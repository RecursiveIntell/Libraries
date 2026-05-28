#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

REQUIRED_FIELDS = {
    "id",
    "risk_surface",
    "input",
    "expected_policy",
    "required_receipts",
    "forbidden_behavior",
}
VALID_POLICIES = {
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
REQUIRED_SURFACES = {
    "high_impact_recommendation",
    "memory_personalization",
    "repeated_nudge",
    "tool_output_persuasion",
    "delegated_influence",
    "exit_respect",
    "synthetic_intimacy",
    "user_requested_manipulation",
    "alternative_set_integrity",
    "agency_degradation",
}


def main() -> int:
    ap = argparse.ArgumentParser(description="Validate P20.2 agency eval JSONL cases.")
    ap.add_argument("path", nargs="?", default="evals/p20_agency_eval_cases.jsonl")
    ap.add_argument("--min-cases", type=int, default=20)
    args = ap.parse_args()
    path = Path(args.path)
    if not path.exists():
        print(f"missing agency eval file: {path}", file=sys.stderr)
        return 2

    ids: set[str] = set()
    surfaces: set[str] = set()
    receipts: set[str] = set()
    count = 0
    errors: list[str] = []
    for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), 1):
        if not line.strip():
            continue
        count += 1
        try:
            obj = json.loads(line)
        except Exception as exc:
            errors.append(f"line {lineno}: invalid json: {exc}")
            continue

        missing = REQUIRED_FIELDS - set(obj)
        if missing:
            errors.append(f"line {lineno}: missing {sorted(missing)}")

        case_id = obj.get("id")
        if not isinstance(case_id, str) or not case_id.strip():
            errors.append(f"line {lineno}: id must be non-empty string")
        elif case_id in ids:
            errors.append(f"line {lineno}: duplicate id {case_id}")
        else:
            ids.add(case_id)

        risk_surface = obj.get("risk_surface")
        if not isinstance(risk_surface, str) or not risk_surface.strip():
            errors.append(f"line {lineno}: risk_surface must be non-empty string")
        else:
            surfaces.add(risk_surface)

        expected_policy = obj.get("expected_policy")
        if expected_policy not in VALID_POLICIES:
            errors.append(f"line {lineno}: invalid expected_policy {expected_policy!r}")

        if not isinstance(obj.get("input"), dict):
            errors.append(f"line {lineno}: input must be object")

        for array_name in ["required_receipts", "forbidden_behavior"]:
            value = obj.get(array_name)
            if not isinstance(value, list) or not value:
                errors.append(f"line {lineno}: {array_name} must be non-empty list[str]")
                continue
            for item in value:
                if not isinstance(item, str) or not item.strip():
                    errors.append(f"line {lineno}: {array_name} entries must be non-empty strings")
                elif array_name == "required_receipts":
                    receipts.add(item)

    if count < args.min_cases:
        errors.append(f"expected at least {args.min_cases} cases, found {count}")

    missing_surfaces = REQUIRED_SURFACES - surfaces
    if missing_surfaces:
        errors.append(f"missing required risk surfaces: {sorted(missing_surfaces)}")

    if errors:
        print("Agency eval validation FAILED", file=sys.stderr)
        for error in errors:
            print(f"- {error}", file=sys.stderr)
        return 2

    print(
        f"Agency eval validation OK: {count} cases, "
        f"{len(surfaces)} surfaces, {len(receipts)} receipt kinds"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
