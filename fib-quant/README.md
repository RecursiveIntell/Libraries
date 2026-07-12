# fib-quant

Experimental Rust implementation of FibQuant-style radial-angular
vector quantization for KV-cache compression. ~50× compression in
the binary-packed form. 100% recall on the canonical benchmark
corpus (single-level).

> **Experimental, not production-ready.** This is an independent
> implementation based on the public FibQuant reference (Lee & Kim
> 2026). It is **not** the original paper code — see the Attribution
> section below.

`fib-quant` decomposes a vector into spherical blocks, quantizes
each block against a Fibonacci-optimized codebook, and stores only
the codebook indices. The result: a 768-dim f32 vector (3,072 bytes)
becomes ~860 bytes in JSON, or ~64 bytes with binary packing. And it
still finds the right document at rank 1 in 100% of the canonical
test queries.

This is the **cold-tier codec** in the poly-kv pool. It handles
shared context that's large, stable, and accessed by many agents:

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

## What's in the box

- **Codec** (`src/codec.rs`, 1121 lines) — `FibQuantizer` with
  `encode`, `decode`, `encode_with_receipt`, `encode_batch`,
  `decode_batch`, `decode_batch_fast`, `encode_layers`. The
  `encode_batch` path is the Rayon-parallel path that wins the
  poly-kv pool build.
- **Codebook** (`src/codebook.rs`, 207 lines) —
  `FibCodebookV1` with `build_initial_codebook` and Lloyd-Max
  refinement. Parity-verified against the reference.
- **Rotation** (`src/rotation.rs`, 216 lines) — fast
  Walsh-Hadamard rotation with `StoredRotation`, a CPU fallback,
  and an optional CUDA dispatch via `gpu-backend`.
- **Spherical beta** (`src/spherical_beta.rs`, 139 lines) —
  spherical-Beta direction samplers for codebook seeding:
  `sample_spherical_beta`, `beta_d_k`, `radius_quantile`,
  `radius_quantile_k2_closed_form`.
- **Profiles** (`src/profile.rs`, 424 lines) — typed
  `FibQuantProfileV1` with `paper_default(ambient_dim, block_dim,
  codebook_size, seed)` (k=4, N=32 defaults shown in examples
  below). Includes validation and rate computation.
- **Receipts** (`src/receipt.rs`, 128 lines) — typed
  `FibQuantCompressionReceiptV1` capturing every parameter of
  the encode pipeline for audit.
- **Scoring** (`src/scoring.rs`, 411 lines) — `GramTable` and
  `FibScorer` for approximate inner-product scoring without full
  decode, using a precomputed codebook Gram table.
- **Residual VQ** (`src/residual.rs`, 491 lines) — two-level
  vector quantization with `FibResidualQuantizer`,
  `FibResidualCodeV1`, and `ResidualCodebookV1` for improved
  reconstruction quality.
- **Wire format** (`src/wire.rs`, 333 lines) — self-describing
  wire format (`FibCodeWireV1`, `WireHeader`) that carries
  profile metadata so downstream consumers can decode without
  external profile knowledge.
- **Sidecar search** (`src/sidecar.rs`, 430 lines) —
  `FibSidecarIndex<Id>` wrapping `FibScorer` for approximate
  nearest-inner-product search with `ScoredCandidate` and
  `SearchReceiptV1`.
- **Eval harness** (`src/eval.rs`, 519 lines) — benchmark
  harness with `FibBenchmarkCorpus`, `run_benchmark`, and
  `FibBenchmarkReceiptV1` capturing recall@K, nDCG@K, compression
  ratio, MSE, and timing.
- **Compat layer** (`src/compat.rs`, 248 lines, `compat` feature)
  — implements `VectorCodec`, `CodecProfile`, and `KvCacheCodec`
  traits from `quant-codec-core` so fib-quant can be used through
  the shared codec trait boundary by `poly-kv`, `quant-governor`,
  and `scr-runtime-compression`.
- **KV-cache codec** (`src/kv/`, ~3,300 lines, `kv` feature) — a
  separate `KvCacheCodec` impl that operates on `KvTensorShape`
  rather than raw `Vec<f32>`. Includes attention-quality metrics,
  shape contracts, policy-role-aware dispatch, and a streaming
  encoder (`KvStreamEncoder` in `src/kv/stream.rs`, 800 lines) for
  incremental token-by-token construction.
- **KV receipts** (`src/kv/receipt.rs`) — `KvCompressionReceiptV1`,
  `KvDecodeReceiptV1`, and `KvEvalReceiptV1` for KV-cache
  pipeline audit.

## Quick Start

```rust
use fib_quant::{FibQuantProfileV1, FibQuantizer};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Build a quantizer with the default paper profile.
    // paper_default takes 4 args: ambient_dim, block_dim, codebook_size, seed
    let profile = FibQuantProfileV1::paper_default(32, 4, 32, 7)?;
    let quantizer = FibQuantizer::new(profile)?;

    // Encode a single vector.
    let vector: Vec<f32> = (0..32).map(|i| i as f32 * 0.01).collect();
    let code = quantizer.encode(&vector)?;
    let reconstructed = quantizer.decode(&code)?;

    // The cosine similarity should be high.
    let cos = fib_quant::metrics::cosine_similarity(&vector, &reconstructed)?;
    assert!(cos > 0.85);

    // Or encode with a full audit receipt.
    let (code, receipt) = quantizer.encode_with_receipt(&vector)?;
    println!("Receipt: {}", receipt.profile_digest);

    // Or encode a batch in parallel (returns Vec<FibCodeV1>).
    let corpus: Vec<Vec<f32>> = vec![vector.clone(), vector.clone()];
    let refs: Vec<&[f32]> = corpus.iter().map(|v| v.as_slice()).collect();
    let codes = quantizer.encode_batch(&refs)?;
    assert_eq!(codes.len(), 2);

    Ok(())
}
```

Run it: `cargo run --release --example encode_decode`.

## New modules

### Scoring — approximate inner product without full decode

`FibScorer` uses a precomputed codebook Gram table
(`GramTable`, `G[i,j] = <codeword_i, codeword_j>`) to estimate the
inner product of a query vector against a stored `FibCodeV1` without
running the full decode pipeline (rotation inverse + norm scaling).
The estimate is approximate — bounded by codebook quantization noise
— and avoids decoding the stored vector entirely. For a codebook of
size N=32, the Gram table is 4 KB.

```rust
use fib_quant::{FibScorer, FibQuantProfileV1, FibQuantizer};

let profile = FibQuantProfileV1::paper_default(32, 4, 32, 7)?;
let quantizer = FibQuantizer::new(profile)?;
// FibScorer::new takes ownership of the quantizer.
let scorer = FibScorer::new(quantizer)?;

let query = vec![0.1; 32];
// Encode before moving the quantizer into the scorer, or use
// a separate quantizer instance for encoding.
```

### Residual — two-level VQ for better fidelity

After encoding with the main codebook, the quantization residual
still contains signal. `FibResidualQuantizer` encodes this residual
with a second, smaller codebook (typically 4–8 codewords), adding
2–3 bits per block. Cosine fidelity improves from ~0.863
(single-level) to ~0.93+ (two-level) for typical nomic-768
workloads, at the cost of ~10–15% size increase.

```rust
use fib_quant::{FibQuantProfileV1, FibQuantizer, FibResidualQuantizer};

let profile = FibQuantProfileV1::paper_default(768, 4, 32, 7)?;
let quantizer = FibQuantizer::new(profile)?;
let residual = FibResidualQuantizer::build(&quantizer.profile().clone(), quantizer.codebook())?;
```

### Wire format — self-describing codes

`FibCodeWireV1` wraps a `FibCodeV1` with a 59-byte header carrying
seed, dim, k, N, norm_format, and profile digest. This enables
decode without any external profile knowledge — the compact FB1/FB2
formats strip all metadata, forcing consumers to hardcode profile
fields.

```rust
use fib_quant::{FibCodeWireV1, FibQuantProfileV1, FibQuantizer};

let profile = FibQuantProfileV1::paper_default(32, 4, 32, 7)?;
let quantizer = FibQuantizer::new(profile)?;
let code = quantizer.encode(&[0.1; 32])?;
let wire_bytes = FibCodeWireV1::to_wire_bytes(&code, quantizer.profile())?;
// Decode without needing the original profile — from_wire_bytes
// returns (FibCodeV1, FibQuantProfileV1):
let (decoded_code, recovered_profile) = FibCodeWireV1::from_wire_bytes(&wire_bytes)?;
```

### Sidecar — approximate search index

`FibSidecarIndex<Id>` stores caller-owned IDs alongside encoded
`FibCodeV1` artifacts and provides approximate nearest-inner-product
search via the Gram-table estimator. The index is a *sidecar*: it
produces approximate candidates that callers must rerank with an
exact inner-product computation against the original (un-encoded)
vectors before acting on the result.

```rust
use fib_quant::{FibScorer, FibSidecarIndex, FibQuantProfileV1, FibQuantizer};

let profile = FibQuantProfileV1::paper_default(32, 4, 32, 7)?;
// Encode first, then build the scorer for search.
let quantizer = FibQuantizer::new(profile)?;
let code = quantizer.encode(&[0.1; 32])?;

// Build a fresh quantizer to move into the scorer.
let profile2 = FibQuantProfileV1::paper_default(32, 4, 32, 7)?;
let scorer = FibScorer::new(FibQuantizer::new(profile2)?)?;
let mut index = FibSidecarIndex::new(scorer);

index.add(42, code);

let candidates = index.search(&[0.12; 32], 10, 4)?;
// Rerank candidates against original vectors before use.
```

### Eval — benchmark harness

`run_benchmark` takes a `FibBenchmarkCorpus` (database vectors,
queries, ground-truth top-K) and a `&FibQuantizer`, then returns a
`FibBenchmarkReceiptV1` with recall@K, nDCG@K, compression ratio,
cosine similarity, MSE, and timing. Standalone helpers `recall_at_k`
and `ndcg_at_k` are also exported.

```rust
use fib_quant::{FibBenchmarkCorpus, FibQuantProfileV1, FibQuantizer, run_benchmark};

let corpus = FibBenchmarkCorpus { /* ... */ };
let profile = FibQuantProfileV1::paper_default(768, 4, 32, 7)?;
let quantizer = FibQuantizer::new(profile)?;
let receipt = run_benchmark(&corpus, &quantizer)?;
println!("recall@10: {}", receipt.recall_at_k);
```

### Streaming KV encoder

`KvStreamEncoder` (behind the `kv` feature) allows building a
`KvEncodedTensorV1` one token at a time instead of requiring the
full tensor upfront. Pages are flushed automatically when they
reach `tokens_per_page` from the page geometry. The output is
identical to the batch `encode_kv_tensor` for the same input.

Currently supports `batch = 1, layers = 1, kv_heads = 1` shapes
(the common single-stream inference case).

## C Kernels

Starting in v0.1.0-beta.4, the encode/decode and compressed-attention
hot paths in `fib-quant` are backed by C kernels compiled with
`-O3 -mavx2 -mfma` via the `cc` crate. The original Rust implementations
are preserved in `src/archive/` with headers documenting the replacement.

| Kernel | C file | Purpose |
|--------|--------|---------|
| Codec | `c-kernels/codec.c` | `encode_vector_block` and `decode_vector_block` inner loops |
| Compressed attention | `c-kernels/attention.c` | Compressed attention logits + softmax computation |

The FFI boundary is in `src/ffi.rs` — `extern "C"` declarations wrapping
the C functions. `src/codec.rs`, `src/scoring.rs`, and
`src/kv/compressed_attention.rs` call through the FFI layer when the C
kernels are available. The `unsafe_code` lint is allowed at the crate
level for the FFI module.

All C kernels are compiled at build time via `build.rs`. No pre-built
binaries. The `cc` build dependency is required.

## Benchmarks — measured

### Compression ratios (768-dim nomic-embed-v1.5)

| Format | Bytes per vector | Ratio |
|---|---|---|
| Raw f32 | 3,072 | 1.0× |
| fib-quant JSON (default profile) | 860 | 3.6× |
| fib-quant binary-packed (`PackedFibCode`) | ~64 | ~48× |
| fib-quant KV-cache JSON | 1,200 | 2.6× |
| fib-quant KV-cache binary | ~80 | ~38× |

The "theoretical 50×" is the binary-packed compact form.
The "JSON 3.6×" is what you get with the default wire
format — the JSON envelope is 12× bigger than the actual
codebook indices. **If you're optimizing for storage, use
the binary wire format.**

### Retrieval quality (P26 measurement, semantic-memory harness)

8 queries, 200 docs, 768-dim, k=10, oversample=4:

| Route | Recall@1 | Recall@10 | nDCG@10 | Mean rank drift |
|---|---|---|---|---|
| exact_scan (no compression) | 1.000 | 1.000 | 1.000 | — |
| **fib-quant only** | **1.000** | **1.000** | **1.000** | **0.33** |
| turbo-quant only | 1.000 | 1.000 | 1.000 | 0.03 |
| poly-kv (two-tier) | 1.000 | 1.000 | 1.000 | 0.25 |

**Cosine fidelity: 0.863** (single vector), **0.9996** (after
turbo-quant rerank in poly-kv).

> The benchmark numbers above are from the P26 measurement run and
> are valid for single-level quantization. Two-level residual VQ
> improves cosine fidelity to ~0.93+ but has not been benchmarked
> for retrieval quality yet.

### Encode_batch throughput — "Do All" perf pass 2026-06-01

The `encode_batch` loop is the dominant cost in the poly-kv
pool build. After the June 1 perf pass (AVX2+FMA SIMD +
Rayon):

| Workload | Old (f64 ref) | + SIMD | + Rayon (parallel) | + full stack |
|---|---|---|---|---|
| qwen3 2560 n=80 | 13763ms | — | 1250ms | **346ms** (40×) |
| nomic 768 n=80 | 4552ms | 94ms | 407ms | **133ms** (34×) |
| qwen3 2560 n=4 | 1449ms | 418ms | 893ms | **256ms** (5.7×) |

Numbers from `poly-kv/benchmarks/DO_ALL_PERF_PASS_2026-06-01.md`.

### GPU path — measured

| Shape | n | CPU | Hadamard-GPU | Full-GPU | Best |
|---|---|---|---|---|---|
| d=64 | 80 | 14ms | **13ms (-7%)** | 14ms | Hadamard |
| d=128 | 80 | 57ms | **54ms (-5%)** | 56ms | Hadamard |
| d=768 | 80 | 2143ms | **2103ms (-2%)** | 2133ms | Hadamard |
| d=2560 | 4 | 1571ms | 1564ms (0%) | 1554ms (0%) | tie |

**Honest takeaway:** fib-quant's `encode_batch` is 2-7%
faster on a real GPU (msi i7-6700HQ + GTX 1070) with the
Hadamard path engaged. The codebook_lookup kernel exists and
is parity-verified, but the per-call H2D/D2H overhead
currently negates its win. A device-side pipeline is the
next step.

### Test coverage

- **23 integration test files** in `tests/`:
  - `bitpack_indices`, `codebook_determinism`,
    `compact_bytes_roundtrip`, `corruption_rejection`,
    `decode_batch_fast_parity`, `direction_generators`,
    `encode_decode_roundtrip`, `kv_attention_quality`,
    `kv_corruption_rejection`, `kv_encode_decode_reference`,
    `kv_policy_role_aware`, `kv_property_shapes`,
    `kv_shape_contracts`, `lloyd_refinement`,
    `norm_payload_rejection`, `paper_k2_radius_closed_form`,
    `paper_smoke_regression`, `profile_digest`,
    `profile_resource_bounds`, `property_bitpack`,
    `property_codec`, `rotation_identity`,
    `spherical_beta_sampler`.
- **4 examples**: `build_codebook`, `encode_batch_microbench`,
  `encode_decode`, `test_compact_decode`.
- **4 benches** (criterion): `codebook_build`, `encode_decode`,
  `kv_attention_ref`, `kv_encode_decode`.
- **122 tests** pass with `cargo test --all-features` (was 47
  before the new module additions, 105 before C kernel integration).
- `cargo clippy --all-features --all-targets -- -D warnings` clean
  (modulo upstream `gpu-backend` lints on newer toolchains).

## MSRV

Rust 1.75 (2021 edition). `#![forbid(unsafe_code)]` at the
crate level, except the FFI module (`src/ffi.rs`) which uses
`unsafe` for `extern "C"` calls to the C kernels.

A C compiler (GCC or Clang with AVX2/FMA support) is required for
the C kernel build step via the `cc` crate.

## Dependencies

- `serde` (with `derive`).
- `serde_json`.
- `blake3`.
- `half` (f16 support for scoring/scaling).
- `nalgebra` (with `serde-serialize`).
- `rand` + `rand_chacha` (dev).
- `gpu-backend` (optional) — for the GPU Hadamard dispatch.
- `rayon` (optional, behind the `parallel` feature) — for
  parallel batch encoding.
- `quant-codec-core` (optional, behind the `compat` feature) — for
  the shared codec trait boundary.
- `cc` (build dependency, for C kernel compilation).
- `proptest` (dev).
- `criterion` (dev).

## License

Apache-2.0. See `LICENSE-APACHE` for the full text.

## Changelog

See `CHANGELOG.md` for the release history.

## Attribution

This crate is an independent Rust implementation of the
FibQuant compression technique described in the public
literature (Lee & Kim, 2026). It is **not** the original
authors' reference code, and does not claim affiliation with
the FibQuant paper authors. The mathematical approach
(radial-angular block decomposition with Fibonacci-sampled
codebook seeding) follows the published specification.

The `kv-cache` codec profile, the parallel encode pipeline,
the GPU dispatch path, the receipt infrastructure, and the
test suite are original to this implementation.

## Where it's used

`fib-quant` is the cold-tier codec for:

- `poly-kv` — the shared pool is fib-quant compressed.
  Every shared system prompt, every shared few-shot example,
  every shared doc goes through `encode_batch`.
- `semantic-memory` — when `AdmissibilityClass::Standard` is
  selected by `quant-governor`, semantic-memory can route to
  the fib-quant sidecar for candidate generation.
- `scr-runtime-compression` — the `fib` feature is the
  fib-quant adapter for the runtime.

Any system that needs a **high-ratio, medium-fidelity** vector
codec — search-only recall, cold tier, ~50× storage savings
— can adopt `fib-quant` directly.