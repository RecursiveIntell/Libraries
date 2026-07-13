# compressed-scorer

Codec-agnostic compressed-domain scoring for retrieval and attention.

The crate defines one shared trait, `CompressedScorer`, plus adapters for `fib-quant` and `turbo-quant`. The point is simple: prepare a query once, score compressed vectors directly, and decode only the small top-k set when exact verification or value aggregation is required.

## Canonical role in the RecursiveIntell stack

`compressed-scorer` is the compression substrate. Codec crates such as `turbo-quant`, `fib-quant`, and experimental `hyperquant` should plug into this trait instead of wiring directly into product search paths. Runtime users should treat compressed scores as candidate-generation evidence, then exact-rerank or exact-decode the selected top-k.

Current receipt-backed product lane: PerDim/int8-style compressed-domain candidate scoring. HyperQuant remains an experimental lattice backend candidate until it implements this trait and beats or meaningfully differs from baselines under the same corpus receipts.

## Why this exists

Normal compressed retrieval does this:

1. load compressed vector
2. decompress to f32
3. dot/cosine against query
4. repeat for every candidate

`compressed-scorer` does this instead:

1. prepare query once
2. score compressed payloads directly
3. rank by approximate compressed-domain score
4. optionally decode top-k only

That is the shared seam for:

- semantic-memory compressed candidate generation
- scr-runtime-compression adapter routing
- ESP32-S3 / embedded attention caches where PSRAM reads dominate

## Supported codecs

- **fib-quant**: Gram-table lookup. `G[i,j] = <codeword_i, codeword_j>`.
  Precomputed at construction time. O(1) per scored vector.
- **turbo-quant**: Polar-coordinate inner product estimate after seeded
  rotation. Data-oblivious — no trained codebook needed.
- **per-dim**: Asymmetric per-dimension uniform quantization over
  unit-normalized vectors. `prepare_query()` builds a query-side contribution
  lookup table, then `score_prepared()` sums table entries indexed by document
  codes. This is ADC-style compressed-domain candidate scoring, not a
  SIMD/QuickADC production-speed claim yet.

## Core API

```rust
use compressed_scorer::CompressedScorer;

fn score_all<S: CompressedScorer>(
    scorer: &S,
    query: &[f32],
    compressed: &[S::Compressed],
) -> compressed_scorer::ScorerResult<Vec<f32>> {
    let prepared = scorer.prepare_query(query)?;
    compressed
        .iter()
        .map(|code| scorer.score_prepared(&prepared, code))
        .collect()
}
```

## AttentionCache

`AttentionCache<S>` is a one-head compressed attention cache over any `CompressedScorer` implementation.

It computes logits from compressed keys without decompression, softmaxes them, then decodes only the selected top-k compressed values:

```rust,ignore
let mut cache = AttentionCache::new(scorer);
cache.push_compressed(compressed_key, compressed_value);
let output = cache.attention_topk(&query, 4)?;
assert!(output.decompression_count <= 4);
```

This is the ESP32-S3-facing API: small trait surface, no database types, no semantic-memory dependency, and no std requirement when built with `--no-default-features --features no_std`.

## Feature flags

| Feature | Default | Meaning |
|---|---:|---|
| `fib` | no | Enables `FibScorerAdapter` over `fib-quant` Gram-table scoring. |
| `turbo` | yes | Enables `TurboScorerAdapter` over `turbo-quant` prepared-query scoring. |
| `c-kernels` | yes | Uses the measured native C PerDim scoring kernel; disable for the pure-Rust fallback and toolchain-free cross builds. |
| `no_std` | no | Builds the trait, candidate list, and `AttentionCache` with `alloc` only. |

For embedded/no_std builds, disable codec features unless the codec dependency itself supports the target:

```bash
cargo check -p compressed-scorer --no-default-features --features no_std --target riscv32imc-unknown-none-elf
cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc
```

## Verification receipts from this integration pass

- `cargo test -p compressed-scorer --all-features` -> 24 passed; 1 doc-test ignored.
- `cargo test -p compressed-scorer --no-default-features --features no_std` -> 19 passed; 1 doc-test ignored.
- `cargo check -p compressed-scorer --no-default-features --features no_std --target riscv32imc-unknown-none-elf` -> passed.
- `cargo +esp check -p compressed-scorer --no-default-features --features no_std --target xtensa-esp32s3-none-elf -Z build-std=core,alloc` -> passed.

## Design boundaries

- This crate does not own codec truth; it wraps codec implementations.
- This crate does not own semantic-memory search semantics or raw-vector authority.
- HyperQuant should be a backend behind this trait, not a direct semantic-memory default path.
- Approximate scores are candidate-generation evidence unless the caller explicitly chooses compressed-only mode.
- Exact rerank/decode remains a caller policy decision.
