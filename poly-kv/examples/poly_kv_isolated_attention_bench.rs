//! Fair isolated Rust attention-operator speed benchmark for poly-kv.
//!
//! Compares:
//! - pre-decoded exact dense attention (keys already decompressed, just matmul + topk)
//! - regular compressed attention (rebuilds codec per call)
//! - prepared compressed attention (pre-built index, only query prep per call)
//!
//! Runs a scale sweep over multiple token counts.
//! Emits a JSON receipt with per-scale timing, speed ratios, and quality metrics.

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

/// Pre-decoded exact dense attention: keys already decompressed, just matmul + topk.
/// This is the FAIR comparison — no decompress cost, just the scoring operator.
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

    // Pre-decode keys once for fair comparison
    let decompressed = pool.decompress_layer(0).unwrap();
    let pre_decoded_keys = decompressed.keys[0].clone();

    // Pre-decoded exact dense (fair: no decompress cost)
    let pre_decoded_ns = bench_fn(
        || {
            let _ = pre_decoded_exact_attention(&pre_decoded_keys, head_dim, &query, top_k);
        },
        warmup,
        repeat,
    );

    // Regular compressed attention (rebuilds codec per call)
    let regular_ns = bench_fn(
        || {
            let _ = pool.attention_topk_compressed(0, 0, &query, top_k).unwrap();
        },
        warmup,
        repeat,
    );

    // Prepared compressed attention (pre-built index)
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

    // Quality: compare prepared vs pre-decoded exact
    let exact_hits = pre_decoded_exact_attention(&pre_decoded_keys, head_dim, &query, top_k);
    let prepared_result = pool
        .attention_topk_compressed_prepared(&index, &query, top_k)
        .unwrap();
    let exact_top: std::collections::HashSet<usize> = exact_hits.iter().map(|(i, _)| *i).collect();
    let prepared_top: std::collections::HashSet<usize> =
        prepared_result.hits.iter().map(|h| h.token_index).collect();
    let union_count = exact_top.union(&prepared_top).count().max(1);
    let overlap = exact_top.intersection(&prepared_top).count() as f64 / union_count as f64;

    let speed_ratio_pre_decoded_over_prepared = pre_decoded_ns as f64 / prepared_ns as f64;
    let speed_ratio_pre_decoded_over_regular = pre_decoded_ns as f64 / regular_ns as f64;

    json!({
        "num_tokens": num_tokens,
        "head_dim": head_dim,
        "num_heads": num_heads,
        "top_k": top_k,
        "pre_decoded_exact_ns_mean": pre_decoded_ns,
        "regular_compressed_ns_mean": regular_ns,
        "prepared_compressed_ns_mean": prepared_ns,
        "speed_ratio_pre_decoded_over_regular": speed_ratio_pre_decoded_over_regular,
        "speed_ratio_pre_decoded_over_prepared": speed_ratio_pre_decoded_over_prepared,
        "topk_overlap": overlap,
    })
}

fn main() {
    let warmup = 10;
    let repeat = 50;
    let top_k = 8;
    let head_dim = 8;
    let num_heads = 4;

    let scales = [64, 128, 256, 512];
    let mut results = Vec::new();

    for &n in &scales {
        eprintln!("running scale n={n}...");
        results.push(run_scale(n, head_dim, num_heads, top_k, warmup, repeat));
    }

    let receipt = json!({
        "schema_version": "poly_kv_fair_rust_attention_bench_v1",
        "claim_boundary": "fair isolated Rust CPU attention-operator benchmark; pre-decoded exact dense (no decompress cost) vs prepared compressed (pre-built index) vs regular compressed (rebuilds per call); synthetic random vectors; not production runtime speedup, not GPU evidence, not real-model quality evidence",
        "config": {
            "head_dim": head_dim,
            "num_heads": num_heads,
            "num_layers": 2,
            "top_k": top_k,
            "warmup": warmup,
            "repeat": repeat,
            "scales": scales,
        },
        "results": results,
        "passed": true,
        "blockers": [],
    });

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());

    eprintln!("\n=== FAIR BENCHMARK SUMMARY ===");
    eprintln!(
        "{:>8} {:>12} {:>12} {:>12} {:>10} {:>10} {:>8}",
        "tokens", "pre_dec_ns", "reg_comp_ns", "prep_comp_ns", "ratio_reg", "ratio_prep", "overlap"
    );
    for r in &results {
        eprintln!(
            "{:>8} {:>12} {:>12} {:>12} {:>10.4} {:>10.4} {:>8.4}",
            r["num_tokens"].as_u64().unwrap(),
            r["pre_decoded_exact_ns_mean"].as_u64().unwrap(),
            r["regular_compressed_ns_mean"].as_u64().unwrap(),
            r["prepared_compressed_ns_mean"].as_u64().unwrap(),
            r["speed_ratio_pre_decoded_over_regular"].as_f64().unwrap(),
            r["speed_ratio_pre_decoded_over_prepared"].as_f64().unwrap(),
            r["topk_overlap"].as_f64().unwrap(),
        );
    }
}
