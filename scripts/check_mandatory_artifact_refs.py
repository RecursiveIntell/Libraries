#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent


def require_pattern(path: str, pattern: str, description: str) -> str | None:
    content = (ROOT / path).read_text(encoding="utf-8")
    if re.search(pattern, content, re.MULTILINE | re.DOTALL):
        return None
    return f"{path}: missing {description}"


def main() -> int:
    failures = [
        require_pattern(
            "verification-policy/src/lib.rs",
            r"pub\s+plan_id:\s*stack_ids::CheckPlanId,",
            "non-optional plan_id on PolicyDecision",
        ),
        require_pattern(
            "verification-policy/src/permit.rs",
            r"if\s+plan\.advisory_only\s*\|\|\s*!plan\.promotable_if_completed",
            "permit minting rejection for advisory-only or non-promotable plans",
        ),
        require_pattern(
            "forge-pilot/src/loop_runner_report.rs",
            r"pub\s+verification_plan_artifact:\s*Option<VerificationPlanArtifact>,",
            "loop report verification-plan artifact field",
        ),
        require_pattern(
            "forge-pilot/src/loop_runner.rs",
            r"verification_plan_artifact:\s*Some\(verification_plan_artifact\),",
            "loop iterations always carrying the verification-plan artifact",
        ),
        require_pattern(
            "forge-pilot/src/receipts.rs",
            r"promotion_blocked_on_missing_proof\s*=\s*!plan\.promotable_if_completed\s*\|\|\s*plan\.advisory_only\s*\|\|\s*!proof_obligations_remaining\.is_empty\(\)\s*\|\|\s*!policy_blockers\.is_empty\(\);",
            "promotion blocked when proof obligations or policy blockers remain",
        ),
        require_pattern(
            "remote-oracle-admission/src/lib.rs",
            r"require_non_empty_slice\(&self\.required_artifact_refs,\s*\"required_artifact_refs\"\)\?;",
            "remote request required_artifact_refs validation",
        ),
        require_pattern(
            "remote-oracle-admission/src/lib.rs",
            r"require_non_empty_slice\(&self\.returned_artifact_refs,\s*\"returned_artifact_refs\"\)\?;",
            "remote result returned_artifact_refs validation",
        ),
        require_pattern(
            "remote-oracle-admission/src/lib.rs",
            r"require_non_empty_slice\(&self\.artifact_refs,\s*\"artifact_refs\"\)\?;",
            "cross-runtime replay artifact_refs validation",
        ),
        require_pattern(
            "effect-runtime/src/effect.rs",
            r"require_non_empty_slice\(&self\.initiating_artifact_refs,\s*\"initiating_artifact_refs\"\)\?;",
            "effect intent initiating_artifact_refs validation",
        ),
        require_pattern(
            "effect-runtime/src/effect.rs",
            r"require_non_empty_slice\(\s*&self\.obligation_refs\.required_obligation_refs,\s*\"required_obligation_refs\",\s*\)\?;",
            "effect receipts required_obligation_refs validation",
        ),
        require_pattern(
            "effect-runtime/src/observation.rs",
            r"require_id\(\s*&self\.effect_execution_receipt_id,\s*\"effect_execution_receipt_id\",\s*\)\?;",
            "effect observation execution receipt validation",
        ),
        require_pattern(
            "forge-memory-bridge/src/transform.rs",
            r"canonical bundle-bearing export is missing episode_id",
            "canonical bundle episode_id rejection",
        ),
        require_pattern(
            "forge-pilot/tests/execution_evidence_lineage_tests.rs",
            r"verification_plan_artifact:\s*Some\(verification_plan_artifact\)",
            "execution lineage test coverage for verification-plan artifacts",
        ),
        require_pattern(
            "forge-memory-bridge/tests/forge_bridge_memory_proof.rs",
            r"canonical_v3_transform_rejects_missing_episode_id_for_bundle_lane",
            "bridge test coverage for missing episode_id rejection",
        ),
        require_pattern(
            "remote-oracle-admission/tests/validation_tests.rs",
            r"fn\s+request_requires_artifact_refs\(\)",
            "remote admission test coverage for required artifact refs",
        ),
        require_pattern(
            "remote-oracle-admission/tests/validation_tests.rs",
            r"fn\s+result_requires_returned_artifact_refs\(\)",
            "remote admission test coverage for returned artifact refs",
        ),
    ]

    failures = [failure for failure in failures if failure is not None]
    if failures:
        print("mandatory artifact-ref checks failed:", file=sys.stderr)
        for failure in failures:
            print(f" - {failure}", file=sys.stderr)
        return 1

    print("mandatory artifact-ref checks passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
