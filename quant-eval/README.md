# quant-eval

`quant-eval` is a Rust crate for compression and semantic-search evaluation scaffolding.

Current status: **prototype benchmark harnesses with synthetic data and simulated compression paths**. It is useful as a typed measurement substrate, but it does **not yet run real codec implementations** and should not be described as proving codec quality, production readiness, or workload-level benchmark performance.

## What is implemented

### Admissibility harness

File: `src/benchmarks/admissibility.rs` (346 lines)

Implemented:

- `CodecProfile` presets: `fast`, `balanced`, and `high_compression`.
- `AdmissibilityTest` over caller-provided `TestSetEntry` values.
- Standard synthetic test vectors for zero, unit, and deterministic pseudo-random vectors.
- Summary counts per profile.

Important limitation:

- The current harness simulates codec behavior from `should_succeed` and profile quality targets. It does not call a codec trait or encode/decode real payloads yet.

### Compression benchmark scaffold

File: `src/benchmarks/compression.rs` (460 lines)

Implemented:

- Deterministic synthetic corpus and query generation.
- Raw nearest-neighbor computation with cosine similarity.
- Recall@K and MRR calculations over exact-vs-estimated result sets.
- Cosine-similarity-style summary statistics over result overlap.

Important limitations:

- Compression is simulated by returning the exact result set.
- The benchmark does not currently measure encoded byte size, compression ratio, per-block ratios, wire formats, or codec theoretical ratios.
- The reported cosine-similarity statistics are derived from top-K overlap, not from comparing raw vectors to decoded vectors.

### Semantic memory benchmark scaffold

File: `src/benchmarks/semantic.rs` (398 lines)

Implemented:

- Deterministic synthetic index and query generation.
- Raw search baseline using cosine similarity.
- Synthetic relevance judgments from the raw top-K results.
- Precision@K, Recall@K, NDCG@K, MAP, and degradation-ratio calculations.

Important limitation:

- Compressed search currently delegates to raw search, so degradation is simulated/minimal by construction. It is not evidence of real codec preservation quality.

### Benchmark receipts

Files: `src/receipt.rs`, `src/fingerprint.rs`

Implemented:

- `BenchmarkReceipt` with timestamp, commit hash, machine fingerprint string, result list, and optional note.
- `BenchmarkResult` timing fields.
- Receipt JSON serialization/deserialization.
- Receipt hash and receipt diff helpers.
- `MachineFingerprint` derived from available host/user/arch/OS/CPU-count/machine-id inputs.

## Public API

The crate currently re-exports:

- `AdmissibilityTest`
- `CodecProfile`
- `CompressionBenchmark`
- `CompressionBenchmarkConfig`
- `SemanticMemoryBenchmark`
- `SemanticMemoryConfig`
- `QuantEvalError`
- `MachineFingerprint`
- `BenchmarkReceipt`
- `BenchmarkResult`
- `ReceiptDiff`

## Quick start

```rust
use quant_eval::{CompressionBenchmark, CompressionBenchmarkConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let benchmark = CompressionBenchmark::with_config(CompressionBenchmarkConfig {
        dim: 64,
        db_size: 100,
        queries: 10,
        seed: 42,
        top_k: 5,
        iterations: 10,
    });

    let report = benchmark.run()?;
    println!("recall@{} = {}", report.top_k, report.recall_at_k);
    Ok(())
}
```

Run tests:

```bash
cargo test -p quant-eval
cargo clippy -p quant-eval --all-targets -- -D warnings
```

## Test coverage

Current verified test surface:

- 19 unit tests in the library modules.
- 5 integration tests in `tests/integration.rs`.
- `cargo test` passes.
- `cargo clippy --all-targets -- -D warnings` passes.

## MSRV

Rust 1.75, 2021 edition.

## Dependencies

Runtime dependencies:

- `serde`
- `serde_json`
- `thiserror`
- `chrono`
- `sha2`
- `blake3`

Dev dependency:

- `tempfile`

The crate currently contains no platform-specific code, FFI, or async runtime dependency.

## What this README does not claim

This README intentionally does **not** claim that `quant-eval`:

- proves real codec admissibility;
- measures actual compression ratios;
- enforces codec-reported admissibility classes;
- integrates with `poly-kv`, `fib-quant`, `turbo-quant`, or `quant-governor`;
- emits `quant_codec_core::EvalReport` values;
- catches theoretical-ratio violations;
- validates production performance.

Those are reasonable next targets, but they need implementation evidence before they become public claims.

## Next implementation targets

1. Add a codec evaluation trait or adapter layer so the harness can call real encode/decode implementations.
2. Replace simulated compression paths with actual compressed/decompressed vector comparisons.
3. Add encoded-byte accounting and compression-ratio reports.
4. Add a typed report shape that either depends on `quant-codec-core` or clearly defines a local `quant-eval` report schema.
5. Add cross-crate integration tests once real codec adapters exist.

## License

MIT. See `LICENSE-MIT` for details.
