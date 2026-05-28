#!/usr/bin/env python3
"""Startup preflight for the standalone fib-quant crate.

This is intentionally local-only. The public repository ignores `.codex/`.
"""
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

REQUIRED = [
    "Cargo.toml",
    "README.md",
    "LICENSE",
    "src/lib.rs",
    "scripts/publish_preflight.py",
    "scripts/publish_final_assert.py",
]


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str] | None:
    try:
        return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    except FileNotFoundError:
        return None


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    errors: list[str] = []
    warnings: list[str] = []

    for rel in REQUIRED:
        if not (repo / rel).exists():
            errors.append(f"missing required fib-quant crate file: {rel}")

    cargo_toml = repo / "Cargo.toml"
    if cargo_toml.exists():
        text = cargo_toml.read_text(encoding="utf-8", errors="replace")
        if 'name = "fib-quant"' not in text:
            errors.append("Cargo.toml is not the fib-quant package manifest")
        if "workspace = true" in text:
            errors.append("Cargo.toml still uses workspace inheritance")

    if run(["git", "rev-parse", "--show-toplevel"], repo) is None:
        warnings.append("git not found; cannot validate source state")
    if run(["cargo", "--version"], repo) is None:
        warnings.append("cargo not found; validation commands may be unavailable")

    print("FibQuant preflight")
    for warning in warnings:
        print(f"WARN: {warning}")
    for error in errors:
        print(f"ERROR: {error}")
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
