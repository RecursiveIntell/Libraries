# Validation Results

Passing:

- `bash scripts/preflight.sh`
- `cargo fmt --all -- --check`
- `cargo check --workspace --all-targets`
- `cargo test --workspace --all-targets`
- `cargo clippy --workspace --all-targets -- -D warnings`
- `cargo doc --workspace --no-deps`
- `cargo test -p quant-codec-core --all-targets`
- `cargo clippy -p quant-codec-core --all-targets -- -D warnings`
- `cargo test -p poly-kv synthetic -- --nocapture`
- `cargo test -p poly-kv memory_accounting`
- `python3 scripts/validate_schemas.py`
- `python3 scripts/check_public_claims.py`
- `python3 scripts/validate_final_state.py`
- `python3 scripts/check_forbidden_patterns.py`
- `cargo check -p poly-kv --benches --features bench`

Skipped:

- `cargo semver-checks check-release`: `cargo-semver-checks` is not installed. This blocks publish/release approval, not local implementation validation.

Crate-name visibility:

- `cargo search poly-kv --limit 5`: no matching output returned.
- `cargo search quant-codec-core --limit 5`: no matching output returned.

Failing checks: none.
