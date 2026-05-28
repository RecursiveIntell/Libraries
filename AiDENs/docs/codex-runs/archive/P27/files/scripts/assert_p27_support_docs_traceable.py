#!/usr/bin/env python3
"""Assert P27 support/quickstart docs are current, traceable, and fenced."""

from pathlib import Path
import sys


ROOT = Path(sys.argv[1]) if len(sys.argv) > 1 else Path(".")
SUPPORT = ROOT / "SUPPORT_PROFILE.md"
README = ROOT / "README.md"
QUICKSTART = ROOT / "docs" / "OPERATOR_QUICKSTART.md"
TRACE = ROOT / "docs" / "P27_SUPPORT_TRACEABILITY.md"
STATUS = ROOT / "STATUS.md"

REQUIRED_SUPPORT = [
    "## Supported-Local",
    "Mock-provider Plan",
    "Durable `AiDENsRunBundleV3`",
    "11A-style semantic disclosure",
    "## Partial",
    "## Deferred",
    "deferred-cloud",
    "deferred-autonomy",
    "design-only",
    "semantic_disclosure",
]

REQUIRED_TRACE = [
    "target/p27/audit/cargo_test_integration_phase17_provider_e2e.log",
    "target/p27/audit/cargo_test_integration_phase17_run_bundle_store.log",
    "target/p27/audit/assert_p27_semantic_disclosure_phase17.log",
    "target/p27/audit/assert_p27_memory_no_local_truth.log",
    "target/p27/audit/assert_p27_ownership_scanner_fail_closed.log",
]

REQUIRED_QUICKSTART = [
    "agent validate",
    "agent doctor",
    "agent run",
    "agent inspect",
    "target/p27/examples/local-coding-agent",
    "P27_SKIP_CARGO=1 bash scripts/verify_current.sh",
    "semantic_disclosure",
]

FORBIDDEN_CURRENT_DOC_SNIPPETS = [
    "P20_2_REQUIRE_CARGO",
    "scripts/p20_2_verify.sh",
    "target/p26/examples",
    "full verifier remains blocked by ownership scanner",
    "These are targets, not claims",
]


def read(path: Path, failures: list[str]) -> str:
    if not path.exists():
        failures.append(f"missing required doc: {path}")
        return ""
    return path.read_text()


def main() -> int:
    failures: list[str] = []
    support = read(SUPPORT, failures)
    readme = read(README, failures)
    quickstart = read(QUICKSTART, failures)
    trace = read(TRACE, failures)
    status = read(STATUS, failures)
    current_docs = {
        str(SUPPORT): support,
        str(README): readme,
        str(QUICKSTART): quickstart,
        str(TRACE): trace,
        str(STATUS): status,
    }

    for snippet in REQUIRED_SUPPORT:
        if snippet not in support:
            failures.append(f"SUPPORT_PROFILE missing: {snippet}")
    for snippet in REQUIRED_TRACE:
        if snippet not in trace:
            failures.append(f"P27_SUPPORT_TRACEABILITY missing evidence path: {snippet}")
    for snippet in REQUIRED_QUICKSTART:
        if snippet not in quickstart:
            failures.append(f"OPERATOR_QUICKSTART missing: {snippet}")

    for path, text in current_docs.items():
        for snippet in FORBIDDEN_CURRENT_DOC_SNIPPETS:
            if snippet in text:
                failures.append(f"{path} contains stale or unsupported snippet: {snippet}")

    if "P27_SUPPORT_TRACEABILITY.md" not in readme:
        failures.append("README does not point to P27 support traceability")
    if "Phase 18" not in status:
        failures.append("STATUS missing Phase 18 ledger entry")

    if failures:
        print("P27 support docs traceability guard FAILED", file=sys.stderr)
        for failure in failures:
            print(f"- {failure}", file=sys.stderr)
        return 1

    print("P27 support docs traceability OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
