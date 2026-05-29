#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(sys.argv[1] if len(sys.argv) > 1 else ".").resolve()
LEDGER = ROOT / "docs" / "codex-runs" / "CURRENT_RUN.json"
CLASS = ROOT / "docs" / "codex-runs" / "CODEX_ARTIFACT_CLASSIFICATION.json"
RUN_ARTIFACT_RE = re.compile(r"(?:^|[/_.-])P\d+[A-Z]?(?:[_./-]|$)|CODEX|codex|handoff|phase", re.I)
ALLOWED_PREFIXES = ("docs/codex-runs/archive/", "docs/root-markdown-archive/", "docs/source-packages/archive/", "target/", ".git/", "crates/aidens-runner/target/")
FINISH_PACK_RE = re.compile(r"^(?:aidens|AiDENs)[-_](?:hostile_audit|p\d+[a-z]?_hermes|p\d+[a-z]?_)*finish_pack\.zip$", re.I)
GENERATED_PACKAGE_RE = re.compile(r"^[^/]+-(?:codex-context|next-codex-context|release-context|codex-run-full|full-context|audit-full|source-clean|research-context)-\d{8}T?\d{0,6}Z?\.(?:zip|manifest\.json|report\.md|excluded\.json|findings\.json|codex-archive\.json)$")


def fail(msgs: list[str]) -> int:
    for m in msgs:
        print(f"FAIL: {m}", file=sys.stderr)
    return 2


def main() -> int:
    errors: list[str] = []
    if not LEDGER.exists():
        errors.append(f"missing {LEDGER.relative_to(ROOT)}")
        active = ""
    else:
        data = json.loads(LEDGER.read_text(encoding="utf-8"))
        active = str(data.get("active_run", "")).upper()
    if not CLASS.exists():
        errors.append(f"missing {CLASS.relative_to(ROOT)}")
        return fail(errors)
    try:
        raw = json.loads(CLASS.read_text(encoding="utf-8"))
    except Exception as e:
        return fail([f"classification JSON invalid: {e}"])
    entries = raw.get("artifacts", raw if isinstance(raw, list) else [])
    classified: dict[str, dict] = {}
    for item in entries:
        if isinstance(item, dict) and item.get("path"):
            classified[str(item["path"]).strip("/")] = item

    violations: list[str] = []
    bad_active: list[str] = []
    for p in ROOT.rglob("*"):
        if not p.is_file():
            continue
        rel = p.relative_to(ROOT).as_posix()
        if rel.startswith(ALLOWED_PREFIXES) or "__pycache__" in p.parts or "target" in p.parts:
            continue
        if GENERATED_PACKAGE_RE.match(rel):
            continue
        if FINISH_PACK_RE.match(rel):
            continue
        if RUN_ARTIFACT_RE.search(rel):
            item = classified.get(rel)
            if not item:
                violations.append(rel)
                continue
            cls = str(item.get("classification", "")).lower()
            run = str(item.get("run", "")).upper()
            if cls in {"active", "current", "current-instruction"} and run != active:
                bad_active.append(f"{rel}: classification={cls} run={run} active={active}")
    if violations:
        errors.append("unclassified run/Codex artifacts remain active:\n  " + "\n  ".join(violations[:300]))
    if bad_active:
        errors.append("non-active run artifacts classified active/current:\n  " + "\n  ".join(bad_active[:300]))
    if errors:
        return fail(errors)
    print("PASS: Codex/run artifacts are classified and active classifications match ledger")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
