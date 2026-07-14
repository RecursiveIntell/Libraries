#!/usr/bin/env python3
"""Verify repository default branch from origin is `main`."""

from __future__ import annotations

import argparse
import subprocess
import sys


def git_default_branch(remote: str) -> str:
    completed = subprocess.run(
        ["git", "remote", "show", remote],
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"failed to inspect remote {remote}: {completed.stderr.strip()}")

    for line in completed.stdout.splitlines():
        stripped = line.strip()
        if stripped.startswith("HEAD branch:"):
            return stripped.split(":", 1)[1].strip()
    raise RuntimeError(f"could not find default branch in `git remote show {remote}` output")


def main() -> int:
    parser = argparse.ArgumentParser(description="Check that git remote default branch is main.")
    parser.add_argument(
        "--remote",
        default="origin",
        help="Remote to inspect (default: origin)",
    )
    parser.add_argument(
        "--expect",
        default="main",
        help="Expected default branch name (default: main)",
    )
    args = parser.parse_args()

    try:
        branch = git_default_branch(args.remote)
    except RuntimeError as error:
        print(f"error: {error}", file=sys.stderr)
        return 1

    if branch != args.expect:
        print(
            f"error: {args.remote} default branch is {branch}, expected {args.expect}",
            file=sys.stderr,
        )
        return 1

    print(f"ok: {args.remote} default branch is {branch}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

