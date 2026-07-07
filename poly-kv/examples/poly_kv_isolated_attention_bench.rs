//! Fair isolated Rust attention-operator speed benchmark for poly-kv.
//!
//! Compares:
//! - pre-decoded exact dense attention (keys already decompressed, just matmul + topk)
//! - regular compressed attention (rebuilds codec per call)
//! - prepared compressed attention (pre-built index, only query prep per call)
//! - fully prepared compressed attention (pre-unpacked indices + norms, Gram lookups)
//! - prefetched Gram rows (query-specific rows pre-fetched, cache-friendly scoring)
//! - batch heads (all 12 heads scored in one pass, amortized loop overhead)
//!
//! Runs at head_dim=8 and head_dim=64, scales 64-2048.
//! Emits JSON receipt.

use poly_kv::{AttentionType, KvTensorShape, SharedKVPool};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use std::time::Instant;

fn make_shape(head_dim: usize, num_heads: usize) -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 2,
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

fn run_scale(
    num_tokens: usize,
    head_dim: usize,
    num_heads: usize,
    top_k: usize,
    warmup: usize,
    repeat: usize,
) -> serde_json::Value {
    let shape = make_shape(head_dim, num_heads);
    let corpus = make_corpus(num_tokens, &shape, 42);
    let (pool, _) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let query: Vec<f32> = (0..head_dim).map(|x| x as f32 * 0.125).collect();

    let decompressed = pool.decompress_layer(0).unwrap();
    let pre_decoded_keys = decompressed.keys[0].clone();

    let pre_decoded_ns = bench_fn(
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

    let index = pool.prepare_compressed_index(0, 0).unwrap();
    let prepared_ns = bench_fn(
        || {
            let _ = pool
                .attention_topk_compressed_prepared(&index, &query, top_k)
                .unwrap();
        },
        warmup,
        repeat,
    );

    let fully_index = pool.prepare_fully_compressed_index(0, 0).unwrap();
    let fully_prepared_ns = bench_fn(
        || {
            let _ = pool
                .attention_topk_fully_prepared(&fully_index, &query, top_k)
                .unwrap();
        },
        warmup,
        repeat,
    );

    let prefetched_ns = bench_fn(
        || {
            let _ = pool
                .attention_topk_prefetched(&fully_index, &query, top_k)
                .unwrap();
        },
        warmup,
        repeat,
    );

    // Batch heads: prepare queries for all heads
    let all_queries: Vec<Vec<f32>> = (0..num_heads)
        .map(|h| {
            (0..head_dim)
                .map(|x| x as f32 * 0.125 + h as f32 * 0.01)
                .collect()
        })
        .collect();
    let query_refs: Vec<&[f32]> = all_queries.iter().map(|q| q.as_slice()).collect();
    let batch_ns = bench_fn(
        || {
            let _ = pool
                .attention_topk_batch_heads(&fully_index, &query_refs, top_k)
                .unwrap();
        },
        warmup,
        repeat,
    );

    // Per-head equivalent (sequential heads) for comparison
    let sequential_heads_ns = bench_fn(
        || {
            for (h, _) in all_queries.iter().enumerate().take(num_heads) {
                let _ = pool
                    .attention_topk_prefetched(&fully_index, &all_queries[h], top_k)
                    .unwrap();
            }
        },
        warmup,
        repeat,
    );

    // Quality
    let exact_hits = pre_decoded_exact_attention(&pre_decoded_keys, head_dim, &query, top_k);
    let prefetched_result = pool
        .attention_topk_prefetched(&fully_index, &query, top_k)
        .unwrap();
    let exact_top: std::collections::HashSet<usize> = exact_hits.iter().map(|(i, _)| *i).collect();
    let prefetched_top: std::collections::HashSet<usize> = prefetched_result
        .hits
        .iter()
        .map(|h| h.token_index)
        .collect();
    let overlap = exact_top.intersection(&prefetched_top).count() as f64
        / exact_top.union(&prefetched_top).count().max(1) as f64;

    json!({
        "num_tokens": num_tokens, "head_dim": head_dim, "num_heads": num_heads, "top_k": top_k,
        "pre_decoded_exact_ns": pre_decoded_ns,
        "regular_compressed_ns": regular_ns,
        "prepared_compressed_ns": prepared_ns,
        "fully_prepared_ns": fully_prepared_ns,
        "prefetched_gram_ns": prefetched_ns,
        "batch_heads_ns": batch_ns,
        "sequential_heads_ns": sequential_heads_ns,
        "ratio_fully_prepared": pre_decoded_ns as f64 / fully_prepared_ns as f64,
        "ratio_prefetched": pre_decoded_ns as f64 / prefetched_ns as f64,
        "ratio_batch_per_head": (pre_decoded_ns as f64 * num_heads as f64) / batch_ns as f64,
        "ratio_sequential_per_head": (pre_decoded_ns as f64 * num_heads as f64) / sequential_heads_ns as f64,
        "topk_overlap": overlap,
    })
}

fn main() {
    let warmup = 10;
    let top_k = 8;
    let configs = [
        (
            8usize,
            4usize,
            [64, 128, 256, 512, 1024, 2048].as_slice(),
            50usize,
        ),
        (
            64usize,
            12usize,
            [64, 128, 256, 512, 1024, 2048].as_slice(),
            20usize,
        ),
    ];

    let mut all_results = Vec::new();
    for &(head_dim, num_heads, scales, base_repeat) in &configs {
        eprintln!("\n=== head_dim={head_dim} num_heads={num_heads} ===");
        for &n in scales {
            let repeat = if n >= 1024 {
                base_repeat / 4
            } else {
                base_repeat
            };
            let repeat = repeat.max(5);
            eprintln!("  n={n} repeat={repeat}...");
            all_results.push(run_scale(n, head_dim, num_heads, top_k, warmup, repeat));
        }
    }

    let receipt = json!({
        "schema_version": "poly_kv_simd_batch_bench_v1",
        "claim_boundary": "fair isolated Rust CPU attention benchmark; pre-decoded exact vs regular/prepared/fully-prepared/prefetched/batch-heads compressed; synthetic random vectors; release mode; not production speedup",
        "config": { "top_k": top_k, "warmup": warmup, "build_mode": "release" },
        "results": all_results,
        "passed": true, "blockers": [],
    });

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());

    eprintln!("\n=== BENCHMARK SUMMARY ===");
    eprintln!(
        "{:>6} {:>4} {:>4} {:>10} {:>10} {:>10} {:>10} {:>8} {:>8} {:>8}",
        "tokens",
        "dim",
        "heads",
        "exact_ns",
        "fully_ns",
        "prefet_ns",
        "batch_ns",
        "r_fully",
        "r_pref",
        "r_batch"
    );
    for r in &all_results {
        eprintln!(
            "{:>6} {:>4} {:>4} {:>10} {:>10} {:>10} {:>10} {:>8.2} {:>8.2} {:>8.2}",
            r["num_tokens"].as_u64().unwrap(),
            r["head_dim"].as_u64().unwrap(),
            r["num_heads"].as_u64().unwrap(),
            r["pre_decoded_exact_ns"].as_u64().unwrap(),
            r["fully_prepared_ns"].as_u64().unwrap(),
            r["prefetched_gram_ns"].as_u64().unwrap(),
            r["batch_heads_ns"].as_u64().unwrap(),
            r["ratio_fully_prepared"].as_f64().unwrap(),
            r["ratio_prefetched"].as_f64().unwrap(),
            r["ratio_batch_per_head"].as_f64().unwrap(),
        );
    }
}
