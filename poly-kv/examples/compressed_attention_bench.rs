//! `compressed_attention_bench` — Rust compressed attention throughput benchmark.
//!
//! Generates a synthetic KV pool, materializes an agent shell, and runs
//! compressed top-k attention for multiple queries. Reports latency,
//! decoded keys, and throughput as a JSON receipt.
//!
//! Usage:
//!   cargo run --release --example compressed_attention_bench
//!   cargo run --release --example compressed_attention_bench -- --num-queries 128 --top-k 64

use std::time::Instant;

use poly_kv::pool::SharedKVPool;
use poly_kv::shape::{AttentionType, KvTensorShape};
use poly_kv::shell;
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};
use serde::Serialize;

#[derive(Debug, Serialize)]
struct BenchReceipt {
    schema_version: String,
    num_queries: usize,
    top_k: usize,
    head_dim: usize,
    num_layers: u32,
    num_kv_heads: u32,
    pool_tokens: usize,
    shell_tokens: usize,
    total_scored: usize,
    decoded_keys_total: usize,
    total_latency_ms: u128,
    per_query_latency_us: f64,
    queries_per_second: f64,
    compression_ratio: f64,
    pool_size_bytes: u64,
}

fn make_shape() -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::GQA,
        num_layers: 24,
        num_heads: 32,
        num_kv_heads: 8,
        head_dim: 64,
        hidden_size: 2048,
    }
}

fn make_corpus(
    head_dim: usize,
    num_layers: u32,
    num_kv_heads: u32,
    n_tokens: usize,
) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(0xDEAD_BEEF);
    let vec_len = num_layers as usize * num_kv_heads as usize * head_dim * 2;
    (0..n_tokens)
        .map(|i| {
            let v: Vec<f32> = (0..vec_len).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (format!("tok_{i}"), v)
        })
        .collect()
}

fn main() {
    let mut num_queries = 64usize;
    let mut top_k = 64usize;
    let mut pool_tokens = 256usize;
    let mut output: Option<String> = None;

    let args: Vec<String> = std::env::args().collect();
    let mut i = 1;
    while i < args.len() {
        match args[i].as_str() {
            "--num-queries" => {
                num_queries = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(64);
                i += 2;
            }
            "--top-k" => {
                top_k = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(64);
                i += 2;
            }
            "--pool-tokens" => {
                pool_tokens = args.get(i + 1).and_then(|s| s.parse().ok()).unwrap_or(256);
                i += 2;
            }
            "--output" => {
                output = args.get(i + 1).cloned();
                i += 2;
            }
            "--help" | "-h" => {
                println!("Usage: compressed_attention_bench [--num-queries N] [--top-k K] [--pool-tokens N] [--output FILE]");
                return;
            }
            _ => {
                i += 1;
            }
        }
    }

    let shape = make_shape();
    let head_dim = shape.head_dim;
    let num_layers = shape.num_layers;
    let num_kv_heads = shape.num_kv_heads;

    println!(
        "Building pool with {pool_tokens} tokens, {num_layers} layers, {num_kv_heads} kv heads, head_dim={head_dim}..."
    );

    let corpus = make_corpus(head_dim, num_layers, num_kv_heads, pool_tokens);
    let pool_start = Instant::now();
    let (pool, pool_receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let pool_build_ms = pool_start.elapsed().as_millis();
    println!(
        "Pool built in {pool_build_ms} ms, ratio={:.2}x, size={} KB",
        pool_receipt.compression_ratio,
        pool_receipt.pool_size_bytes / 1024
    );

    let (shell, shell_receipt) = shell::materialize_shell(&pool, "bench_agent", &[], 42).unwrap();
    println!(
        "Shell materialized: {} unique tokens",
        shell_receipt.num_unique_tokens
    );

    let turbo_config = poly_kv::policy::TurboConfig::default_8bit();

    let mut rng = ChaCha8Rng::seed_from_u64(0x5E21_7EED);
    let queries: Vec<Vec<f32>> = (0..num_queries)
        .map(|_| (0..head_dim).map(|_| rng.gen_range(-1.0..1.0)).collect())
        .collect();

    let mut total_decoded = 0usize;
    let mut total_scored = 0usize;

    println!(
        "Running {num_queries} compressed top-k attention queries on layer 0 (top_k={top_k})..."
    );
    let bench_start = Instant::now();
    for (qi, query) in queries.iter().enumerate() {
        let (hits, receipt) = shell
            .attention_topk_compressed(&pool, 0, query, top_k, &turbo_config)
            .unwrap();
        total_decoded += receipt.decoded_keys;
        total_scored += receipt.candidate_count;
        if qi == 0 {
            println!(
                "  Query 0: {} hits, {} candidates scored, {} decoded keys",
                hits.len(),
                receipt.candidate_count,
                receipt.decoded_keys
            );
        }
    }
    let total_latency = bench_start.elapsed();

    let per_query_us = total_latency.as_micros() as f64 / num_queries as f64;
    let qps = num_queries as f64 / total_latency.as_secs_f64();

    println!(
        "Done: {} queries in {} ms ({:.1} us/query, {:.1} queries/s)",
        num_queries,
        total_latency.as_millis(),
        per_query_us,
        qps
    );
    println!("Total scored: {total_scored}, total decoded: {total_decoded}");

    let bench_receipt = BenchReceipt {
        schema_version: "compressed_attention_bench_v1".to_string(),
        num_queries,
        top_k,
        head_dim,
        num_layers,
        num_kv_heads,
        pool_tokens,
        shell_tokens: 0,
        total_scored,
        decoded_keys_total: total_decoded,
        total_latency_ms: total_latency.as_millis(),
        per_query_latency_us: per_query_us,
        queries_per_second: qps,
        compression_ratio: pool_receipt.compression_ratio,
        pool_size_bytes: pool_receipt.pool_size_bytes,
    };

    let json = serde_json::to_string_pretty(&bench_receipt).unwrap();
    println!("\n{}", json);

    if let Some(path) = output {
        std::fs::write(&path, &json).unwrap();
        println!("\nReceipt written to {path}");
    }
}
