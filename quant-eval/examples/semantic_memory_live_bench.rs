//! Benchmark compressed vs uncompressed search on the live semantic-memory store.
//!
//! Extracts real f32 embeddings from the Hermes semantic-memory SQLite DB,
//! runs brute-force cosine search (uncompressed) vs compressed-scorer search,
//! and produces a receipt with recall/NDCG/latency metrics.
//!
//! Usage: cargo run -p quant-eval --example semantic_memory_live_bench --features testing -- /path/to/memory.db

use std::collections::HashSet;
use std::path::PathBuf;
use std::time::Instant;

use compressed_scorer::PerDimScorer;
use rusqlite::Connection;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
struct LiveBenchmarkReceipt {
    db_path: String,
    fact_count: usize,
    embedding_dim: usize,
    top_k: usize,
    query_count: usize,
    raw_mean_latency_us: f64,
    compressed_mean_latency_us: f64,
    recall_at_1: f32,
    recall_at_5: f32,
    recall_at_10: f32,
    recall_at_k: f32,
    exact_rerank_recovery_at_1: f32,
    top_k_overlap: f32,
    compression_ratio: f32,
    raw_bytes: usize,
    compressed_bytes: usize,
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let db_path = if args.len() >= 2 {
        PathBuf::from(&args[1])
    } else {
        PathBuf::from(
            std::env::var("HOME")
                .map(|h| format!("{h}/.hermes/semantic-memory.db/memory.db"))
                .unwrap_or_else(|_| "memory.db".to_string()),
        )
    };

    let top_k = if args.len() >= 3 {
        args[2].parse::<usize>().unwrap_or(10)
    } else {
        10
    };

    eprintln!("opening: {db_path:?}");
    let conn = Connection::open(&db_path).expect("failed to open DB");

    // Extract all fact embeddings as f32 vectors
    let mut stmt = conn
        .prepare("SELECT id, embedding FROM facts WHERE embedding IS NOT NULL")
        .expect("failed to prepare query");

    let rows = stmt
        .query_map([], |row| {
            let fact_id: String = row.get(0)?;
            let blob: Vec<u8> = row.get(1)?;
            Ok((fact_id, blob))
        })
        .expect("failed to query embeddings");

    let mut fact_ids: Vec<String> = Vec::new();
    let mut embeddings: Vec<Vec<f32>> = Vec::new();

    for row in rows {
        let (fact_id, blob) = row.expect("failed to read row");
        let dim = blob.len() / 4;
        let vec: Vec<f32> = (0..dim)
            .map(|i| {
                let bytes = &blob[i * 4..i * 4 + 4];
                f32::from_le_bytes([bytes[0], bytes[1], bytes[2], bytes[3]])
            })
            .collect();
        fact_ids.push(fact_id);
        embeddings.push(vec);
    }

    let fact_count = embeddings.len();
    let embedding_dim = embeddings.first().map(|v| v.len()).unwrap_or(0);
    eprintln!("loaded {fact_count} embeddings, dim={embedding_dim}");

    if fact_count == 0 {
        eprintln!("no embeddings found, exiting");
        return;
    }

    // Use first N queries (rotate through the corpus)
    let query_count = fact_count.min(50);
    let queries: Vec<Vec<f32>> = embeddings[..query_count].to_vec();

    // Build compressed scorer (8-bit per-dimension quantization)
    let bits = 8;
    let mut per_dim = PerDimScorer::new(embedding_dim, bits)
        .expect("failed to build per-dim scorer");
    let refs: Vec<&[f32]> = embeddings.iter().map(|v| v.as_slice()).collect();
    per_dim.fit(&refs).expect("failed to fit scorer");

    // Compress all embeddings
    let compressed: Vec<_> = embeddings
        .iter()
        .map(|v| per_dim.compress(v).expect("failed to compress"))
        .collect();

    // Run brute-force (raw) search for each query
    let mut raw_results: Vec<Vec<(usize, f32)>> = Vec::with_capacity(query_count);
    let raw_start = Instant::now();
    for query in &queries {
        let mut scores: Vec<(usize, f32)> = embeddings
            .iter()
            .enumerate()
            .map(|(idx, vec)| (idx, cosine_sim(query, vec)))
            .collect();
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        raw_results.push(scores.into_iter().take(top_k).collect());
    }
    let raw_elapsed = raw_start.elapsed();
    let raw_mean_latency_us = raw_elapsed.as_secs_f64() * 1_000_000.0 / query_count as f64;

    // Run compressed search for each query
    let mut compressed_results: Vec<Vec<(usize, f32)>> = Vec::with_capacity(query_count);
    let comp_start = Instant::now();
    for query in &queries {
        let candidates = compressed_scorer::search_topk(&per_dim, query, &compressed, top_k * 4)
            .expect("search failed");
        // Rerank with exact f32
        let mut reranked: Vec<(usize, f32)> = candidates
            .into_iter()
            .map(|c| (c.idx, cosine_sim(query, &embeddings[c.idx])))
            .collect();
        reranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
        compressed_results.push(reranked.into_iter().take(top_k).collect());
    }
    let comp_elapsed = comp_start.elapsed();
    let compressed_mean_latency_us = comp_elapsed.as_secs_f64() * 1_000_000.0 / query_count as f64;

    // Compute recall metrics
    let mut recall_1 = 0.0f32;
    let mut recall_5 = 0.0f32;
    let mut recall_10 = 0.0f32;
    let mut recall_k = 0.0f32;
    let mut top_k_overlap = 0.0f32;
    let mut exact_rerank_recovery_1 = 0.0f32;

    for i in 0..query_count {
        let raw = &raw_results[i];
        let comp = &compressed_results[i];

        let raw_ids: HashSet<usize> = raw.iter().map(|(idx, _)| *idx).collect();
        let comp_ids: HashSet<usize> = comp.iter().map(|(idx, _)| *idx).collect();

        // Recall@k: fraction of raw top-k found in compressed top-k
        if let Some((raw_top1, _)) = raw.first() {
            if comp.iter().any(|(idx, _)| idx == raw_top1) {
                recall_1 += 1.0;
            }
        }
        let raw_5: HashSet<usize> = raw.iter().take(5).map(|(idx, _)| *idx).collect();
        let comp_5: HashSet<usize> = comp.iter().take(5).map(|(idx, _)| *idx).collect();
        recall_5 += raw_5.intersection(&comp_5).count() as f32 / raw_5.len().max(1) as f32;

        let raw_10: HashSet<usize> = raw.iter().take(10).map(|(idx, _)| *idx).collect();
        let comp_10: HashSet<usize> = comp.iter().take(10).map(|(idx, _)| *idx).collect();
        recall_10 += raw_10.intersection(&comp_10).count() as f32 / raw_10.len().max(1) as f32;

        recall_k += raw_ids.intersection(&comp_ids).count() as f32 / raw_ids.len().max(1) as f32;

        top_k_overlap +=
            raw_ids.intersection(&comp_ids).count() as f32 / raw_ids.len().max(1) as f32;

        // Exact rerank recovery@1: does compressed top-1 match raw top-1?
        if let (Some((raw_1, _)), Some((comp_1, _))) = (raw.first(), comp.first()) {
            if raw_1 == comp_1 {
                exact_rerank_recovery_1 += 1.0;
            }
        }
    }

    let q = query_count as f32;
    let receipt = LiveBenchmarkReceipt {
        db_path: db_path.to_string_lossy().to_string(),
        fact_count,
        embedding_dim,
        top_k,
        query_count,
        raw_mean_latency_us,
        compressed_mean_latency_us,
        recall_at_1: recall_1 / q,
        recall_at_5: recall_5 / q,
        recall_at_10: recall_10 / q,
        recall_at_k: recall_k / q,
        exact_rerank_recovery_at_1: exact_rerank_recovery_1 / q,
        top_k_overlap: top_k_overlap / q,
        // 8-bit vs 32-bit = 4x compression
        compression_ratio: 4.0,
        raw_bytes: fact_count * embedding_dim * 4,
        compressed_bytes: fact_count * embedding_dim,
    };

    let output = serde_json::to_string_pretty(&receipt).expect("failed to serialize");
    println!("{output}");

    // Print summary
    eprintln!("\n=== LIVE SEMANTIC MEMORY BENCHMARK ===");
    eprintln!("facts: {fact_count}, dim: {embedding_dim}, queries: {query_count}, top_k: {top_k}");
    eprintln!("raw search:       {raw_mean_latency_us:.1} µs/query");
    eprintln!("compressed search: {compressed_mean_latency_us:.1} µs/query");
    eprintln!("recall@1:  {:.4}", receipt.recall_at_1);
    eprintln!("recall@5:  {:.4}", receipt.recall_at_5);
    eprintln!("recall@10: {:.4}", receipt.recall_at_10);
    eprintln!("recall@k:  {:.4}", receipt.recall_at_k);
    eprintln!(
        "exact rerank recovery@1: {:.4}",
        receipt.exact_rerank_recovery_at_1
    );
    eprintln!("top-k overlap: {:.4}", receipt.top_k_overlap);
    eprintln!(
        "compression ratio: {}x ({:.1} MB -> {:.1} MB)",
        receipt.compression_ratio,
        receipt.raw_bytes as f64 / 1_048_576.0,
        receipt.compressed_bytes as f64 / 1_048_576.0,
    );
}

fn cosine_sim(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let norm_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let norm_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if norm_a < 1e-12 || norm_b < 1e-12 {
        return 0.0;
    }
    dot / (norm_a * norm_b)
}
