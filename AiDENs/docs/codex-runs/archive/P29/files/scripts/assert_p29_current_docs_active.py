#!/usr/bin/env python3
from pathlib import Path
import sys

required = [
    "P29_STATUS_EVIDENCE_MANIFEST.json",
    "docs/p29/P29_FINAL_AUDIT_REPORT.md",
    "docs/p29/P29_KNOWN_LIMITATIONS_REGISTER.md",
    "docs/p29/P29_SUPPORT_TRACEABILITY.md",
    "handoffs/p29/FINAL_AUDITOR_HANDOFF.md",
    "scripts/p29_verify.sh",
]
missing = [p for p in required if not Path(p).exists()]
if missing:
    print("Missing active P29 docs/scripts:")
    for p in missing: print(" -", p)
    sys.exit(1)
print("active P29 docs/scripts present")
