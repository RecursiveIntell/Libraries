# HyperQuant Crate Creation Receipt — 2026-06-26

Repo: /home/sikmindz/Coding/Libraries
Crate: /home/sikmindz/Coding/Libraries/hyperquant
Package: hyperquant v0.1.0

## What shipped

Created a new Rust workspace crate: `hyperquant`.

Workspace changes:
- Added `hyperquant` to `[workspace].members`.
- Added `hyperquant` to `default-members`.
- Cargo.lock updated by Cargo resolution; includes the new `hyperquant` package and lock sync for already-dirty workspace manifests.

Crate files:
- `hyperquant/Cargo.toml`
- `hyperquant/README.md`
- `hyperquant/LICENSE-MIT`
- `hyperquant/LICENSE-APACHE`
- `hyperquant/src/lib.rs`
- `hyperquant/src/error.rs`
- `hyperquant/src/scalar.rs`
- `hyperquant/src/lattice.rs`
- `hyperquant/src/receipt.rs`
- `hyperquant/tests/error_contract.rs`
- `hyperquant/tests/lattice_contract.rs`
- `hyperquant/tests/receipt_contract.rs`

## Public API

Exports:
- `HyperQuantError`
- `Result`
- `LatticeKind`
- `HyperQuantConfig`
- `HyperQuantResult`
- `quantize_z1`
- `quantize_a2`
- `ClaimBoundary`
- `HyperQuantReceiptV1`

Implemented lattices:
- `Z1`: scalar integer-lattice quantization.
- `A2`: two-dimensional triangular-lattice nearest-point quantization using basis `b1=(1,0)`, `b2=(1/2,sqrt(3)/2)`.

Explicitly unsupported:
- `D4`
- `E8`

Unsupported lattices return `HyperQuantError::UnsupportedLattice`, not placeholder/fake results.

## Safety / claim boundaries

The crate explicitly does not claim:
- HuggingFace integration
- CUDA/GPU implementation
- paper parity
- model-quality preservation
- compression superiority
- D4/E8 implementation

Receipt claim boundary:
- `ClaimBoundary::ExperimentalPrimitiveOnly`

Non-finite behavior:
- Empty input rejected.
- NaN/Inf inputs rejected with `HyperQuantError::NonFiniteInput`.
- Finite inputs that would produce non-finite artifact metrics are rejected with `HyperQuantError::NonFiniteArtifact`.
- Receipts are bound to the quantization result's stored input/config digests; `receipt()` does not accept arbitrary input.

## TDD receipt

Initial tests were written before implementation and failed on missing API:

```text
cargo test -p hyperquant -- --nocapture
error[E0432]: unresolved imports hyperquant::quantize_a2, quantize_z1, ...
error[E0425]: cannot find type HyperQuantReceiptV1 in crate hyperquant
```

Then implementation was added and tests were driven green.

## Verification receipts

Final focused crate gates:

```text
cargo fmt -p hyperquant: PASS
cargo test -p hyperquant -- --nocapture: PASS
  - 4 unit tests
  - 3 error contract tests
  - 7 lattice contract tests
  - 4 receipt contract tests
  - 18 total tests

cargo check -p hyperquant --all-targets: PASS
cargo clippy -p hyperquant --all-targets -- -D warnings: PASS
cargo publish -p hyperquant --dry-run --allow-dirty: PASS
```

Workspace gate:

```text
cargo check --workspace: PASS
```

Pre-existing warnings observed during workspace check:
- `gpu-backend/src/simd_nearest.rs` unused import warnings.
- `hnsw-bench/src/main.rs` unused import/dead-code warnings.
- workspace warning: `quant-governor` package profile ignored because profiles belong at workspace root.

Security scan:

```text
security scan: no matches
```

Scanned new crate files for hardcoded secrets, shell injection patterns, eval/exec, pickle, and SQL format patterns.

## Independent review

First independent review: REQUEST_CHANGES
- Blocker: receipts accepted arbitrary input and could be misleading.
- Blocker: non-finite input/artifact behavior could silently produce non-finite metrics.

Fixes applied:
- `HyperQuantResult` now stores `input_len`, `input_digest`, and `config_digest`.
- `HyperQuantResult::receipt()` no longer accepts arbitrary input.
- `quantize_z1` and `quantize_a2` reject NaN/Inf inputs.
- `quantize_z1` and `quantize_a2` reject finite inputs that would produce non-finite MSE/receipt metrics.
- Added regression coverage for non-finite and overflow-artifact paths.

Final independent review: APPROVED
- Re-ran test/check/clippy/dry-run gates.
- Verified blocker fixes.
- No blockers remained.

## Semantic memory

Saved implementation fact in semantic memory namespace `libraries`:
- `c144122c-360a-4b73-8b2e-3981d2133b24`

## Not published yet

`cargo publish --dry-run --allow-dirty` passed, but the crate was not actually published.

Next publish steps, if desired:
1. Confirm crate name availability on crates.io.
2. Review README/public wording one more time.
3. Commit the crate and workspace lockfile intentionally.
4. Run `cargo publish -p hyperquant --allow-dirty` only after explicit publish decision.
