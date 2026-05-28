#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

INCLUDE_RE = re.compile(r'include_(?:str|bytes)!\s*\(\s*"([^"]+)"\s*\)', re.S)
SKIP_DIRS = {".git", "target"}

REQUIRED_PACKAGE_PATHS = [
    "MANIFEST.json",
    "MANIFEST.txt",
    "docs/P21_PROVIDER_EXPANSION_PLAN.md",
    "docs/p20/P21_PROVIDER_EXPANSION_PLAN.md",
    "evals/p20_agency_eval_cases.jsonl",
    "scripts/assert_no_fake_completion.sh",
    "scripts/assert_no_shadow_truth.sh",
    "scripts/p20_2_generate_audit_bundle.sh",
    "scripts/p20_2_scanner_selftest.py",
    "scripts/p20_2_scan_package_integrity.py",
    "scripts/p20_2_scan_testkit_purity.py",
    "scripts/p20_2_validate_agency_cases.py",
    "scripts/p20_2_run_test_agent.sh",
    "scripts/p20_2_verify.sh",
    "scripts/p20_2_verify_release_zip.sh",
    "fixtures/test-agent/basic-agent.toml",
    "fixtures/test-agent/coding-agent.toml",
    "fixtures/runner/test_agent_basic.json",
    "fixtures/runner/expected_test_agent_event_log.ndjson",
]


def rel(root: Path, path: Path) -> str:
    try:
        return path.resolve().relative_to(root.resolve()).as_posix()
    except ValueError:
        return str(path)


def should_skip(path: Path) -> bool:
    return any(part in SKIP_DIRS or part.startswith("target-") for part in path.parts)


def manifest_txt_entries(path: Path) -> list[str]:
    entries: list[str] = []
    for raw in path.read_text(encoding="utf-8", errors="ignore").splitlines():
        line = raw.strip()
        if not line or line.startswith("#") or line.endswith(":"):
            continue
        if line.startswith("- "):
            line = line[2:].strip()
        entries.append(line)
    return entries


def manifest_json_entries(path: Path) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    try:
        data = json.loads(path.read_text(encoding="utf-8"))
    except Exception as exc:
        return [], [f"invalid json: {exc}"]

    raw_entries = data if isinstance(data, list) else data.get("files", []) if isinstance(data, dict) else []
    if not isinstance(raw_entries, list):
        return [], ["manifest files field must be a list"]

    entries: list[str] = []
    for index, entry in enumerate(raw_entries):
        if isinstance(entry, str):
            entries.append(entry)
        elif isinstance(entry, dict) and isinstance(entry.get("path"), str):
            entries.append(entry["path"])
        else:
            errors.append(f"entry {index} must be string or object with path")
    if isinstance(data, dict) and "file_count" in data:
        declared = data["file_count"]
        if not isinstance(declared, int):
            errors.append("file_count must be an integer")
        elif declared != len(entries):
            errors.append(f"file_count {declared} does not match files length {len(entries)}")
    return entries, errors


def scan(root: Path) -> dict:
    missing_includes = []
    include_count = 0
    for rs in root.rglob("*.rs"):
        if should_skip(rs.relative_to(root)):
            continue
        text = rs.read_text(encoding="utf-8", errors="ignore")
        for include_rel in INCLUDE_RE.findall(text):
            include_count += 1
            target = (rs.parent / include_rel).resolve()
            if not target.exists():
                missing_includes.append(
                    {
                        "file": rel(root, rs),
                        "include": include_rel,
                        "resolved": rel(root, target),
                    }
                )

    missing_required = [path for path in REQUIRED_PACKAGE_PATHS if not (root / path).exists()]
    manifest_missing = []
    manifest_omissions = []
    manifest_errors = []
    manifest_counts = {}

    manifest_specs = []
    txt_manifest = root / "MANIFEST.txt"
    if txt_manifest.exists():
        manifest_specs.append(("MANIFEST.txt", manifest_txt_entries(txt_manifest), []))
    json_manifest = root / "MANIFEST.json"
    if json_manifest.exists():
        entries, errors = manifest_json_entries(json_manifest)
        manifest_specs.append(("MANIFEST.json", entries, errors))

    manifest_entry_sets: dict[str, set[str]] = {}
    for manifest_name, entries, errors in manifest_specs:
        manifest_counts[manifest_name] = len(entries)
        manifest_entry_sets[manifest_name] = set(entries)
        for error in errors:
            manifest_errors.append({"manifest": manifest_name, "error": error})
        entry_set = set(entries)
        if len(entry_set) != len(entries):
            manifest_errors.append({"manifest": manifest_name, "error": "duplicate manifest entries"})
        for entry in entries:
            if not (root / entry).exists():
                manifest_missing.append({"manifest": manifest_name, "entry": entry})
        for required in REQUIRED_PACKAGE_PATHS:
            if required not in entry_set:
                manifest_omissions.append({"manifest": manifest_name, "entry": required})

    if {"MANIFEST.txt", "MANIFEST.json"}.issubset(manifest_entry_sets):
        txt_only = sorted(manifest_entry_sets["MANIFEST.txt"] - manifest_entry_sets["MANIFEST.json"])
        json_only = sorted(manifest_entry_sets["MANIFEST.json"] - manifest_entry_sets["MANIFEST.txt"])
        if txt_only or json_only:
            manifest_errors.append(
                {
                    "manifest": "MANIFEST.txt/MANIFEST.json",
                    "error": "manifest entry sets differ",
                    "txt_only": txt_only,
                    "json_only": json_only,
                }
            )

    ok = not (
        missing_includes
        or missing_required
        or manifest_missing
        or manifest_omissions
        or manifest_errors
    )
    return {
        "ok": ok,
        "include_count": include_count,
        "required_path_count": len(REQUIRED_PACKAGE_PATHS),
        "manifest_counts": manifest_counts,
        "missing_includes": missing_includes,
        "missing_required": missing_required,
        "manifest_missing": manifest_missing,
        "manifest_omissions": manifest_omissions,
        "manifest_errors": manifest_errors,
    }


def main() -> int:
    ap = argparse.ArgumentParser(description="Scan P20.2 package integrity.")
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--json-out", default="target/aidens-p20-2-audit/package-integrity.json")
    args = ap.parse_args()

    root = Path(args.root).resolve()
    report = scan(root)
    out = root / args.json_out
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text(json.dumps(report, indent=2), encoding="utf-8")
    print(json.dumps(report, indent=2))
    return 0 if report["ok"] else 2


if __name__ == "__main__":
    raise SystemExit(main())
