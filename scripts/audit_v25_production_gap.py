#!/usr/bin/env python3
from __future__ import annotations

import json
import re
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent

TARGETS = {
    "effect-runtime": {
        "files": [
            "effect-runtime/src/lib.rs",
            "effect-runtime/src/effect.rs",
            "effect-runtime/src/observation.rs",
            "effect-runtime/src/compensation.rs",
        ],
        "typed_id_markers": [
            "EffectIntentId",
            "EffectWindowId",
            "EffectPreflightReportId",
            "EffectCommitDecisionId",
            "EffectExecutionReceiptId",
            "EffectObservationBundleId",
            "CompensationPlanId",
            "CompensationExecutionReceiptId",
        ],
        "v25_ref_markers": [
            "ApplicabilityContextId",
            "ProfileSetId",
            "CompositionReceiptId",
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionConflictSetId",
            "ProfileExceptionBundleId",
        ],
    },
    "verification-control": {
        "files": ["verification-control/src/lib.rs"],
        "v25_ref_markers": [
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionReceiptId",
            "ApplicabilityContextId",
            "ProfileSetId",
            "CompositionConflictSetId",
            "ProfileExceptionBundleId",
        ],
    },
    "verification-policy": {
        "files": ["verification-policy/src/lib.rs"],
        "v25_ref_markers": [
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionReceiptId",
            "ProfileExceptionBundleId",
        ],
    },
    "verification-adjudication": {
        "files": ["verification-adjudication/src/lib.rs"],
        "v25_ref_markers": [
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionReceiptId",
            "ApplicabilityContextId",
            "ProfileSetId",
            "CompositionConflictSetId",
            "ProfileExceptionBundleId",
        ],
    },
    "remote-oracle-admission": {
        "files": ["remote-oracle-admission/src/lib.rs"],
        "v25_ref_markers": [
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionReceiptId",
            "ApplicabilityContextId",
            "ProfileSetId",
            "CompositionConflictSetId",
            "ProfileExceptionBundleId",
        ],
    },
    "federated-settlement": {
        "files": ["federated-settlement/src/lib.rs"],
        "v25_ref_markers": [
            "EffectiveConstitutionId",
            "CompiledObligationSetId",
            "CompositionReceiptId",
            "ApplicabilityContextId",
            "ProfileSetId",
            "CompositionConflictSetId",
            "ProfileExceptionBundleId",
        ],
    },
}

END_STATE_SCHEMA_STEMS = [
    "effect-intent-v1",
    "effect-preflight-report-v1",
    "effect-commit-decision-v1",
    "effect-execution-receipt-v1",
    "effect-observation-bundle-v1",
    "compensation-plan-v1",
    "compensation-execution-receipt-v1",
    "control-receipt-v1",
    "effect-review-case-v1",
    "effect-block-receipt-v1",
    "delegation-review-case-v1",
    "release-gate-case-v1",
    "continuity-review-case-v1",
    "policy-decision-v1",
    "promotion-decision-v1",
    "refutation-decision-v1",
    "rollback-plan-v1",
    "effect-adjudication-receipt-v1",
    "release-rollback-decision-v1",
    "remote-slice-request-v1",
    "remote-slice-result-v1",
    "cross-runtime-replay-ticket-v1",
    "settlement-case-v1",
    "settlement-receipt-v1",
    "shared-replay-slice-v1",
    "shared-divergence-report-v1",
    "shared-view-downgrade-v1",
    "local-dissent-record-v1",
]

SCHEMA_REG_MARKERS = {
    "effect-review-case-v1": "write_schema::<verification_control::EffectReviewCaseV1>",
    "effect-block-receipt-v1": "write_schema::<verification_control::EffectBlockReceiptV1>",
    "delegation-review-case-v1": "write_schema::<verification_control::DelegationReviewCaseV1>",
    "release-gate-case-v1": "write_schema::<verification_control::ReleaseGateCaseV1>",
    "continuity-review-case-v1": "write_schema::<verification_control::ContinuityReviewCaseV1>",
    "effect-adjudication-receipt-v1": "write_schema::<verification_adjudication::EffectAdjudicationReceiptV1>",
    "release-rollback-decision-v1": "write_schema::<verification_adjudication::ReleaseRollbackDecisionV1>",
}

EXAMPLE_TEST_FILES = {
    "effect-runtime": [
        "effect-runtime/tests/serde_roundtrip.rs",
        "effect-runtime/tests/fixture_conformance.rs",
        "effect-runtime/tests/v25_citation_flow.rs",
    ],
    "verification-control": [
        "verification-control/tests/v25_review_case_roundtrip.rs",
        "verification-control/tests/v25_citation_requirements.rs",
    ],
    "verification-policy": [
        "verification-policy/tests/v25_policy_citation_flow.rs",
        "verification-policy/tests/policy_profile_example_roundtrip.rs",
    ],
    "verification-adjudication": [
        "verification-adjudication/tests/policy_flow_integration.rs",
        "verification-adjudication/tests/v25_adjudication_citation_flow.rs",
    ],
    "remote-oracle-admission": [
        "remote-oracle-admission/tests/v25_local_constitution_refs.rs",
    ],
    "federated-settlement": [
        "federated-settlement/tests/v25_local_constitution_refs.rs",
    ],
}

RAW_PROFILE_FIELD_PATTERNS = [
    r"allowed_run_modes",
    r"required_preflight_checks",
    r"required_observation_classes",
    r"requires_compensation_plan_for",
    r"max_delegation_depth",
    r"forbidden_role_combinations",
    r"required_assurance_sections",
    r"required_monitor_classes",
    r"continuity_exception_ttl_minutes",
    r"requires_postmortem_for_severity",
    r"allowed_execution_regions",
    r"forbidden_transfer_classes",
    r"lossy_fields",
    r"break_glass_requires_post_hoc_review",
]
NO_LOCAL_RECOMPOSITION_SCAN_DIRS = [
    "effect-runtime",
    "verification-control",
    "verification-adjudication",
    "remote-oracle-admission",
    "federated-settlement",
]


def read(rel: str) -> str:
    return (ROOT / rel).read_text(encoding="utf-8")


def has_any_markers(rel_files: list[str], markers: list[str]) -> dict[str, bool]:
    text = "\n".join(read(rel) for rel in rel_files)
    return {marker: (marker in text) for marker in markers}


def existing(rel: str) -> bool:
    return (ROOT / rel).exists()


def current_ci_state() -> dict[str, bool]:
    ci_text = read(".github/workflows/ci.yml")
    make_text = read("Makefile")
    return {
        "ci_runs_v25_repo_truth": "check_v25_repo_truth.sh" in ci_text,
        "ci_runs_v25_json_surface": "check_v25_json_surface.py" in ci_text,
        "ci_runs_no_local_recomposition": "check_no_local_recomposition.sh" in ci_text,
        "make_has_v25_local_checks": "run_v25_local_checks" in make_text,
        "make_has_production_closure_target": "v25-production-closure" in make_text,
    }


def schema_and_example_state() -> dict[str, dict[str, bool]]:
    out: dict[str, dict[str, bool]] = {}
    for stem in END_STATE_SCHEMA_STEMS:
        out[stem] = {
            "schema": existing(f"schemas/{stem}.schema.json"),
            "example": existing(f"examples/{stem}.example.json"),
            "registered": SCHEMA_REG_MARKERS.get(stem, "") in read("contract-schema-gen/src/lib.rs") if stem in SCHEMA_REG_MARKERS else True,
        }
    return out


def no_local_recomposition_findings() -> list[dict[str, str]]:
    findings: list[dict[str, str]] = []
    combined = re.compile("|".join(RAW_PROFILE_FIELD_PATTERNS))
    for directory in NO_LOCAL_RECOMPOSITION_SCAN_DIRS:
        for path in (ROOT / directory).rglob("*.rs"):
            rel = path.relative_to(ROOT).as_posix()
            for lineno, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                if combined.search(line):
                    findings.append({"file": rel, "line": str(lineno), "text": line.strip()})
    return findings


report: dict[str, object] = {
    "repo_root": str(ROOT),
    "targets": {},
    "schema_and_example_state": schema_and_example_state(),
    "ci_and_make": current_ci_state(),
    "no_local_recomposition_findings": no_local_recomposition_findings(),
}

for target, config in TARGETS.items():
    target_report: dict[str, object] = {
        "files": config["files"],
        "v25_ref_markers": has_any_markers(config["files"], config.get("v25_ref_markers", [])),
        "test_files": {rel: existing(rel) for rel in EXAMPLE_TEST_FILES[target]},
    }
    if "typed_id_markers" in config:
        target_report["typed_id_markers"] = has_any_markers(config["files"], config["typed_id_markers"])
    report["targets"][target] = target_report

print(json.dumps(report, indent=2, sort_keys=True))
