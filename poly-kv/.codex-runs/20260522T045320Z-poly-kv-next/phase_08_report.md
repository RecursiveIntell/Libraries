# Phase 08 Report

## Scope

Full validation pass.

## Validation results

| Command | Result |
|---|---|
| `cargo fmt --all -- --check` | pass |
| `cargo check --workspace --all-targets` | pass |
| `cargo test --workspace --all-targets` | pass |
| first `cargo clippy --workspace --all-targets -- -D warnings` | fail: `KvCacheShapeV2::gqa` had too many arguments |
| second `cargo fmt --all -- --check` | pass |
| second `cargo clippy --workspace --all-targets -- -D warnings` | pass |
| `cargo doc --workspace --no-deps` | pass |
| `python3 scripts/validate_schemas.py` | pass |
| `python3 scripts/check_public_claims.py` | pass |
| `python3 scripts/validate_final_state.py` | pass |
| `python3 scripts/assert_no_boundary_drift.py` | pass |
| `python3 scripts/assert_receipt_integrity.py` | pass |
| `python3 scripts/assert_realized_accounting.py` | pass |
| `python -m compileall python` | pass |
| `PYTHONPATH=python python -m pytest -q python/tests` | pass overall: 1 passed, 3 skipped |

## Repairs during validation

Added `#[allow(clippy::too_many_arguments)]` to `KvCacheShapeV2::gqa`, matching the explicit constructor boundary already used for `KvCacheShapeV2::new`.

## Skips

Python native interop tests skipped because `poly_kv._native` is not installed and `maturin` is unavailable. This remains a sidecar build-validation blocker, not a Rust core blocker.
