# HyperQuant ROI Decision + Quant-Eval Integration Receipt — 2026-06-26

Repo: /home/sikmindz/Coding/Libraries

## Decision

Winner: implement the integration/evidence path, not paper parity.

Reason:
- Paper parity for arXiv:2606.23406 requires a full rate-distortion optimizer and model/layer-level reproduction. That is high effort, high claim risk, and not locally evidenced yet.
- The first integration step creates immediate leverage: quant-eval can now measure HyperQuant primitives before any governor, codec-core, turbo/fib/poly-kv integration promotes them.
- This preserves the doctrine boundary: evidence first, then policy, then runtime integration.

## Implemented

Added HyperQuant evaluation support to `quant-eval`.

Files changed:
- `quant-eval/Cargo.toml`
  - Added local `hyperquant` dependency.
- `quant-eval/src/lib.rs`
  - Exported HyperQuant eval API.
- `quant-eval/src/hyperquant_eval.rs`
  - New deterministic fixture evaluation module.
- `quant-eval/tests/hyperquant_eval.rs`
  - New contract tests.

New public API:
- `HyperQuantEvalConfig`
- `HyperQuantProfileEval`
- `HyperQuantEvalResult`
- `run_hyperquant_eval`

## Behavior

The harness evaluates the current local `hyperquant` crate, not paper parity.

It reports for Z1 and A2:
- `mean_mse`
- `max_mse`
- `mean_bytes_per_vector`
- `estimated_raw_bytes_per_vector`
- `estimated_compressed_bytes_per_vector`
- `rejected_vectors`
- `receipt_count`
- conservative claim boundary string

Fixtures:
- General deterministic synthetic vector fixture.
- Triangular A2 fixture where A2 should match or beat Z1.

## TDD receipt

Red test first:

```text
cargo test -p quant-eval hyperquant_eval -- --nocapture
error[E0432]: unresolved imports `quant_eval::run_hyperquant_eval`, `quant_eval::HyperQuantEvalConfig`
error[E0432]: unresolved import `hyperquant`
error[E0425]: cannot find type `HyperQuantEvalResult` in crate `quant_eval`
```

Green implementation followed.

## Verification receipts

```text
cargo fmt -p quant-eval: PASS
cargo test -p quant-eval hyperquant_eval -- --nocapture: PASS, 6 focused tests
cargo check -p quant-eval --all-targets: PASS
cargo clippy -p quant-eval --all-targets -- -D warnings: PASS
cargo test -p quant-eval -- --nocapture: PASS, 35 tests
cargo test -p hyperquant -- --nocapture: PASS, 18 tests
security scan over new quant-eval files: no matches
```

Independent review:

```text
APPROVED
```

Reviewer confirmed:
- ROI decision is sane.
- Harness is evidence-producing and lower risk than paper parity.
- Z1/A2 fixture coverage is deterministic and narrowly scoped.
- Claim boundaries are conservative.
- Publish dry-run failure is packaging-order limitation, not code correctness failure.

## Known packaging limitation

```text
cargo publish -p quant-eval --dry-run --allow-dirty: FAIL
```

Reasons:
- `quant-eval@0.1.0` already exists on crates.io.
- `hyperquant` is not published on crates.io yet, so the path+version dependency cannot resolve for publish packaging.

This does not block local correctness, but it blocks publishing this `quant-eval` state until:
1. `hyperquant` is published, and
2. `quant-eval` version is bumped.

## Semantic memory

Saved fact:
- `libraries:01f5a500-f770-4154-92f7-3957b2eb3429`

## Next highest-ROI step

Publish or otherwise package `hyperquant` only after operator approval. Then either:
1. bump `quant-eval`, rerun `cargo publish --dry-run`, and publish if desired; or
2. implement the `quant-codec-core` adapter behind a feature gate after benchmark receipts are accepted.
