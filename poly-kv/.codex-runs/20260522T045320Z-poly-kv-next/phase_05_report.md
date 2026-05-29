# Phase 05 Report

## Scope

Validated Python interop receipt tests and skip behavior for native-only operations.

## Python-visible operations implemented

In `python/poly_kv/receipts.py` with native implementations in `crates/poly-kv-python/src/lib.rs`:

- import package without requiring the extension;
- construct typed `ShapeV2`;
- validate shape via native JSON bulk API;
- build synthetic pool and return manifest/build/reader receipts;
- attach synthetic reader and return injection receipt;
- decode synthetic slice and return decode receipt;
- build pool from CPU f32 JSON fixture and return receipts;
- reject unsupported shapes/dtypes/layouts through custom Python exceptions;
- disclose copy behavior through decode receipts.

## Validation

Commands and results:

- `PYTHONPATH=python python -m pytest -q python/tests/test_import.py`: pass
- `PYTHONPATH=python python -m pytest -q python/tests/test_receipt_parity.py`: skipped, `_native` not built
- `PYTHONPATH=python python -m pytest -q python/tests/test_shape_rejection.py`: skipped, `_native` not built
- `PYTHONPATH=python python -m pytest -q python/tests/test_no_silent_copy.py`: skipped, `_native` not built
- `PYTHONPATH=python python -m pytest -q python/tests`: pass overall, 1 passed and 3 skipped

## Skips

Native interop execution is skipped because `maturin` is unavailable and `poly_kv._native` is not installed in this Python environment. The tests skip with the explicit message: `poly_kv._native is not built; run maturin develop or maturin build`.
