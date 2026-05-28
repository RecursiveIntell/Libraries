# 01 — Codex Master Prompt: FibQuant Paper-Core Pass

## Role

You are Codex operating as a deterministic implementation agent. You must prioritize mathematical fidelity, reproducibility, bounded changes, and evidence over speed.

## Goal

Implement a new `fib-quant` Rust crate that faithfully implements the core FibQuant paper pipeline:

```text
x in R^d
nu = ||x||_2
u is stored as fp16 side header
u_hat may be f32 in reference mode only
u != 0 for normal encode path
u_vector = x / nu
y = Pi u                         # shared deterministic Haar-like orthogonal rotation
split y into d/k contiguous k-blocks
for each block b:
    index = nearest codeword in C_{d,k,N}
store fixed-rate indices
reconstruct block by codeword lookup
x_hat = nu Pi^T y_hat
```

## Required crate placement

- Add top-level workspace member: `fib-quant`.
- Do not add to `default-members` in this pass unless the entire workspace already requires every member there and final tests prove it is safe. Default expected behavior: member only, not default-member.

## Existing repo context

You must inspect existing compression surfaces before changing anything:

- `semantic-memory/src/vector_codec.rs`
- `semantic-memory/tests/vector_codec.rs`
- `semantic-memory/Cargo.toml`
- `turbo-quant/src/lib.rs`
- `turbo-quant/src/rotation.rs`
- `turbo-quant/src/turbo.rs`
- `turbo-quant/src/polar.rs`
- `turbo-quant/src/qjl.rs`
- `turbo-quant/src/kv.rs`
- root `Cargo.toml`

## Explicit non-goals

- No product integration.
- No semantic-memory adapter.
- No Gloss UI.
- No default-on codec.
- No FEUT/SCR variant.
- No rewrite of `turbo-quant`.
- No change to `semantic-memory` codec behavior.
- No public benchmark claims.

## Mathematical target

The crate must implement:

1. Profile law:
   - `FibQuantProfileV1` with explicit `ambient_dim d`, `block_dim k`, `codebook_size N`, rates, seeds, source mode, radius method, direction method, Lloyd settings, norm format.
   - Deterministic profile digest that changes when any math-bearing field changes.

2. Source law:
   - Spherical-Beta block source.
   - `R^2 ~ Beta(k/2, (d-k)/2)`.
   - `U ~ Uniform(S^{k-1})`.
   - `X = R U`.
   - Reference sampler from normalized Gaussian in R^d for comparison.

3. Radius law:
   - Bennett-Gersho / Beta-quantile radii.
   - k=2 closed form.
   - All radius math in f64.

4. Direction law:
   - k=2 planar Fibonacci spiral.
   - k=3 Fibonacci sphere.
   - k>=4 Roberts-Kronecker sequence mapped through inverse normal and projected to S^{k-1}.

5. Codebook law:
   - radial-angular deterministic initialization;
   - deterministic digest;
   - row-major codeword storage.

6. Lloyd-Max law:
   - multi-restart;
   - random orthogonal rotation per restart;
   - nearest assignment;
   - centroid update;
   - deterministic empty-cell repair;
   - keep lowest-MSE result;
   - emit `LloydReportV1`.

7. Codec law:
   - fixed-rate payload;
   - fp16 norm header by default;
   - index packing uses `ceil(log2(N))` wire bits;
   - distinguish paper rate `log2(N)/k` from wire rate;
   - reject corrupt digest, wrong dimension, wrong profile, out-of-range index, non-finite vectors.

8. Receipt law:
   - every encode can produce a receipt with source digest, profile digest, codebook digest, encoded digest, d/k/N, rate fields, seeds, MSE/cosine if measured, fallback availability, and recorded time.

## Required files

Create at minimum:

```text
fib-quant/Cargo.toml
fib-quant/README.md
fib-quant/src/lib.rs
fib-quant/src/error.rs
fib-quant/src/profile.rs
fib-quant/src/digest.rs
fib-quant/src/rotation.rs
fib-quant/src/spherical_beta.rs
fib-quant/src/beta_inv.rs
fib-quant/src/directions.rs
fib-quant/src/codebook.rs
fib-quant/src/lloyd.rs
fib-quant/src/bitpack.rs
fib-quant/src/codec.rs
fib-quant/src/metrics.rs
fib-quant/src/receipt.rs
fib-quant/tests/profile_digest.rs
fib-quant/tests/spherical_beta_sampler.rs
fib-quant/tests/paper_k2_radius_closed_form.rs
fib-quant/tests/direction_generators.rs
fib-quant/tests/codebook_determinism.rs
fib-quant/tests/lloyd_refinement.rs
fib-quant/tests/bitpack_indices.rs
fib-quant/tests/encode_decode_roundtrip.rs
fib-quant/tests/corruption_rejection.rs
fib-quant/tests/paper_smoke_regression.rs
docs/compression/FIBQUANT_SOURCE_BASIS.md
docs/compression/FIBQUANT_MATH_CONFORMANCE.md
docs/compression/FIBQUANT_BENCHMARK_PLAN.md
docs/compression/FIBQUANT_ROLLBACK_PLAN.md
```

## Dependency discipline

Prefer workspace dependencies where present. If a needed dependency is not present, add it narrowly to `fib-quant/Cargo.toml` first. Do not change unrelated crates.

Recommended dependencies:

```toml
serde = { workspace = true, features = ["derive"] }
serde_json = { workspace = true }
schemars = { workspace = true }
thiserror = { workspace = true }
blake3 = { workspace = true }
half = "2"
rand = "0.8"
rand_chacha = "0.3"
rand_distr = "0.4"
nalgebra = { version = "0.33", features = ["serde-serialize"] }
statrs = "0.17"
```

If using non-workspace dependency versions conflicts with the workspace, stop and report the conflict instead of force-upgrading the workspace.

## Completion definition

The pass is complete only if:

- `fib-quant` compiles;
- tests pass or failures are exact and classified;
- docs are written;
- no forbidden surfaces changed;
- final assertion script passes or reports exact blockers;
- final response includes receipts and unresolved deviations.
