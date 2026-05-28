#!/usr/bin/env python3
"""Static checks for P23 z.py contract."""
from pathlib import Path
import re, sys
z = Path(sys.argv[1] if len(sys.argv) > 1 else "z.py")
text = z.read_text(errors="replace")
errors = []
if "CURRENT_P22_PHASE_PROMPTS" in text:
    errors.append("P22-specific CURRENT_P22_PHASE_PROMPTS still present")
if re.search(r'--codex-current-run"[^\n]+default="P22"', text):
    errors.append("--codex-current-run still defaults to P22")
for mode in ["release-context", "next-codex-context", "audit-full"]:
    if mode not in text:
        errors.append(f"missing package mode or documented equivalent: {mode}")
if "script-ref-not-archived" not in text and "script-ref-excluded" not in text:
    errors.append("script reference inclusion/exclusion checks not evident")
if "CODEX_ARTIFACT_CLASSIFICATION" not in text:
    errors.append("classification registry not evident")
if errors:
    print("FAIL: z.py P23 contract issues:", file=sys.stderr)
    for e in errors: print("  - " + e, file=sys.stderr)
    sys.exit(2)
print("ok: z.py static P23 contract checks passed")
