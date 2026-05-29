#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
DOCS = [
    ROOT / "README.md",
    ROOT / "STATUS.md",
    ROOT / "SOURCE_BASIS.md",
    ROOT / "SUPPORT_PROFILE.md",
    ROOT / "docs" / "codex-runs" / "CURRENT_RUN.md",
]
POSITIVE_PATTERNS = [
    ("build_certified", re.compile(r"\b(build|cargo|workspace)\b.{0,80}\b(certified|passed|green|verified)\b", re.I | re.S)),
    ("package_certified", re.compile(r"\b(package|sidecar|manifest)\b.{0,80}\b(certified|passed|green|verified)\b", re.I | re.S)),
    ("extracted_replay_certified", re.compile(r"\b(extracted|self[- ]?replay|replay)\b.{0,80}\b(certified|passed|green|verified)\b", re.I | re.S)),
]
FORBIDDEN_PHRASES = [
    "fully v11a compliant",
    "v11b complete",
    "production cloud ready",
    "complete autonomous platform",
]

# Exact-forbidden: these trigger even inside negations, so use exclusion terms instead
EXACT_FORBIDDEN = [
    "production-cloud-ready",
]


def is_forbidden(text_lower: str) -> list[str]:
    hits = []
    for phrase in FORBIDDEN_PHRASES:
        if phrase in text_lower:
            hits.append(phrase)
    # "production-cloud-ready" only triggers if NOT preceded by "not " or similar negation
    for phrase in EXACT_FORBIDDEN:
        idx = 0
        while True:
            pos = text_lower.find(phrase, idx)
            if pos == -1:
                break
            # Check if preceded by a negation prefix
            start = max(0, pos - 4)
            prefix = text_lower[start:pos].rstrip()
            if prefix.endswith("not") or prefix.endswith("no ") or prefix.endswith("non-"):
                idx = pos + len(phrase)
                continue
            hits.append(phrase)
            break
    return hits


def fail(errors: list[str]) -> int:
    for e in errors:
        print(f"FAIL: {e}", file=sys.stderr)
    return 2


def main() -> int:
    if not LEDGER.exists():
        return fail([f"missing {LEDGER.relative_to(ROOT)}"])
    data = json.loads(LEDGER.read_text(encoding="utf-8"))
    active = str(data.get("active_run", "")).upper()
    last = str(data.get("last_certified_run", "")).upper()
    status = str(data.get("certification_status", ""))
    support = str(data.get("support_label", ""))
    errors: list[str] = []

    for doc in DOCS:
        if not doc.exists():
            errors.append(f"missing doc {doc.relative_to(ROOT)}")
            continue
        text = doc.read_text(encoding="utf-8", errors="replace")
        low = text.lower()
        for phrase in is_forbidden(low):
            errors.append(f"{doc.relative_to(ROOT)} contains forbidden support/release phrase: {phrase}")
        for expected in [active, last, status, support]:
            if expected and expected not in text and expected.lower() not in low:
                errors.append(f"{doc.relative_to(ROOT)} missing ledger value {expected!r}")
        for key, rx in POSITIVE_PATTERNS:
            if rx.search(text) and data.get(key) is not True:
                errors.append(f"{doc.relative_to(ROOT)} appears to claim {key} but ledger has {key}=false")

    if errors:
        return fail(errors)
    print("PASS: release truth docs are consistent with CURRENT_RUN.json")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
