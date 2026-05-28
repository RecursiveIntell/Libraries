#![allow(clippy::expect_used)]

use knowledge_runtime::{
    ContinuityRuntimeViewV1, DelegationRuntimeViewV1, DeployabilityRuntimeViewV1,
    EffectRuntimeViewV1,
};
use stack_ids::{
    AssuranceCaseId, AuthorityLeaseId, DeploymentProfileId, EffectIntentId, EffectWindowId,
    ErrorBudgetLedgerId, IncidentCaseId, RecoveryPlanId,
};

#[test]
fn v21_v24_runtime_views_roundtrip() {
    let effect = EffectRuntimeViewV1 {
        schema_version: "effect_runtime_view_v1".into(),
        effect_intent_id: EffectIntentId::new("effect-intent-1"),
        effect_window_id: Some(EffectWindowId::new("effect-window-1")),
        reversibility_class: "compensatable".into(),
        run_mode: "live".into(),
        publication_status: "required".into(),
        degradation_markers: vec!["none".into()],
    };
    let delegation = DelegationRuntimeViewV1 {
        schema_version: "delegation_runtime_view_v1".into(),
        authority_lease_id: AuthorityLeaseId::new("authority-lease-1"),
        validity_state: "valid".into(),
        advisory_only: false,
        revocation_refs: vec![],
        missing_backpointers: vec!["acting_on_behalf_receipt".into()],
    };
    let deployability = DeployabilityRuntimeViewV1 {
        schema_version: "deployability_runtime_view_v1".into(),
        deployment_profile_id: DeploymentProfileId::new("deployment-profile-1"),
        assurance_case_id: AssuranceCaseId::new("assurance-case-1"),
        release_readiness_decision_id: None,
        readiness_state: "advisory_only".into(),
        advisory_only: true,
        open_gaps: vec!["monitoring plan pending".into()],
    };
    let continuity = ContinuityRuntimeViewV1 {
        schema_version: "continuity_runtime_view_v1".into(),
        incident_case_id: IncidentCaseId::new("incident-case-1"),
        recovery_plan_id: Some(RecoveryPlanId::new("recovery-plan-1")),
        continuity_exception_id: None,
        error_budget_ledger_id: Some(ErrorBudgetLedgerId::new("error-budget-ledger-1")),
        replay_completeness: "partial".into(),
        degradation_markers: vec!["missing forensic freeze".into()],
    };

    let json = serde_json::to_string(&(effect, delegation, deployability, continuity))
        .expect("serialize runtime views");
    let _: (
        EffectRuntimeViewV1,
        DelegationRuntimeViewV1,
        DeployabilityRuntimeViewV1,
        ContinuityRuntimeViewV1,
    ) = serde_json::from_str(&json).expect("deserialize runtime views");
}
