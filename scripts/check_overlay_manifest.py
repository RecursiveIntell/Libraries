#!/usr/bin/env python3
import json
from pathlib import Path
import sys

kit_root = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(__file__).resolve().parents[1]
manifest = json.loads((kit_root / "11_CHANGED_FILE_MANIFEST.json").read_text())
missing = []
for rel in manifest["changed_files"]:
    if not (kit_root / "repo_overlay" / rel).exists():
        missing.append(rel)
if missing:
    print("missing overlay files:")
    for rel in missing:
        print(rel)
    raise SystemExit(1)
print("overlay manifest check passed")
