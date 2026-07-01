# fib-quant Improvement Plan
Josh Stevenson / RecursiveIntell — 2026-06-25

Current state: v0.1.0-alpha.2. 4,941 lines src, 1,089 lines tests (47 passing, 0 failures),
153 lines benches, 167 lines examples. Compiles clean --all-features. clippy clean.
README claims "100% recall on canonical benchmark corpus" and "50x compression" — those
are real P26 measurements, not paper parity.

## GAP 1: No quant-codec-core trait implementation [ARCHITECTURE]

fib-quant does not implement `VectorCodec`, `KvCacheCodec`, or `CodecProfile` from
quant-codec-core. turbo-quant does. This means poly-kv and quant-governor cannot route
to fib-quant through the shared trait boundary — they use a manual adapter instead.

**Fix**: Add a `compat` feature gate that implements:
- `CodecProfile` for `FibQuantProfileV1` (codec_id = "fib_quant", fixed_rate_bits = wire_index_bits, is_lossy = true)
- `VectorCodec` for `FibQuantizer` (encode_block / decode_block delegating to existing encode/decode)
- `KvCacheCodec` for the KV codec (encode_kv_cache / decode_slice delegating to encode_kv_tensor / a new random-access decode path)

Add `quant-codec-core` as an optional dependency behind `compat` feature.
This is additive — no existing API breaks.

Files: new `src/compat.rs`, Cargo.toml feature, lib.rs re-export.

## GAP 2: No approximate inner product / scoring path [ALGORITHM]

turbo-quant has `inner_product_estimate(code, query)` — score without decompressing.
fib-quant has nothing. Every comparison requires full decode → rotation inverse → norm
scaling → dot product. For a 10K-vector corpus, that's 10K decodes just to find top-K.

The FibQuant paper describes the codebook as supporting approximate scoring: the inner
product of two quantized vectors can be estimated from the codebook indices alone, using
precomputed codebook Gram tables. This is the same pattern as turbo-quant's QJL estimate.

**Fix**: Add `inner_product_estimate(code, query) -> f32` to `FibQuantizer`.
Method:
1. Normalize query, apply rotation → get rotated query blocks
2. For each block, find nearest codeword index (same as encode)
3. Precompute codebook Gram matrix: G[i,j] = <codeword_i, codeword_j> (N×N, computed once)
4. Estimate = sum over blocks of G[query_index, code_index] * norm_query * norm_code
5. This gives an approximate inner product without decoding the stored code

Add `precompute_gram_table()` to FibQuantizer (lazy, cached).
Add `FibSidecarIndex` with `search(query, top_k, oversample) -> Vec<ScoredCandidate>` +
`SearchReceiptV1`, mirroring turbo-quant's pattern.

Files: new `src/scoring.rs`, new `src/sidecar.rs`, codec.rs additions.

## GAP 3: No residual / second-level quantization [ALGORITHM]

The FibQuant paper describes a two-level scheme: coarse codebook + residual codebook.
The current implementation is single-level only. The Lloyd-Max refinement improves the
single codebook but there's no residual stage that captures the quantization error and
encodes it with a second, smaller codebook.

turbo-quant has this pattern: PolarQuant (coarse) + QJL residual sketch (fine).

**Fix**: Add `FibResidualCodeV1`:
- After encoding with the main codebook, compute residual = rotated_block - codeword[index]
- Encode residual with a second small codebook (e.g., 4 codewords, 2 bits)
- Store as `residual_indices` alongside the main indices
- Decode: codeword[main_index] + residual_codebook[residual_index]
- This should push cosine fidelity from 0.863 → 0.93+ based on paper claims

Profile field: `residual_codebook_size: Option<u32>` (None = single-level, backward compatible).
Wire format: append residual_indices after main indices, with a flag bit in the header.

Files: new `src/residual.rs`, profile.rs field, codec.rs encode/decode path.

## GAP 4: Rotation is QR/Gaussian, not Hadamard [PERFORMANCE]

The README says "fast Walsh-Hadamard rotation" but the actual `StoredRotation` uses
nalgebra QR decomposition of a Gaussian matrix. This is O(d^3) to build and O(d^2) per
apply. The Hadamard rotation is O(d log d) to apply and O(d) to build (just a seed +
recursive structure).

turbo-quant has `FastHadamardRotation` in its rotation module. gpu-backend has
`hadamard_batch` for GPU. fib-quant's rotation is the slow path.

**Fix**: Add `HadamardRotation` as an alternative rotation strategy:
- `RotationMethod::Hadamard` in the profile (additive, non-breaking)
- Deterministic from seed: apply sign-flip pattern + recursive Hadamard
- O(d log d) apply, no nalgebra dependency for this path
- f32-native (no f64 intermediate like the current StoredRotation)
- The existing QR rotation stays as the `RotationMethod::QrGaussian` default for
  backward compatibility

Files: rotation.rs new struct + apply/apply_inverse, profile.rs enum variant.

## GAP 5: KV per-channel and role-aware policies are stubs [CORRECTNESS]

`KvAxisPolicyV1::PerChannel` and `RoleAwareKiviStyle` both fall back to raw or error:
```rust
KvAxisPolicyV1::PerChannel | KvAxisPolicyV1::RoleAwareKiviStyle => {
    if profile.fallback_policy.mode == KvFallbackModeV1::KeepRaw {
        Ok(raw_block(...))  // <-- stub: just stores raw
    } else {
        Err(FibQuantError::DependencyUnsupported(...))
    }
}
```

These are declared in the policy enum and tested in `kv_policy_role_aware.rs` but never
actually compress. The KIVI-style split (keys channel-wise, values token-wise) is a
declared strategy that doesn't work.

**Fix**: Implement per-channel quantization:
- For PerChannel: transpose the head_dim × tokens slice, quantize each channel vector
  (length = tokens) instead of each token vector (length = head_dim)
- For RoleAwareKiviStyle: keys use PerChannel, values use PerToken (already works)
- This requires a new vector extraction path in the KV codec (channel-major instead of
  token-major)
- Add tests verifying that per-channel actually compresses (not raw fallback)

Files: kv/codec.rs encode_vector_block, kv/layout.rs channel extraction.

## GAP 6: No random-access KV decode [PERFORMANCE]

`decode_kv_pages` decodes ALL pages and ALL blocks into a flat f32 buffer. For a
128K-token KV cache with 32 layers, 8 heads, 128 head_dim, that's 128K × 32 × 8 × 128 =
524M floats = 2GB decoded just to read one token's key vector.

The quant-codec-core `KvCacheCodec::decode_slice` trait method exists for this —
decode only the requested slice. fib-quant doesn't implement it.

**Fix**: Add `decode_kv_slice(encoded, request: KvSliceRequest) -> Vec<f32>`:
- Parse the request (batch, layer, head, token range)
- Only decode blocks matching the request
- Skip pages entirely outside the token range
- This is the random-access property the FibQuant paper claims

Files: kv/codec.rs new function, kv/page.rs random-access block iteration.

## GAP 7: No streaming append [ALGORITHM]

KV caches are built incrementally — each generated token appends one key and one value
vector. The current codec requires the full tensor upfront. You can't append to an
existing `KvEncodedTensorV1`.

The FibQuant paper explicitly claims "random-access" and "streaming" as key properties.

**Fix**: Add `KvStreamEncoder`:
- `KvStreamEncoder::new(shape, layout, profile) -> Self`
- `append_token(key_vector, value_vector) -> AppendReceipt`
- `finish() -> KvEncodedTensorV1`
- Internally: build pages incrementally, flush when page is full
- The receipt tracks append order, token count, fallback events

Files: new `src/kv/stream.rs`, kv/mod.rs re-export.

## GAP 8: No schemars::JsonSchema on wire types [INTEROP]

turbo-quant derives `JsonSchema` on all wire types. fib-quant doesn't. This means any
system that generates JSON Schema from the codec types (for validation, documentation,
or cross-language binding) can't include fib-quant types.

**Fix**: Add `schemars` as an optional dependency behind a `schema` feature.
Add `#[derive(JsonSchema)]` to: FibCodeV1, FibCodebookV1, FibQuantProfileV1,
FibQuantCompressionReceiptV1, KvEncodedTensorV1, KvDecodedTensorV1, and all KV sub-types.

Files: Cargo.toml schemars dep, derive annotations across src/.

## GAP 9: No benchmark eval harness [VALIDATION]

turbo-quant has `BenchmarkCorpus`, `BenchmarkReceiptV1`, `CompressionEvalV1` in its eval
module. quant-eval has `fingerprint` and `receipt` modules. fib-quant has benches
(criterion) but no eval harness that produces typed compression eval receipts.

The governed compression spec requires: "no codec is enabled by default without local
replay fixtures, top-K retrieval drift, contradiction-recall, protected-span behavior,
and compression eval receipts."

**Fix**: Add `eval` module to fib-quant:
- `FibBenchmarkCorpus` — fixture vectors + queries + ground truth top-K
- `run_benchmark(corpus, profile) -> FibBenchmarkReceiptV1`
- Receipt includes: compression ratio, mean cosine, recall@K, encode time, decode time,
  per-vector MSE distribution, byte accounting
- This is what quant-governor needs to make routing decisions

Files: new `src/eval.rs`.

## GAP 10: Calibration never recommends bits_per_coord [CORRECTNESS]

`KvCalibrationSummaryV1::recommended_bits_per_coord` is always `None`. The calibration
function collects norm statistics but never uses them to recommend a bit rate. The whole
point of calibration is to pick the bit rate that meets a quality budget.

**Fix**: Add quality-budget-driven recommendation:
- Input: `KvQualityBudgetV1` (target MSE or target cosine)
- Calibration measures norm distribution → predicts MSE at each bit rate
- Uses the Beta-distribution radius quantile model to predict per-bitrate MSE
- Sets `recommended_bits_per_coord` to the lowest rate meeting the budget
- This is what the KV policy needs to auto-select codebook size

Files: kv/calibration.rs, kv/profile.rs quality budget types.

## GAP 11: SIMD rotation apply [PERFORMANCE]

`StoredRotation::apply` and `apply_inverse_f32` are scalar loops. The matrix is row-major
f64 but the apply is a naive `for row { for col { sum += matrix[row*dim+col] * input[col] } }`.
No SIMD, no cache-friendly blocking.

gpu-backend has `simd_nearest.rs` with AVX2+FMA for the codebook lookup. The rotation
apply is equally hot (called per-vector in encode, per-vector in decode) but has no SIMD.

**Fix**: Add SIMD-accelerated rotation apply for the f32 path:
- Use `std::simd` (portable SIMD, stable since 1.75) for 4/8-wide f32 dot products
- Or: use the gpu-backend `simd_nearest` pattern with `std::arch::x86_64` AVX2 intrinsics
  behind a cfg gate
- Target: 2-4x speedup on the rotation apply for d=768 (nomic) and d=2560 (qwen3)

Files: rotation.rs new `apply_simd` methods, feature gate.

## GAP 12: Repo hygiene [CLEANUP]

- `fib-quant/z.py` — junk file from a codex run, should be deleted
- `fib-quant/docs/codex-runs/` — archive bloat, should be cleaned per repo hygiene rules
  (keep at most the latest run, archive the rest or delete)
- `fib-quant/docs/compression/` and `fib-quant/docs/kv/` — included in package but may
  contain stale content

**Fix**: Delete z.py, archive or delete old codex-runs, verify docs/ is current.

## PRIORITY ORDER

1. GAP 1 (quant-codec-core traits) — unblocks poly-kv and quant-governor integration
2. GAP 2 (approximate scoring + sidecar) — biggest functionality gap vs turbo-quant
3. GAP 3 (residual quantization) — biggest quality improvement (0.863 → 0.93+ cosine)
4. GAP 5 (per-channel/role-aware KV) — declared features that don't work
5. GAP 6 (random-access KV decode) — required for real KV-cache use
6. GAP 9 (eval harness) — required for quant-governor routing decisions
7. GAP 4 (Hadamard rotation) — performance, but QR works correctly today
8. GAP 11 (SIMD rotation) — performance, after Hadamard if both are done
9. GAP 7 (streaming append) — important for real KV-cache but can be layered on top
10. GAP 10 (calibration bits_per_coord) — completes the calibration loop
11. GAP 8 (schemars) — interop, low risk, can be done anytime
12. GAP 12 (repo hygiene) — cleanup, do last

## WHAT TO BORROW FROM OTHER CRATES

From turbo-quant:
- Sidecar index pattern (TurboSidecarIndex → FibSidecarIndex)
- SearchReceiptV1 pattern
- ByteAccountingV1 pattern
- Packed wire format pattern (PackedTurboCode → PackedFibCode)
- BenchmarkCorpus / CompressionEvalV1 pattern
- CodecProfile trait impl pattern

From quant-codec-core:
- VectorCodec, KvCacheCodec, CodecProfile traits (implement them)

From gpu-backend:
- SIMD nearest-codeword (already used) → extend to SIMD rotation apply

From quant-eval:
- Fingerprint + receipt patterns for benchmark harness

From scr-runtime-compression:
- CompressedSearchPath + ExactFallbackAdapter (already references fib-quant, but fib-quant
  doesn't expose the right interface for it — the sidecar index from GAP 2 fixes this)

## ESTIMATED EFFORT

- GAP 1: 2-3 hours (trait impls, feature gate, tests)
- GAP 2: 4-6 hours (Gram table, scoring, sidecar, search receipt, tests)
- GAP 3: 3-4 hours (residual codebook, encode/decode, tests, benches)
- GAP 4: 2-3 hours (Hadamard rotation, tests, parity verification)
- GAP 5: 3-4 hours (per-channel extraction, policy impl, tests)
- GAP 6: 2-3 hours (random-access decode, tests)
- GAP 7: 3-4 hours (stream encoder, append receipt, tests)
- GAP 8: 1-2 hours (schemars derives, feature gate)
- GAP 9: 3-4 hours (eval harness, fixtures, receipts)
- GAP 10: 2-3 hours (calibration math, budget types, tests)
- GAP 11: 2-3 hours (SIMD, benchmarks)
- GAP 12: 30 min (cleanup)

Total: ~30-40 hours of focused implementation work.