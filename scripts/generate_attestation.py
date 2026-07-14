#!/usr/bin/env python3
"""Generate machine-readable command attestations."""

from __future__ import annotations

import argparse
import hashlib
import json
import os
import platform
import shlex
import subprocess
import sys
from datetime import datetime, timezone
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def capture_toolchain() -> dict[str, str]:
    def run_for(command: list[str]) -> str:
        completed = subprocess.run(
            command,
            capture_output=True,
            text=True,
            check=False,
            cwd=ROOT,
        )
        return completed.stdout.strip() or completed.stderr.strip()

    return {
        "rustc": run_for(["rustc", "--version"]),
        "cargo": run_for(["cargo", "--version"]),
        "platform": platform.platform(),
    }


def gather_env(selected: list[str] | None = None) -> dict[str, str]:
    env = dict(os.environ)
    selected = selected or [
        "PATH",
        "RUSTUP_TOOLCHAIN",
        "RUSTFLAGS",
        "CARGO_HOME",
        "CARGO_NET_OFFLINE",
        "CI",
        "GITHUB_ACTIONS",
    ]
    return {key: env.get(key, "") for key in selected}


def command_digest(data: str) -> str:
    return hashlib.sha256(data.encode("utf-8")).hexdigest()


def generate_attestation(command: list[str] | None, env_keys: list[str] | None = None) -> dict[str, object]:
    started_at = datetime.now(timezone.utc).isoformat()
    head = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    commit = head.stdout.strip() if head.returncode == 0 else "unknown"

    if not command:
        return {
            "generated_at_utc": started_at,
            "command": [],
            "command_rendered": "",
            "env": gather_env(env_keys),
            "toolchain": capture_toolchain(),
            "exit_status": 0,
            "commit": commit,
            "notes": "no command supplied",
        }

    result = subprocess.run(
        command,
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
        env=os.environ.copy(),
    )
    return {
        "generated_at_utc": started_at,
        "command": command,
        "command_rendered": " ".join(shlex.quote(part) for part in command),
        "env": gather_env(env_keys),
        "toolchain": capture_toolchain(),
        "exit_status": result.returncode,
        "commit": commit,
        "stdout_digest": command_digest(result.stdout),
        "stderr_digest": command_digest(result.stderr),
    }


def main() -> int:
    parser = argparse.ArgumentParser(description="Generate machine-readable test attestation.")
    parser.add_argument(
        "--command",
        nargs="*",
        help="Command to execute for attestation; if omitted only template output is produced.",
    )
    parser.add_argument(
        "--env",
        action="append",
        default=None,
        help="Environment variables to include (repeatable).",
    )
    parser.add_argument(
        "--output",
        type=Path,
        default=None,
        help="Write attestation JSON to this path instead of stdout.",
    )
    args = parser.parse_args()

    attestation = generate_attestation(args.command, args.env)
    payload = json.dumps(attestation, indent=2, sort_keys=True)
    if args.output:
        args.output.write_text(payload + "\n", encoding="utf-8")
    else:
        print(payload)

    return int(attestation["exit_status"])


if __name__ == "__main__":
    raise SystemExit(main())

