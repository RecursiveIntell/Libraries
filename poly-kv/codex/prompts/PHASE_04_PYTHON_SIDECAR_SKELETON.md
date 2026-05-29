# Phase 04 — Python sidecar skeleton

Add optional sidecar layout:

```text
crates/poly-kv-python/
  Cargo.toml
  src/lib.rs
pyproject.toml
python/poly_kv/
  __init__.py
  _native.pyi
  py.typed
  exceptions.py
  receipts.py
  adapters/__init__.py
python/tests/test_import.py
```

Rules:

- Use PyO3 + maturin.
- Native extension is `poly_kv._native`.
- Rust core crates must not depend on PyO3.
- Handwrite stubs.
- Create custom exceptions.
- Expose only bulk APIs.
- Do not implement daemon mode.

Gate: `python -m compileall python` and `maturin build` or `maturin develop` documented; if maturin unavailable, record skip reason.
