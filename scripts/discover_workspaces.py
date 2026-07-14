#!/usr/bin/env python3
"""Discover active Cargo workspace roots and verify that CI certifies each one."""

from __future__ import annotations

import argparse
import re
import sys
import tomllib
from pathlib import Path


ROOT = Path(__file__).resolve().parents[1]
DEFAULT_WORKFLOW = ROOT / ".github" / "workflows" / "ci.yml"
LANE_RE = re.compile(r"^\s*#\s*workspace-lane:\s*(\S+)\s*$", re.MULTILINE)
IGNORED_PARTS = {".git", "target", "docs", "_salvage_from_libraries2"}
IGNORED_COMPONENTS = {"fixtures", "fuzz"}


def is_ignored(path: Path) -> bool:
    relative = path.relative_to(ROOT)
    for part in relative.parts:
        if part in IGNORED_PARTS or part in IGNORED_COMPONENTS or part.startswith("target-"):
            return True
    return False


def discover() -> list[str]:
    roots: list[str] = []
    for manifest in ROOT.rglob("Cargo.toml"):
        if is_ignored(manifest):
            continue
        try:
            document = tomllib.loads(manifest.read_text(encoding="utf-8"))
        except (OSError, tomllib.TOMLDecodeError) as error:
            raise RuntimeError(f"cannot parse {manifest.relative_to(ROOT)}: {error}") from error
        if "workspace" not in document:
            continue
        parent = manifest.parent.relative_to(ROOT).as_posix()
        roots.append("." if parent == "." else parent)
    return sorted(roots, key=lambda item: (item != ".", item.casefold()))


def configured_lanes(workflow: Path) -> set[str]:
    try:
        text = workflow.read_text(encoding="utf-8")
    except OSError as error:
        raise RuntimeError(f"cannot read workflow {workflow}: {error}") from error
    return set(LANE_RE.findall(text))


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--workflow", type=Path, default=DEFAULT_WORKFLOW)
    parser.add_argument(
        "--list-only",
        action="store_true",
        help="print workspace roots without checking CI lane declarations",
    )
    args = parser.parse_args()

    try:
        roots = discover()
        lanes = configured_lanes(args.workflow) if not args.list_only else set()
    except RuntimeError as error:
        print(f"workspace discovery failed: {error}", file=sys.stderr)
        return 1

    for root in roots:
        print(root)

    if args.list_only:
        return 0

    missing = sorted(set(roots) - lanes)
    stale = sorted(lanes - set(roots))
    if missing:
        print(f"workspace discovery failed: CI lanes missing for {missing}", file=sys.stderr)
    if stale:
        print(f"workspace discovery failed: stale CI lane declarations {stale}", file=sys.stderr)
    if missing or stale:
        return 1

    print(f"workspace discovery passed: {len(roots)} active workspace roots are certified")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
