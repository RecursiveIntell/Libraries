//! End-to-end encode/decode benchmark for turbo-quant KV cache path.
//! Measures full encode + decode + shadow attention with realistic dimensions.
use std::hint::black_box;
use std::time::Instant;

use turbo_quant::{KvCacheCompressor, KvQuantPolicy, KvRuntimeConfig, TurboQuantizer};

fn main() {
    println!("=== turbo-quant end-to-end benchmark (release) ===\n");

    // Test 1: KV cache compress/decompress at various dimensions
    for dim in [64, 128, 256, 512] {
        let config = KvRuntimeConfig {
            head_dim: dim,
            key_policy: KvQuantPolicy::quantized(8, 16),
            value_policy: KvQuantPolicy::Exact,
            seed: 42,
            keep_exact_shadow: true,
        };
        let mut cache = KvCacheCompressor::new_runtime(config).unwrap();

        // Generate 64 tokens worth of K/V
        let tokens = 64;
        let keys: Vec<Vec<f32>> = (0..tokens)
            .map(|t| {
                (0..dim)
                    .map(|i| ((t * dim + i) as f32 * 0.017).sin())
                    .collect()
            })
            .collect();
        let values: Vec<Vec<f32>> = (0..tokens)
            .map(|t| {
                (0..dim)
                    .map(|i| ((t * dim + i) as f32 * 0.019).cos())
                    .collect()
            })
            .collect();

        // Populate the cache whose shadow-attention path is measured below.
        // The encode benchmark intentionally creates independent caches, but
        // those temporary values are not the attention workload.
        for (key, value) in keys.iter().zip(values.iter()) {
            cache.compress_token(key, value).unwrap();
        }
        assert_eq!(
            cache.len(),
            tokens,
            "shadow-attention workload must contain every token"
        );

        // Warmup
        for _ in 0..5 {
            let mut c = KvCacheCompressor::new_runtime(KvRuntimeConfig {
                head_dim: dim,
                key_policy: KvQuantPolicy::quantized(8, 16),
                value_policy: KvQuantPolicy::Exact,
                seed: 42,
                keep_exact_shadow: true,
            })
            .unwrap();
            for (k, v) in keys.iter().zip(values.iter()) {
                c.compress_token(k, v).unwrap();
            }
        }

        // Measure encode
        let iters = 100;
        let start = Instant::now();
        for _ in 0..iters {
            let mut c = KvCacheCompressor::new_runtime(KvRuntimeConfig {
                head_dim: dim,
                key_policy: KvQuantPolicy::quantized(8, 16),
                value_policy: KvQuantPolicy::Exact,
                seed: 42,
                keep_exact_shadow: true,
            })
            .unwrap();
            for (k, v) in keys.iter().zip(values.iter()) {
                c.compress_token(black_box(k), black_box(v)).unwrap();
            }
        }
        let encode_ns = start.elapsed().as_nanos() as f64 / iters as f64;

        // Measure shadow attention
        let query: Vec<f32> = (0..dim).map(|i| (i as f32 * 0.023).sin()).collect();
        for _ in 0..5 {
            black_box(cache.shadow_attention_scores(&query).unwrap());
        }
        let start = Instant::now();
        for _ in 0..iters {
            black_box(cache.shadow_attention_scores(black_box(&query)).unwrap());
        }
        let shadow_ns = start.elapsed().as_nanos() as f64 / iters as f64;

        let ns_per_tok = encode_ns / tokens as f64;
        println!("KV dim={dim:>4} tokens={tokens:>3}: encode {encode_ns:>10.0}ns ({ns_per_tok:.0}ns/tok)  shadow_attn {shadow_ns:>8.0}ns");
    }

    // Test 2: Full encode/decode cycle (PolarQuantizer)
    for dim in [128, 256, 512, 768] {
        let q = TurboQuantizer::new(dim, 8, 32, 42).unwrap();
        let vector: Vec<f32> = (0..dim).map(|i| i as f32 / dim as f32 - 0.5).collect();

        // Warmup
        for _ in 0..10 {
            black_box(q.encode(black_box(&vector)).unwrap());
        }

        let iters = 1000;
        let start = Instant::now();
        for _ in 0..iters {
            black_box(q.encode(black_box(&vector)).unwrap());
        }
        let encode_ns = start.elapsed().as_nanos() as f64 / iters as f64;

        let code = q.encode(&vector).unwrap();
        for _ in 0..10 {
            black_box(q.decode_approximate(black_box(&code)).unwrap());
        }
        let start = Instant::now();
        for _ in 0..iters {
            black_box(q.decode_approximate(black_box(&code)).unwrap());
        }
        let decode_ns = start.elapsed().as_nanos() as f64 / iters as f64;

        // Measure IP estimate (search path)
        let query: Vec<f32> = (0..dim).map(|i| i as f32 / dim as f32 - 0.3_f32).collect();
        let prepared = q.prepare_query(&query).unwrap();
        for _ in 0..10 {
            black_box(
                q.inner_product_estimate_prepared(black_box(&code), black_box(&prepared))
                    .unwrap(),
            );
        }
        let start = Instant::now();
        for _ in 0..iters {
            black_box(
                q.inner_product_estimate_prepared(black_box(&code), black_box(&prepared))
                    .unwrap(),
            );
        }
        let ip_ns = start.elapsed().as_nanos() as f64 / iters as f64;

        println!("Polar dim={dim:>4} bits=8 proj=32: encode {encode_ns:>8.0}ns  decode {decode_ns:>8.0}ns  ip_est {ip_ns:>6.0}ns");
    }

    println!("\n=== done ===");
}
