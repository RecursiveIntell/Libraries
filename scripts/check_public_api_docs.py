#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


import json

ROOT = Path(__file__).resolve().parent.parent
LANE_MANIFEST = ROOT / "scripts" / "lane_manifest.json"

def _load_doc_certified() -> list[str]:
    if LANE_MANIFEST.is_file():
        manifest = json.loads(LANE_MANIFEST.read_text(encoding="utf-8"))
        return manifest.get("doc_certified_lane", [])
    return []

# Prefer lane_manifest.json if present; otherwise use hardcoded fallback.
DOC_CERTIFIED_CRATES = _load_doc_certified() or [
    "forge-pilot",
    "kernel-conformance",
    "llm-tool-runtime",
    "kernel-execution",
    "contract-schema-gen",
    "kernel-oracles",
    "profile-runtime",
    "recursive-kernel-core",
    "constraint-compiler",
    "effect-runtime",
    "verification-control",
    "verification-policy",
    "semantic-memory-forge",
]
DEMOTED_COMPATIBILITY_CRATES = [
    "assurance-runtime",
    "attestation-exchange",
    "authority-delegation",
    "constitutional-memory",
    "continuity-runtime",
    "discovery-portfolio",
    "federated-settlement",
    "mechanism-runtime",
    "spec-execution",
]
SCOPE_NOTES_PATH = ROOT / "SCOPE_NOTES.md"
DECISION_TABLE_PATH = (
    ROOT / "docs" / "closeout_v21_v24" / "governance_surface_decision_table.md"
)
PUB_FN_RE = re.compile(r"^\s*pub fn\s+[A-Za-z_][A-Za-z0-9_]*\s*\(")


def has_doc_comment(lines: list[str], index: int) -> bool:
    cursor = index - 1
    saw_doc = False
    while cursor >= 0:
        stripped = lines[cursor].strip()
        if stripped.startswith("///") or stripped.startswith("#[doc ="):
            saw_doc = True
            cursor -= 1
            continue
        if stripped.startswith("#["):
            cursor -= 1
            continue
        if not stripped:
            return saw_doc
        return saw_doc
    return saw_doc


def crate_counts(crate: str) -> tuple[int, int]:
    total = 0
    documented = 0
    for path in sorted((ROOT / crate / "src").rglob("*.rs")):
        lines = path.read_text(encoding="utf-8").splitlines()
        for idx, line in enumerate(lines):
            if PUB_FN_RE.match(line):
                total += 1
                if has_doc_comment(lines, idx):
                    documented += 1
    return total, documented


def crate_has_lib_docs(crate: str) -> bool:
    lib = ROOT / crate / "src" / "lib.rs"
    if not lib.is_file():
        return False
    for line in lib.read_text(encoding="utf-8").splitlines():
        stripped = line.strip()
        if not stripped:
            continue
        return stripped.startswith("//!")
    return False


def main() -> int:
    failures: list[str] = []
    print("public api doc coverage report")
    for crate in DOC_CERTIFIED_CRATES:
        total, documented = crate_counts(crate)
        print(f"{crate}: documented {documented}/{total} public functions")
        if total > 0 and documented != total:
            failures.append(crate)

    if not SCOPE_NOTES_PATH.is_file():
        failures.append("SCOPE_NOTES.md missing for demoted compatibility crates")
    if not DECISION_TABLE_PATH.is_file():
        failures.append(
            "docs/closeout_v21_v24/governance_surface_decision_table.md missing"
        )

    scope_notes = (
        SCOPE_NOTES_PATH.read_text(encoding="utf-8")
        if SCOPE_NOTES_PATH.is_file()
        else ""
    )
    decision_table = (
        DECISION_TABLE_PATH.read_text(encoding="utf-8")
        if DECISION_TABLE_PATH.is_file()
        else ""
    )
    for crate in DEMOTED_COMPATIBILITY_CRATES:
        readme = ROOT / crate / "README.md"
        if not readme.is_file():
            failures.append(f"{crate} missing README.md")
        if crate not in scope_notes:
            failures.append(f"{crate} missing from SCOPE_NOTES.md")
        if crate not in decision_table:
            failures.append(f"{crate} missing from governance surface decision table")
        if not crate_has_lib_docs(crate):
            failures.append(f"{crate} missing crate-level lib docs")
        total, documented = crate_counts(crate)
        print(
            f"{crate}: compatibility-name crate, documented {documented}/{total} public functions"
        )

    if failures:
        print("public api doc truth is incomplete: " + ", ".join(failures), file=sys.stderr)
        return 1

    print("public api doc coverage check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
