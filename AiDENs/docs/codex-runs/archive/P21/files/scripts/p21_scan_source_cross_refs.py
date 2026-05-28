#!/usr/bin/env python3
"""P21 source cross-reference scanner.

Catches common package drift: code referencing root scripts/evals/fixtures absent from source.
"""
from __future__ import annotations
import json
import re
import sys
from pathlib import Path

root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
patterns = [
    re.compile(r'"((?:scripts|evals|fixtures|tests/fixtures|examples)/[^"\s]+)"'),
    re.compile(r"'((?:scripts|evals|fixtures|tests/fixtures|examples)/[^'\s]+)'"),
]
missing = []
seen = set()
for path in list(root.rglob("*.rs")) + list(root.rglob("*.sh")) + list(root.rglob("*.py")) + list(root.rglob("*.md")) + list(root.rglob("*.toml")):
    if any(part in {"target", ".git"} for part in path.parts):
        continue
    try:
        text = path.read_text(encoding="utf-8")
    except Exception:
        continue
    for pat in patterns:
        for m in pat.finditer(text):
            ref = m.group(1).rstrip(".,);]")
            if "$" in ref or "{" in ref or "<" in ref:
                continue
            # Ignore intentionally generic example paths.
            if ref.endswith("/"):
                continue
            key = (str(path.relative_to(root)), ref)
            if key in seen:
                continue
            seen.add(key)
            if not (root / ref).exists():
                missing.append({"source": str(path.relative_to(root)), "ref": ref})
report = {"root": str(root), "missing_cross_refs": missing, "ok": not missing}
print(json.dumps(report, indent=2, sort_keys=True))
if missing:
    sys.exit(1)
