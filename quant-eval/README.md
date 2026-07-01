# quant-eval

`quant-eval` is a Rust crate for evidence-first compression and retrieval evaluation. It provides deterministic benchmark scaffolds, typed result shapes, benchmark receipts, RAG fixture metrics, and HyperQuant primitive evaluation before any codec is promoted into a governor or runtime path.

Current status: prototype-to-evidence benchmark substrate. It contains real metric code and deterministic fixtures, but some harnesses still use synthetic data or simulated compression paths. It should not be described as proving workload-level codec quality, production readiness, or model performance until real codec adapters and corpus receipts exist.

![quant-eval evidence pipeline](docs/quant-eval-pipeline.svg)

## What this gives you

`quant-eval` gives compression and retrieval crates a place to produce evidence before integration:

- **Compression benchmark scaffolding** — deterministic synthetic vector corpus, exact nearest-neighbor baseline, recall@K, MRR, and overlap-derived similarity summaries.
- **Semantic-memory search scaffolding** — synthetic index/query generation, precision@K, recall@K, NDCG@K, MAP, and degradation-ratio calculations.
- **Admissibility harness** — profile-oriented checks over deterministic standard vectors.
- **Benchmark receipts** — timestamped receipt structures with machine fingerprint, result list, JSON serialization, hashes, and diffs.
- **RAG fixture metrics** — local recall@K, NDCG@K, and exact-rerank recovery over caller-supplied query/retrieval fixtures.
- **HyperQuant primitive evaluation** — deterministic Z1/A2/D4 evaluation through the published `hyperquant` crate, with mean/max MSE, estimated bytes, rejected-vector counts, receipt counts, and explicit claim boundaries.
- **Governed HyperQuant receipts** — joins `quant-governor` embedding decisions, admission/block rationale, measured HyperQuant fixture metrics, and Q8/Q4/HyperQuant byte baselines into a serializable receipt.
- **HyperQuant retrieval + latency benchmark** — synthetic clustered embedding retrieval benchmark with raw-vs-HyperQuant latency percentiles, recall@K, NDCG@K, top-K overlap, exact-rerank recovery, rank drift, score error, and compression accounting.
- **Conservative public surface** — measurement APIs first; no silent production claims.

## Evidence pipeline

```text
fixtures / synthetic corpora
        ↓
codec or retrieval harness
        ↓
metrics: MSE, recall@K, MRR, NDCG, MAP, recovery
        ↓
benchmark receipts + diffs
        ↓
policy/admission decisions in downstream crates
```

The crate is intentionally upstream of runtime policy. It measures and records; it does not decide that a codec is admissible for a truth-bearing system.

## Installation

```toml
[dependencies]
quant-eval = "0.1.1"
```

From the RecursiveIntell Libraries workspace:

```toml
[dependencies]
quant-eval = { path = "../quant-eval" }
```

## Quick start: compression benchmark scaffold

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
    println!("mrr = {}", report.mrr);
    Ok(())
}
```

## Quick start: HyperQuant primitive evaluation

```rust
use quant_eval::{run_hyperquant_eval, HyperQuantEvalConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let result = run_hyperquant_eval(&HyperQuantEvalConfig {
        dim: 16,
        vectors: 64,
        seed: 42,
        scale: 8.0,
    })?;

    for profile in &result.profiles {
        println!(
            "{:?}: mean_mse={} max_mse={} receipts={}",
            profile.kind,
            profile.mean_mse,
            profile.max_mse,
            profile.receipt_count
        );
    }

    println!("claim boundary: {}", result.claim_boundary);
    Ok(())
}
```

## Quick start: governed HyperQuant receipt

```rust
use quant_eval::{
    run_governed_hyperquant_eval, GovernedHyperQuantEvalConfig,
    GovernedHyperQuantPolicyPreset, HyperQuantEvalConfig,
};
use quant_governor::AdmissibilityClass;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = run_governed_hyperquant_eval(&GovernedHyperQuantEvalConfig {
        fixture: HyperQuantEvalConfig::default(),
        policy_preset: GovernedHyperQuantPolicyPreset::StorageEfficient,
        size_bytes: 500_000,
        accuracy_requirement: 0.89,
        latency_tolerance_ms: 200,
        admissibility: AdmissibilityClass::Standard,
    })?;

    println!("selected codec: {}", receipt.decision.selected_codec);
    println!("admitted: {}", receipt.admitted);
    println!("reason: {}", receipt.admission_reason);
    println!("claim boundary: {}", receipt.claim_boundary);
    Ok(())
}
```

## Quick start: HyperQuant retrieval + latency benchmark

```rust
use hyperquant::LatticeKind;
use quant_eval::{run_hyperquant_retrieval_benchmark, HyperQuantRetrievalBenchmarkConfig};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let receipt = run_hyperquant_retrieval_benchmark(&HyperQuantRetrievalBenchmarkConfig {
        dim: 32,
        docs: 128,
        queries: 12,
        clusters: 8,
        top_k: 5,
        seed: 11,
        scale: 16.0,
        lattice: LatticeKind::D4,
    })?;

    println!("recall@k: {}", receipt.quality.recall_at_k);
    println!("ndcg@k: {}", receipt.quality.ndcg_at_k);
    println!("raw p95 ns: {}", receipt.raw_latency_ns.p95);
    println!("hyperquant p95 ns: {}", receipt.hyperquant_latency_ns.p95);
    println!("compression ratio: {}", receipt.compression_ratio);
    println!("claim boundary: {}", receipt.claim_boundary);
    Ok(())
}
```

Run the JSON-emitting example:

```bash
cargo run -p quant-eval --example hyperquant_retrieval_benchmark
```

## Public API

The crate re-exports:

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
- `evaluate_rag_fixture`
- `RagEvalResult`
- `RagQueryFixture`
- `RagRetrievedDoc`
- `run_hyperquant_eval`
- `run_governed_hyperquant_eval`
- `HyperQuantEvalConfig`
- `HyperQuantEvalResult`
- `HyperQuantProfileEval`
- `GovernedHyperQuantEvalConfig`
- `GovernedHyperQuantEvalReceipt`
- `GovernedHyperQuantPolicyPreset`
- `GovernedHyperQuantDecisionTrace`
- `GovernedHyperQuantBaseline`
- `run_hyperquant_retrieval_benchmark`
- `HyperQuantRetrievalBenchmarkConfig`
- `HyperQuantRetrievalBenchmarkReceipt`
- `HyperQuantRetrievalQuality`
- `HyperQuantRetrievalThresholds`
- `LatencySummaryNs`
- `RankDriftSummary`
- `ErrorSummary`

## Implemented modules

### Admissibility harness

File: `src/benchmarks/admissibility.rs`

Implemented:

- `CodecProfile` presets: `fast`, `balanced`, and `high_compression`.
- `AdmissibilityTest` over caller-provided `TestSetEntry` values.
- Standard synthetic test vectors for zero, unit, and deterministic pseudo-random vectors.
- Summary counts per profile.

Important limitation:

- This harness still simulates codec behavior from `should_succeed` and profile quality targets. It does not yet call a shared `quant-codec-core` trait.

### Compression benchmark scaffold

File: `src/benchmarks/compression.rs`

Implemented:

- Deterministic synthetic corpus and query generation.
- Raw nearest-neighbor computation with cosine similarity.
- Recall@K and MRR calculations over exact-vs-estimated result sets.
- Similarity-style summary statistics over top-K overlap.

Important limitations:

- Compression is currently simulated by returning exact result sets.
- It does not yet measure real encoded byte size, compression ratio, per-block ratios, wire formats, or codec theoretical ratios.
- The reported cosine-similarity statistics are derived from top-K overlap, not raw-vs-decoded vector cosine.

### Semantic-memory benchmark scaffold

File: `src/benchmarks/semantic.rs`

Implemented:

- Deterministic synthetic index and query generation.
- Raw search baseline using cosine similarity.
- Synthetic relevance judgments from raw top-K results.
- Precision@K, Recall@K, NDCG@K, MAP, and degradation-ratio calculations.

Important limitation:

- Compressed search currently delegates to raw search, so degradation is simulated/minimal by construction. It is not evidence of real codec preservation quality.

### RAG fixture harness

File: `src/rag.rs`

Implemented:

- Query fixtures with explicit relevant document IDs.
- Retrieved document list with scores.
- Recall@K.
- NDCG@K.
- Exact-rerank recovery for top-ranked relevant result.
- Duplicate retrieved-doc suppression.

### HyperQuant primitive harness

File: `src/hyperquant_eval.rs`

Implemented:

- `HyperQuantEvalConfig`
- `HyperQuantProfileEval`
- `HyperQuantEvalResult`
- `run_hyperquant_eval`
- `GovernedHyperQuantEvalConfig`
- `GovernedHyperQuantEvalReceipt`
- `GovernedHyperQuantPolicyPreset`
- `run_governed_hyperquant_eval`
- deterministic synthetic fixture generation;
- triangular A2 fixture where A2 should match or beat Z1;
- Z1/A2/D4 metrics through the published `hyperquant` crate;
- governed embedding admission/block receipts using `quant-governor`;
- Q8/Q4/HyperQuant byte-accounting baselines for fixture-level ROI review;
- conservative claim-boundary string on every result.

Important limitation:

- This is primitive-level and synthetic clustered-fixture evidence only. It is not HyperQuant paper parity, BEIR/TREC RAG evidence, model-quality evidence, production admissibility, or superiority evidence.

### HyperQuant retrieval + latency benchmark

File: `src/hyperquant_retrieval.rs`

Implemented:

- deterministic clustered embedding fixture generation;
- raw exact-search baseline over normalized vectors;
- HyperQuant search over reconstructed Z1/A2/D4 vectors;
- raw and HyperQuant latency summaries (`p50`, `p95`, `max`, `mean`);
- retrieval quality metrics: recall@K, top-K overlap, NDCG@K, exact top-1 recovery in HyperQuant top-K;
- rank drift and score-error summaries;
- raw-vs-Rice-estimated HyperQuant byte accounting and compression ratio;
- explicit thresholds and blockers in each receipt.

Important limitation:

- This benchmark is synthetic and local. It is useful for regression and first-pass ROI screening, but real retrieval claims still require real corpus/qrels evidence such as BEIR or TREC RAG.

### Benchmark receipts

Files: `src/receipt.rs`, `src/fingerprint.rs`

Implemented:

- `BenchmarkReceipt` with timestamp, commit hash, machine fingerprint string, result list, and optional note.
- `BenchmarkResult` timing fields.
- Receipt JSON serialization/deserialization.
- Receipt hash and receipt diff helpers.
- `MachineFingerprint` derived from available host/user/arch/OS/CPU-count/machine-id inputs.

## Claim boundary

Safe to claim today:

- `quant-eval` provides deterministic Rust benchmark scaffolds and fixture metrics.
- `quant-eval` can evaluate current HyperQuant Z1/A2/D4 primitive behavior.
- `quant-eval` can emit governed HyperQuant fixture receipts that join policy admission/block rationale with measured fixture metrics and Q8/Q4/HyperQuant byte baselines.
- `quant-eval` can benchmark HyperQuant synthetic clustered retrieval latency and quality with receipt-backed recall@K, NDCG@K, exact top-1 recovery, rank drift, score error, and compression accounting.
- `quant-eval` emits typed metrics and benchmark receipt structures.
- `quant-eval` has local tests, clippy, and publish dry-run receipts for this release.

Not safe to claim today:

- real corpus retrieval quality or TREC/BEIR RAG performance;
- actual compression-ratio measurements for all codecs;
- model-quality preservation;
- superiority of any codec;
- production readiness;
- integrated production policy enforcement for `poly-kv`, `fib-quant`, `turbo-quant`, or `semantic-memory`;
- `quant_codec_core::EvalReport` emission.

Those are reasonable next targets, but they need implementation evidence before becoming public claims.

## Verification

Release gate for v0.1.1:

```bash
cargo fmt -p quant-eval
cargo test -p quant-eval -- --nocapture
cargo test -p hyperquant -- --nocapture
cargo check -p quant-eval --all-targets
cargo clippy -p quant-eval --all-targets -- -D warnings
cargo publish -p quant-eval --dry-run --allow-dirty
```

Expected current test surface:

- 21 unit tests in `quant-eval` library modules.
- 5 HyperQuant integration tests.
- 5 governed HyperQuant receipt integration tests.
- 3 HyperQuant retrieval benchmark integration tests.
- 5 general integration tests.
- 5 RAG fixture tests.
- 44 `quant-eval` tests total.
- 26 `hyperquant` tests for the dependency surface.

## Development

Run focused HyperQuant evaluation tests:

```bash
cargo test -p quant-eval hyperquant_eval -- --nocapture
```

Run all quant-eval tests:

```bash
cargo test -p quant-eval -- --nocapture
```

Run lint gate:

```bash
cargo clippy -p quant-eval --all-targets -- -D warnings
```

## Integration path

Recommended adoption order:

```text
quant-eval fixture metrics
  -> quant-codec-core adapter reports
  -> quant-governor policy/admissibility
  -> turbo-quant / fib-quant comparative benchmarks
  -> poly-kv or semantic-memory only with exact fallback and disclosure
```

`quant-eval` should remain evidence infrastructure. Policy decisions belong in governor/runtime crates.

## Dependencies

Runtime dependencies:

- `serde`
- `serde_json`
- `thiserror`
- `chrono`
- `sha2`
- `blake3`
- `hyperquant`
- `quant-governor`

Dev dependency:

- `tempfile`

The crate currently contains no platform-specific code, FFI, async runtime dependency, CUDA, or HuggingFace integration.

## Roadmap

Near-term:

1. Add a codec evaluation trait or adapter layer so harnesses can call real encode/decode implementations.
2. Replace simulated compression paths with actual compressed/decompressed vector comparisons.
3. Add encoded-byte accounting and compression-ratio reports.
4. Emit or convert into `quant-codec-core` report shapes when that boundary is ready.
5. Add cross-crate integration tests for `hyperquant`, `fib-quant`, and `turbo-quant` adapters.

Medium-term:

1. Add real corpus fixtures for semantic-memory embeddings.
2. Add before/after receipt diffs for codec promotion reviews.
3. Add admissibility gates that can be consumed by `quant-governor`.
4. Add visual report export for benchmark receipts.

## License

MIT. See `LICENSE-MIT` for details.
