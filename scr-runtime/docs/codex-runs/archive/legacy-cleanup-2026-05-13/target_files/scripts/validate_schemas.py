#!/usr/bin/env python3
import json
from pathlib import Path
import sys

root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
schema_dir = root / "schemas" / "generated"

if not schema_dir.exists():
    print("FAIL: schemas/generated does not exist")
    sys.exit(1)

schemas = sorted(schema_dir.glob("*.json"))
if not schemas:
    print("FAIL: no generated schemas found")
    sys.exit(1)

ok = True
for path in schemas:
    try:
        json.loads(path.read_text(encoding="utf-8"))
        print(f"PASS schema json: {path}")
    except Exception as exc:
        print(f"FAIL schema json: {path}: {exc}")
        ok = False

sys.exit(0 if ok else 1)
