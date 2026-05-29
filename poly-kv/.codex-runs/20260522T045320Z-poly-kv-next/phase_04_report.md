# Phase 04 Report

## Scope

Added optional PyO3/maturin Python sidecar skeleton. Rust core crates remain independent from Python dependencies.

## Changed files

- `Cargo.toml`
- `Cargo.lock`
- `crates/poly-kv-python/Cargo.toml`
- `crates/poly-kv-python/src/lib.rs`
- `pyproject.toml`
- `python/poly_kv/__init__.py`
- `python/poly_kv/_native.pyi`
- `python/poly_kv/py.typed`
- `python/poly_kv/exceptions.py`
- `python/poly_kv/receipts.py`
- `python/poly_kv/adapters/__init__.py`
- `python/tests/test_import.py`
- `python/tests/test_receipt_parity.py`
- `python/tests/test_shape_rejection.py`
- `python/tests/test_no_silent_copy.py`

## Implementation

- Added workspace member `crates/poly-kv-python`.
- Added PyO3 native extension target `poly_kv._native`.
- Added handwritten stub file `_native.pyi` and `py.typed`.
- Added Python custom exceptions.
- Added bulk-oriented Python wrappers; no daemon mode or persistent background service.
- Added import smoke test and native-dependent tests that skip explicitly if the extension is unavailable.
- Native sidecar functions return JSON-compatible receipts for shape validation, synthetic build, reader attach, decode, and CPU f32 JSON fixture build.

## Boundary check

Read `docs/SOURCE_OF_TRUTH_MAP.md` per the Rust crate boundary skill. PyO3/maturin dependencies are isolated to `crates/poly-kv-python`; `crates/quant-codec-core/Cargo.toml` and `crates/poly-kv/Cargo.toml` do not depend on Python binding packages.

## Validation

Commands and results:

- `cargo search pyo3 --limit 1`: pass; latest reported `pyo3 = "0.28.3"`.
- `cargo fmt --all`: pass
- `cargo check -p poly-kv-python`: pass
- `python -m compileall python`: pass
- `python3 scripts/assert_python_sidecar_layout.py`: pass
- `PYTHONPATH=python python -m pytest -q python/tests/test_import.py`: pass
- `python -m maturin --version`: fail, `No module named maturin`
- `python -m maturin build`: fail/skip, `No module named maturin`

## Skips

`maturin build` is recorded as skipped for this environment because the `maturin` Python module is not installed. This blocks wheel/build validation, but not Rust crate compilation or pure Python import validation.
