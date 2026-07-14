#!/usr/bin/env python3
"""Reject mutable GitHub Action references in workflow `uses:` entries."""

from __future__ import annotations

import argparse
import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
USES_RE = re.compile(r"^\s*-?\s*uses:\s*([^\s#]+)(?:\s+#\s*(\S.*))?\s*$")
PINNED_RE = re.compile(r"^[^/@\s]+/[^@\s]+@[0-9a-fA-F]{40}$")


def workflow_paths(arguments: list[str]) -> list[Path]:
    if arguments:
        return [Path(item).resolve() for item in arguments]
    return sorted((ROOT / ".github" / "workflows").glob("*.y*ml"))


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("workflows", nargs="*", help="workflow files (defaults to all workflows)")
    args = parser.parse_args()

    errors: list[str] = []
    paths = workflow_paths(args.workflows)
    if not paths:
        errors.append("no GitHub workflow files found")

    checked = 0
    for path in paths:
        try:
            lines = path.read_text(encoding="utf-8").splitlines()
        except OSError as error:
            errors.append(f"{path}: cannot read workflow: {error}")
            continue
        for line_number, line in enumerate(lines, 1):
            match = USES_RE.match(line)
            if not match:
                continue
            checked += 1
            reference, version_comment = match.groups()
            if reference.startswith("./") or reference.startswith("docker://"):
                continue
            if not PINNED_RE.fullmatch(reference):
                errors.append(
                    f"{path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}:{line_number}: "
                    f"mutable action reference {reference!r}; expected owner/repo@<40-hex-sha>"
                )
            elif not version_comment:
                errors.append(
                    f"{path.relative_to(ROOT) if path.is_relative_to(ROOT) else path}:{line_number}: "
                    "SHA-pinned action is missing a readable version comment"
                )

    if errors:
        for error in errors:
            print(f"action pinning error: {error}", file=sys.stderr)
        return 1

    print(f"action pinning ok: {checked} uses entries checked across {len(paths)} workflow(s)")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
