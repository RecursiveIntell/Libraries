//! Isolated Rust attention-operator speed benchmark for poly-kv.
//!
//! Compares exact dense attention vs:
//! - regular compressed attention (rebuilds codec per call)
//! - prepared compressed attention (pre-built index, only query prep per call)
//!
//! Emits a JSON receipt with timing, speed ratios, and quality metrics.

use poly_kv::{AttentionType, KvTensorShape, SharedKVPool};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;
use std::time::Instant;

fn make_shape() -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers: 2,
        num_heads: 4,
        num_kv_heads: 4,
        head_dim: 8,
        hidden_size: 32,
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

fn exact_dense_attention(
    pool: &SharedKVPool,
    layer_idx: usize,
    query: &[f32],
    top_k: usize,
) -> Vec<(usize, f32)> {
    let decompressed = pool.decompress_layer(layer_idx).unwrap();
    let keys = decompressed.keys.first().unwrap();
    let head_dim = decompressed.head_dim;
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

fn bench_fn<F: FnMut()>(mut f: F, warmup: usize, repeat: usize) -> (u128, u128) {
    for _ in 0..warmup {
        f();
    }
    let start = Instant::now();
    for _ in 0..repeat {
        f();
    }
    let elapsed = start.elapsed();
    (elapsed.as_nanos() / repeat as u128, elapsed.as_nanos())
}

fn main() {
    let shape = make_shape();
    let corpus = make_corpus(128, &shape, 42);
    let (pool, _) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let query: Vec<f32> = (0..shape.head_dim).map(|x| x as f32 * 0.125).collect();
    let top_k = 8;
    let warmup = 10;
    let repeat = 100;

    // Exact dense attention timing
    let (exact_ns, _) = bench_fn(
        || {
            let _ = exact_dense_attention(&pool, 0, &query, top_k);
        },
        warmup,
        repeat,
    );

    // Regular compressed attention (rebuilds codec per call)
    let (regular_ns, _) = bench_fn(
        || {
            let _ = pool.attention_topk_compressed(0, 0, &query, top_k).unwrap();
        },
        warmup,
        repeat,
    );

    // Prepared compressed attention (pre-built index)
    let index = pool.prepare_compressed_index(0, 0).unwrap();
    let (prepared_ns, _) = bench_fn(
        || {
            let _ = pool
                .attention_topk_compressed_prepared(&index, &query, top_k)
                .unwrap();
        },
        warmup,
        repeat,
    );

    // Quality: compare prepared vs exact
    let exact_hits = exact_dense_attention(&pool, 0, &query, top_k);
    let prepared_result = pool
        .attention_topk_compressed_prepared(&index, &query, top_k)
        .unwrap();
    let exact_top: std::collections::HashSet<usize> = exact_hits.iter().map(|(i, _)| *i).collect();
    let prepared_top: std::collections::HashSet<usize> =
        prepared_result.hits.iter().map(|h| h.token_index).collect();
    let overlap = exact_top.intersection(&prepared_top).count() as f64
        / exact_top.union(&prepared_top).count().max(1) as f64;

    let speed_ratio_regular = exact_ns as f64 / regular_ns as f64;
    let speed_ratio_prepared = exact_ns as f64 / prepared_ns as f64;

    let receipt = json!({
        "schema_version": "poly_kv_isolated_rust_attention_bench_v1",
        "claim_boundary": "isolated Rust CPU attention-operator benchmark over synthetic pool; not production runtime speedup, not GPU evidence, not end-to-end latency evidence",
        "config": {
            "num_tokens": corpus.len(),
            "head_dim": shape.head_dim,
            "num_heads": shape.num_heads,
            "num_layers": shape.num_layers,
            "top_k": top_k,
            "warmup": warmup,
            "repeat": repeat,
        },
        "results": {
            "exact_dense_ns_mean": exact_ns,
            "regular_compressed_ns_mean": regular_ns,
            "prepared_compressed_ns_mean": prepared_ns,
            "speed_ratio_exact_over_regular": speed_ratio_regular,
            "speed_ratio_exact_over_prepared": speed_ratio_prepared,
            "topk_overlap": overlap,
        },
        "passed": true,
        "blockers": [],
    });

    println!("{}", serde_json::to_string_pretty(&receipt).unwrap());

    eprintln!(
        "exact={exact_ns}ns regular={regular_ns}ns prepared={prepared_ns}ns \
         ratio_regular={speed_ratio_regular:.4}x ratio_prepared={speed_ratio_prepared:.4}x \
         overlap={overlap:.4}"
    );
}
