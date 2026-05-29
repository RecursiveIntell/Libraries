# Python Sidecar Spec

## Architecture

First sidecar is in-process, not daemon-first:

```text
Python user code
→ poly_kv package
→ poly_kv._native PyO3 extension
→ poly-kv Rust crate
→ SharedKvPool / PoolReader / receipts
```

## Package layout

```text
pyproject.toml
crates/poly-kv-python/
  Cargo.toml
  src/lib.rs
python/poly_kv/
  __init__.py
  _native.pyi
  py.typed
  exceptions.py
  receipts.py
  adapters/
    __init__.py
python/tests/
  test_import.py
  test_receipt_parity.py
  test_shape_rejection.py
  test_no_silent_copy.py
```

## Packaging rules

- Build with PyO3 + maturin mixed layout.
- Public import namespace: `poly_kv`.
- Native extension: `poly_kv._native`.
- Do not use abi3 in the first fast path if Python 3.10 + buffer protocol support matters.
- Ship handwritten `.pyi` and `py.typed`.
- Do not implement daemon mode in this pass.

## Current PyO3 rules

- First sidecar surface is bulk JSON-compatible functions in `poly_kv._native`.
- Rust core crates must not depend on PyO3 or maturin.
- Do not implement daemon mode in this pass.
- Map Rust errors through custom Python exceptions where Python wrappers are used:
  - `PolyKvError`
  - `PolyKvNativeUnavailable`
  - `PolyKvShapeError`
- Treat owned-copy disclosure as required receipt data.

## API surface

Bulk-only first pass:

```python
import poly_kv

shape = poly_kv.ShapeV2(
    batch=1,
    layers=2,
    num_q_heads=2,
    num_kv_heads=2,
    seq_len=8,
    head_dim=4,
)
receipts = poly_kv.build_synthetic_pool(shape)
decoded = poly_kv.decode_synthetic_slice(
    shape, role="key", layer=0, start=0, end=2
)
```

## Receipt requirements

Every Python operation that crosses a material boundary emits or returns a JSON-compatible receipt with:

- input dtype/layout
- copy performed
- shape validation result
- manifest digest
- encoded bytes
- fallback state
- errors/degradation

## Data interchange

- CPU f32 JSON fixture path exists for this pass.
- memoryview / buffer protocol / NumPy validation is planned.
- Tensor exchange: DLPack is deferred.
- Arrow: defer to metadata/reporting only.
- mmap/shared memory: defer to later daemon/persistent artifact pass.

## Adapter boundary

No Hugging Face, PyTorch, vLLM, llama.cpp, Candle, Burn, or tch adapter is implemented in this pass.
