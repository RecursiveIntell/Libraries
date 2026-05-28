#!/usr/bin/env python3
from __future__ import annotations
import argparse, json, sys
from pathlib import Path

REQUIRED_TOP_LEVEL = {
    "id", "risk_surface", "input", "expected_policy", "required_receipts", "forbidden_behavior"
}
ALLOWED_EXPECTED = {
    "allow",
    "allow_with_disclosure",
    "block",
    "defer_to_professional_or_external_source",
    "degrade",
    "gate",
    "quarantine",
    "redact",
    "require_alternatives",
    "require_user_confirmation",
    "warn",
}

def main() -> int:
    ap = argparse.ArgumentParser(description="Validate P20/P20.1 agency eval JSONL cases.")
    ap.add_argument("path", nargs="?", default="evals/p20_agency_eval_cases.jsonl")
    ap.add_argument("--min-cases", type=int, default=8)
    args = ap.parse_args()
    path = Path(args.path)
    if not path.exists():
        print(f"ERROR: agency eval file missing: {path}", file=sys.stderr)
        return 2
    case_count = 0
    ids: set[str] = set()
    errors: list[str] = []
    surfaces: set[str] = set()
    receipts: set[str] = set()
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        case_count += 1
        try:
            obj = json.loads(line)
        except Exception as exc:
            errors.append(f"line {lineno}: invalid JSON: {exc}")
            continue
        missing = REQUIRED_TOP_LEVEL - set(obj)
        if missing:
            errors.append(f"line {lineno}: missing keys {sorted(missing)}")
        cid = obj.get("id")
        if not isinstance(cid, str) or not cid.strip():
            errors.append(f"line {lineno}: id must be non-empty string")
        elif cid in ids:
            errors.append(f"line {lineno}: duplicate id {cid}")
        else:
            ids.add(cid)
        risk = obj.get("risk_surface")
        if not isinstance(risk, str) or not risk.strip():
            errors.append(f"line {lineno}: risk_surface must be non-empty string")
        else:
            surfaces.add(risk)
        expected = obj.get("expected_policy")
        if isinstance(expected, str):
            if expected not in ALLOWED_EXPECTED:
                errors.append(f"line {lineno}: expected_policy {expected!r} not in {sorted(ALLOWED_EXPECTED)}")
        elif isinstance(expected, dict):
            action = expected.get("action")
            if action not in ALLOWED_EXPECTED:
                errors.append(f"line {lineno}: expected_policy.action {action!r} not in {sorted(ALLOWED_EXPECTED)}")
        else:
            errors.append(f"line {lineno}: expected_policy must be string or object")
        req = obj.get("required_receipts")
        if not isinstance(req, list) or not req:
            errors.append(f"line {lineno}: required_receipts must be non-empty list")
        else:
            for r in req:
                if not isinstance(r, str) or not r.strip():
                    errors.append(f"line {lineno}: required_receipts entries must be non-empty strings")
                else:
                    receipts.add(r)
        forbidden = obj.get("forbidden_behavior")
        if not isinstance(forbidden, list) or not forbidden:
            errors.append(f"line {lineno}: forbidden_behavior must be non-empty list")
        inp = obj.get("input")
        if not isinstance(inp, (str, dict)):
            errors.append(f"line {lineno}: input must be string or object")
    if case_count < args.min_cases:
        errors.append(f"expected at least {args.min_cases} cases, found {case_count}")
    expected_surfaces = {
        "high_impact_recommendation",
        "repeated_nudge",
        "memory_personalization",
        "tool_output_persuasion",
        "delegated_influence",
        "synthetic_intimacy",
        "exit_respect",
        "alternative_set_integrity",
    }
    missing_surfaces = expected_surfaces - surfaces
    if missing_surfaces:
        errors.append(f"missing required risk surfaces: {sorted(missing_surfaces)}")
    if errors:
        print("Agency eval validation FAILED", file=sys.stderr)
        for e in errors:
            print(f"- {e}", file=sys.stderr)
        return 2
    print(f"Agency eval validation OK: {case_count} cases, {len(surfaces)} surfaces, {len(receipts)} receipt kinds")
    return 0

if __name__ == "__main__":
    raise SystemExit(main())
