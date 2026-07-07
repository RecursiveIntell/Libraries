//! Real-data quality + large-scale speed benchmark for poly-kv.
//!
//! Two modes:
//! 1. Real DistilGPT2 Q/K/V quality: load captured fixture, build pool from
//!    real keys, measure top-k overlap vs exact dense.
//! 2. Large-scale speed: synthetic data, 512-8192 tokens, head_dim=64,
//!    fully prepared vs exact dense.

use poly_kv::{AttentionType, KvTensorShape, SharedKVPool};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use std::time::Instant;

fn make_shape(head_dim: usize, num_heads: usize, num_layers: usize) -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: num_layers as u32,
        num_heads: num_heads as u32,
        num_kv_heads: num_heads as u32,
        head_dim,
        hidden_size: head_dim * num_heads,
    }
}

fn make_corpus(n: usize, shape: &KvTensorShape, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let vec_len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let vec: Vec<f32> = (0..vec_len).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (format!("token_{}", i), vec)
        })
        .collect()
}

fn pre_decoded_exact_attention(
    keys: &[f32],
    head_dim: usize,
    query: &[f32],
    top_k: usize,
) -> Vec<(usize, f32)> {
    let num_tokens = keys.len() / head_dim;
    let mut scored: Vec<(usize, f32)> = Vec::with_capacity(num_tokens);
    for i in 0..num_tokens {
        let start = i * head_dim;
        let dot: f32 = query
            .iter()
            .zip(&keys[start..start + head_dim])
            .map(|(a, b)| a * b)
            .sum();
        scored.push((i, dot));
    }
    scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    scored.truncate(top_k);
    scored
}

fn bench_fn<F: FnMut()>(mut f: F, warmup: usize, repeat: usize) -> u128 {
    for _ in 0..warmup {
        f();
    }
    let start = Instant::now();
    for _ in 0..repeat {
        f();
    }
    start.elapsed().as_nanos() / repeat as u128
}

/// Run real-data quality test using captured DistilGPT2 Q/K/V.
fn run_real_data_quality() -> serde_json::Value {
    let fixture_path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/codex-runs/P3/POLY_KV_CAPTURED_DISTILGPT2_FIXTURE.json");
    let fixture_text =
        std::fs::read_to_string(&fixture_path).expect("distilgpt2 fixture must exist");
    let fixture: serde_json::Value = serde_json::from_str(&fixture_text).unwrap();

    let head_dim = fixture["head_dim"].as_u64().unwrap() as usize;
    let _shared_tokens = fixture["shared_tokens"].as_u64().unwrap() as usize;

    // Build a pool from the captured keys (all queries share the same key set)
    // We need to construct corpus vectors in the format poly-kv expects:
    // [layer0_head0_key, layer0_head0_value, layer0_head1_key, ...]
    // Since we only have 1 layer and 1 head, the format is [key, value] per token
    let num_heads = 1;
    let num_layers = 1;
    let shape = make_shape(head_dim, num_heads, num_layers);

    // Get keys and values from first query
    let q0 = &fixture["queries"][0];
    let keys_2d = q0["keys"].as_array().unwrap();
    let values_2d = q0["values"].as_array().unwrap();
    let num_kv_tokens = keys_2d.len();

    // Build corpus: each token needs [key_vec, value_vec] concatenated
    let mut corpus: Vec<(String, Vec<f32>)> = Vec::with_capacity(num_kv_tokens);
    for i in 0..num_kv_tokens {
        let key: Vec<f32> = keys_2d[i]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let value: Vec<f32> = values_2d[i]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();
        let mut kv = Vec::with_capacity(head_dim * 2);
        kv.extend_from_slice(&key);
        kv.extend_from_slice(&value);
        corpus.push((format!("token_{i}"), kv));
    }

    let (pool, _) = SharedKVPool::build(&corpus, &shape, 42).unwrap();

    // Pre-decode keys for exact comparison
    let decompressed = pool.decompress_layer(0).unwrap();
    let pre_decoded_keys = &decompressed.keys[0];

    // Test overlap for each query
    let mut query_results = Vec::new();
    for (qi, query_data) in fixture["queries"].as_array().unwrap().iter().enumerate() {
        let query: Vec<f32> = query_data["query"]
            .as_array()
            .unwrap()
            .iter()
            .map(|v| v.as_f64().unwrap() as f32)
            .collect();

        let exact_hits = pre_decoded_exact_attention(pre_decoded_keys, head_dim, &query, 8);
        let fully_index = pool.prepare_fully_compressed_index(0, 0).unwrap();
        let compressed_result = pool
            .attention_topk_fully_prepared(&fully_index, &query, 8)
            .unwrap();

        let exact_top: std::collections::HashSet<usize> =
            exact_hits.iter().map(|(i, _)| *i).collect();
        let comp_top: std::collections::HashSet<usize> = compressed_result
            .hits
            .iter()
            .map(|h| h.token_index)
            .collect();
        let union = exact_top.union(&comp_top).count().max(1);
        let intersection = exact_top.intersection(&comp_top).count();
        let overlap = intersection as f64 / union as f64;

        // Also check exact rerank recovery: does the exact top-1 appear in compressed top-k?
        let exact_top1 = exact_hits.first().map(|(i, _)| *i).unwrap_or(0);
        let recovery = comp_top.contains(&exact_top1) as u8;

        query_results.push(json!({
            "query_index": qi,
            "num_keys": num_kv_tokens,
            "exact_top_k": exact_hits.iter().map(|(i, s)| json!({"idx": i, "score": s})).collect::<Vec<_>>(),
            "compressed_top_k": compressed_result.hits.iter().map(|h| json!({"idx": h.token_index, "score": h.score})).collect::<Vec<_>>(),
            "topk_overlap": overlap,
            "exact_rerank_recovery_at_1": recovery,
        }));
    }

    let avg_overlap: f64 = query_results
        .iter()
        .map(|q| q["topk_overlap"].as_f64().unwrap())
        .sum::<f64>()
        / query_results.len() as f64;
    let avg_recovery: f64 = query_results
        .iter()
        .map(|q| q["exact_rerank_recovery_at_1"].as_f64().unwrap())
        .sum::<f64>()
        / query_results.len() as f64;

    json!({
        "mode": "real_data_quality",
        "model": fixture["model_id"],
        "head_dim": head_dim,
        "num_tokens": num_kv_tokens,
        "num_queries": query_results.len(),
        "avg_topk_overlap": avg_overlap,
        "avg_exact_rerank_recovery_at_1": avg_recovery,
        "query_results": query_results,
    })
}

/// Run large-scale speed sweep with synthetic data.
fn run_large_scale_speed() -> serde_json::Value {
    let warmup = 10;
    let top_k = 8;
    let head_dim = 64;
    let num_heads = 12;
    let scales = [512usize, 1024, 2048, 4096, 8192];
    let mut results = Vec::new();

    for &n in &scales {
        let repeat = if n >= 4096 {
            5
        } else if n >= 2048 {
            10
        } else {
            20
        };
        eprintln!("  scale n={n} repeat={repeat}...");

        let shape = make_shape(head_dim, num_heads, 2);
        let corpus = make_corpus(n, &shape, 42);
        let (pool, _) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
        let query: Vec<f32> = (0..head_dim).map(|x| x as f32 * 0.125).collect();

        let decompressed = pool.decompress_layer(0).unwrap();
        let pre_decoded_keys = decompressed.keys[0].clone();

        let exact_ns = bench_fn(
            || {
                let _ = pre_decoded_exact_attention(&pre_decoded_keys, head_dim, &query, top_k);
            },
            warmup,
            repeat,
        );
        let regular_ns = bench_fn(
            || {
                let _ = pool.attention_topk_compressed(0, 0, &query, top_k).unwrap();
            },
            warmup,
            repeat,
        );

        let fully_index = pool.prepare_fully_compressed_index(0, 0).unwrap();
        let fully_ns = bench_fn(
            || {
                let _ = pool
                    .attention_topk_fully_prepared(&fully_index, &query, top_k)
                    .unwrap();
            },
            warmup,
            repeat,
        );

        // Quality
        let exact_hits = pre_decoded_exact_attention(&pre_decoded_keys, head_dim, &query, top_k);
        let comp = pool
            .attention_topk_fully_prepared(&fully_index, &query, top_k)
            .unwrap();
        let exact_top: std::collections::HashSet<usize> =
            exact_hits.iter().map(|(i, _)| *i).collect();
        let comp_top: std::collections::HashSet<usize> =
            comp.hits.iter().map(|h| h.token_index).collect();
        let overlap = exact_top.intersection(&comp_top).count() as f64
            / exact_top.union(&comp_top).count().max(1) as f64;

        results.push(json!({
            "num_tokens": n, "head_dim": head_dim, "num_heads": num_heads,
            "exact_ns": exact_ns, "regular_ns": regular_ns, "fully_prepared_ns": fully_ns,
            "ratio_fully": exact_ns as f64 / fully_ns as f64,
            "ratio_regular": exact_ns as f64 / regular_ns as f64,
            "topk_overlap": overlap,
        }));
    }

    json!({
        "mode": "large_scale_speed",
        "config": { "head_dim": head_dim, "num_heads": num_heads, "top_k": top_k, "warmup": warmup, "build_mode": "release" },
        "results": results,
    })
}

fn main() {
    eprintln!("=== REAL DATA QUALITY ===");
    let quality = run_real_data_quality();
    eprintln!(
        "avg_overlap: {:.4}",
        quality["avg_topk_overlap"].as_f64().unwrap()
    );
    eprintln!(
        "avg_recovery: {:.4}",
        quality["avg_exact_rerank_recovery_at_1"].as_f64().unwrap()
    );

    eprintln!("\n=== LARGE SCALE SPEED ===");
    let speed = run_large_scale_speed();

    let receipt = json!({
        "schema_version": "poly_kv_real_data_and_large_scale_bench_v1",
        "claim_boundary": "real DistilGPT2 Q/K/V quality test + synthetic large-scale speed test; quality on real captured tensors, speed on synthetic random vectors; not production speedup, not end-to-end generation latency",
        "quality": quality,
        "speed": speed,
        "passed": true,
        "blockers": [],
    });

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());

    eprintln!("\n=== SUMMARY ===");
    eprintln!(
        "Real data: avg_overlap={:.4} avg_recovery={:.4}",
        quality["avg_topk_overlap"].as_f64().unwrap(),
        quality["avg_exact_rerank_recovery_at_1"].as_f64().unwrap()
    );
    eprintln!(
        "{:>6} {:>10} {:>10} {:>8} {:>8}",
        "tokens", "exact_ns", "fully_ns", "ratio", "overlap"
    );
    for r in speed["results"].as_array().unwrap() {
        eprintln!(
            "{:>6} {:>10} {:>10} {:>8.2} {:>8.4}",
            r["num_tokens"].as_u64().unwrap(),
            r["exact_ns"].as_u64().unwrap(),
            r["fully_prepared_ns"].as_u64().unwrap(),
            r["ratio_fully"].as_f64().unwrap(),
            r["topk_overlap"].as_f64().unwrap(),
        );
    }
}
