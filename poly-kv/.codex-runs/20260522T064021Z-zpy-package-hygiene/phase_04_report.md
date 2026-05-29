# Phase 04 Report - Validators And Regression Tests

## Changes

- `scripts/assert_source_package_hygiene.py`: now validates required paths in both manifest JSON and the generated ZIP.
- `scripts/assert_source_package_hygiene.py`: still validates command evidence and `root_package_archive` summary presence/errors.
- `scripts/test_zpy_hygiene_regression.py`: now checks ZIP contents, root package archive manifest existence, and invokes the package hygiene validator in the temp fixture.
- `scripts/assert_no_boundary_drift.py`: replaced literal `0x08` regex bytes with `\\b` word-boundary escapes.

## Commands

- `python3 -m py_compile z.py scripts/assert_source_package_hygiene.py scripts/test_zpy_hygiene_regression.py scripts/assert_no_boundary_drift.py` - passed.
- `python3 scripts/assert_no_boundary_drift.py` - passed.
- `python3 scripts/test_zpy_hygiene_regression.py` - passed.

## Manual Guardrail

1. Source-of-truth owner: validators remain under `scripts/`; package behavior remains in `z.py`.
2. No duplicate abstraction or shadow implementation introduced.
3. No schema widening, shape coercion, hidden fallback, or fake compatibility layer introduced.
4. Validator material operations are read-only except temp fixture creation by the regression test.
5. Exact fallback/compressed fixtures not touched.
6. Optional adapters not touched.
7. Validators now prove manifest and ZIP self-containment for required Python sidecar and command evidence.
8. No skipped/failed phase 04 checks remain.
9. Scope exclusions hold.
10. Rollback path: revert validator edits and restore the previous regex bytes if needed.
