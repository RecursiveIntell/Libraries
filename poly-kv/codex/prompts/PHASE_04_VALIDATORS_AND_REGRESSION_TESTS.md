# Phase 04 — Validators and regression tests

Add/copy:

- `scripts/assert_source_package_hygiene.py`
- `scripts/test_zpy_hygiene_regression.py`
- `scripts/build_handoff_package.sh`

Make them pass. These scripts must be useful without external dependencies.

Also fix `scripts/assert_no_boundary_drift.py` if it still contains literal backspace regex instead of `\b` boundaries.
