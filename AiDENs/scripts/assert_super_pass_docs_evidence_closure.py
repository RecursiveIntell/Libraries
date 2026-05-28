#!/usr/bin/env python3
"""Assert super-pass docs/evidence registers are present and label-honest."""

from pathlib import Path
import csv
import json
import sys


REQUIRED_PATHS = [
    "docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md",
    "docs/super-pass/SUPPORT_TRACEABILITY.md",
    "handoffs/super-pass/FINAL_AUDITOR_HANDOFF.md",
    "target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json",
]

REQUIRED_TEXT = {
    "STATUS.md": [
        "clean source bundle is accepted as source basis",
        "not a product-conformance or release-package claim",
        "zip-byte hashes",
    ],
    "SUPPORT_PROFILE.md": [
        "skipped post-bundle operator gate is not counted as a product defect",
        "Regenerated package sidecars and extracted-package self-replay",
        "do not widen support labels",
    ],
    "SOURCE_BASIS.md": [
        "minimal executable seed only",
        "reserved/quarantined",
        "final package sidecars and extracted-package self-replay",
    ],
}

FORBIDDEN_CLAIMS = [
    "v11B-complete",
    "v11C-complete",
    "production-cloud-ready",
    "broad-autonomy-ready",
    "canonical-truth-owner",
]


def main() -> int:
    failures = []
    for rel in REQUIRED_PATHS:
        if not Path(rel).exists():
            failures.append(f"missing required evidence path: {rel}")

    for rel, needles in REQUIRED_TEXT.items():
        path = Path(rel)
        text = path.read_text(encoding="utf-8", errors="ignore") if path.exists() else ""
        for needle in needles:
            if needle not in text:
                failures.append(f"{rel} missing required text: {needle}")

    hash_manifest = Path("target/super-pass/audit/PHASE_15_AUDIT_LOG_HASHES.json")
    if hash_manifest.exists():
        data = json.loads(hash_manifest.read_text())
        if data.get("entry_count", 0) <= 0:
            failures.append("audit hash manifest contains no log entries")
        for entry in data.get("entries", []):
            if len(entry.get("sha256", "")) != 64:
                failures.append(f"invalid sha256 entry: {entry.get('path')}")

    known_limits = Path("docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md").read_text(
        encoding="utf-8", errors="ignore"
    ) if Path("docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md").exists() else ""
    for required_id in ["CLAUDE-F-001", "CLAUDE-F-003", "CLAUDE-F-004", "CLAUDE-F-017", "CLAUDE-F-020"]:
        if required_id not in known_limits:
            failures.append(f"known limitations register missing linked row {required_id}")

    with Path("matrices/P29_MASTER_ISSUE_MATRIX.csv").open(newline="") as f:
        p29_rows = list(csv.DictReader(f))
    if any(row.get("Status") == "open" for row in p29_rows):
        failures.append("P29 master issue matrix still contains raw open rows")

    claim_text = "\n".join(
        path.read_text(encoding="utf-8", errors="ignore")
        for path in [
            Path("STATUS.md"),
            Path("SUPPORT_PROFILE.md"),
            Path("SOURCE_BASIS.md"),
            Path("docs/super-pass/KNOWN_LIMITATIONS_REGISTER.md"),
            Path("docs/super-pass/SUPPORT_TRACEABILITY.md"),
            Path("handoffs/super-pass/FINAL_AUDITOR_HANDOFF.md"),
        ]
        if path.exists()
    ).lower()
    for label in FORBIDDEN_CLAIMS:
        idx = claim_text.find(label.lower())
        if idx >= 0:
            context = claim_text[max(0, idx - 80): idx + 120]
            if not any(word in context for word in ["not", "no ", "forbidden", "do not accept", "must not"]):
                failures.append(f"forbidden label appears as a claim: {label}")

    if failures:
        print("super-pass docs/evidence closure FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1
    print("super-pass docs/evidence closure OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
