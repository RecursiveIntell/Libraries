#!/usr/bin/env python3
"""P21 package integrity scanner.

Checks required directories/files and compile-time include_str/include_bytes targets.
"""
from __future__ import annotations
import json
import os
import re
import sys
from pathlib import Path

root = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
required_paths = [
    "Cargo.toml",
    "crates/aidens-cli/Cargo.toml",
    "crates/aidens-integration-tests/Cargo.toml",
    "crates/aidens-testkit/Cargo.toml",
    "scripts/verify.sh",
    "scripts/p20_verify.sh",
    "scripts/p20_2_verify.sh",
    "scripts/p21_verify.sh",
    "scripts/p21_scan_package_integrity.py",
    "scripts/p21_scan_source_cross_refs.py",
    "evals/p20_agency_eval_cases.jsonl",
    "fixtures/test-agent/basic-agent.toml",
    "fixtures/runner/expected_test_agent_event_log.ndjson",
]
missing = [p for p in required_paths if not (root / p).exists()]

include_missing = []
include_refs = 0
pattern = re.compile(r'include_(?:str|bytes)!\("([^"]+)"\)')
for rs in root.rglob("*.rs"):
    if any(part in {"target", ".git"} for part in rs.parts):
        continue
    try:
        text = rs.read_text(encoding="utf-8")
    except UnicodeDecodeError:
        text = rs.read_text(errors="ignore")
    for match in pattern.finditer(text):
        include_refs += 1
        target = (rs.parent / match.group(1)).resolve()
        try:
            target.relative_to(root)
        except ValueError:
            pass
        if not target.exists():
            include_missing.append({"source": str(rs.relative_to(root)), "target": match.group(1), "resolved": str(target)})

manifest_missing = []
manifest = root / "MANIFEST.txt"
if manifest.exists():
    for line in manifest.read_text(errors="ignore").splitlines():
        item = line.strip()
        if not item or item.startswith("#") or item.endswith("/"):
            continue
        if not (root / item).exists():
            manifest_missing.append(item)

report = {
    "root": str(root),
    "required_missing": missing,
    "include_refs": include_refs,
    "include_missing": include_missing,
    "manifest_missing": manifest_missing,
    "ok": not missing and not include_missing and not manifest_missing,
}
print(json.dumps(report, indent=2, sort_keys=True))
if not report["ok"]:
    sys.exit(1)
