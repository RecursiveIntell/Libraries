#!/usr/bin/env python3
"""Assert that a z.py manifest or zip excludes stale Codex history by default."""
from __future__ import annotations
import argparse, json, sys, zipfile, re
from pathlib import Path

FORBIDDEN = [
    re.compile(r"(^|/)\.codex/"),
    re.compile(r"(^|/)\.codex_evidence/"),
    re.compile(r"^\.?CODEX_[^/]*$"),
    re.compile(r"^\.?NEXT_CODEX_[^/]*$"),
    re.compile(r"^CODEX_PROMPTS/"),
    re.compile(r"(^|/).*_CODEX_RUN_PROMPT\.md(?:\..*)?$"),
    re.compile(r"(^|/)docs/codex-runs/archive/"),
    re.compile(r"(^|/)docs/p22/(?:p20|p21)(?:/|$)", re.IGNORECASE),
    re.compile(r"(^|/)docs/p22/[Pp](?:20|21)"),
    re.compile(r"(^|/)docs/[Pp](?!22\b)\d"),
    re.compile(r"(^|/)prompts/[Pp](?!22\b)\d"),
    re.compile(r"(^|/)handoffs/[Pp](?!22\b)\d"),
    re.compile(r"(^|/)scripts/[Pp](?:20|21)(?:[_-]?\d+)?[_-]"),
    re.compile(r"(^|/)install_[Pp](?:20|21)(?:[_-]?\d+)?_overlay\.sh$"),
]


def load_paths(path: Path) -> list[str]:
    if path.suffix == ".zip":
        with zipfile.ZipFile(path) as zf:
            return zf.namelist()
    payload = json.loads(path.read_text())
    if "files" in payload:
        return [entry["path"] for entry in payload["files"]]
    if "report" in payload and "files" in payload:
        return [entry["path"] for entry in payload["files"]]
    raise SystemExit(f"unsupported manifest/zip shape: {path}")


def path_variants(path: str) -> list[str]:
    normalized = path.strip("/")
    variants = [normalized]
    parts = normalized.split("/")
    if "AiDENs" in parts:
        idx = parts.index("AiDENs")
        if idx + 1 < len(parts):
            variants.append("/".join(parts[idx + 1 :]))
    return variants


def is_forbidden(path: str) -> bool:
    return any(rx.search(variant) for variant in path_variants(path) for rx in FORBIDDEN)


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("path", nargs="?", help="z.py manifest JSON or zip path")
    ap.add_argument("--manifest", dest="manifest", help="z.py manifest JSON path")
    ap.add_argument("--zip", dest="zip_path", help="zip path")
    args = ap.parse_args()
    selected = args.manifest or args.zip_path or args.path
    if not selected:
        ap.error("provide a manifest/zip path, or use --manifest/--zip")
    paths = load_paths(Path(selected))
    bad = [p for p in paths if is_forbidden(p)]
    if bad:
        print("FAIL: normal package contains stale/archived Codex history:")
        for p in bad[:200]:
            print(f"  - {p}")
        if len(bad) > 200:
            print(f"  ... {len(bad)-200} more")
        return 1
    print("PASS: package/manifest excludes stale and archived Codex history")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
