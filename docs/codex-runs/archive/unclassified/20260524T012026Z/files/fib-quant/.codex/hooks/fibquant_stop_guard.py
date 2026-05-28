#!/usr/bin/env python3
"""Stop hook guard for the standalone fib-quant crate.

The guard catches concrete forbidden public-claim text and accidental changes
to parent-workspace surfaces when invoked from this nested repository.
"""
from __future__ import annotations

import argparse
import subprocess
from pathlib import Path

FORBIDDEN_CHANGE_PREFIXES = [
    "../semantic-memory/src/",
    "../turbo-quant/src/",
    "../AiDENs/crates/aidens-contracts/src/",
]
FORBIDDEN_TEXT = [
    "zero accuracy loss",
    "guaranteed lossless",
    "beats turboquant",
    "production ready",
]
SCAN_SUFFIXES = {".rs", ".md", ".toml"}
SCAN_DIRS = [
    "src",
    "docs/compression",
    "docs/kv",
    "examples",
    "tests",
    "benches",
]
SCAN_FILES = ["README.md", "Cargo.toml", "CHANGELOG.md", "RELEASE_CHECKLIST.md"]


def run(cmd: list[str], cwd: Path) -> subprocess.CompletedProcess[str] | None:
    try:
        return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    except FileNotFoundError:
        return None


def changed_paths(repo: Path) -> list[str]:
    proc = run(["git", "diff", "--name-only", "HEAD"], repo)
    if proc is None or proc.returncode != 0:
        proc = run(["git", "status", "--porcelain"], repo)
        if proc is None or proc.returncode != 0:
            return []
        return [line[3:].strip() for line in proc.stdout.splitlines() if len(line) > 3]
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]


def scan_files(repo: Path) -> list[Path]:
    paths: list[Path] = []
    for rel in SCAN_FILES:
        path = repo / rel
        if path.exists():
            paths.append(path)
    for rel in SCAN_DIRS:
        root = repo / rel
        if root.exists():
            paths.extend(
                path
                for path in root.rglob("*")
                if path.is_file() and path.suffix in SCAN_SUFFIXES
            )
    return paths


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--repo", default=".")
    args = parser.parse_args()
    repo = Path(args.repo).resolve()
    errors: list[str] = []

    for changed in changed_paths(repo):
        for prefix in FORBIDDEN_CHANGE_PREFIXES:
            if changed.startswith(prefix):
                errors.append(f"forbidden FibQuant pass source change: {changed}")

    for path in scan_files(repo):
        data = path.read_text(encoding="utf-8", errors="replace").lower()
        for needle in FORBIDDEN_TEXT:
            if needle in data:
                errors.append(f"forbidden claim text '{needle}' in {path.relative_to(repo)}")

    if errors:
        print("FibQuant stop guard violations:")
        for error in errors:
            print(f"ERROR: {error}")
        return 1

    print("FibQuant stop guard: no forbidden surface/claim violations detected")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
