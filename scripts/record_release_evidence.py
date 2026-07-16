#!/usr/bin/env python3
"""Explicitly record release-gate evidence; never used by verification.

The recorder requires a clean tree before execution. It runs every configured
command, retains stdout/stderr by content digest, records failed commands rather
than omitting them, and then writes the manifest plus derivative receipt.
"""
from __future__ import annotations

import argparse
import json
import os
import shlex
import subprocess
from datetime import datetime, timezone
from pathlib import Path

from evidence_common import git_status_porcelain, sha256_bytes, source_binding
from release_gate_set import RELEASE_GATE_COMMANDS, gate_sha256

ROOT = Path(__file__).resolve().parent.parent
MANIFEST = ROOT / "STATUS_EVIDENCE_MANIFEST.json"
LOG_DIR = ROOT / "release" / "evidence" / "logs"


def save_log(stream: str, data: bytes) -> dict[str, object]:
    digest = sha256_bytes(data)
    LOG_DIR.mkdir(parents=True, exist_ok=True)
    path = LOG_DIR / f"{digest}.{stream}.log"
    if not path.exists():
        path.write_bytes(data)
    return {"path": str(path.relative_to(ROOT)), "sha256": digest, "bytes": len(data)}


def run_gate(command: str) -> dict[str, object]:
    completed = subprocess.run(
        command,
        shell=True,
        cwd=ROOT,
        stdout=subprocess.PIPE,
        stderr=subprocess.PIPE,
        env=os.environ.copy(),
        check=False,
    )
    return {
        "command": command,
        "argv": ["/bin/sh", "-c", command],
        "cwd": ".",
        "exit_code": completed.returncode,
        "result": "pass" if completed.returncode == 0 else "fail",
        "stdout": save_log("stdout", completed.stdout),
        "stderr": save_log("stderr", completed.stderr),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Record source-bound release evidence")
    parser.add_argument("--allow-dirty", action="store_true", help="forensic only; marks the record non-clean")
    parser.add_argument("--command", action="append", default=[], help="record only a specific command; repeatable")
    args = parser.parse_args()

    before = git_status_porcelain(ROOT)
    if before and not args.allow_dirty:
        parser.error("refusing to record evidence from a dirty tree")

    commands = args.command or RELEASE_GATE_COMMANDS
    receipts = [run_gate(command) for command in commands]
    captured_at = datetime.now(timezone.utc)
    results = [{"command": receipt["command"], "result": receipt["result"]} for receipt in receipts]
    manifest = {
        "schema_version": "libraries.status-evidence.v2",
        "snapshot": f"{captured_at.date().isoformat()}-source-bound-release-evidence",
        "captured_at": captured_at.date().isoformat(),
        "captured_at_utc": captured_at.isoformat(timespec="seconds"),
        "tree_was_clean_before_recording": not bool(before),
        "dirty_status_before_recording": before.splitlines() if before else [],
        "gate_definition": {
            "path": "scripts/release_gate_set.py",
            "sha256": gate_sha256(),
            "command_count": len(commands),
        },
        "proof_commands": commands,
        "proof_results": results,
        "source_binding": source_binding(ROOT, receipts),
        "notes": [
            "Recorded by the explicit record-release-evidence path; verification is read-only.",
            "Every selected command has an exit code and content-addressed stdout/stderr logs.",
            "A failed command is recorded as failed; it is never omitted or rewritten by verification.",
        ],
    }
    MANIFEST.write_text(json.dumps(manifest, indent=2) + "\n", encoding="utf-8")

    generated = subprocess.run(["python3", "scripts/generate_closeout_receipt.py"], cwd=ROOT, check=False)
    if generated.returncode:
        return generated.returncode
    return 0 if all(receipt["exit_code"] == 0 for receipt in receipts) else 1


if __name__ == "__main__":
    raise SystemExit(main())
