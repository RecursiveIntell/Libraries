#!/usr/bin/env python3
"""Assert that stale Codex-run artifacts are not active outside the archive."""
from __future__ import annotations
import argparse
import re
import sys
from pathlib import Path

RUN_SEGMENT_RE = re.compile(r"^(p|P)(\d{1,3})(?:[_-]?\d+)?$")
RUN_PREFIX_RE = re.compile(r"^(p|P)\d{1,3}(?:[_-]?\d+)?")

DEFAULT_ALLOWED_ACTIVE_PREFIXES = {
    "docs/codex-runs/",
    "target/",
}

PROTECTED_ACTIVE_FILES = {
    "README.md", "STATUS.md", "SOURCE_BASIS.md", "AGENTS.md", "AGENTS_P22.md",
    "Cargo.toml", "Cargo.lock", "rust-toolchain.toml",
    "P22_RUN_ORDER.md",
}

ACTIVE_RUN_ALLOWED_PATTERNS = [
    re.compile(r"^prompts/p22/"),
    re.compile(r"^docs/p22/"),
    re.compile(r"^tasks/p22/"),
    re.compile(r"^handoffs/p22/"),
    re.compile(r"^scripts/p22_"),
    re.compile(r"^scripts/assert_p22_"),
]

STALE_PATH_PATTERNS = [
    re.compile(r"^\.codex/"),
    re.compile(r"^\.codex_evidence/"),
    re.compile(r"^prompts/[Pp](?!22\b)\d"),
    re.compile(r"^docs/[Pp](?!22\b)\d"),
    re.compile(r"^handoffs/[Pp](?!22\b)\d"),
    re.compile(r"^\.?CODEX_.*"),
    re.compile(r"^\.?NEXT_CODEX_.*"),
    re.compile(r"^.*_CODEX_RUN_PROMPT\.md(?:\..*)?$"),
    re.compile(r"^scripts/[Pp](?!22\b)\d"),
    re.compile(r"^install_[Pp](?:20|21)(?:[_-]?\d+)?_overlay\.sh$"),
]


def is_under_archive(rel: str) -> bool:
    return rel.startswith("docs/codex-runs/archive/")


def is_allowed_current(rel: str, current_run: str | None) -> bool:
    if rel in PROTECTED_ACTIVE_FILES:
        return True
    if is_under_archive(rel):
        return True
    if rel.startswith("docs/codex-runs/") and not rel.startswith("docs/codex-runs/archive/"):
        return True
    if current_run and current_run.upper() == "P22":
        return any(p.search(rel) for p in ACTIVE_RUN_ALLOWED_PATTERNS)
    return False


def is_stale(rel: str) -> bool:
    return any(p.search(rel) for p in STALE_PATH_PATTERNS)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("root", nargs="?", default=".")
    ap.add_argument("--current-run", default="P22")
    args = ap.parse_args()
    root = Path(args.root)
    offenders: list[str] = []
    for path in root.rglob("*"):
        if not path.is_file():
            continue
        rel = path.relative_to(root).as_posix()
        if "/.git/" in f"/{rel}/" or rel.startswith("target/"):
            continue
        if is_allowed_current(rel, args.current_run):
            continue
        if is_stale(rel):
            offenders.append(rel)
    if offenders:
        print("FAIL: stale Codex-run artifacts remain active outside archive:")
        for item in offenders[:200]:
            print(f"  - {item}")
        if len(offenders) > 200:
            print(f"  ... {len(offenders)-200} more")
        return 1
    print("PASS: no stale active Codex-run artifacts detected outside archive/current P22 allowlist")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
