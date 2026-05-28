#!/usr/bin/env python3
from __future__ import annotations

import fnmatch
import json
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
PROTECTED = {
    "AGENTS.md", "README.md", "SOURCE_BASIS.md", "STATUS.md", "SUPPORT_PROFILE.md",
    "Cargo.toml", "Cargo.lock", "Makefile",
}
ACTIVE_ALLOWED = {
    "SHADOW_SEMANTICS_AUDIT.md",
}
NOISE_PATTERNS = [
    "*AUDIT*.MD", "*HARD_AUDIT*.MD", "*ISSUE_MATRIX*.MD", "*RISK_REGISTER*.MD",
    "*PROMPT*.MD", "*MASTER*.MD", "*SNAPSHOT*.MD", "*STATUS_DASHBOARD*.MD",
    "*IMPLEMENTATION_PLAYBOOK*.MD", "*CONFORMANCE*.MD", "*HARDENING*.MD", "*PLAN*.MD",
    "*TENSOR*.MD", "P[0-9]*_*.MD", "*_CODEX_*.MD", "*_PHASE_*.MD",
]
RUN_RE = re.compile(r"^P\d+[A-Z]?[_-]", re.I)


def active_run() -> str:
    if LEDGER.exists():
        data = json.loads(LEDGER.read_text(encoding="utf-8"))
        return str(data.get("active_run", "")).upper()
    return "P31A"


def main() -> int:
    active = active_run()
    errors: list[str] = []
    ambiguous: list[str] = []
    stale_active: list[str] = []
    for p in ROOT.glob("*.md"):
        name = p.name
        upper = name.upper()
        if name in PROTECTED or name in ACTIVE_ALLOWED:
            continue
        if upper.startswith(active + "_") or upper.startswith(active + "-"):
            # Active run root markdown is still discouraged; require classification file to own it.
            ambiguous.append(name)
            continue
        if RUN_RE.match(name):
            stale_active.append(name)
            continue
        if any(fnmatch.fnmatch(upper, pat) for pat in NOISE_PATTERNS):
            ambiguous.append(name)
    if stale_active:
        errors.append("stale root run Markdown artifacts must be archived/classified, not active:\n  " + "\n  ".join(sorted(stale_active)[:300]))
    if ambiguous:
        errors.append("ambiguous root Markdown artifacts remain active:\n  " + "\n  ".join(sorted(ambiguous)[:300]))
    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        return 2
    print("PASS: root Markdown archive policy clean")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
