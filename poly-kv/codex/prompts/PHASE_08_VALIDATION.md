# Phase 08 — Full validation

Run and record:

```bash
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo doc --workspace --no-deps
python3 scripts/validate_schemas.py
python3 scripts/check_public_claims.py
python3 scripts/validate_final_state.py
python3 scripts/assert_no_boundary_drift.py
python3 scripts/assert_receipt_integrity.py
python3 scripts/assert_realized_accounting.py
python -m compileall python
python -m pytest -q python/tests
```

If a tool is unavailable, record exact command, stderr, and whether it blocks release.

Gate: no skipped check is omitted from final report.
