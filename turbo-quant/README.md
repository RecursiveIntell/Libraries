# turbo-quant

[![Crates.io](https://img.shields.io/crates/v/turbo-quant.svg)](https://crates.io/crates/turbo-quant)
[![Docs.rs](https://docs.rs/turbo-quant/badge.svg)](https://docs.rs/turbo-quant)
[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE-MIT)

**Experimental vector-compression sidecars and approximate scoring for Rust.**

TurboQuant combines polar quantization with an optional QJL residual sketch. The crate also exposes packed representations, wire formats, sidecar search receipts, and KV-cache shadow measurements. Keep exact vectors or another authoritative source for reranking and recovery.

Quality, latency, and byte savings depend on the selected profile and workload. The earlier unqualified agent-shell timings, universal ranking-parity language, and projected compression ratios are not current source-bound guarantees. Measure the complete representation, including metadata and retained exact data.

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

## Integration boundary

The [PolyKV workspace](../poly-kv/README.md) has its own pool and adapter APIs. Its TurboQuant adapter currently returns `UnsupportedAdapter`; the older `SharedKVPool`/`AgentShell` examples are not a supported integration path. Use this crate's codec, index, or KV APIs directly when evaluating TurboQuant.

## How It Works

1. **Normalize** the vector to unit length
2. **Rotate** with the selected deterministic, seeded rotation policy
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

## Choosing parameters

Choose dimensions, bit width, projection count, and rotation policy explicitly, then evaluate the profile against an exact baseline. `TurboQuantizer::new` defaults to polar quantization plus QJL. The explicit constructors also expose mode and rotation selection. Inspect [the constructor validation](src/turbo.rs) and [benchmark harness](benches/turbo_quant_search.rs) before comparing profiles.

More bits or projections also cost bytes and computation. A nominal bit budget is not the serialized size of a complete sidecar or a measured application memory saving.

## What This Crate Is

- Deterministic sidecar codec (reconstructible from four integers)
- PolarQuant + QJL compression with inner product estimation
- Sidecar index with explicit approximate-only receipts
- KV-cache shadow mode for quality measurement
- A checked-in [0.1 API compatibility smoke example](examples/compat_0_1_smoke.rs)

## What This Crate Is Not

- Not a canonical vector store — keep your exact vectors
- Not reversible — decoded vectors are approximations
- Not production-guaranteed — requires workload-specific benchmark gates
- Not a replacement for exact reranking — `receipt.exact_rerank_required` is always true

## Package and source

```toml
[dependencies]
turbo-quant = "0.2"
```

The local manifest is version 0.2.0 and declares Rust 1.75.0. Registry releases and this workspace checkout can differ; use a path dependency when evaluating the exact source in this tree.

## Testing

```bash
cargo test -p turbo-quant --all-targets --all-features
cargo clippy -p turbo-quant --all-targets --all-features -- -D warnings
```

## License

The package manifest declares MIT; see [LICENSE-MIT](LICENSE-MIT).
