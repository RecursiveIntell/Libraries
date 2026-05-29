#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
DOCS = [ROOT / "README.md", ROOT / "STATUS.md", ROOT / "SUPPORT_PROFILE.md"]
FORBIDDEN = [
    "complete autonomous platform",
    "fully verified",
    "fully v11a compliant",
    "v11b complete",
    "production cloud ready",
]

# Negation-safe forbidden: trigger only if NOT preceded by "not " or similar
EXACT_FORBIDDEN = [
    "production-cloud-ready",
]


def is_forbidden_claim(text_lower: str) -> list[str]:
    hits = []
    for phrase in FORBIDDEN:
        if phrase in text_lower:
            hits.append(phrase)
    for phrase in EXACT_FORBIDDEN:
        idx = 0
        while True:
            pos = text_lower.find(phrase, idx)
            if pos == -1:
                break
            start = max(0, pos - 4)
            prefix = text_lower[start:pos].rstrip()
            if prefix.endswith("not") or prefix.endswith("no ") or prefix.endswith("non-"):
                idx = pos + len(phrase)
                continue
            hits.append(phrase)
            break
    return hits
REQUIRED_DISCLOSURES = [
    "supported-local-candidate",
    "not production-cloud-ready",
    "CURRENT_RUN.json",
]


def main() -> int:
    errors: list[str] = []
    if not LEDGER.exists():
        errors.append(f"missing {LEDGER.relative_to(ROOT)}")
        data = {}
    else:
        data = json.loads(LEDGER.read_text(encoding="utf-8"))
    evidence = data.get("evidence", {}) if isinstance(data.get("evidence"), dict) else {}

    for key, required in {
        "build_certified": ["cargo_metadata_log", "fmt_log", "check_log", "test_log", "clippy_log", "final_verify_log"],
        "package_certified": ["package_manifest", "package_findings", "package_report"],
        "extracted_replay_certified": ["package_replay_receipt"],
    }.items():
        if data.get(key) is True:
            for ref in required:
                value = evidence.get(ref)
                if not isinstance(value, str) or not value.strip():
                    errors.append(f"{key}=true requires evidence.{ref}")

    combined = ""
    for doc in DOCS:
        if not doc.exists():
            errors.append(f"missing {doc.relative_to(ROOT)}")
            continue
        text = doc.read_text(encoding="utf-8", errors="replace")
        combined += "\n" + text
        low = text.lower()
        for phrase in is_forbidden_claim(low):
                errors.append(f"{doc.relative_to(ROOT)} contains forbidden claim: {phrase}")
    low_combined = combined.lower()
    for marker in REQUIRED_DISCLOSURES:
        if marker.lower() not in low_combined:
            errors.append(f"public support docs missing disclosure marker: {marker}")

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        return 2
    print("PASS: support claims are evidence-bounded")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
