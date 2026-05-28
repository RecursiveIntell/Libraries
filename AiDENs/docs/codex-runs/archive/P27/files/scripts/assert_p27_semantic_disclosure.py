#!/usr/bin/env python3
"""Assert P27 evidence-bearing CLI outputs expose 11A disclosure labels."""

from pathlib import Path
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
CLI_SRC = ROOT / "crates" / "aidens-cli" / "src"
LIB = CLI_SRC / "lib.rs"
AGENT = CLI_SRC / "agent.rs"
TESTS = CLI_SRC / "tests.rs"

REQUIRED_HELPER_SNIPPETS = [
    "fn semantic_disclosure_value",
    '"semantic_status"',
    '"exactness"',
    '"support_tier"',
    '"degradation"',
    '"proof_checks"',
    '"known_limits"',
    '"reference_semantics"',
]

REQUIRED_AGENT_SURFACES = {
    "agent_validate_command": "AgentSpecValidationReportV1",
    "agent_doctor_command": "AgentSpecDoctorReportV1",
    "agent_loop_output_json": "PlanActVerifyLoopV1OutputDisplay",
    "inspect_run_bundle_command": "AiDENsRunInspectReportV3",
}

REQUIRED_LABELS = [
    "exact_check",
    "degraded_exact_check",
    "failed_exact_check",
    "display_only",
]


def main() -> int:
    failures: list[str] = []
    lib_text = LIB.read_text() if LIB.exists() else ""
    agent_text = AGENT.read_text() if AGENT.exists() else ""
    tests_text = TESTS.read_text() if TESTS.exists() else ""

    if not lib_text:
        failures.append(f"missing CLI facade: {LIB}")
    if not agent_text:
        failures.append(f"missing agent CLI module: {AGENT}")
    if not tests_text:
        failures.append(f"missing CLI tests module: {TESTS}")

    for snippet in REQUIRED_HELPER_SNIPPETS:
        if snippet not in lib_text:
            failures.append(f"semantic helper missing snippet: {snippet}")

    combined = lib_text + "\n" + agent_text
    for label in REQUIRED_LABELS:
        if label not in combined:
            failures.append(f"missing semantic label: {label}")

    for function_name, report_schema in REQUIRED_AGENT_SURFACES.items():
        index = agent_text.find(function_name)
        if index < 0:
            failures.append(f"missing agent evidence surface: {function_name}")
            continue
        window = agent_text[index : index + 9000]
        if report_schema not in window:
            failures.append(f"{function_name} missing report schema {report_schema}")
        if "semantic_disclosure_value(" not in window:
            failures.append(f"{function_name} missing semantic disclosure block")

    for snippet in [
        'raw_report["semantic_disclosure"]["semantic_status"]',
        'valid["semantic_disclosure"]["exactness"]',
        'inspected_from_output["semantic_disclosure"]["support_tier"]',
    ]:
        if snippet not in tests_text:
            failures.append(f"missing semantic disclosure test assertion: {snippet}")

    if failures:
        print("P27 semantic disclosure guard FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("P27 semantic disclosure guard OK: CLI evidence surfaces carry 11A labels")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
