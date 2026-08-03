# fib-quant

[![Crates.io](https://img.shields.io/crates/v/fib-quant.svg)](https://crates.io/crates/fib-quant)
[![Docs.rs](https://docs.rs/fib-quant/badge.svg)](https://docs.rs/fib-quant)
[![License](https://img.shields.io/badge/license-Apache--2.0-blue.svg)](LICENSE)

**Experimental CPU-first vector and KV artifact codec.**

This crate is an alpha research implementation. It does not claim production KV-cache serving, paper-benchmark reproduction, universal compression ratios, or retrieval guarantees.

> Based on Namyoon Lee and Yongjune Kim, "FibQuant: Universal Vector Quantization for Random-Access KV-Cache Compression", arXiv:2605.11478.

fib-quant provides CPU reference paths for spherical-block quantization, deterministic profiles, authenticated receipts, and an optional typed KV artifact contract. Size and quality outcomes depend on the tensor shape, profile, fallback policy, and metadata overhead.

## Where It Fits

fib-quant is the **cold-tier codec** in poly-kv. It handles shared context that's large, stable, and accessed by many agents:

```
┌──────────────────────────────────┐
│    CPU reference KV artifacts     │  ← optional `kv` feature
│    Typed shape, profile, receipt   │
│    No production serving claim     │
└──────────┬──────────┬────────────┘
           │          │
      ┌────▼───┐ ┌───▼────┐
      │ Agent0 │ │ Agent1 │  ...  ← turbo-quant hot tier
      └────────┘ └────────┘
```

## Local accounting smoke fixture (2026-08-01)

This is one small synthetic KV tensor, not a representative benchmark or a paper reproduction:

| Representation | Total bytes |
|---|---:|
| Raw f32 values | 16 |
| JSON logical envelope | 4,256 |
| Framed binary wire | 2,339 |
| Framed-wire header | 44 |

The framed wire was smaller than JSON for this fixture, but larger than raw f32 because profile, codebook, page, and receipt metadata dominate at this size. Do not extrapolate these values to production workloads.

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

## Framed binary wire

The optional `kv` feature owns a versioned `FQKV` frame with explicit flags, checked payload length, BLAKE3 payload digest, bounded deserialization, and exact trailing-byte rejection. The payload contains the typed shape, layout, profile, pages, and receipts. The wire is an experimental local contract; compatibility migrations and durable restart/recovery are not claimed.

## Integration with poly-kv

PolyKV can opt into the `fibquant-adapter` feature and select `FibQuantValueCodec` through `PoolBuilder::value_codec`. The integration is CPU-only and experimental. It requires an explicit finite value-quality budget and retains exact fallback separately; it does not auto-select compression or claim persistence/recovery.

## What's Implemented

- Fixed-rate block quantization
- Deterministic stored rotations (Fibonacci/random/Roberts-Kronecker)
- Spherical-Beta source sampling for codebook initialization
- Lloyd-Max refinement with non-worsening fallback
- Fixed-width bit packing for indices
- Fail-closed digests and compression receipts
- Versioned `FQKV` framed wire under the optional `kv` feature
- Optional typed KV-cache contracts

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
