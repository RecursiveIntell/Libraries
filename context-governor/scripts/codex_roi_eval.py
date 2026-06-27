#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import shutil
import subprocess
import sys
import time
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]


def run(cmd: list[str], timeout: int = 120) -> dict:
    started = time.time()
    proc = subprocess.run(
        cmd,
        cwd=ROOT,
        text=True,
        capture_output=True,
        timeout=timeout,
        check=False,
    )
    return {
        "command": cmd,
        "ok": proc.returncode == 0,
        "returncode": proc.returncode,
        "elapsed_ms": round((time.time() - started) * 1000, 2),
        "stdout_tail": proc.stdout[-4000:],
        "stderr_tail": proc.stderr[-4000:],
    }


def optional_codex_smoke() -> dict:
    if os.environ.get("RUN_CODEX_EVAL") != "1":
        return {
            "skipped": True,
            "reason": "set RUN_CODEX_EVAL=1 to run Codex non-interactive smoke checks",
        }
    if not shutil.which("codex"):
        return {"skipped": True, "reason": "codex binary not found"}
    prompt = (
        "Read the context-governor crate and report the single highest-risk "
        "remaining correctness issue. Do not edit files."
    )
    return run(
        [
            "codex",
            "exec",
            "--json",
            "--sandbox",
            "read-only",
            "--cd",
            str(ROOT),
            prompt,
        ],
        timeout=600,
    )


def main() -> int:
    checks = [
        run(["cargo", "fmt", "--check"], timeout=60),
        run(["cargo", "clippy", "--all-targets", "--", "-D", "warnings"], timeout=180),
        run(["cargo", "test", "--all-targets"], timeout=180),
        run(["cargo", "run", "--example", "replay_eval"], timeout=180),
    ]
    report = {
        "schema": "CodexRoiEvalV1",
        "crate": str(ROOT),
        "checks": checks,
        "codex_smoke": optional_codex_smoke(),
    }
    print(json.dumps(report, indent=2))
    return 0 if all(check["ok"] for check in checks) else 1


if __name__ == "__main__":
    raise SystemExit(main())
