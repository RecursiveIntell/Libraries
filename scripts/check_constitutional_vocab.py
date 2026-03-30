#!/usr/bin/env python3
from __future__ import annotations

import re
import sys
from pathlib import Path


ROOT = Path(__file__).resolve().parent.parent
TARGETS = [
    ROOT / "effect-runtime" / "src",
    ROOT / "assurance-runtime" / "src",
    ROOT / "attestation-exchange" / "src",
    ROOT / "authority-delegation" / "src",
    ROOT / "continuity-runtime" / "src",
    ROOT / "remote-oracle-admission" / "src",
    ROOT / "verification-policy" / "src",
    ROOT / "verification-control" / "src",
    ROOT / "verification-adjudication" / "src",
]
CONTROLLED_VOCAB_FIELDS = {
    "admission_check_result",
    "blast_radius_ceiling",
    "budget_sufficiency_result",
    "close_midflight_behavior",
    "closure_recommendation",
    "commit_atomicity",
    "compensation_class",
    "coverage_state",
    "current_validity_state",
    "decision_state",
    "dependency_reachability_result",
    "disclosure_ceiling",
    "default_severity",
    "effect_class",
    "exactness_class",
    "exactness_class_ceiling",
    "exactness_target",
    "execution_state",
    "exercise_kind",
    "input_integrity_result",
    "local_admission_recommendation",
    "observation_state",
    "publication_status",
    "replayability_class",
    "replay_obligation",
    "replay_state",
    "retry_owner",
    "reversibility_class",
    "rotation_channel",
    "rotation_policy",
    "run_mode",
    "severity",
    "status",
    "trigger_kind",
    "allowed_disclosure",
    "admissibility_judgment",
    "admission_impact",
    "expiration_policy",
    "failure_behavior",
    "final_state",
    "final_disposition",
    "block_reason",
    "invalidation_radius",
    "lifecycle_state",
    "revocation_channel",
    "replay_impact",
    "rollback_scope",
    "rollback_class",
    "risk_class",
    "refutation_class",
    "refuter_kind",
    "promotion_impact",
    "state",
    "exactness_spend",
    "selected_decision",
    "current_disposition",
    "decision_class",
    "allowed_run_modes",
    "approval_requirement",
    "required_preflight_checks",
    "required_observation_classes",
    "requires_compensation_plan_for",
    "default_retention_class",
    "cross_border_transfer_default",
    "default_disclosure_budget_class",
    "default_decision",
    "disclosure_budget_class",
    "export_package_format",
    "forbidden_role_combinations",
    "required_assurance_sections",
    "required_monitor_classes",
    "required_forensic_freeze_surfaces",
    "requires_postmortem_for_severity",
    "tenant_key_kind",
    "isolation_class",
    "cross_tenant_query_default",
    "required_attestation",
    "required_disclosure_policy_class",
    "translation_mode",
    "retry_owner",
    "decision_state",
    "budget_class",
    "max_exactness",
    "refuter_allowance",
    "oracle_allowance",
    "replay_allowance",
    "human_review_allowance",
    "exhaustion_behavior",
    "required_replayability_class",
    "downgrade_behavior",
    "replay_visibility",
    "allowed_reveal_class",
}
FIELD_RE = re.compile(
    r"^\s*pub\s+(?P<field>[A-Za-z_][A-Za-z0-9_]*)\s*:\s*(?P<ty>Option<String>|String|Vec<String>)\s*,\s*$"
)


def main() -> int:
    failures: list[str] = []
    for target in TARGETS:
        for path in sorted(target.rglob("*.rs")):
            for line_no, line in enumerate(path.read_text(encoding="utf-8").splitlines(), start=1):
                match = FIELD_RE.match(line)
                if match is None:
                    continue
                field = match.group("field")
                if field in CONTROLLED_VOCAB_FIELDS:
                    failures.append(
                        f"{path.relative_to(ROOT)}:{line_no}: raw controlled-vocab String field `{field}`"
                    )

    if failures:
        print("constitutional vocab check failed:", file=sys.stderr)
        for failure in failures:
            print(f" - {failure}", file=sys.stderr)
        return 1

    print("constitutional vocab check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
