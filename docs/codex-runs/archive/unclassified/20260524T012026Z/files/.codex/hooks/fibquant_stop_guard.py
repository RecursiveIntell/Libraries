#!/usr/bin/env python3
"""Stop hook guard for the FibQuant paper-core pass.
This hook intentionally does NOT require final files on every turn. It only fails on concrete forbidden-surface violations and forbidden claim text.
"""
from __future__ import annotations
import argparse, subprocess
from pathlib import Path

FORBIDDEN_CHANGE_PREFIXES = [
    "semantic-memory/src/",
    "turbo-quant/src/",
    "AiDENs/crates/aidens-contracts/src/",
]
FORBIDDEN_TEXT = [
    "zero accuracy loss",
    "guaranteed lossless",
    "beats turboquant",
    "production ready",
]

def run(cmd, cwd):
    try:
        return subprocess.run(cmd, cwd=cwd, text=True, capture_output=True, check=False)
    except FileNotFoundError:
        return None

def changed_paths(repo: Path):
    proc = run(["git", "diff", "--name-only", "HEAD"], repo)
    if proc is None or proc.returncode != 0:
        proc = run(["git", "status", "--porcelain"], repo)
        if proc is None or proc.returncode != 0:
            return []
        return [line[3:].strip() for line in proc.stdout.splitlines() if len(line) > 3]
    return [line.strip() for line in proc.stdout.splitlines() if line.strip()]

def main() -> int:
    import argparse
    ap = argparse.ArgumentParser()
    ap.add_argument("--repo", default=".")
    args = ap.parse_args()
    repo = Path(args.repo).resolve()
    errors = []
    for p in changed_paths(repo):
        for prefix in FORBIDDEN_CHANGE_PREFIXES:
            if p.startswith(prefix):
                errors.append(f"forbidden FibQuant pass source change: {p}")

    for sr in [repo / "fib-quant", repo / "docs/compression"]:
        if not sr.exists():
            continue
        for path in sr.rglob("*"):
            if path.is_file() and path.suffix in {".rs", ".md", ".toml"}:
                data = path.read_text(encoding="utf-8", errors="replace").lower()
                for needle in FORBIDDEN_TEXT:
                    if needle in data:
                        errors.append(f"forbidden claim text '{needle}' in {path.relative_to(repo)}")

    if errors:
        print("FibQuant stop guard violations:")
        for e in errors:
            print(f"ERROR: {e}")
        return 1
    print("FibQuant stop guard: no forbidden surface/claim violations detected")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
