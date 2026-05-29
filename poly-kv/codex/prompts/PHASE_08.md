# PHASE 08 — Regression fixtures and validation scripts

Tasks:

1. Add `scripts/test_zpy_universal_packager_regression.py` or equivalent.
2. Add fixtures for Rust/Python/Node/Git/Docker/unsafe/Codex audit.
3. Wire tests into `scripts/run_next_validation.sh` or a new `scripts/run_zpy_validation.sh`.
4. Ensure existing `poly-kv` validators still pass.

Acceptance:

```bash
python3 scripts/test_zpy_hygiene_regression.py
python3 scripts/test_zpy_universal_packager_regression.py
python3 scripts/assert_source_package_hygiene.py --repo-root . --manifest <manifest> --package <package> --mode manifest
```
