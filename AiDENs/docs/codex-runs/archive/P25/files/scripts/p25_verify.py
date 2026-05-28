#!/usr/bin/env python3
"""P25 verifier and evidence manifest generator."""
from __future__ import annotations

import json
import hashlib
import os
import re
import shlex
import subprocess
from datetime import datetime, timezone
from pathlib import Path

ROOT = Path.cwd()
AUDIT_DIR = ROOT / "target" / "p25" / "audit"
PACKAGE_DIR = ROOT / "target" / "p25" / "package"
PACKAGE_PREFIX = "AiDENs-p25-codex-context"
MANIFEST_PATH = ROOT / "P25_STATUS_EVIDENCE_MANIFEST.json"
COMMAND_LOG = AUDIT_DIR / "phase05_command_log.txt"
COMMAND_RESULTS = AUDIT_DIR / "phase05_command_results.jsonl"
PHASE_REPORT_PREFIX = "phase05"

COMMANDS = [
    ("assert_phase_gate_integrity", "python3 scripts/assert_phase_gate_integrity.py"),
    ("assert_root_markdown_archive_policy", "python3 scripts/assert_root_markdown_archive_policy.py"),
    ("assert_current_run_truth", "python3 scripts/assert_current_run_truth.py"),
    ("assert_codex_artifact_classification", "python3 scripts/assert_codex_artifact_classification.py ."),
    ("assert_root_markdown_archive_manifest", "python3 scripts/assert_root_markdown_archive_manifest.py"),
    ("assert_support_claims", "python3 scripts/assert_support_claims.py"),
    ("assert_source_basis_staleness", "bash scripts/assert_docs_source_basis_current.sh"),
    ("assert_no_fake_completion", "bash scripts/assert_no_fake_completion.sh"),
    ("assert_no_local_substitute_dependencies", "bash scripts/assert_no_local_substitute_dependencies.sh"),
    (
        "root_markdown_archive_dry_run",
        "python3 z.py --root . --profile aidens --mode codex-context --no-strict --verify-root-markdown-noise-hygiene --archive-root-markdown-noise --root-markdown-archive-dry-run",
    ),
]


def run(cmd: str, label: str, idx: int, env: dict[str, str] | None = None) -> dict[str, str]:
    safe = f"{PHASE_REPORT_PREFIX}_{idx:02d}_{label}.txt"
    log_path = AUDIT_DIR / safe
    with open(log_path, "w", encoding="utf-8") as fp:
        fp.write(f"$ {cmd}\n")
        fp.flush()
        result = subprocess.run(
            cmd,
            cwd=ROOT,
            shell=True,
            text=True,
            stdout=fp,
            stderr=subprocess.STDOUT,
            executable="/bin/bash",
            env=env,
        )
    status = "pass" if result.returncode == 0 else "fail"
    return {
        "command": cmd,
        "status": status,
        "log": log_path.as_posix(),
        "return_code": result.returncode,
    }


def run_with_output(cmd: str, label: str, idx: int, env: dict[str, str] | None = None) -> dict[str, str]:
    result = run(cmd, label, idx, env=env)
    return result


def cargo_command(cmd: str, idx: int) -> dict[str, str]:
    result = subprocess.run(
        "command -v cargo >/dev/null 2>&1",
        cwd=ROOT,
        shell=True,
        text=True,
        executable="/bin/bash",
    )
    if result.returncode != 0:
        label = cmd.replace(" ", "_")
        safe = f"{PHASE_REPORT_PREFIX}_{idx:02d}_{label}.txt"
        log_path = AUDIT_DIR / safe
        log_path.write_text("[cargo unavailable]\n", encoding="utf-8")
        return {
            "command": cmd,
            "status": "deferred",
            "log": log_path.as_posix(),
            "return_code": 0,
        }
    return run(cmd, cmd.replace(" ", "_"), idx)


def sha256(path: Path) -> str:
    h = hashlib.sha256()
    with path.open("rb") as fp:
        for chunk in iter(lambda: fp.read(1024 * 1024), b""):
            h.update(chunk)
    return h.hexdigest()


def latest_root_manifest() -> Path | None:
    root = ROOT / "docs" / "root-markdown-archive"
    if not root.exists():
        return None
    ts_dirs = [p for p in root.iterdir() if p.is_dir() and re.fullmatch(r"[0-9TZX_\\-]+", p.name)]
    if not ts_dirs:
        return None
    ts_dirs.sort(key=lambda p: p.name)
    candidate = ts_dirs[-1] / "ROOT_MARKDOWN_ARCHIVE_MANIFEST.json"
    return candidate if candidate.exists() else None


def load_root_archive_summary(path: Path) -> dict:
    manifest = json.loads(path.read_text(encoding="utf-8"))
    summary = manifest.get("summary", {})
    return {
        "manifest_path": path.as_posix(),
        "inspected_count": summary.get("inspected_count", 0),
        "protected_count": summary.get("protected_count", 0),
        "candidate_count": summary.get("candidate_count", 0),
        "ambiguous_count": summary.get("ambiguous_count", 0),
        "moved_count": len(manifest.get("files", [])),
    }


def summarize(results: list[dict[str, str]]) -> None:
    print("[P25] command summary")
    for idx, row in enumerate(results, 1):
        status = row["status"]
        code = row["return_code"]
        print(f" {idx:02d}. [{status}] ({code}) {row['command']}")
        print(f"     log: {row['log']}")


def main() -> int:
    AUDIT_DIR.mkdir(parents=True, exist_ok=True)
    PACKAGE_DIR.mkdir(parents=True, exist_ok=True)

    with open(COMMAND_LOG, "w", encoding="utf-8") as log:
        log.write(f"phase=05 run_id=P25 started_utc={datetime.now(timezone.utc).isoformat()}\n")

    command_records: list[dict[str, str]] = []
    failed_checks = []

    idx = 1
    # Core checks
    for label, cmd in COMMANDS:
        rec = run_with_output(cmd, label, idx)
        idx += 1
        with open(COMMAND_RESULTS, "a", encoding="utf-8") as fp:
            fp.write(json.dumps(rec) + "\n")
        command_records.append(rec)
        with open(COMMAND_LOG, "a", encoding="utf-8") as fp:
            fp.write(f"{label}: {rec['status']} ({rec['return_code']})\\n")
            fp.write(f"  log={rec['log']}\\n")
        if rec["status"] == "fail":
            failed_checks.append(f"{label}: rc={rec['return_code']}")

    # Package validation is required in hardening phase.
    package_output = str(PACKAGE_DIR / f"{PACKAGE_PREFIX}.zip")
    package_cmd = (
        "python3 z.py --root . --profile aidens --mode codex-context "
        "--strict --check-script-refs --codex-current-run P25 --output "
        f"{shlex.quote(package_output)}"
    )
    rec = run_with_output(package_cmd, "package_validation", idx)
    idx += 1
    with open(COMMAND_RESULTS, "a", encoding="utf-8") as fp:
        fp.write(json.dumps(rec) + "\n")
    command_records.append(rec)
    with open(COMMAND_LOG, "a", encoding="utf-8") as fp:
        fp.write(f"package_validation: {rec['status']} ({rec['return_code']})\\n")
        fp.write(f"  log={rec['log']}\\n")
    if rec["status"] == "fail":
        failed_checks.append("package_validation: package command failed")

    for label, cmd in (
        ("assert_package_validation", "python3 scripts/assert_package_validation.py"),
    ):
        rec = run_with_output(cmd, label, idx)
        idx += 1
        with open(COMMAND_RESULTS, "a", encoding="utf-8") as fp:
            fp.write(json.dumps(rec) + "\n")
        command_records.append(rec)
        with open(COMMAND_LOG, "a", encoding="utf-8") as fp:
            fp.write(f"{label}: {rec['status']} ({rec['return_code']})\\n")
            fp.write(f"  log={rec['log']}\\n")
        if rec["status"] == "fail":
            failed_checks.append(f"{label}: rc={rec['return_code']}")

    # Cargo checks where available.
    cargo_checks = [
        ("cargo_fmt", "cargo fmt --all -- --check"),
        ("cargo_check", "cargo check --workspace"),
        ("cargo_test", "cargo test --workspace"),
        ("cargo_clippy", "cargo clippy --workspace --all-targets -- -D warnings"),
        ("cargo_doc", "cargo doc --workspace --no-deps"),
    ]
    for label, cmd in cargo_checks:
        rec = cargo_command(cmd, idx)
        idx += 1
        with open(COMMAND_RESULTS, "a", encoding="utf-8") as fp:
            fp.write(json.dumps(rec) + "\n")
        command_records.append(rec)
        with open(COMMAND_LOG, "a", encoding="utf-8") as fp:
            fp.write(f"{label}: {rec['status']} ({rec['return_code']})\\n")
            fp.write(f"  log={rec['log']}\\n")
        if rec["status"] == "fail":
            failed_checks.append(f"{label}: rc={rec['return_code']}")

    # Post-run derived checks.
    root_manifest_path = latest_root_manifest()
    root_archive_info: dict[str, int | str | None] = {
        "manifest_path": None,
        "inspected_count": 0,
        "protected_count": 0,
        "candidate_count": 0,
        "ambiguous_count": 0,
        "moved_count": 0,
    }
    if root_manifest_path is not None:
        root_archive_info.update(load_root_archive_summary(root_manifest_path))

    package_zip = PACKAGE_DIR / f"{PACKAGE_PREFIX}.zip"
    package_sha = None
    if package_zip.exists():
        package_sha = sha256(package_zip)

    unresolved_risks = []
    if any(r["status"] == "deferred" for r in command_records):
        unresolved_risks.append("Cargo tooling unavailable in this environment; cargo checks recorded as deferred.")
    if root_manifest_path and root_archive_info.get("candidate_count", 0) > 0:
        unresolved_risks.append("Root markdown candidates still present after dry-run-only checks.")
        failed_checks.append("root markdown candidates remain")

    support_claims = [
        "Supported-local and fixture-backed support claims remain evidence-bearing; no additional cloud/autonomy support claims were introduced in this phase."
    ]

    validation_results = [
        {"check": row["command"], "status": row["status"], "log": row["log"], "return_code": row["return_code"]}
        for row in command_records
    ]

    changed_files = [
        "scripts/p25_verify.sh",
        "scripts/p25_verify.py",
        "scripts/verify_current.sh",
        "scripts/assert_root_markdown_archive_manifest.py",
        "scripts/assert_support_claims.py",
        "scripts/assert_package_validation.py",
        "handoffs/p25/PHASE_05_GATE_REVALIDATION.md",
        "handoffs/p25/PHASE_05_REPORT.md",
        "target/p25/audit/phase05_command_log.txt",
        "target/p25/audit/phase05_command_results.jsonl",
        "P25_STATUS_EVIDENCE_MANIFEST.json",
        "target/p25/package/AiDENs-p25-codex-context.zip",
        "target/p25/package/AiDENs-p25-codex-context.manifest.json",
        "target/p25/package/AiDENs-p25-codex-context.findings.json",
        "target/p25/package/AiDENs-p25-codex-context.report.md",
        "target/p25/package/AiDENs-p25-codex-context.excluded.json",
        "target/p25/package/AiDENs-p25-codex-context.codex-archive.json",
        root_manifest_path.as_posix() if root_manifest_path else "",
    ]
    changed_files = [p for p in changed_files if p]

    manifest = {
        "run_id": "P25",
        "created_utc": datetime.now(timezone.utc).isoformat(),
        "package_sha256": package_sha,
        "commands": [
            {
                "command": row["command"],
                "status": row["status"],
                "log": row["log"],
            }
            for row in command_records
        ],
        "changed_files": sorted(set(changed_files)),
        "phase_gates": [
            {
                "phase": "05",
                "name": "Verifier and evidence manifest hardening",
                "status": "pass" if not failed_checks else "fail",
                "log": COMMAND_LOG.as_posix(),
            }
        ],
        "root_markdown_archive": root_archive_info,
        "validation_results": validation_results,
        "support_claims": support_claims,
        "known_limitations": ["No additional runtime/semantic capability expansion in this phase."],
        "unresolved_risks": unresolved_risks,
        "failed_checks": failed_checks,
    }

    with open(MANIFEST_PATH, "w", encoding="utf-8") as fp:
        json.dump(manifest, fp, indent=2, sort_keys=False)

    summarize(command_records)
    print()
    print(f"[P25] manifest written: {MANIFEST_PATH}")
    print(f"[P25] failed checks: {len(failed_checks)}")
    for line in failed_checks:
        print(f" - {line}")

    with open(COMMAND_LOG, "a", encoding="utf-8") as fp:
        fp.write(f"finished_utc={datetime.now(timezone.utc).isoformat()}\n")
        fp.write(f"failed_checks={len(failed_checks)}\n")
        for item in failed_checks:
            fp.write(f" - {item}\n")

    return 0 if not failed_checks else 1


if __name__ == "__main__":
    raise SystemExit(main())
