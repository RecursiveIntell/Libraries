#!/usr/bin/env python3
from pathlib import Path
p = Path(__file__).resolve().parents[1] / "docs/provekv/PROVEKV_EVIDENCE_INDEX.md"
s = p.read_text()
for term in ["11.1304x", "4.7607620871", "7.19x", "JSON", "GTX 1070"]:
    if term not in s:
        raise SystemExit(f"missing evidence marker: {term}")
print("provekv evidence index: PASS")
