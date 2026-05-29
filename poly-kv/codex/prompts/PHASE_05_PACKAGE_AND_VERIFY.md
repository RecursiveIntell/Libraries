# Phase 05 — Package and verify

Run the full validation chain:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/assert_python_sidecar_layout.py
python3 scripts/assert_no_boundary_drift.py
python3 scripts/test_zpy_hygiene_regression.py
bash scripts/build_handoff_package.sh
```

Then inspect the generated manifest and prove required files are present.
