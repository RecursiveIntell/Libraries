#!/usr/bin/env python3
"""Validate latest root markdown archive manifest shape and entry fields."""
from pathlib import Path
import json
import re

ROOT = Path.cwd()
ARCHIVE_ROOT = ROOT / "docs" / "root-markdown-archive"

REQUIRED_ENTRY_KEYS = {
    "archived_path",
    "original_path",
    "sha256",
    "bytes",
    "mtime_utc",
    "reason",
    "classification",
}


def main() -> int:
    if not ARCHIVE_ROOT.exists():
        print("FAIL: root markdown archive root missing")
        return 1

    timestamp_dirs = [p for p in ARCHIVE_ROOT.iterdir() if p.is_dir() and re.fullmatch(r"[0-9TZ_\\-]+", p.name)]
    if not timestamp_dirs:
        print("FAIL: no root markdown archive timestamp directories")
        return 1

    manifest_path = sorted(timestamp_dirs, key=lambda p: p.name)[-1] / "ROOT_MARKDOWN_ARCHIVE_MANIFEST.json"
    if not manifest_path.exists():
        print(f"FAIL: latest manifest missing: {manifest_path}")
        return 1

    try:
        manifest = json.loads(manifest_path.read_text(encoding="utf-8"))
    except Exception as e:
        print(f"FAIL: unable to parse manifest: {e}")
        return 1

    if "files" not in manifest or not isinstance(manifest["files"], list):
        print("FAIL: manifest missing files list")
        return 1

    missing = []
    for idx, row in enumerate(manifest["files"]):
        if not isinstance(row, dict):
            missing.append(f"entry_{idx}:not_object")
            continue
        missing_keys = sorted(REQUIRED_ENTRY_KEYS - set(row))
        if missing_keys:
            missing.append(f"entry_{idx}:{','.join(missing_keys)}")

    if missing:
        print("FAIL: manifest entries missing required fields")
        for item in missing[:200]:
            print(f"  {item}")
        return 1

    if manifest.get("collisions"):
        print("FAIL: manifest has collisions (strict fail-closed rule violated)")
        print(f"  collisions={manifest.get('collisions')}")
        return 1

    summary = manifest.get("summary", {})
    print("PASS: root markdown archive manifest checks passed")
    print(
        "manifest={0}".format(manifest_path.as_posix())
    )
    print(
        "inspected={0} candidate={1} ambiguous={2} moved={3}".format(
            summary.get("inspected_count", 0),
            summary.get("candidate_count", 0),
            summary.get("ambiguous_count", 0),
            len(manifest.get("files", [])),
        )
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
