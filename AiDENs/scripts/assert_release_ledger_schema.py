#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
RUN_RE = re.compile(r"^P\d+[A-Z]?$", re.I)
CERT_STATUSES = {"uncertified", "blocked", "failed", "certified"}
REQUIRED_STR = [
    "schema_version", "project", "last_certified_run", "active_run", "target_run",
    "parent_run", "active_run_role", "certification_status", "support_label",
    "build_scope_file", "known_limitations_file",
]
REQUIRED_BOOL = [
    "feature_expansion_allowed", "boundary_compiler_deferred", "runtime_receipt_changes_deferred",
    "build_certified", "package_certified", "extracted_replay_certified",
]
REQUIRED_EVIDENCE_KEYS = [
    "build_receipt", "cargo_metadata_log", "fmt_log", "check_log", "test_log", "clippy_log",
    "package_manifest", "package_findings", "package_report", "package_replay_receipt", "final_verify_log",
]


def fail(msg: str) -> int:
    print(f"FAIL: {msg}", file=sys.stderr)
    return 2


def exists_ref(value: object) -> bool:
    return isinstance(value, str) and bool(value.strip())


def main() -> int:
    if not LEDGER.exists():
        return fail(f"missing {LEDGER.relative_to(ROOT)}")
    try:
        data = json.loads(LEDGER.read_text(encoding="utf-8"))
    except Exception as e:
        return fail(f"invalid JSON in {LEDGER.relative_to(ROOT)}: {e}")

    errors: list[str] = []
    for key in REQUIRED_STR:
        if not isinstance(data.get(key), str) or not data.get(key).strip():
            errors.append(f"{key} must be a non-empty string")
    for key in REQUIRED_BOOL:
        if not isinstance(data.get(key), bool):
            errors.append(f"{key} must be boolean")

    if data.get("schema_version") != "aidens.current-run.v1":
        errors.append("schema_version must be aidens.current-run.v1")
    if data.get("project") != "AiDENs":
        errors.append("project must be AiDENs")

    for key in ["last_certified_run", "active_run", "target_run", "parent_run"]:
        value = str(data.get(key, ""))
        if not RUN_RE.match(value):
            errors.append(f"{key} has invalid run id: {value!r}")

    if str(data.get("target_run", "")).upper() != str(data.get("active_run", "")).upper():
        errors.append("target_run must equal active_run during P31A")
    if data.get("certification_status") not in CERT_STATUSES:
        errors.append(f"certification_status must be one of {sorted(CERT_STATUSES)}")
    if data.get("feature_expansion_allowed") is not False:
        errors.append("feature_expansion_allowed must be false for P31A")
    if data.get("boundary_compiler_deferred") is not True:
        errors.append("boundary_compiler_deferred must be true for P31A")
    if data.get("runtime_receipt_changes_deferred") is not True:
        errors.append("runtime_receipt_changes_deferred must be true for P31A")

    evidence = data.get("evidence")
    if not isinstance(evidence, dict):
        errors.append("evidence must be an object")
        evidence = {}
    for key in REQUIRED_EVIDENCE_KEYS:
        if key not in evidence:
            errors.append(f"evidence.{key} missing")

    if data.get("build_certified"):
        for key in ["cargo_metadata_log", "fmt_log", "check_log", "test_log", "clippy_log", "final_verify_log"]:
            if not exists_ref(evidence.get(key)):
                errors.append(f"build_certified=true requires evidence.{key}")
    if data.get("package_certified"):
        for key in ["package_manifest", "package_findings", "package_report"]:
            if not exists_ref(evidence.get(key)):
                errors.append(f"package_certified=true requires evidence.{key}")
    if data.get("extracted_replay_certified") and not exists_ref(evidence.get("package_replay_receipt")):
        errors.append("extracted_replay_certified=true requires evidence.package_replay_receipt")

    if data.get("certification_status") == "certified":
        for key in ["build_certified", "package_certified", "extracted_replay_certified"]:
            if data.get(key) is not True:
                errors.append(f"certification_status=certified requires {key}=true")
        if not exists_ref(evidence.get("final_verify_log")):
            errors.append("certification_status=certified requires evidence.final_verify_log")

    if str(data.get("active_run", "")).upper() == str(data.get("last_certified_run", "")).upper() and data.get("certification_status") != "certified":
        errors.append("active_run must not equal last_certified_run unless certification_status=certified")

    if errors:
        for e in errors:
            print(f"FAIL: {e}", file=sys.stderr)
        return 2

    print(f"PASS: release ledger schema valid ({LEDGER.relative_to(ROOT)})")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
