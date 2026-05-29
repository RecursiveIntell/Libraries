#!/usr/bin/env python3
from pathlib import Path
import sys

required = [
    'pyproject.toml',
    'python/poly_kv/__init__.py',
    'python/poly_kv/_native.pyi',
    'python/poly_kv/py.typed',
    'python/tests/test_import.py',
]
missing = [p for p in required if not Path(p).exists()]
if missing:
    print('missing Python sidecar files:')
    for p in missing:
        print(' -', p)
    sys.exit(1)

pyproject = Path('pyproject.toml').read_text(encoding='utf-8', errors='ignore')
checks = ['maturin', 'poly_kv._native', 'python-source']
for c in checks:
    if c not in pyproject:
        print(f'pyproject.toml missing expected token: {c}')
        sys.exit(1)
print('python sidecar layout ok')
