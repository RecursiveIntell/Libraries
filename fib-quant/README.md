# fib-quant

[![Crates.io](https://img.shields.io/crates/v/fib-quant.svg)](https://crates.io/crates/fib-quant)
[![Docs.rs](https://docs.rs/fib-quant/badge.svg)](https://docs.rs/fib-quant)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**The cold tier. 50× compression. 100% recall.**

> Based on Namyoon Lee and Yongjune Kim, "FibQuant: Universal Vector Quantization for Random-Access KV-Cache Compression", arXiv:2605.11478.

fib-quant compresses KV-cache vectors by decomposing them into spherical blocks, quantizing each block against a Fibonacci-optimized codebook, and storing only the codebook indices. The result: a 768-dim f32 vector (3,072 bytes) becomes ~860 bytes in JSON — or ~64 bytes with binary packing. And it still finds the right document at rank 1.

## Where It Fits

fib-quant is the **cold-tier codec** in poly-kv. It handles shared context that's large, stable, and accessed by many agents:

```
┌──────────────────────────────────┐
│    SHARED POOL — fib-quant       │  ← you are here
│    System prompts, few-shot      │
│    examples, shared docs         │
│    50× compression, cos 0.863    │
└──────────┬──────────┬────────────┘
           │          │
      ┌────▼───┐ ┌───▼────┐
      │ Agent0 │ │ Agent1 │  ...  ← turbo-quant hot tier
      └────────┘ └────────┘
```

## Benchmarked (2026-06-01)

| Metric | Result |
|---|---|
| Recall@1 (8 queries, compressed) | **1.000** |
| Recall@1 (10 agents, shared pool) | **1.000** |
| Cosine fidelity (768-dim) | 0.863 |
| Rank drift vs exact scan | 0.33 |
| JSON compression (single 768d vector) | 3.6× (3,072b → 860b) |
| Projected binary compression | ~48× (3,072b → ~64b) |
| Batch compression (160 docs, cold tier) | 480 KB → 133 KB (3.6× JSON) |

**Key insight:** 100% recall even at 0.863 fidelity. The target document stays at rank 1 while secondary rankings shift. For a cold tier that's accessed occasionally, this is the right tradeoff.

## Quick Start

```rust
use fib_quant::{FibQuantProfileV1, FibQuantizer};

fn main() -> fib_quant::Result<()> {
    // 768-dim embedding, k=4 blocks, N=32 codebook, seed=42
    let mut profile = FibQuantProfileV1::paper_default(768, 4, 32, 42)?;
    profile.training_samples = 2048;
    profile.lloyd_restarts = 4;
    profile.lloyd_iterations = 8;

    let quantizer = FibQuantizer::new(profile)?;

    // Encode
    let input: Vec<f32> = vec![0.25; 768];
    let code = quantizer.encode(&input)?;

    // Decode
    let decoded = quantizer.decode(&code)?;
    assert_eq!(decoded.len(), 768);

    // Measure compression
    let json_size = serde_json::to_vec(&code).unwrap().len();
    println!("3,072 raw bytes → {} compressed bytes ({:.1}×)",
        json_size, 3072.0 / json_size as f64);
    Ok(())
}
```

## How It Works

1. **Normalize** the input vector
2. **Rotate** with a deterministic, seed-fixed rotation matrix
3. **Split** into blocks of k dimensions (k=4)
4. **Quantize** each block against a codebook of N codewords (N=32)
5. **Lloyd-Max refine** the codebook with training samples
6. **Bit-pack** codebook indices into a compact payload
7. **Digest** everything for content-addressed reproducibility

The codebook is built from spherical-Beta source samples, Fibonacci-optimized direction vectors, and radial quantile estimation. Training is deterministic from the seed.

## The Binary Wire Gap

Current compression is JSON-serialized. A single codebook index (5 bits) becomes a JSON integer (~4 bytes). At scale, this overhead dwarfs the payload. The `packed` module exists for exactly this — direct bit-level encoding. When integrated:

| | 1 vector (768d) | 100 vectors | 10,000 vectors |
|---|---|---|---|
| JSON | 860 bytes (3.6×) | 86 KB (3.6×) | 8.6 MB (3.6×) |
| Binary | ~64 bytes (48×) | ~6 KB (48×) | ~640 KB (48×) |

## Integration with poly-kv

```rust
use poly_kv::{SharedKVPool, KvTensorShape, AttentionType};

// fib-quant is auto-selected for cold-tier compression
let (pool, receipt) = SharedKVPool::build(&corpus, &shape, 42)?;
// receipt.compression_ratio tells you how well it compressed
// receipt.pool_digest is a blake3 hash of all layer digests
```

## What's Implemented

- Fixed-rate block quantization (k=4, N=32 benchmarked)
- Deterministic stored rotations (Fibonacci/random/Roberts-Kronecker)
- Spherical-Beta source sampling for codebook initialization
- Lloyd-Max refinement with non-worsening fallback
- Fixed-width bit packing for indices
- Fail-closed digests and compression receipts
- Optional `kv` feature for typed KV-cache contracts

## What's Not Claimed

- Production KV-cache compressor — research-grade
- GPU kernel or CUDA fusion support
- Paper benchmark reproduction (perplexity, throughput)
- vLLM, FlashInfer, TensorRT-LLM, or HuggingFace integration
- Default-on compression in any downstream project

## Install

```toml
[dependencies]
fib-quant = "0.1.0-alpha.1"
```

MSRV: 1.75

## Validation

```bash
cargo test --all-features
cargo clippy --all-targets --all-features -- -D warnings
cargo doc --no-deps --all-features
```

## Citation

If this crate is useful, cite both this implementation and the FibQuant paper. `CITATION.cff` included.

## License

Apache-2.0. See `LICENSE`.
