#!/usr/bin/env python3
from pathlib import Path
import sys

forbidden = [
    "v11b-complete",
    "v11c-complete",
    "production-cloud-ready",
    "broad-autonomy-ready",
    "canonical memory truth owner",
]
claim_surfaces = [
    Path("STATUS.md"),
    Path("SUPPORT_PROFILE.md"),
    Path("SOURCE_BASIS.md"),
    Path("P29_STATUS_EVIDENCE_MANIFEST.json"),
    *Path("docs/p29").glob("**/*.md"),
    *Path("handoffs/p29").glob("**/*.md"),
]
policy_allowlist = {
    "P29_FORBIDDEN_FINAL_STATE.md",
    "P29_SUPPORT_LABEL_POLICY.md",
    "P29_ACCEPTANCE_GATES.md",
    "P29_MASTER_PACKET.md",
}

bad = []
for p in claim_surfaces:
    if not p.exists() or p.name in policy_allowlist:
        continue
    text = p.read_text(encoding="utf-8", errors="ignore").lower()
    for f in forbidden:
        idx = text.find(f)
        if idx >= 0 and "forbidden" not in text[max(0, idx - 80):idx + 120] and "not " not in text[max(0, idx - 20):idx]:
            bad.append((str(p), f))
if bad:
    print("Forbidden claims found:")
    for p,f in bad: print(f" - {p}: {f}")
    sys.exit(1)
print("no forbidden claims found")
