#!/usr/bin/env python3
"""Startup preflight for the FibQuant paper-core Codex pass.
Fails only on concrete, local violations. Designed to be safe in partial repos.
"""
from __future__ import annotations
import argparse, os, subprocess, sys
from pathlib import Path

REQUIRED = [
    "Cargo.toml",
    "semantic-memory/src/vector_codec.rs",
    "turbo-quant/src/lib.rs",
    "turbo-quant/src/rotation.rs",
]

def run(cmd, cwd):
    try:
        return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    except FileNotFoundError:
        return None

def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    errors = []
    warnings = []

    for rel in REQUIRED:
        if not (repo / rel).exists():
            errors.append(f"missing required source file: {rel}")

    git = run(["git", "rev-parse", "--show-toplevel"], repo)
    if git is None:
        warnings.append("git not found; cannot validate clean source state")
    elif git.returncode != 0:
        warnings.append("not a git repository; hooks/final diff checks may be weaker")

    cargo = run(["cargo", "--version"], repo)
    if cargo is None:
        warnings.append("cargo not found; Codex must report validation as environmental if tests cannot run")

    root_toml = repo / "Cargo.toml"
    if root_toml.exists():
        text = root_toml.read_text(encoding="utf-8", errors="replace")
        if "semantic-memory" not in text or "turbo-quant" in text and '"turbo-quant"' in text:
            warnings.append("root workspace shape differs from expected; inspect before editing")

    print("FibQuant preflight")
    for w in warnings:
        print(f"WARN: {w}")
    for e in errors:
        print(f"ERROR: {e}")
    return 1 if errors else 0

if __name__ == "__main__":
    raise SystemExit(main())
