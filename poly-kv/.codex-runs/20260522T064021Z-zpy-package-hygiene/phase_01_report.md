# Phase 01 Report - Allowlist And Command Evidence

## Changes

- `z.py`: added `.pyi` to `ALLOWED_TEXT_EXTENSIONS`.
- `z.py`: added `py.typed` to `ALLOWED_BASENAMES`.
- `z.py`: added narrow context evidence inclusion for `.codex-runs/**/commands_run.log` and `.codex-runs/**/commands_run.receipts.jsonl` in `next-codex-context`, `codex-run-full`, and `audit-full`.
- `z.py`: added strict context package evidence checks for required Python sidecar files and command evidence.

## Commands

- `python3 -m py_compile z.py scripts/assert_source_package_hygiene.py scripts/test_zpy_hygiene_regression.py scripts/assert_no_boundary_drift.py` - passed.
- `python3 scripts/test_zpy_hygiene_regression.py` - initially failed on root Markdown ambiguity for package artifacts; repaired by deferring those files to root package archival.
- `python3 scripts/test_zpy_hygiene_regression.py` - passed after repair.

## Manual Guardrail

1. Source-of-truth owner: package inclusion policy remains in `z.py`; validator expectations remain in `scripts/assert_source_package_hygiene.py`.
2. No duplicate codec/compression/runtime abstraction introduced.
3. No schema widening, shape coercion, hidden fallback, or fake compatibility layer introduced.
4. Material operation changed: context package file inclusion. Evidence remains subject to text validation and secret scanning.
5. Exact fallback/compressed fixtures not touched.
6. Optional adapters not touched.
7. Fixture regression proves `_native.pyi`, `py.typed`, and `.codex-runs/current/commands_run.log` manifest inclusion.
8. Initial regression failure recorded and repaired.
9. Scope exclusions hold.
10. Rollback path: revert `z.py` allowlist/context evidence edits.
