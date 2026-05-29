# Validation Results

## Passed

- `bash scripts/preflight.sh`
- `cargo test -p quant-codec-core shape`
- `cargo test -p poly-kv`
- `cargo check -p poly-kv-python`
- `python -m compileall python`
- `python3 scripts/assert_python_sidecar_layout.py`
- `PYTHONPATH=python python -m pytest -q python/tests/test_import.py`
- `python3 scripts/bench_rust_synthetic.py --run-id 20260522T045320Z-poly-kv-next`
- `python3 scripts/bench_boundary.py --run-id 20260522T045320Z-poly-kv-next`
- `python3 scripts/compare_receipts.py --run-id 20260522T045320Z-poly-kv-next`
- `python -m json.tool` for all Phase 06 JSON artifacts
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- second `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`
- `python3 scripts/validate_schemas.py`
- `python3 scripts/check_public_claims.py`
- `python3 scripts/validate_final_state.py`
- `python3 scripts/assert_no_boundary_drift.py`
- `python3 scripts/assert_receipt_integrity.py`
- `python3 scripts/assert_realized_accounting.py`
- `PYTHONPATH=python python -m pytest -q python/tests` overall, with expected native skips
- `bash scripts/run_rust_gates.sh`

## Failed Then Repaired

- First `python3 scripts/assert_realized_accounting.py`: failed with baseline `SyntaxError`; repaired newline literal and reran successfully.
- First `cargo clippy --workspace --all-targets -- -D warnings`: failed on `KvCacheShapeV2::gqa` argument count; added targeted allowance and reran successfully.

## Skipped

- `python -m maturin --version` and `python -m maturin build`: skipped/failed because `maturin` is not installed.
- Native Python sidecar operation tests: skipped because `poly_kv._native` is not installed.
- `cargo-semver-checks check-release`: skipped by `scripts/run_rust_gates.sh` because `cargo-semver-checks` is not installed.
