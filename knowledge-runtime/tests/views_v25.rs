#![allow(clippy::expect_used)]

use knowledge_runtime::{
    CompiledObligationRuntimeViewV1, CompositionConflictRuntimeViewV1, EffectiveConstitutionViewV1,
    PolicyImpactDiffRuntimeViewV1,
};
use stack_ids::{
    ApplicabilityContextId, CompiledObligationSetId, CompositionConflictSetId,
    EffectiveConstitutionId, PolicyImpactDiffId, ProfileSetId,
};

#[test]
fn v25_runtime_views_roundtrip() {
    let constitution = EffectiveConstitutionViewV1 {
        schema_version: "effective_constitution_view_v1".into(),
        effective_constitution_id: EffectiveConstitutionId::new("effective-constitution-1"),
        applicability_context_id: ApplicabilityContextId::new("applicability-context-1"),
        profile_set_id: ProfileSetId::new("profile-set-1"),
        mode_classification: "normal".into(),
        blocking: false,
        approval_requirements: vec!["locality-review-board".into()],
        hard_block_summary: vec![],
        active_warning_summary: vec!["vendor_translation_lossy".into()],
        valid_as_of: "2026-03-16T14:00:00Z".into(),
        recorded_as_of: "2026-03-16T14:00:05Z".into(),
    };
    let obligations = CompiledObligationRuntimeViewV1 {
        schema_version: "compiled_obligation_runtime_view_v1".into(),
        compiled_obligation_set_id: CompiledObligationSetId::new("compiled-obligation-set-1"),
        effective_constitution_id: constitution.effective_constitution_id.clone(),
        required_approvals: vec!["locality-review-board".into()],
        required_checks: vec!["integrity".into(), "admission".into()],
        required_monitors: vec!["cross_region_replay_monitor".into()],
        required_disclosure_obligations: vec!["audit_redaction_rule_set:rrs_001".into()],
        required_post_hoc_review_obligations: vec!["locality_exception_post_review".into()],
        unresolved_dependency_refs: vec![],
        effect_eligibility_summary: "eligible_with_approvals".into(),
    };
    let conflicts = CompositionConflictRuntimeViewV1 {
        schema_version: "composition_conflict_runtime_view_v1".into(),
        composition_conflict_set_id: CompositionConflictSetId::new("composition-conflict-set-1"),
        effective_constitution_id: Some(EffectiveConstitutionId::new("effective-constitution-2")),
        blocking: true,
        conflict_keys: vec!["disclosure.export_format:export_package_format".into()],
        review_paths: vec!["human_constitution_review".into()],
        admissible_exception_classes: vec!["disclosure_exception".into()],
    };
    let diff = PolicyImpactDiffRuntimeViewV1 {
        schema_version: "policy_impact_diff_runtime_view_v1".into(),
        policy_impact_diff_id: PolicyImpactDiffId::new("policy-impact-diff-1"),
        from_effective_constitution_id: EffectiveConstitutionId::new("effective-constitution-0"),
        to_effective_constitution_id: constitution.effective_constitution_id.clone(),
        changed_obligation_families: vec!["monitor.required".into()],
        newly_blocked_paths: vec![],
        newly_admitted_paths: vec![
            "residency.forbidden_transfer_classes:customer_pii_to_non_us".into(),
        ],
        migration_consequences: vec!["monitoring plan update required".into()],
        simulation_only: false,
    };

    let json = serde_json::to_string(&(constitution, obligations, conflicts, diff))
        .expect("serialize v25 runtime views");
    let _: (
        EffectiveConstitutionViewV1,
        CompiledObligationRuntimeViewV1,
        CompositionConflictRuntimeViewV1,
        PolicyImpactDiffRuntimeViewV1,
    ) = serde_json::from_str(&json).expect("deserialize v25 runtime views");
}

#[test]
fn v25_runtime_view_examples_parse() {
    let constitution =
        std::fs::read_to_string("../examples/effective-constitution-view-v1.example.json")
            .expect("read constitution view example");
    let _: EffectiveConstitutionViewV1 =
        serde_json::from_str(&constitution).expect("parse constitution view example");

    let obligations =
        std::fs::read_to_string("../examples/compiled-obligation-runtime-view-v1.example.json")
            .expect("read obligation view example");
    let _: CompiledObligationRuntimeViewV1 =
        serde_json::from_str(&obligations).expect("parse obligation view example");

    let conflicts =
        std::fs::read_to_string("../examples/composition-conflict-runtime-view-v1.example.json")
            .expect("read conflict view example");
    let _: CompositionConflictRuntimeViewV1 =
        serde_json::from_str(&conflicts).expect("parse conflict view example");

    let diff =
        std::fs::read_to_string("../examples/policy-impact-diff-runtime-view-v1.example.json")
            .expect("read diff view example");
    let _: PolicyImpactDiffRuntimeViewV1 =
        serde_json::from_str(&diff).expect("parse diff view example");
}
