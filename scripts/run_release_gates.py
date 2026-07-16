#!/usr/bin/env python3
"""Read-only verification for source-bound release evidence.

Recording gate outcomes is deliberately performed only by
`scripts/record_release_evidence.py`; this command never writes evidence,
logs, receipts, or source files.
"""
from __future__ import annotations

import argparse
import json
from pathlib import Path

from evidence_common import REQUIRED_BINDING_FIELDS, verify_binding

ROOT = Path(__file__).resolve().parent.parent
EVIDENCE_MANIFEST = ROOT / "STATUS_EVIDENCE_MANIFEST.json"
RECEIPT_PATH = ROOT / "release" / "closeout_receipt_v1.json"


def main() -> int:
    parser = argparse.ArgumentParser(description="Read-only release evidence verifier")
    parser.add_argument("--repo", type=Path, default=ROOT)
    args = parser.parse_args()
    repo = args.repo.resolve()
    findings: list[str] = []

    manifest_path = repo / EVIDENCE_MANIFEST.name
    receipt_path = repo / "release" / RECEIPT_PATH.name
    if not manifest_path.is_file():
        findings.append("missing STATUS_EVIDENCE_MANIFEST.json")
    if not receipt_path.is_file():
        findings.append("missing release/closeout_receipt_v1.json")
    if findings:
        print(json.dumps({"schema_version": "libraries.evidence-verification.v1", "findings": findings}, indent=2))
        return 1

    manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    receipt = json.loads(receipt_path.read_text(encoding="utf-8"))
    for field in ("snapshot", "captured_at"):
        if manifest.get(field) != receipt.get(field):
            findings.append(f"{field} mismatch")
    manifest_results = {item.get("command"): item.get("result") for item in manifest.get("proof_results", [])}
    if manifest_results != receipt.get("gate_results"):
        findings.append("gate result mismatch")

    binding = manifest.get("source_binding")
    if not isinstance(binding, dict):
        findings.append("missing source_binding")
    else:
        missing = sorted(REQUIRED_BINDING_FIELDS - set(binding))
        if missing:
            findings.append("missing source_binding fields: " + ", ".join(missing))
        if receipt.get("source_binding") != binding:
            findings.append("receipt source_binding mismatch")
        if not missing:
            findings.extend(verify_binding(repo, binding))

    print(json.dumps({"schema_version": "libraries.evidence-verification.v1", "findings": findings}, indent=2))
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
