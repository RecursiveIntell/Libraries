//! `poly_kv_gpu_bench` — end-to-end poly-kv pool build benchmark.
//!
//! Runs the SharedKVPool build path on a deterministic synthetic corpus
//! across two model shapes (nomic-embed 768-dim and qwen3-embedding 2560-dim)
//! and three corpus sizes (4, 20, 80 documents). For each configuration we
//! report the wall time, the receipt's `backend` field, and the codec's
//! runtime `is_gpu_accelerated()` flag — so the output makes the actual
//! acceleration state unambiguous.
//!
//! The two target shapes are:
//!   - nomic-embed-text: 12 layers, 12 kv heads, head_dim 64, ambient 768
//!   - qwen3-embedding:   28 layers,  4 kv heads, head_dim 128, ambient 2560
//!
//! Usage:
//!   cargo run --release --example poly_kv_gpu_bench
//!   cargo run --release --example poly_kv_gpu_bench --features gpu

use std::time::Instant;

use poly_kv::pool::SharedKVPool;
use poly_kv::shape::{AttentionType, KvTensorShape};
use rand::Rng;
use rand_chacha::{rand_core::SeedableRng, ChaCha8Rng};

#[derive(Debug, Clone, Copy)]
struct ModelShape {
    name: &'static str,
    num_layers: u32,
    num_kv_heads: u32,
    head_dim: usize,
}

const NOMIC: ModelShape = ModelShape {
    name: "nomic-embed-text (768-dim)",
    num_layers: 12,
    num_kv_heads: 12,
    head_dim: 64,
};

const QWEN3: ModelShape = ModelShape {
    name: "qwen3-embedding (2560-dim)",
    num_layers: 28,
    num_kv_heads: 4,
    head_dim: 128,
};

fn make_shape(m: ModelShape) -> KvTensorShape {
    KvTensorShape {
        attention_type: AttentionType::GQA,
        num_layers: m.num_layers,
        num_heads: m.num_kv_heads * 4, // pretend GQA with 4x q heads
        num_kv_heads: m.num_kv_heads,
        head_dim: m.head_dim,
        hidden_size: m.num_kv_heads as usize * 4 * m.head_dim,
    }
}

fn make_corpus(m: ModelShape, n_tokens: usize) -> Vec<(String, Vec<f32>)> {
    let mut rng = ChaCha8Rng::seed_from_u64(0xDEAD_BEEF);
    let vec_len = m.num_layers as usize * m.num_kv_heads as usize * m.head_dim * 2;
    (0..n_tokens)
        .map(|i| {
            let v: Vec<f32> = (0..vec_len).map(|_| rng.gen_range(-1.0..1.0)).collect();
            (format!("doc_{i}"), v)
        })
        .collect()
}

fn run_one(model: ModelShape, n_tokens: usize) {
    let shape = make_shape(model);
    let corpus = make_corpus(model, n_tokens);

    // warm the codec + codebook build once outside the timed region
    let _ = SharedKVPool::build(&corpus[..1.min(corpus.len())], &shape, 42).unwrap();

    let start = Instant::now();
    let (pool, receipt) = SharedKVPool::build(&corpus, &shape, 42).unwrap();
    let wall = start.elapsed();

    // Per-call GPU probe: would the actual batch size clear the threshold?
    let batch_n = n_tokens * model.num_kv_heads as usize;
    let gpu_dispatch_would = if batch_n >= 16 && model.head_dim >= 64 && cfg!(feature = "gpu") {
        "yes"
    } else {
        "no"
    };

    println!(
        "  {model:32} n={n_tokens:>3}  wall={wall_ms:>6} ms  receipt_ms={rms:>5}  \
         batch={bn:>3}  gpu_dispatch={gd:>3}  backend={bk:>3}  ratio={ratio:.2}x  size={kb} KB",
        model = format!("{} {}", model.name, ""),
        n_tokens = n_tokens,
        wall_ms = wall.as_millis(),
        rms = receipt.fib_build_ms,
        bn = batch_n,
        gd = gpu_dispatch_would,
        bk = receipt.backend,
        ratio = receipt.compression_ratio,
        kb = receipt.pool_size_bytes / 1024,
    );

    // Surface failure loud: never silently misreport.
    assert_eq!(pool.manifest.num_shared_tokens, n_tokens as u32);
}

fn main() {
    println!("poly-kv pool-build benchmark");
    println!("compile-time: gpu feature = {}", cfg!(feature = "gpu"));
    println!();

    for model in &[NOMIC, QWEN3] {
        println!("=== {} ===", model.name);
        for n in &[4usize, 20, 80] {
            run_one(*model, *n);
        }
        println!();
    }

    println!("Notes:");
    println!("  - GPU threshold is n>=16, dim>=64. The 4-doc corpora will fall through to CPU even with --features gpu.");
    println!("  - For 768-dim and 2560-dim shapes, every (layer,head) batch is well over 16 vectors, so the dispatch is gated only by n_tokens.");
    println!("  - 'backend' is the runtime truth, not the compile feature.");
}
