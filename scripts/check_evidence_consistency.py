#!/usr/bin/env python3
"""Validate that the evidence manifest corresponds to the current git HEAD."""

from __future__ import annotations

import argparse
import json
import os
import re
import subprocess
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT / "STATUS_EVIDENCE_MANIFEST.json"


def load_manifest(path: Path) -> tuple[dict, str]:
    text = path.read_text(encoding="utf-8")
    return json.loads(text), text


def read_commit_from_manifest(data: object) -> str | None:
    candidate_keys = ("commit", "commit_sha", "head", "head_sha", "git_commit", "git_head")

    if isinstance(data, dict):
        for key in candidate_keys:
            value = data.get(key)
            if isinstance(value, str) and re.fullmatch(r"[0-9a-fA-F]{40}", value):
                return value.lower()
        for value in data.values():
            commit = read_commit_from_manifest(value)
            if commit is not None:
                return commit

    elif isinstance(data, list):
        for value in data:
            commit = read_commit_from_manifest(value)
            if commit is not None:
                return commit

    return None


def git_head_hash() -> str:
    completed = subprocess.run(
        ["git", "rev-parse", "HEAD"],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        raise RuntimeError(f"failed to read HEAD: {completed.stderr.strip()}")
    return completed.stdout.strip()


def git_remote_head_hash(path: Path, remote: str) -> str:
    completed = subprocess.run(
        ["git", "log", "-1", "--format=%H", "--", str(path)],
        cwd=ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if completed.returncode != 0:
        return ""
    return completed.stdout.strip()


def main() -> int:
    parser = argparse.ArgumentParser(description="Check STATUS_EVIDENCE_MANIFEST commit consistency.")
    parser.add_argument(
        "--manifest",
        default=str(MANIFEST_PATH),
        help="Path to evidence manifest.",
    )
    parser.add_argument(
        "--remote",
        default="origin",
        help="Remote used for HEAD branch checks if manifest lacks explicit commit fields.",
    )
    args = parser.parse_args()

    manifest_path = Path(args.manifest)
    if not manifest_path.is_absolute():
        manifest_path = ROOT / manifest_path
    if not manifest_path.exists():
        print(f"error: manifest not found: {manifest_path}", file=sys.stderr)
        return 1

    try:
        manifest, raw = load_manifest(manifest_path)
    except (json.JSONDecodeError, OSError) as error:
        print(f"error: cannot load manifest: {error}", file=sys.stderr)
        return 1

    if not raw:
        print(f"error: manifest is empty: {manifest_path}", file=sys.stderr)
        return 1

    head = git_head_hash()
    recorded = read_commit_from_manifest(manifest)
    if recorded is None:
        if os.environ.get("STATUS_EVIDENCE_REQUIRE_EXPLICIT_COMMIT", "").lower() in {"1", "true", "yes"}:
            print(
                "error: manifest does not include a 40-char commit SHA field "
                "(commit, commit_sha, head, head_sha)",
                file=sys.stderr,
            )
            return 1
        recorded = git_remote_head_hash(manifest_path, args.remote)
        if not recorded:
            print(
                "error: unable to infer manifest commit from git history",
                file=sys.stderr,
            )
            return 1

    if recorded != head:
        print(
            f"error: manifest commit mismatch; manifest {recorded} != HEAD {head}",
            file=sys.stderr,
        )
        return 1

    print(f"ok: evidence manifest matches HEAD {head}")
    print(json.dumps({"manifest": str(manifest_path), "head": head, "recorded": recorded}))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())

