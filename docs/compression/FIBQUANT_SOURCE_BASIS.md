# FibQuant Source Basis

Created: 2026-05-16

## Scope

This document is the Phase 0 no-code source basis for a paper-faithful `fib-quant` math-core pass. It exists before any `fib-quant` crate code is written.

The intended implementation target is a new top-level Rust workspace crate named `fib-quant`. This pass is limited to math core, codebook generation, fixed-rate encode/decode, tests, receipts, and documentation. It is not a semantic-memory integration pass.

## Source Hierarchy

1. Current repository files in `/home/sikmindz/Coding/Libraries`.
2. This document.
3. `FibQuant: Universal Vector Quantization for Random-Access KV-Cache Compression`, Namyoon Lee and Yongjune Kim, arXiv:2605.11478v1, submitted 2026-05-12.
4. Existing `turbo-quant` and `semantic-memory` codec surfaces as compatibility context only.

## Paper Source Used

Primary paper artifacts inspected:

- arXiv abstract page: `https://arxiv.org/abs/2605.11478`
- arXiv TeX source downloaded from `https://arxiv.org/e-print/2605.11478`
- Extracted local source during Phase 0: `/tmp/fibquant-paper/Arxiv_fibquant_neurips.tex`

The arXiv page identifies the paper as `arXiv:2605.11478v1`, submitted on 2026-05-12, with title `FibQuant: Universal Vector Quantization for Random-Access KV-Cache Compression`.

## Repository Shape Observed

Root `Cargo.toml` is a workspace with resolver `2`, many members, and an explicit `default-members` list. `turbo-quant` and `semantic-memory` are already workspace members and are also in `default-members`.

The expected new crate should be added as a workspace member only in a later phase. It should not be added to `default-members` unless a later validation pass explicitly proves that this is required and safe.

The working tree was already dirty before Phase 0 document creation. Existing modifications include root workspace files and many files under `semantic-memory/src/**`. Those are treated as pre-existing workspace state. This pass must not add or modify anything under:

- `semantic-memory/src/**`
- `turbo-quant/src/**`
- Gloss or product repositories

## Local Compatibility Surfaces Inspected

Required local files inspected during Phase 0:

- `Cargo.toml`
- `semantic-memory/Cargo.toml`
- `semantic-memory/src/vector_codec.rs`
- `semantic-memory/tests/vector_codec.rs`
- `turbo-quant/src/lib.rs`
- `turbo-quant/src/rotation.rs`
- `turbo-quant/src/turbo.rs`
- `turbo-quant/src/polar.rs`
- `turbo-quant/src/qjl.rs`
- `turbo-quant/src/kv.rs`
- `turbo-quant/src/wire.rs`

Observed compatibility facts:

- `semantic-memory` exposes `VectorCodecProfileV1`, `VectorArtifactV1`, and a `VectorCodec` trait. Artifacts carry profile and encoded digests and fail closed on profile or artifact digest mismatch.
- `semantic-memory` has raw f32 and SQ8 codecs, plus an optional `turbo-quant-codec` feature.
- `turbo-quant` provides seeded stored rotations, PolarQuant, QJL, TurboQuant, KV cache helper types, and a deterministic wire format.
- `turbo-quant` is scalar/polar/QJL compatibility context only. It must not be rewritten or used as a hidden semantic widening layer for FibQuant.

## Paper-Faithful Math Obligations

The paper pipeline to implement in later phases is:

1. For a nonzero cached vector `x in R^d`, compute norm `nu = ||x||_2`.
2. Store `nu` as an fp16 norm header by default.
3. Normalize and rotate with a shared deterministic Haar-like orthogonal rotation: `y = Pi x / nu`.
4. Split `y` into `d / k` contiguous blocks, requiring `k | d`.
5. Encode each block by nearest-codeword lookup against a shared codebook `C = {c_1, ..., c_N} subset B^k`.
6. Store fixed-rate indices with `ceil(log2(N))` wire bits per index.
7. Decode by table lookup, inverse rotation, and multiplication by the stored norm.

The canonical block source is the spherical-Beta law on the unit ball. For `U ~ Uniform(S^{d-1})` and `X = U_{1:k}`:

```text
R^2 ~ Beta(k/2, (d-k)/2)
E[R^2] = k / d
Var(R^2) = 2k(d-k) / (d^2(d+2))
X / R is uniform on S^{k-1} and independent of R
```

The codebook initialization must use radial-angular construction:

```text
beta_{d,k} = (k / (k + 2)) * ((d - k - 2) / 2) + 1
q_n = (n - 1/2) / N
r_n = sqrt(BetaInv(q_n; k/2, beta_{d,k}))
```

For `k = 2`, the paper gives a closed form:

```text
r_n = sqrt(1 - (1 - q_n)^(4/d))
```

Direction generation obligations:

- `k = 2`: planar Fibonacci spiral with golden-angle sequence.
- `k = 3`: Fibonacci sphere with equal-area latitude bands and golden-angle azimuth.
- `k >= 4`: Roberts-Kronecker rank-one sequence, inverse-normal mapped, then projected to `S^{k-1}`.

Lloyd-Max refinement is mandatory. Initialization alone is not sufficient. The paper pseudocode requires:

1. Sample training blocks from the spherical-Beta source.
2. Run multiple restarts.
3. Apply a random orthogonal rotation to the initial codebook per restart.
4. Alternate nearest assignment and centroid update.
5. Repair empty cells by splitting high-distortion cells.
6. Keep the codebook with the lowest training MSE.

## Rate and Wire Law

The implementation must keep mathematical and practical rates distinct:

```text
paper_rate_bits_per_coord = log2(N) / k
wire_index_bits = ceil(log2(N))
wire_bits_per_coord = wire_index_bits / k
```

The paper rate is dense and can include fractional or sub-one-bit operating points. The first-pass wire format is fixed-rate and byte-packed, so it must use `ceil(log2(N))` bits per emitted index unless a future enumerative/fractional fixed-rate coder is explicitly implemented and tested.

Variable-length payloads are forbidden in the paper-faithful path.

## Required Fail-Closed Behavior

Later code must reject, not repair silently:

- zero dimension;
- invalid `k`;
- invalid `N`;
- `d % k != 0`;
- non-finite input vectors;
- zero norm on the normal encode path;
- corrupt profile digest;
- corrupt codebook digest;
- wrong vector dimension;
- wrong profile;
- out-of-range decoded index;
- malformed fixed-rate payload.

No silent padding is allowed when `d % k != 0`.

## Non-Goals

This pass must not:

- modify `semantic-memory/src/**`;
- modify `turbo-quant/src/**`;
- modify Gloss or product repositories;
- make FibQuant default anywhere;
- integrate FibQuant into `semantic-memory`;
- add FEUT or SCR variants;
- claim paper benchmark wins as local measurements;
- claim lossless accuracy;
- implement only `k = 2`;
- skip Lloyd-Max refinement;
- write compatibility shims that silently widen semantics.

## Phase 0 Stop Status

Phase 0 has produced this source-basis document and has not created the `fib-quant` crate. Coding can begin only in Phase 1 after this document is reviewed as the source basis.
