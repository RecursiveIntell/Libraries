#!/usr/bin/env python3
import csv
import subprocess
import sys
from pathlib import Path

ROOT = Path.cwd()
OUT_DIR = ROOT / "docs" / "contract-ownership"
script = ROOT / "scripts" / "make_type_ownership_inventory.py"

if not script.exists():
    print("error: scripts/make_type_ownership_inventory.py missing", file=sys.stderr)
    sys.exit(2)

subprocess.run([sys.executable, str(script)], check=True)

findings_path = OUT_DIR / "CANONICAL_DUPLICATE_FINDINGS.csv"
if not findings_path.exists():
    print("error: duplicate findings CSV missing", file=sys.stderr)
    sys.exit(2)

rows = list(csv.DictReader(findings_path.open()))
blocking = [r for r in rows if r.get("severity") in {"P0", "P1_REVIEW"}]

if blocking:
    print("FAIL: local aidens-contracts public definitions duplicate canonical public type names.")
    for r in blocking:
        print(f"{r['severity']}: {r['type_name']} local {r['aidens_file']}:{r['aidens_line']} duplicates {r['canonical_owner']} {r['canonical_file']}:{r['canonical_line']}")
    print("Fix by deleting the local definition or converting it to an explicit canonical pub use. Do not add compatibility shims.")
    sys.exit(1)

print("PASS: no local aidens-contracts public type definitions duplicate canonical public type names.")
