# turbo-quant

[![Crates.io](https://img.shields.io/crates/v/turbo-quant.svg)](https://crates.io/crates/turbo-quant)
[![Docs.rs](https://docs.rs/turbo-quant/badge.svg)](https://docs.rs/turbo-quant)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

**The hot tier. Near-lossless. 17ms per agent. Cosine 0.9996.**

turbo-quant compresses vectors via polar-coordinate quantization with an optional QJL residual sketch. It's not the highest compression in the stack — that's fib-quant at 50×. What turbo-quant gives you is reconstruction so close to the original that rankings barely shift (rank drift 0.03).

## Where It Fits

turbo-quant is the **hot-tier codec** in poly-kv. It handles agent-private context that needs to stay sharp:

```
┌──────────────────────────────────┐
│  SHARED POOL — fib-quant (50×)   │
└──────┬──────────┬──────────┬─────┘
       │          │          │
  ┌────▼─────┐┌───▼────┐┌────▼─────┐
  │ Agent 0  ││Agent 1 ││ Agent 9  │  ← you are here
  │ turbo 8b ││turbo 8b││ turbo 8b │
  │cos .9996 ││cos.9996││ cos .9996 │
  │  17ms    ││  17ms  ││   17ms   │
  └──────────┘└────────┘└──────────┘
```

This is where conversation turns, tool outputs, and unique agent state live. High fidelity, low latency.

## Benchmarked (2026-06-01)

| Metric | Result |
|---|---|
| Recall@1 (8 queries) | **1.000** |
| Recall@1 (10 agents, shells) | **1.000** — all 10 |
| Cosine fidelity (768-dim) | **0.9996** |
| Rank drift vs exact scan | **0.03** |
| Shell materialize (12 docs) | 17ms avg per agent |
| Cross-agent interference | **0/90 pairs** |
| JSON compression (single 768d vector) | 0.6× (JSON overhead > raw f32) |
| Projected binary compression | ~7× |

**Key insight:** 0.9996 cosine fidelity with 0.03 rank drift. For all practical purposes, the compressed representation preserves the exact ranking. When your hot tier needs to be trustworthy, this is what you use.

## Quick Start

```rust
use turbo_quant::TurboQuantizer;

fn main() -> turbo_quant::Result<()> {
    let dim = 768;
    let quantizer = TurboQuantizer::new(dim, 8, 32, 42)?;

    let vector = vec![0.1_f32; dim];
    let query = vec![0.1_f32; dim];

    // Encode & decode
    let code = quantizer.encode(&vector)?;
    let decoded = quantizer.decode_approximate(&code)?;

    // Approximate inner product without full decompression
    let score = quantizer.inner_product_estimate(&code, &query)?;
    println!("score: {score:.4}");

    Ok(())
}
```

## Two-Tier with poly-kv

```rust
use poly_kv::{SharedKVPool, KvTensorShape, AttentionType};

// Shared pool built once (fib-quant cold tier)
let (pool, _) = SharedKVPool::build(&shared_corpus, &shape, 42)?;

// Each agent's shell is turbo-quant — 17ms, near-lossless
let (shell, receipt) = pool.materialize_shell("agent_7", &agent_tokens, 43)?;
// receipt.shell_digest is deterministic — same tokens + same seed = same digest
```

## How It Works

1. **Normalize** the vector to unit length
2. **Rotate** with a fast Hadamard transform (deterministic, seeded)
3. **Polar encode** — compress angles into discrete bins (8-bit by default)
4. **QJL sketch** — quantized Johnson-Lindenstrauss residual for extra precision
5. **Pack** into a compact binary representation (`PackedTurboCode`)
6. **Search** via approximate inner product without full decompression

The key property: **data-oblivious construction.** No k-means. No trained codebook. The entire quantizer is reconstructed from four integers: `(dim, bits, projections, seed)`.

## Sidecar Search

turbo-quant includes a sidecar index for approximate candidate retrieval:

```rust
use turbo_quant::{SearchOptions, TurboQuantizer, TurboSidecarIndex};

let mut index = TurboSidecarIndex::new(quantizer);
index.add("doc-a", &vec![0.1; 768], None)?;
index.add("doc-b", &vec![0.2; 768], None)?;

let (candidates, receipt) = index.search(
    &query,
    SearchOptions { top_k: 10, oversample: 4 },
)?;

// receipt.exact_rerank_required is always true — this is a sidecar, not ground truth
assert!(receipt.exact_rerank_required);
```

## KV-Cache Shadow Mode

For measuring compression quality before promoting to production:

```rust
use turbo_quant::{KvCacheCompressor, KvQuantPolicy, KvRuntimeConfig};

let mut cache = KvCacheCompressor::new_runtime(KvRuntimeConfig {
    head_dim: 768,
    key_policy: KvQuantPolicy::quantized(8, 32),
    value_policy: KvQuantPolicy::Exact,
    seed: 42,
    keep_exact_shadow: true,  // ← critical: measure before trusting
})?;

cache.compress_token(&key_vector, &value_vector)?;

// Compare approximate vs exact attention scores
let shadow = cache.shadow_scores(&query)?;
// Promote only after local benchmarks pass your quality gate
```

## Choosing Parameters

| Use case | bits | projections | Compression | Fidelity |
|---|---|---|---|---|
| Hot tier (agent shells) | 8 | 32 | ~8× binary | cos 0.9996 |
| Semantic search | 8 | dim/4 | ~8× binary | workload-dependent |
| Maximum compression | 3-4 | dim/16 | ~15-20× binary | expect quality loss |

## What This Crate Is

- Deterministic sidecar codec (reconstructible from four integers)
- PolarQuant + QJL compression with inner product estimation
- Sidecar index with explicit approximate-only receipts
- KV-cache shadow mode for quality measurement
- Source-compatible upgrade from 0.1.x

## What This Crate Is Not

- Not a canonical vector store — keep your exact vectors
- Not reversible — decoded vectors are approximations
- Not production-guaranteed — requires workload-specific benchmark gates
- Not a replacement for exact reranking — `receipt.exact_rerank_required` is always true

## Install

```toml
[dependencies]
turbo-quant = "0.2"
```

MSRV: 1.75

## Testing

```bash
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

## License

MIT
