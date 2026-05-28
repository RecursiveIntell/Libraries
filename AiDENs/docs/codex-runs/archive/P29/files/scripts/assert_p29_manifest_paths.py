#!/usr/bin/env python3
from pathlib import Path
import json, sys

manifest = Path("P29_STATUS_EVIDENCE_MANIFEST.json")
if not manifest.exists():
    print("P29_STATUS_EVIDENCE_MANIFEST.json not present yet; skipping until final phase")
    sys.exit(0)

data = json.loads(manifest.read_text())
missing = []
def check_path(value):
    if isinstance(value, str):
        if value.startswith("<") or value.startswith("external:") or value.startswith("degraded:"):
            return
        if any(value.endswith(ext) for ext in [".md",".json",".log",".txt",".csv",".sh"]):
            if not Path(value).exists():
                missing.append(value)

def walk(x):
    if isinstance(x, dict):
        for k,v in x.items():
            if k in ("log","evidence","report","manifest","findings","excluded") or k.endswith("_path"):
                if isinstance(v, list):
                    for item in v: check_path(item)
                else:
                    check_path(v)
            walk(v)
    elif isinstance(x, list):
        for i in x: walk(i)
walk(data)

if missing:
    print("Missing manifest paths:")
    for m in sorted(set(missing)):
        print(" -", m)
    sys.exit(1)
print("manifest paths resolved")
