# Validation gates

Run from repo root.

```bash
python3 scripts/preflight_next_pass.sh
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
python3 scripts/assert_no_boundary_drift.py
python3 scripts/assert_python_sidecar_layout.py
python3 scripts/assert_source_package_hygiene.py --repo-root . --mode prepackage
python3 scripts/test_zpy_hygiene_regression.py
python3 z.py --root . --profile generic-rust --mode next-codex-context --strict --archive-root-package-artifacts --archive-root-markdown-noise
python3 scripts/assert_source_package_hygiene.py --repo-root . --manifest poly-kv-generic-rust-next-codex-context-*.manifest.json
```

If maturin is available:

```bash
python -m maturin develop
python -m pytest -q python/tests
python -m maturin build
```

Strict pass criteria:

- zero z.py findings
- no missing Python sidecar typing files
- no omitted current command evidence
- no root package sidecars remain before collection
- no stale Codex artifacts active outside allowed current-run surfaces
- no ambiguous root Markdown under strict hygiene mode
