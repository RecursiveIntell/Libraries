#!/usr/bin/env python3
"""Assert that z.py exposes the P22 Codex archival contract.

This is intentionally lexical. It does not prove behavior; it prevents Codex from
claiming P22 while forgetting the main contract surface.
"""
from __future__ import annotations
import sys
from pathlib import Path

REQUIRED_SNIPPETS = [
    "archive_codex",
    "--archive-codex-runs",
    "--no-archive-codex-runs",
    "--archive-only",
    "--verify-codex-archive-hygiene",
    "--include-codex-archive",
    "--codex-current-run",
    "--codex-archive-root",
    "ARCHIVE_MANIFEST.json",
    "SUPERSESSION.md",
    "CODEX_RUN_INDEX.md",
    "CURRENT_RUN.md",
    "audit-full",
]

FORBIDDEN_SNIPPETS = [
    "include_codex_artifacts = (\n        args.include_codex_artifacts\n        if args.include_codex_artifacts is not None\n        else mode in {\"codex-context\", \"full-context\"}",
]


def main() -> int:
    path = Path(sys.argv[1] if len(sys.argv) > 1 else "z.py")
    text = path.read_text(encoding="utf-8", errors="replace")
    missing = [snippet for snippet in REQUIRED_SNIPPETS if snippet not in text]
    forbidden = [snippet for snippet in FORBIDDEN_SNIPPETS if snippet in text]
    if missing:
        print("FAIL: z.py missing P22 archival contract snippets:")
        for item in missing:
            print(f"  - {item}")
    if forbidden:
        print("FAIL: z.py still appears to include Codex artifacts by default for codex-context/full-context")
    if missing or forbidden:
        return 1
    print("PASS: z.py exposes P22 archival contract surface")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
