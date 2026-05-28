#!/usr/bin/env bash
set -euo pipefail

ROOT="${1:-$(cd "$(dirname "$0")/.." && pwd)}"

need_files=(
  "22_PROFILE_COMPLETION_EXECUTION_LANE.md"
  "03_NO_V25_RATIONALE_AND_PROFILE_LAYER_RULE.md"
  "04_PROFILE_SUITE_OVERVIEW.md"
  "05_PROFILE_IMPLEMENTATION_SEQUENCE.md"
  "06_PROFILE_MASTER_ISSUE_MATRIX.md"
  "07_SCHEMA_REGISTRY_AND_COMPATIBILITY_PLAN.md"
  "08_CRATE_BOUNDARY_AND_OWNERSHIP_MAP.md"
  "09_EXACT_FILE_TOUCH_MAP.md"
  "10_RELEASE_BAR_AND_ACCEPTANCE.md"
  "11_PER_CRATE_APPLY_PLAN.md"
  "12_RISK_REGISTER.md"
  "13_FILE_CREATION_BACKLOG.md"
  "20_CURRENT_CODE_SNAPSHOT_NOTES_20260315.md"
  "CANONICAL_STACK_PROFILE_SPEC_P1_PRIVACY_RETENTION_DISCLOSURE_AND_REDACTION.md"
  "CANONICAL_STACK_PROFILE_SPEC_P2_LOCALITY_TENANCY_RESIDENCY_AND_BOUNDARY_OVERLAY.md"
  "CANONICAL_STACK_PROFILE_SPEC_P3_ROLE_CATALOG_DUTY_SEGREGATION_AND_APPROVAL_MATRIX.md"
  "CANONICAL_STACK_PROFILE_SPEC_P4_REGULATED_DEPLOYMENT_CONTROL_MAPPING_AND_RECERTIFICATION.md"
  "CANONICAL_STACK_PROFILE_SPEC_P5_SECTOR_HAZARD_LIBRARY_MONITOR_CATALOG_AND_MITIGATION_PLAYBOOK.md"
  "CANONICAL_STACK_PROFILE_SPEC_P6_VENDOR_CERTIFICATION_ADAPTER_AND_EXTERNAL_EVIDENCE_TRANSLATION.md"
  "CANONICAL_STACK_PROFILE_SPEC_P7_INCIDENT_TAXONOMY_ESCALATION_AND_PAGER_ROUTING_PROFILE.md"
  "docs/profile_completion_post_v24/README.md"
  "docs/profile_completion_post_v24/MASTER_ISSUE_MATRIX.md"
  "docs/profile_completion_post_v24/EXACT_FILE_TOUCH_MAP.md"
  "docs/profile_completion_post_v24/PER_CRATE_APPLY_PLAN.md"
  "docs/profile_completion_post_v24/RELEASE_BAR_AND_ACCEPTANCE.md"
  "docs/profile_completion_post_v24/REPO_GAP_REPORT_20260315.md"
  "plans/post-v24-profile-completion.execplan.md"
  "verification-policy/src/profile_p1_privacy.rs"
  "verification-policy/src/profile_p2_locality.rs"
  "authority-delegation/src/profile_p3_roles.rs"
  "assurance-runtime/src/profile_p4_regulated.rs"
  "assurance-runtime/src/profile_p5_hazard.rs"
  "attestation-exchange/src/profile_p6_vendor.rs"
  "continuity-runtime/src/profile_p7_incident_routing.rs"
)

for rel in "${need_files[@]}"; do
  test -f "$ROOT/$rel" || { echo "missing required file: $rel" >&2; exit 1; }
done

schema_files=(
  "access-purpose-matrix-v1.schema.json"
  "approval-matrix-v1.schema.json"
  "audit-extraction-policy-v1.schema.json"
  "conflict-class-catalog-v1.schema.json"
  "cross-boundary-transfer-class-v1.schema.json"
  "delegation-matrix-v1.schema.json"
  "escalation-clock-policy-v1.schema.json"
  "evidence-collection-plan-v1.schema.json"
  "hazard-library-v1.schema.json"
  "hazard-scenario-v1.schema.json"
  "incident-taxonomy-v1.schema.json"
  "locality-exception-v1.schema.json"
  "mitigation-playbook-v1.schema.json"
  "monitor-catalog-v1.schema.json"
  "pager-route-profile-v1.schema.json"
  "privacy-retention-profile-v1.schema.json"
  "recertification-schedule-v1.schema.json"
  "redaction-rule-set-v1.schema.json"
  "regulatory-regime-profile-v1.schema.json"
  "requirement-control-map-v1.schema.json"
  "residency-policy-profile-v1.schema.json"
  "role-catalog-v1.schema.json"
  "severity-matrix-v1.schema.json"
  "tenant-boundary-profile-v1.schema.json"
  "vendor-certification-adapter-v1.schema.json"
  "vendor-evidence-translation-v1.schema.json"
  "vendor-revocation-handling-v1.schema.json"
  "vendor-trust-root-binding-v1.schema.json"
)
for name in "${schema_files[@]}"; do
  test -f "$ROOT/schemas/$name" || { echo "missing schema: $name" >&2; exit 1; }
done

example_files=(
  "access-purpose-matrix-v1.example.json"
  "approval-matrix-v1.example.json"
  "audit-extraction-policy-v1.example.json"
  "conflict-class-catalog-v1.example.json"
  "cross-boundary-transfer-class-v1.example.json"
  "delegation-matrix-v1.example.json"
  "escalation-clock-policy-v1.example.json"
  "evidence-collection-plan-v1.example.json"
  "hazard-library-v1.example.json"
  "hazard-scenario-v1.example.json"
  "incident-taxonomy-v1.example.json"
  "locality-exception-v1.example.json"
  "mitigation-playbook-v1.example.json"
  "monitor-catalog-v1.example.json"
  "pager-route-profile-v1.example.json"
  "privacy-retention-profile-v1.example.json"
  "recertification-schedule-v1.example.json"
  "redaction-rule-set-v1.example.json"
  "regulatory-regime-profile-v1.example.json"
  "requirement-control-map-v1.example.json"
  "residency-policy-profile-v1.example.json"
  "role-catalog-v1.example.json"
  "severity-matrix-v1.example.json"
  "tenant-boundary-profile-v1.example.json"
  "vendor-certification-adapter-v1.example.json"
  "vendor-evidence-translation-v1.example.json"
  "vendor-revocation-handling-v1.example.json"
  "vendor-trust-root-binding-v1.example.json"
)
for name in "${example_files[@]}"; do
  test -f "$ROOT/examples/$name" || { echo "missing example: $name" >&2; exit 1; }
done

fixture_files=(
  "p1/audit_extraction_escalation.bundle.json"
  "p1/privacy_redaction_happy_path.bundle.json"
  "p2/locality_exception_happy_path.bundle.json"
  "p2/residency_transfer_blocked.bundle.json"
  "p3/conflict_recusal_blocked.bundle.json"
  "p3/delegation_matrix_happy_path.bundle.json"
  "p4/recertification_overdue_blocked.bundle.json"
  "p4/regulated_release_happy_path.bundle.json"
  "p5/hazard_monitor_happy_path.bundle.json"
  "p5/hazard_playbook_activation.bundle.json"
  "p6/vendor_adapter_happy_path.bundle.json"
  "p6/vendor_revocation_downgrade.bundle.json"
  "p7/incident_taxonomy_happy_path.bundle.json"
  "p7/pager_route_escalation_timeout.bundle.json"
)
for rel in "${fixture_files[@]}"; do
  test -f "$ROOT/contracts/fixtures/$rel" || { echo "missing fixture: $rel" >&2; exit 1; }
done

for dir in p1 p2 p3 p4 p5 p6 p7; do
  test -d "$ROOT/conformance/$dir" || { echo "missing conformance dir: $dir" >&2; exit 1; }
  test -f "$ROOT/conformance/$dir/README.md" || { echo "missing conformance note: $dir/README.md" >&2; exit 1; }
  test -f "$ROOT/contracts/schemas/$dir/manifest.json" || { echo "missing manifest: contracts/schemas/$dir/manifest.json" >&2; exit 1; }
done

grep -q "PrivacyRetentionProfileId" "$ROOT/stack-ids/src/ids.rs" || { echo "stack-ids missing PrivacyRetentionProfileId" >&2; exit 1; }
grep -q "EscalationClockPolicyId" "$ROOT/stack-ids/src/ids.rs" || { echo "stack-ids missing EscalationClockPolicyId" >&2; exit 1; }

grep -q "write_schema::<verification_policy::PrivacyRetentionProfileV1>" "$ROOT/contract-schema-gen/src/lib.rs" || { echo "contract-schema-gen missing PrivacyRetentionProfileV1 registry entry" >&2; exit 1; }
grep -q "write_schema::<continuity_runtime::EscalationClockPolicyV1>" "$ROOT/contract-schema-gen/src/lib.rs" || { echo "contract-schema-gen missing EscalationClockPolicyV1 registry entry" >&2; exit 1; }

grep -q "pub mod profile_p1_privacy;" "$ROOT/verification-policy/src/lib.rs" || { echo "verification-policy lib missing p1 module export" >&2; exit 1; }
grep -q "pub mod profile_p2_locality;" "$ROOT/verification-policy/src/lib.rs" || { echo "verification-policy lib missing p2 module export" >&2; exit 1; }
grep -q "pub mod profile_p3_roles;" "$ROOT/authority-delegation/src/lib.rs" || { echo "authority-delegation lib missing p3 module export" >&2; exit 1; }
grep -q "pub mod profile_p4_regulated;" "$ROOT/assurance-runtime/src/lib.rs" || { echo "assurance-runtime lib missing p4 module export" >&2; exit 1; }
grep -q "pub mod profile_p5_hazard;" "$ROOT/assurance-runtime/src/lib.rs" || { echo "assurance-runtime lib missing p5 module export" >&2; exit 1; }
grep -q "pub mod profile_p6_vendor;" "$ROOT/attestation-exchange/src/lib.rs" || { echo "attestation-exchange lib missing p6 module export" >&2; exit 1; }
grep -q "pub mod profile_p7_incident_routing;" "$ROOT/continuity-runtime/src/lib.rs" || { echo "continuity-runtime lib missing p7 module export" >&2; exit 1; }

echo "post-v24 profile repo truth checks passed"
