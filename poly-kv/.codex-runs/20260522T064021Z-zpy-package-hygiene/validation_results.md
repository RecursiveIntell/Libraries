# Validation Results

## Passed

- `bash scripts/preflight.sh`
- `python3 -m py_compile z.py scripts/assert_source_package_hygiene.py scripts/test_zpy_hygiene_regression.py scripts/assert_no_boundary_drift.py`
- `python3 scripts/assert_python_sidecar_layout.py`
- `python3 scripts/assert_no_boundary_drift.py`
- `python3 scripts/test_zpy_hygiene_regression.py`
- `RUN_ID=20260522T064021Z-zpy-package-hygiene bash scripts/build_handoff_package.sh`
- `RUN_ID=20260522T064021Z-zpy-package-hygiene bash scripts/run_next_validation.sh`
- `python3 scripts/assert_source_package_hygiene.py --repo-root . --manifest poly-kv-generic-rust-next-codex-context-20260522T064721Z.manifest.json --mode manifest`
- `python3 scripts/validate_final_state.py`
- `python3 scripts/check_public_claims.py`
- `bash scripts/run_rust_gates.sh`

## Validation Repairs During Run

- `scripts/preflight_next_pass.sh` had invalid shell quoting around the Cargo path-dependency grep. Fixed before package build.
- `scripts/run_next_validation.sh` needed `PYTHONPATH=python` for source-layout Python tests. Fixed and rerun successfully.

## Skipped

- `cargo-semver-checks check-release` was skipped by `scripts/run_rust_gates.sh` because `cargo-semver-checks` is not installed.
