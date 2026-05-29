# Phase 05 — Python interop receipts and tests

Implement minimal Python-visible operations:

- import package;
- construct shape;
- build pool from CPU data fixture;
- attach reader;
- decode layer/slice;
- return JSON-compatible receipts;
- reject bad shapes/dtypes;
- disclose copy behavior.

Add tests:

```bash
python -m pytest -q python/tests/test_import.py
python -m pytest -q python/tests/test_receipt_parity.py
python -m pytest -q python/tests/test_shape_rejection.py
python -m pytest -q python/tests/test_no_silent_copy.py
```

If NumPy/PyTorch are unavailable, tests must skip explicitly and write skip reasons.

Gate: no silent copy or zero-copy claim without receipt flags.
