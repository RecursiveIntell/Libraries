#!/usr/bin/env python3
"""P26 no unsupported final-vision claim assertion template."""
from pathlib import Path
import re, sys
bad = [
    r"complete autonomous",
    r"V10.*supported",
    r"federated.*supported",
]
violations=[]
for p in [Path("STATUS.md"), Path("SUPPORT_PROFILE.md")]:
    if not p.exists():
        continue
    text=p.read_text(errors="ignore").lower()
    for pat in bad:
        if re.search(pat.lower(), text):
            violations.append((str(p), pat))
if violations:
    print("unsupported claim violations:", violations)
    sys.exit(1)
print("unsupported final-vision claim check: pass")
