#!/usr/bin/env python3
"""Assert P27 agency/governance eval harness coverage and honesty labels."""

from pathlib import Path
import json
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
EVALS = ROOT / "evals" / "p20_agency_eval_cases.jsonl"
AGENCY_SRC = ROOT / "crates" / "aidens-agency-kit" / "src" / "lib.rs"
REQUIRED_CASES = {
    "p27_tool_scarcity_deadline": "tool_output_persuasion",
    "p27_high_impact_housing_urgency_single_path": "high_impact_recommendation",
    "p27_decorative_alternatives_one_viable": "alternative_set_integrity",
    "p27_requested_guilt_manipulation": "user_requested_manipulation",
    "p27_dependency_loop_direct": "emotional_dependence",
    "p27_tool_conflict_without_urgency": "tool_conflict_of_interest",
    "p27_low_impact_urgency_control": "low_impact_advice",
}
REQUIRED_LABEL = 'aidens-heuristic-boundary-classifier-v1'


def main() -> int:
    failures: list[str] = []
    if not EVALS.exists():
        failures.append(f"missing agency eval fixture: {EVALS}")
        cases = []
    else:
        cases = []
        for line_no, line in enumerate(EVALS.read_text().splitlines(), 1):
            if not line.strip():
                continue
            try:
                cases.append(json.loads(line))
            except json.JSONDecodeError as error:
                failures.append(f"{EVALS}:{line_no}: invalid JSON: {error}")

    ids = [case.get("id") for case in cases]
    if len(ids) != len(set(ids)):
        failures.append("agency eval fixture has duplicate ids")
    by_id = {case.get("id"): case for case in cases}
    for case_id, risk_surface in REQUIRED_CASES.items():
        case = by_id.get(case_id)
        if case is None:
            failures.append(f"missing P27 agency eval case: {case_id}")
            continue
        if case.get("risk_surface") != risk_surface:
            failures.append(
                f"{case_id} risk_surface={case.get('risk_surface')} expected {risk_surface}"
            )
        for key in ["expected_policy", "required_receipts", "forbidden_behavior"]:
            if key not in case:
                failures.append(f"{case_id} missing {key}")

    if not AGENCY_SRC.exists():
        failures.append(f"missing agency source: {AGENCY_SRC}")
    else:
        text = AGENCY_SRC.read_text()
        if REQUIRED_LABEL not in text:
            failures.append("agency classifier label is not heuristic-v1")
        if "AgencyPolicyClassifierKindV1::HeuristicBoundaryClassifier" not in text:
            failures.append("agency classifier kind is not explicit heuristic classifier")

    if failures:
        print("agency eval harness guard FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print(
        f"agency eval harness OK: cases={len(cases)} p27_cases={len(REQUIRED_CASES)} heuristic_label={REQUIRED_LABEL}"
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
