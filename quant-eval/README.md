# quant-eval

Compression and semantic search evaluation benchmark suite.

`quant-eval` is the **measurement layer** for the governed
compression workspace. It defines three benchmark harnesses:

- **`admissibility`** — when is a codec admissible for a given
  use case (admissibility classes: Exact, Lossless,
  Approximate).
- **`compression`** — what's the byte cost of encoding a
  corpus with a given codec profile (raw bytes, encoded bytes,
  ratio).
- **`semantic`** — does the encoded representation preserve
  retrieval quality (cosine similarity, NDCG@k, mean rank
  drift, exact-rerank recovery rate).

The output of every benchmark is a typed `EvalReport` that
matches the `quant_codec_core::EvalReport` shape.

## What's in the box

### `admissibility.rs` (346 lines)

Tests that a codec correctly classifies itself as
`Admissibility::Exact`, `Admissibility::Lossless`, or
`Admissibility::Approximate`. The classification is
**enforced** — a codec that says "Exact" but is lossy gets
rejected by the harness, not just warned.

### `compression.rs` (460 lines)

Measures the raw byte cost of encoding a corpus:

- **Raw bytes** — the corpus in f32, summed.
- **Encoded bytes** — the corpus in the codec's wire format,
  summed.
- **Ratio** — raw / encoded.
- **Per-block ratio** — min, max, mean, p50, p95, p99.
- **Theoretical ratio** — the codec's expected ratio for the
  given profile.

The harness catches: codecs that claim a ratio better than
theoretical, codecs that have outliers, codecs that have
non-uniform block sizes.

### `semantic.rs` (398 lines)

Measures retrieval quality:

- **NDCG@k** — Normalized Discounted Cumulative Gain at
  top-k. 1.0 = perfect ranking.
- **Mean rank drift** — average position change between
  raw-vector top-k and encoded-vector top-k.
- **Cosine similarity** — inner product agreement.
- **Exact-rerank recovery rate** — fraction of queries where
  the encoded top-k, after exact rerank against the raw
  vectors, equals the raw top-k.

The harness uses synthetic corpora with known ground truth,
so the metrics are reproducible and not workload-dependent.

## Quick Start

```rust
use quant_eval::compression::measure_compression;
use quant_eval::semantic::measure_retrieval;
use quant_eval::EvalReport;

fn main() {
    // Build a synthetic corpus.
    let corpus: Vec<Vec<f32>> = /* ... */;

    // Measure the compression ratio.
    let report: EvalReport = measure_compression(&corpus, /* profile */);
    assert!(report.passed);
    assert!(report.cosine_similarity.unwrap_or(0.0) > 0.99);
}
```

Run it: `cargo test --release --test integration`.

## Test coverage

- **3 internal benchmarks** in `src/benchmarks/`
  (admissibility.rs, compression.rs, semantic.rs).
- **1 integration test** in `tests/integration.rs` (98 lines)
  that runs the full evaluation pipeline on a synthetic
  corpus and validates the report.
- `cargo test` clean, `cargo clippy --all-targets -- -D warnings` clean.

## MSRV

Rust 1.75 (2021 edition). Stable features only.

## Dependencies

- `serde` (with `derive`).
- `thiserror`.
- `serde_json`.
- `chrono` (for receipt timestamps).
- `sha2` (for digest computation).
- `blake3` (for content-addressable digests).
- `tempfile` (dev only, for test fixtures).

Zero platform-specific code, zero FFI, zero async.

## License

MIT. See `LICENSE-MIT` for the full text.

## Changelog

See `CHANGELOG.md` for the release history.

## Where it's used

`quant-eval` is the measurement layer for:

- `poly-kv` — the pool build emits `EvalReport`s for the
  shared pool and the per-agent shell.
- `fib-quant` — every codec profile change is benchmarked
  with `quant-eval` before being released.
- `turbo-quant` — the experimental vector sidecar is
  benchmarked the same way.
- `quant-governor` — the policy layer uses the
  `Admissibility` classifications produced by this crate.

Any system that needs to **measure** a compression codec
(ratio, quality, admissibility) can adopt `quant-eval`
directly.
