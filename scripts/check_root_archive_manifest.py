#!/usr/bin/env python3
from __future__ import annotations

import json
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
MANIFEST_PATH = ROOT / "docs" / "archive" / "root_closeout_history" / "manifest.json"
REQUIRED_ACTIVE = [
    "README.md",
    "00_START_HERE.md",
    "09_RISK_REGISTER.md",
    "11_BENCHMARK_PLAN.md",
    "AGENTS.md",
    "CONFORMANCE_GATES.md",
    "IMPLEMENTATION_PLAYBOOK.md",
    "MASTER_ISSUE_MATRIX.md",
    "PACK_README.md",
    "PHASED_EXECUTION_PLAN.md",
    "PROMPT.md",
    "RISKS_AND_FORBIDDEN_SHORTCUTS.md",
    "SCOPE_NOTES.md",
    "SOURCE_BASIS.md",
    "SUPPORT_PROFILE.md",
    "STATUS_DASHBOARD.md",
    "RELEASE_CHECKLIST.md",
    "STATUS_EVIDENCE_MANIFEST.json",
    "release/closeout_receipt_v1.json",
]


def main() -> int:
    if not MANIFEST_PATH.is_file():
        print(f"missing archive manifest: {MANIFEST_PATH.relative_to(ROOT)}", file=sys.stderr)
        return 1

    manifest = json.loads(MANIFEST_PATH.read_text(encoding="utf-8"))
    errors: list[str] = []

    if manifest.get("archive_mode") != "logical_supersession_map":
        errors.append("archive_mode must be logical_supersession_map for this snapshot")

    active = manifest.get("active_root_closeout_pack")
    if not isinstance(active, list) or not active:
        errors.append("active_root_closeout_pack must be a non-empty list")
        active = []

    active_set = set(active)
    for path in REQUIRED_ACTIVE:
        if path not in active_set:
            errors.append(f"active_root_closeout_pack missing {path}")

    for rel in active:
        if not (ROOT / rel).exists():
            errors.append(f"active root file missing on disk: {rel}")

    groups = manifest.get("superseded_root_groups")
    if not isinstance(groups, list) or not groups:
        errors.append("superseded_root_groups must be a non-empty list")
        groups = []

    for idx, group in enumerate(groups, start=1):
        if not isinstance(group, dict):
            errors.append(f"group {idx} is not an object")
            continue
        if not group.get("group"):
            errors.append(f"group {idx} missing group name")
        root_globs = group.get("root_globs")
        archived_dir = group.get("archived_dir")
        archived_count = group.get("archived_count")
        superseded_by = group.get("superseded_by")
        if not isinstance(root_globs, list) or not root_globs:
            errors.append(f"group {group.get('group', idx)} missing root_globs")
        if archived_dir is not None:
            archive_path = ROOT / archived_dir
            if not archive_path.is_dir():
                errors.append(
                    f"group {group.get('group', idx)} archived_dir missing on disk: {archived_dir}"
                )
            elif archived_count is not None:
                actual_count = len([path for path in archive_path.iterdir() if path.is_file()])
                if actual_count != archived_count:
                    errors.append(
                        f"group {group.get('group', idx)} archived_dir file count {actual_count} != {archived_count}"
                    )
        if not isinstance(superseded_by, list) or not superseded_by:
            errors.append(f"group {group.get('group', idx)} missing superseded_by")
            continue
        for rel in superseded_by:
            if not (ROOT / rel).exists():
                errors.append(
                    f"group {group.get('group', idx)} superseded_by target missing on disk: {rel}"
                )

    if errors:
        for error in errors:
            print(error, file=sys.stderr)
        print("root archive manifest check failed", file=sys.stderr)
        return 1

    print("root archive manifest check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
