# Phase 03 Report - Strict Hygiene Gates

## Changes

- `z.py`: strict context package checks now fail when existing `python/poly_kv/_native.pyi` or `python/poly_kv/py.typed` are absent from package inclusion.
- `z.py`: strict context package checks now fail when no `.codex-runs/**/commands_run.log` or `.codex-runs/**/commands_run.receipts.jsonl` is included.
- `z.py`: strict packaging fails when root package artifacts remain after archival normalization.
- `z.py`: `root_package_archive` is included in console output, markdown report summary, `report`, and top-level manifest JSON.

## Commands

- `python3 -m py_compile z.py scripts/assert_source_package_hygiene.py scripts/test_zpy_hygiene_regression.py scripts/assert_no_boundary_drift.py` - passed.
- `python3 scripts/test_zpy_hygiene_regression.py` - passed.

## Manual Guardrail

1. Source-of-truth owner: `z.py` owns package strict gates; `scripts/assert_source_package_hygiene.py` independently validates generated outputs.
2. No duplicate compression, receipt, fallback, or runtime authority introduced.
3. No schema widening, shape coercion, hidden fallback, or compatibility layer introduced.
4. Material package actions now emit manifest/report summaries and archive manifests.
5. Exact fallback/compressed fixtures not touched.
6. Optional adapters not touched.
7. Regression test covers strict package success for required files and command evidence.
8. No skipped/failed phase 03 checks remain.
9. Scope exclusions hold.
10. Rollback path: revert `z.py` strict gate/report integration edits.
