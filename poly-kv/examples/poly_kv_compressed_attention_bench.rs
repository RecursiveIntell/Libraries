use std::time::Instant;

use poly_kv::{AttentionType, KvTensorShape, SharedKVPool};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use serde_json::json;

fn env_usize(name: &str, default: usize) -> usize {
    std::env::var(name)
        .ok()
        .and_then(|v| v.parse().ok())
        .unwrap_or(default)
}

fn percentile(sorted: &[u128], pct: f64) -> u128 {
    if sorted.is_empty() {
        return 0;
    }
    let idx = ((sorted.len() - 1) as f64 * pct).round() as usize;
    sorted[idx.min(sorted.len() - 1)]
}

fn ns_stats(values: &[u128]) -> serde_json::Value {
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let sum: u128 = values.iter().sum();
    json!({
        "mean_ns": if values.is_empty() { 0 } else { (sum / values.len() as u128) as u64 },
        "p50_ns": percentile(&sorted, 0.50) as u64,
        "p95_ns": percentile(&sorted, 0.95) as u64,
        "max_ns": sorted.last().copied().unwrap_or(0) as u64,
    })
}

fn make_shape(num_layers: u32, num_heads: u32, head_dim: usize) -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::MHA,
        num_layers,
        num_heads,
        num_kv_heads: num_heads,
        head_dim,
        hidden_size: num_heads as usize * head_dim,
    }
}

fn make_corpus(n: usize, shape: &KvTensorShape, seed: u64) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    let vec_len = shape.num_layers as usize * shape.num_kv_heads as usize * shape.head_dim * 2;
    (0..n)
        .map(|i| {
            let mut v = Vec::with_capacity(vec_len);
            for j in 0..vec_len {
                let base = ((i as f32 * 0.017) + (j as f32 * 0.013)).sin() * 0.5;
                v.push(base + rng.gen_range(-0.25f32..0.25));
            }
            (format!("token_{i}"), v)
        })
        .collect()
}

fn make_queries(n: usize, head_dim: usize, seed: u64) -> Vec<Vec<f32>> {
    let mut rng = ChaCha8Rng::seed_from_u64(seed);
    (0..n)
        .map(|i| {
            (0..head_dim)
                .map(|j| ((i as f32 * 0.031) + (j as f32 * 0.07)).cos() + rng.gen_range(-0.1..0.1))
                .collect()
        })
        .collect()
}

fn overlap(a: &[(usize, bool)], b: &[(usize, bool)]) -> f64 {
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let inter = a.iter().filter(|x| b.contains(x)).count();
    let union = a.len() + b.len() - inter;
    inter as f64 / union.max(1) as f64
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let shared_tokens = env_usize("PKV_SHARED_TOKENS", 128);
    let shell_tokens = env_usize("PKV_SHELL_TOKENS", 24);
    let num_queries = env_usize("PKV_QUERIES", 24);
    let top_k = env_usize("PKV_TOP_K", 16);
    let head_dim = env_usize("PKV_HEAD_DIM", 64);
    let num_heads = env_usize("PKV_HEADS", 4) as u32;
    let num_layers = env_usize("PKV_LAYERS", 2) as u32;
    let seed = 42u64;

    let shape = make_shape(num_layers, num_heads, head_dim);
    let shared = make_corpus(shared_tokens, &shape, seed);
    let shell_corpus = make_corpus(shell_tokens, &shape, seed + 1);

    let t_build = Instant::now();
    let (pool, pool_receipt) = SharedKVPool::build(&shared, &shape, seed)?;
    let build_ns = t_build.elapsed().as_nanos();

    let t_shell = Instant::now();
    let (shell, shell_receipt) = pool.materialize_shell("bench_agent", &shell_corpus, seed + 2)?;
    let shell_ns = t_shell.elapsed().as_nanos();

    let queries = make_queries(num_queries, head_dim, seed + 3);
    let mut legacy_ns = Vec::with_capacity(num_queries);
    let mut compressed_ns = Vec::with_capacity(num_queries);
    let mut overlaps = Vec::with_capacity(num_queries);
    let mut selected_pool_total = 0u64;
    let mut selected_shell_total = 0u64;
    let mut decoded_values_total = 0u64;
    let mut last_receipt = None;

    for query in &queries {
        let t = Instant::now();
        let legacy = shell.attention_topk(&pool, 0, query, top_k, &pool.policy.turbo_config)?;
        legacy_ns.push(t.elapsed().as_nanos());

        let t = Instant::now();
        let compressed = shell.attention_topk_compressed(&pool, 0, 0, query, top_k)?;
        compressed_ns.push(t.elapsed().as_nanos());

        let legacy_ids: Vec<(usize, bool)> = legacy
            .iter()
            .map(|hit| (hit.token_index, hit.from_shell))
            .collect();
        let compressed_ids: Vec<(usize, bool)> = compressed
            .hits
            .iter()
            .map(|hit| (hit.token_index, hit.from_shell))
            .collect();
        overlaps.push(overlap(&legacy_ids, &compressed_ids));
        selected_pool_total += compressed.receipt.selected_pool_count as u64;
        selected_shell_total += compressed.receipt.selected_shell_count as u64;
        decoded_values_total += compressed.receipt.decoded_value_vectors;
        last_receipt = Some(compressed.receipt);
    }

    let mean_overlap = overlaps.iter().sum::<f64>() / overlaps.len().max(1) as f64;
    let min_overlap = overlaps.iter().copied().fold(f64::INFINITY, f64::min);
    let legacy_mean = legacy_ns.iter().sum::<u128>() / legacy_ns.len() as u128;
    let compressed_mean = compressed_ns.iter().sum::<u128>() / compressed_ns.len() as u128;
    let speed_ratio = legacy_mean as f64 / compressed_mean.max(1) as f64;
    let candidate_count = shared_tokens + shell_tokens;
    let decoded_possible_full_values = (candidate_count * num_queries) as u64;
    let value_decode_reduction =
        decoded_possible_full_values as f64 / decoded_values_total.max(1) as f64;

    let passed = mean_overlap >= 0.30
        && decoded_values_total as usize == num_queries * top_k.min(candidate_count)
        && last_receipt
            .as_ref()
            .map(|r| !r.full_layer_decoded && r.exact_fallback_required)
            .unwrap_or(false);

    let receipt = json!({
        "schema": "poly-kv-compressed-attention-bench-v1",
        "claim_boundary": "local synthetic benchmark of compressed candidate selection over reconstructed KV artifacts; not model-quality, logit, PPL, or production latency evidence",
        "config": {
            "shared_tokens": shared_tokens,
            "shell_tokens": shell_tokens,
            "num_queries": num_queries,
            "top_k": top_k,
            "head_dim": head_dim,
            "num_heads": num_heads,
            "num_layers": num_layers,
            "seed": seed
        },
        "build": {
            "pool_build_ns": build_ns as u64,
            "shell_materialize_ns": shell_ns as u64,
            "pool_backend": pool_receipt.backend,
            "pool_bytes": pool_receipt.pool_size_bytes,
            "pool_raw_bytes": pool_receipt.raw_size_bytes,
            "pool_compression_ratio": pool_receipt.compression_ratio,
            "shell_bytes": shell_receipt.shell_size_bytes,
            "shell_unique_tokens": shell_receipt.num_unique_tokens
        },
        "paths": {
            "legacy_full_decode_key_score": "AgentShell::attention_topk(pool decompress_layer + shell key decode)",
            "compressed_candidate_score": "AgentShell::attention_topk_compressed(Fib cold-pool codes + Turbo hot-shell codes + selected value decode)"
        },
        "metrics": {
            "legacy_full_decode_score_latency": ns_stats(&legacy_ns),
            "compressed_candidate_latency": ns_stats(&compressed_ns),
            "legacy_over_compressed_speed_ratio": speed_ratio,
            "top_k_overlap_mean_vs_legacy": mean_overlap,
            "top_k_overlap_min_vs_legacy": min_overlap,
            "decoded_values_total": decoded_values_total,
            "decoded_possible_full_values": decoded_possible_full_values,
            "value_decode_reduction_vs_full_value_decode": value_decode_reduction,
            "selected_pool_total": selected_pool_total,
            "selected_shell_total": selected_shell_total
        },
        "last_selection_receipt": last_receipt,
        "passed": passed,
        "blockers": if passed { Vec::<String>::new() } else { vec!["failed overlap/decode/receipt gate".to_string()] }
    });

    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
