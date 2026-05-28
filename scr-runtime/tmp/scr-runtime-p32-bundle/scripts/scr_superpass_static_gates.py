#!/usr/bin/env python3
"""
Static gates for SCR runtime P32.

These gates are intentionally conservative. They do not replace Rust tests.
They catch known false-completion patterns from the current audit.
"""
from __future__ import annotations

import json
import pathlib
import re
import sys

MODE = sys.argv[1] if len(sys.argv) > 1 else "before"
ROOT = pathlib.Path.cwd()
errors: list[str] = []
warnings: list[str] = []

def text(path: str) -> str:
    p = ROOT / path
    if not p.exists():
        return ""
    return p.read_text(encoding="utf-8", errors="replace")

def require_contains(path: str, needles: list[str], label: str) -> None:
    body = text(path)
    for needle in needles:
        if needle not in body:
            errors.append(f"{label}: {path} missing {needle!r}")

def forbid_regex(path: str, pattern: str, label: str) -> None:
    body = text(path)
    if re.search(pattern, body):
        errors.append(f"{label}: forbidden pattern {pattern!r} in {path}")

# 1. Hooks must not be inert in final mode.
hooks = text(".codex/hooks.json")
if MODE == "final":
    if not hooks.strip():
        errors.append("missing .codex/hooks.json")
    if ":null" in hooks.replace(" ", ""):
        errors.append(".codex/hooks.json contains null hook entries")

# 2. Evaluator must not treat opaque evidence refs as signals.
ref_lib = text("crates/scr-reference/src/lib.rs")
forbidden_signal_patterns = [
    r"ref_kind\s*==\s*\"signal\"",
    r"ref_kind\.as_str\(\)\s*==\s*\"signal\"",
    r"normalize_signal\(&ref_?\.ref_value",
    r"evidence_refs.*filter.*signal",
]
for pat in forbidden_signal_patterns:
    if re.search(pat, ref_lib, flags=re.DOTALL):
        errors.append(f"opaque ref signal scanning remains in scr-reference: {pat}")

# 3. Kernel must contain expected stronger receipt/contract types in final mode.
if MODE == "final":
    kernel = text("crates/scr-kernel/src/lib.rs")
    expected_terms = [
        "ControlSignalV1",
        "ActionCandidateV1",
        "CandidateTrace",
        "AuthorityCheck",
        "EvidenceCheck",
    ]
    for term in expected_terms:
        if term not in kernel:
            errors.append(f"kernel missing expected strengthened type/term: {term}")

# 4. Generated schemas should not omit minLength entirely in final mode.
if MODE == "final":
    schema_paths = list((ROOT / "schemas/generated").glob("*.json"))
    if not schema_paths:
        errors.append("no generated schemas found")
    for sp in schema_paths:
        body = sp.read_text(encoding="utf-8", errors="replace")
        if '"type": "string"' in body and '"minLength"' not in body:
            errors.append(f"schema has string fields but no minLength constraints: {sp}")

# 5. Final docs must exist in final mode.
if MODE == "final":
    for path in [
        "docs/P32_COMPLETION_REPORT.md",
        "docs/P32_COMMAND_RECEIPTS.md",
        "docs/P32_CHANGED_FILES.md",
        "docs/P32_UNRESOLVED_RISKS.md",
        "docs/P32_HOSTILE_AUDITOR_HANDOFF.md",
        "docs/P32_POLICY_CHANGE_RECEIPT.md",
        "docs/P32_ROLLBACK_PLAN.md",
        "docs/SCR_CANONICAL_JSON_V1.md",
        "docs/SCR_ADAPTER_SEAMS.md",
        "docs/SCR_ACTION_SEMANTICS.md",
        "docs/SCHEMA_RUST_PARITY.md",
        "docs/EVALUATOR_BUILD_DIGEST.md",
    ]:
        if not (ROOT / path).exists():
            errors.append(f"missing required final doc: {path}")

payload = {"mode": MODE, "errors": errors, "warnings": warnings}
print(json.dumps(payload, indent=2))
if errors:
    sys.exit(1)
