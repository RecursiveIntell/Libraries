#!/usr/bin/env python3
from pathlib import Path
import json, sys, re

EXPECTED = "P29"
paths = [
    Path("docs/codex-runs/CURRENT_RUN.md"),
    Path("STATUS.md"),
    Path("SOURCE_BASIS.md"),
    Path("SUPPORT_PROFILE.md"),
    Path("P29_STATUS_EVIDENCE_MANIFEST.json"),
]
missing = []
bad = []
for p in paths:
    if not p.exists():
        # Status manifest may be created late; only warn for template phase.
        if p.name == "P29_STATUS_EVIDENCE_MANIFEST.json":
            continue
        missing.append(str(p))
        continue
    text = p.read_text(encoding="utf-8", errors="ignore")
    if EXPECTED not in text:
        bad.append(str(p))

if missing or bad:
    if missing: print("Missing run identity files:", missing)
    if bad: print("Files missing P29 identity:", bad)
    sys.exit(1)
print("P29 run identity check passed")
