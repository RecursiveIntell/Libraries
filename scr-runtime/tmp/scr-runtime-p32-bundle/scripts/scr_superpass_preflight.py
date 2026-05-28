#!/usr/bin/env python3
"""
SCR runtime P32 super-pass preflight/final checker.

Usage:
  python3 scripts/scr_superpass_preflight.py before
  python3 scripts/scr_superpass_preflight.py final
"""
from __future__ import annotations

import json
import os
import pathlib
import shutil
import subprocess
import sys

MODE = sys.argv[1] if len(sys.argv) > 1 else "before"
ROOT = pathlib.Path.cwd()
errors: list[str] = []
warnings: list[str] = []

def exists(path: str) -> bool:
    return (ROOT / path).exists()

def read(path: str) -> str:
    p = ROOT / path
    return p.read_text(encoding="utf-8", errors="replace") if p.exists() else ""

def check_tool(name: str, required_final: bool = True) -> None:
    if shutil.which(name) is None:
        msg = f"missing required tool: {name}"
        if MODE == "final" and required_final:
            errors.append(msg)
        else:
            warnings.append(msg)

def cmd(args: list[str]) -> tuple[int, str]:
    try:
        out = subprocess.run(args, cwd=ROOT, text=True, stdout=subprocess.PIPE, stderr=subprocess.STDOUT, timeout=20)
        return out.returncode, out.stdout.strip()
    except Exception as e:
        return 127, str(e)

for tool in ["python3", "git"]:
    check_tool(tool, required_final=True)
check_tool("cargo", required_final=True)

required_root = ["Cargo.toml", "README.md", "AGENTS.md", "crates/scr-kernel/src/lib.rs", "crates/scr-reference/src/lib.rs"]
for path in required_root:
    if not exists(path):
        errors.append(f"missing required path: {path}")

hooks = read(".codex/hooks.json")
if '"before_phase":null' in hooks or '"after_phase":null' in hooks or '"completion":null' in hooks:
    if MODE == "final":
        errors.append(".codex/hooks.json still contains inert null hooks")
    else:
        warnings.append(".codex/hooks.json contains inert null hooks")

if MODE == "final":
    required_final_docs = [
        "docs/P32_COMPLETION_REPORT.md",
        "docs/P32_COMMAND_RECEIPTS.md",
        "docs/P32_CHANGED_FILES.md",
        "docs/P32_UNRESOLVED_RISKS.md",
        "docs/P32_HOSTILE_AUDITOR_HANDOFF.md",
        "docs/P32_ROLLBACK_PLAN.md",
    ]
    for path in required_final_docs:
        if not exists(path):
            errors.append(f"missing final artifact: {path}")

status_code, status_out = cmd(["git", "status", "--short"])
payload = {
    "mode": MODE,
    "root": str(ROOT),
    "git_status_exit": status_code,
    "git_status": status_out.splitlines()[:200],
    "warnings": warnings,
    "errors": errors,
}
print(json.dumps(payload, indent=2))
if errors:
    sys.exit(1)
