# turbo-quant

Experimental codec substrate: PolarQuant, TurboQuant, and QJL sketches,
with bit-packed wire formats and a KV-cache shadow mode.

`turbo-quant` is a **research-grade vector codec**. It ships three
quantization primitives (`PolarQuant`, `TurboQuant`, and QJL sketches)
and the surrounding infrastructure needed to use them: bit-packed wire
formats, candidate generation, exact rerank, KV-cache shadow mode, and a
benchmark harness that validates quality against a raw-vector reference.

**Status:** experimental / research substrate. The P27 real-workload
audit ([`docs/codex-runs/P27/REAL_BENCH_AUDIT.md](docs/codex-runs/P27/REAL_BENCH_AUDIT.md))
shows the candidate-search sidecar story **fails on BEIR scifact** (5,181
docs, 300 queries, 384-dim `all-minilm`): top-k overlap 0.049 and
top-1 rerank recovery 0.307. The crate's surviving use case is
**KV-cache shadow mode** (different problem: per-vector reconstruction,
not ranking). See the "Scope and limits" section for the full claim law.

## What's in the box

- **PolarQuant** — angular quantization of vectors onto a uniform
  angle grid. Asymmetric: scoring in compressed space only
  (no decode). Source code: `src/polar.rs`.
- **TurboQuant** — adds a QJL residual sketch on top of PolarQuant
  to recover accuracy. Symmetric: the residual sketch lets you
  approximate the inner product, and exact rerank uses the raw
  vector. Source: `src/turbo.rs`.
- **QJL sketches** — randomized sign-based Johnson-Lindenstrauss
  projections for cheap approximate inner product estimation.
  Source: `src/qjl.rs`.
- **Bit-packed wire formats** — `PackedPolarCode`,
  `PackedQjlSketch`, `PackedTurboCode` with a fixed
  `storage_layout: "polar_radii_f32_angles_bitpacked_qjl_signs_bitpacked"`.
  Source: `src/packed.rs`, `src/wire.rs`.
- **KV-cache shadow mode** — `KvRuntimeConfig`, `KvShadowToken`.
  Lets you score a compressed KV cache against a raw baseline and
  emit a `KvShadowReceipt`. The **surviving** use case for this crate.
  Source: `src/kv.rs`.
- **Codec profiles** — typed `CodecProfileV1` that captures the
  codec kind, dim, bits, projections, rotation, and a
  `profile_digest` (FNV-1a 64-bit) for receipt comparison.
  Source: `src/profile.rs`.
- **Benchmark harness** — `tools/semantic_memory_harness/` runs a
  synthetic smoke test; `examples/real_bench.rs` runs a real-workload
  BEIR benchmark and emits a `RealBenchmarkReceiptV1`.

## Quick Start

```rust
use turbo_quant::{CodecProfile, TurboSidecarCode, TurboSidecarIndex};
use nalgebra::DVector;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a codec profile.
    let profile = CodecProfile::turbo_quant_8bit(32)
        .with_projections(16)
        .with_seed(42);

    // Encode a corpus.
    let corpus: Vec<DVector<f32>> = /* your vectors */;
    let code = TurboSidecarCode::encode(&profile, &corpus)?;
    let index = TurboSidecarIndex::build(&profile, code)?;

    // Search — get candidates in compressed space, then rerank on raw.
    let query = DVector::from_vec(/* query vector */);
    let candidates = index.candidates(&profile, &query, 40, 10)?;  // oversample × top_k
    let reranked = index.exact_rerank(&candidates, &corpus, &query, 10)?;

    Ok(())
}
```

The `examples/` directory has runnable versions of this and four
other flows: `bench_embeddings.rs`, `kv_shadow.rs`,
`profile_receipt.rs`, `compat_0_1_smoke.rs` (the P26
release-gate smoke test), and `real_bench.rs` (real-embedding
benchmark with semantic-memory harness).

## Benchmarks — measured

### P27 real-workload audit (BEIR scifact, 2026-06-10)

The current load-bearing evidence. Receipt at
[`docs/codex-runs/P27/REAL_BENCH_RECEIPT.json`](docs/codex-runs/P27/REAL_BENCH_RECEIPT.json);
writeup at
[`docs/codex-runs/P27/REAL_BENCH_AUDIT.md`](docs/codex-runs/P27/REAL_BENCH_AUDIT.md).

Run on BEIR `scifact` (5,181 docs, 300 test queries, 339 positive qrels,
384-dim `all-minilm` embeddings via local Ollama), codec: TurboQuant
8-bit, 32 QJL projections, seed=42, rotation=Auto, oversample=4 (40
candidates → top-10):

| Metric | Value | Threshold | Verdict |
|---|---|---|---|
| **Top-k overlap (cand vs exact top-10, Jaccard)** | 0.049 | ≥ 0.30 | **FAIL** (6.1× too low) |
| **Exact-rerank recovery @1 (top-1 gt in candidate top-40)** | 0.307 | ≥ 0.80 | **FAIL** (2.6× too low) |
| Recall@1 (post-rerank) | 0.777 | — | misleading — see audit |
| Recall@5 (post-rerank) | 0.433 | — | |
| Recall@10 (post-rerank) | 0.787 | — | |
| Rank drift mean / p95 / max | 10.46 / 29 / 39 | — | wrong docs consistently ranked higher |
| Score error mean / p95 / max | 0.20 / 0.45 / 0.80 | — | large vs [-1, 1] range |
| Candidate-search latency p50 / p95 | 35 ms / 250 ms | — | full scan of 5,181 codes |

**Storage** (the part that kills the "sidecar" framing):

| Layout | Bytes | vs raw |
|---|---|---|
| Raw fp32 | 7,958,016 | 1.00× |
| Sidecar only | 4,870,140 | 1.63× compression |
| **Sidecar + raw** (for exact rerank) | **12,828,156** | **0.62× (1.6× LARGER than raw)** |

The sidecar is **not actually a sidecar** — it is a primary index that
still requires the raw vectors to produce useful results, and together
they're worse than storing raw alone.

**Verdict:** the candidate-search sidecar story does not survive a
real-workload test. The crate's surviving use case is **KV-cache shadow
mode**, which is a different problem (per-vector reconstruction, not
ranking) and is supported by separate P26 evidence (see
`docs/codex-runs/P26/SEMANTIC_MEMORY_PROOF_RECEIPT.json` for the
KV-shadow proof).

Reproduce:
```bash
TQ_EMBED_MODEL=all-minilm:latest \
  cargo run --release --example real_bench -- \
    --corpus docs/codex-runs/P27/corpus.tqcb \
    --out /tmp/bench/receipt.json \
    --bits 8 --projections 32 --seed 42 --rotation auto
```

The corpus binary (8.4 MB) and full build/bench logs are in
`docs/codex-runs/P27/`. The harness skill is at
`~/.hermes/skills/mlops/turbo-quant-beir-bench-harness/`.

### Prior evidence (P26, synthetic)

The earlier P26 evidence at
`docs/codex-runs/P26/SEMANTIC_MEMORY_PROOF_RECEIPT.json` is **synthetic**
(1,000×384 random unit vectors, 50 queries) and shows `recall@10 = 1.0`
on that corpus. P27 supersedes it for any real-workload claim: the same
codec that scored 1.0 on synthetic scores 0.787 (post-rerank, with
candidates missing the top-1 70% of the time) on BEIR scifact. The
synthetic number is not wrong but is not representative of real
retrieval.

The `tools/semantic_memory_harness/` synthetic harness is retained for
fast CI smoke tests, not as deployment evidence.

## C Kernels

Starting in v0.2.3, the hot paths in `turbo-quant` are backed by C kernels
compiled with optimization via the `cc` crate. AVX2/FMA is enabled only when
the target advertises those features. The original Rust
implementations are preserved in `src/archive/` with headers documenting the
replacement.

| Kernel | C file | Purpose | Speedup vs Rust |
|--------|--------|---------|-----------------|
| FWHT | `c-kernels/fwht.c` | Fast Walsh-Hadamard Transform | 2.75× |
| Polar encode/decode | `c-kernels/polar.c` | `atan2` + angle quantize, dequantize | 1.8× |
| QJL sketch/project/IP | `c-kernels/qjl.c` | Sign projection, query projection, inner-product estimate | 1.5× |
| Bitpack | `c-kernels/bitpack.c` | Bit packing/unpacking | reverted to Rust (FFI overhead) |

The bitpack kernel was reverted because branch-heavy bit manipulation has
high FFI call overhead that negates the compiler advantage. The FWHT and
scoring kernels benefit from GCC's auto-vectorization on AVX2 targets.

All C kernels are compiled at build time via `build.rs`. No pre-built
binaries. The `cc` build dependency is required.

## Scope and limits

This crate is **experimental**. The following claims are explicitly
**forbidden** in documentation, rustdoc, README, and release notes
unless scoped to a specific external paper claim or local receipt
evidence:

- "zero accuracy loss"
- "zero overhead"
- "production KV cache runtime"
- "drop-in replacement"
- "better than semantic-memory"
- "proven deployment quality"
- "no dataset-specific calibration needed"
- "candidate search sidecar" / "vector search sidecar" — see P27 audit;
  this story fails on real workloads and is **not allowed** as a
  crate-level claim.
- "exact-rerank recovers the sidecar's missing candidates" — the
  P27 number (0.307 top-1 recovery at 4× oversample) is the falsified
  claim this rule replaces.

What's allowed:

- "experimental codec substrate"
- "quantization primitives for PolarQuant / TurboQuant / QJL"
- "KV-cache shadow mode (per-vector reconstruction, not ranking)"
- "workload-specific benchmark receipts required"
- "approximate scoring; exact fallback or rerank is caller responsibility"
- "P26 evidence supports KV-cache shadow; P27 evidence shows vector-search
  sidecar fails on BEIR scifact — see `docs/codex-runs/P27/REAL_BENCH_AUDIT.md`"

The full release-claim law is at
`turbo-quant/AGENTS.md` (P26 patch) and
`~/.hermes/skills/software-development/recursiveintell-doctrine/`.

## What's verified

- `cargo test --all-targets --all-features --locked` passes
  (123 tests).
- `cargo check --all-targets --all-features --locked` clean.
- `cargo fmt --all -- --check` clean.
- `python3 scripts/assert_p26_invariants.py .` passes — all
  required P26 release artifacts are present and content-addressed.
- `cargo package` succeeds.
- The `SemanticMemoryHarnessSummaryV1` is emitted and SHA-256
  recorded in `docs/release-evidence/0.2.0/release_receipt.json`.

## Test coverage

- 18 integration test files in `tests/`:
  - `api_compat.rs`, `bitpack.rs`, `determinism.rs`,
    `encoded_size.rs`, `inner_product.rs`, `invalid_inputs.rs`,
    `kv_policy.rs`, `malformed_artifacts.rs`, `packed_index.rs`,
    `profile_receipt.rs`, `query_workspace.rs`, `readiness.rs`,
    `rotation_policy.rs`, `serialization.rs`, `wire_format.rs`,
    `workspace.rs` — plus 2 more.
- 4 examples: `bench_embeddings`, `compat_0_1_smoke`,
  `kv_shadow`, `profile_receipt`.
- 1 criterion bench: `benches/turbo_quant_search.rs`.

## MSRV

Rust 1.75 (2021 edition). Stable features only.

A C compiler (GCC or Clang; AVX2/FMA is optional) is required for the
C kernel build step via the `cc` crate.

## Dependencies

- `serde`, `nalgebra` (with `serde-serialize`).
- `bitvec` (transitive, for `BitPack`).
- `cc` (build dependency, for C kernel compilation).
- Workspace `Cargo.toml` pin.

C kernels (FWHT, polar, QJL) are compiled at build time via `build.rs`
with optimization and target-appropriate SIMD flags. The `unsafe` keyword is used in the FFI
boundary (`extern "C"` calls); the `unsafe_code` lint is allowed at
the crate level for these specific modules.

## License

MIT OR Apache-2.0 (dual-licensed). See `LICENSE-MIT` and
`LICENSE-APACHE` for the full texts.

## Changelog

See `CHANGELOG.md` for the release history. The v0.2.0 release
notes are in `RELEASE_NOTES.md` and the receipts in
`docs/release-evidence/0.2.0/`.

## Where it's used

`turbo-quant` is the experimental vector codec substrate for:

- [`semantic-memory`](../semantic-memory) — the codec primitives
  (`PolarQuant`, `TurboQuant`, QJL sketches) are available, but
  the **candidate-search sidecar route is gated off by default** as
  of P27 (2026-06-10). The P27 audit
  (`docs/codex-runs/P27/REAL_BENCH_AUDIT.md`) shows the sidecar fails
  on real workloads (BEIR scifact). Subsystems that need approximate
  scoring must run their own workload-specific benchmark and prove
  exact_rerank_recovery_at_1 ≥ 0.80 on their data before enabling
  the sidecar route.
- [`scr-runtime-compression`](../scr-runtime-compression) — the
  cross-runtime compression scheduler can use the codec primitives
  for batched compression; this path is unaffected by P27 (it is a
  compression, not a retrieval, use case).
- The KV-cache shadow mode is used by the
  `tools/semantic_memory_harness/` to validate the codec against
  a raw-vector reference. This is the **surviving** use case for
  the crate.

Adopting `turbo-quant` directly is appropriate for systems that
need a research-grade vector codec (per-vector reconstruction,
not approximate ranking) and that can run their own workload-specific
benchmarks to confirm the codec is appropriate for their use case.
The candidate-search sidecar is **not** recommended as a drop-in
component — see P27.
