//! Decisions/reports cannot authorize a different mutable plan or case.
use stack_ids::{AttemptId, ScopeKey, TraceCtx};
use verification_control::{
    CaseRegion, CheckMethod, CheckPlan, PromotionClass, ReversibilityClass, VerificationCase,
    VerificationCaseClass,
};
use verification_policy::{evaluate_policy, PolicyDecision, PolicySnapshot};

fn fixture() -> (VerificationCase, CheckPlan, PolicySnapshot) {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: "isolated".into(),
            scope_key: Some(ScopeKey::namespace_only("isolated")),
            target_key: "candidate".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("binding-test"),
        "2026-11-12T00:00:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::PairedPatch,
        vec!["test".into()],
        PromotionClass::P2,
        ReversibilityClass::RequiresSupersession,
        true,
        false,
        false,
        "test binding",
        serde_json::json!({"patch_hash":"original"}),
    );
    let policy = PolicySnapshot::permissive("explicit-test-policy", "2026-01-01T00:00:00Z");
    (case, plan, policy)
}
#[test]
fn changed_plan_or_case_with_reused_ids_cannot_consume_decision() {
    let (case, plan, policy) = fixture();
    let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    assert!(decision.issue_execution_permit(&case, &plan, &[]).is_ok());
    let mut changed = plan.clone();
    changed.input = serde_json::json!({"patch_hash":"substituted"});
    assert!(
        decision
            .issue_execution_permit(&case, &changed, &[])
            .is_err(),
        "mutated plan reused authorization ID"
    );
    let mut changed_case = case.clone();
    changed_case.region.namespace = "another-namespace".into();
    assert!(
        decision
            .issue_execution_permit(&changed_case, &plan, &[])
            .is_err(),
        "mutated case reused authorization ID"
    );
}
#[test]
fn policy_report_roundtrip_is_projection_not_execution_authority() {
    let (case, plan, policy) = fixture();
    let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    let report: PolicyDecision =
        serde_json::from_str(&serde_json::to_string(&decision).unwrap()).unwrap();
    assert!(report.validate().is_ok());
    assert!(
        report.issue_execution_permit(&case, &plan, &[]).is_err(),
        "serialized report manufactured runtime authority"
    );
}
#[test]
fn modifying_public_decision_flags_cannot_turn_denial_into_permission() {
    let (case, plan, mut policy) = fixture();
    for rule in &mut policy.method_rules {
        rule.allowed = false;
    }
    let mut decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    decision.method_allowed = true;
    decision.promotion_allowed = true;
    decision.autonomy_allowed = true;
    assert!(
        decision.issue_execution_permit(&case, &plan, &[]).is_err(),
        "modified report flags authorized dispatch"
    );
}
