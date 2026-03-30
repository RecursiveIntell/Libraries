use serde_json::json;
use stack_ids::{AttemptId, ClaimVersionId, ScopeKey, TraceCtx, TrialId};
use verification_adjudication::{adjudicate_case, VerificationDisposition};
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    schedule_check_plan, BudgetLineage, CaseRegion, CheckMethod, CheckPlan, ControlReceipt,
    DegradationMarker, PromotionClass, QueueHop, ReversibilityClass, VerificationAttempt,
    VerificationAttemptState, VerificationCase, VerificationCaseClass,
};
use verification_policy::{
    evaluate_policy, ApprovalRequirement, AutonomyCeiling, MethodPolicy, PolicySnapshot,
};

fn build_case() -> VerificationCase {
    VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "claim:v25-adjudication".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v25")),
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-v25-adjudication"),
        "2026-03-17T00:00:00Z",
        false,
        false,
    )
}

fn build_plan(case: &VerificationCase) -> CheckPlan {
    CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["exact".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "v25 adjudication lane",
        json!({
            "target_key": case.region.target_key,
            "claim_version_id": case.region.claim_version_id,
        }),
    )
}

fn build_attempt(case: &VerificationCase, plan: &CheckPlan, degraded: bool) -> VerificationAttempt {
    VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        case.attempt_id.clone(),
        Some(TrialId::new("trial-v25")),
        VerificationAttemptState::Succeeded,
        false,
        degraded,
        "2026-03-17T00:00:05Z",
        "2026-03-17T00:00:06Z",
        Some("oracle:matched".into()),
    )
}

fn build_policy(approval_requirement: ApprovalRequirement) -> PolicySnapshot {
    PolicySnapshot {
        schema_version: verification_policy::POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-25".into(),
        effective_from: "2026-03-16T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method: CheckMethod::ExactBoundedOracle,
            allowed: true,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: verification_policy::V25CitationContext::missing(),
        obligation_refs: verification_policy::V25ControlObligationRefs::missing(),
    }
}

fn build_control_receipt(
    case: &VerificationCase,
    plan: &CheckPlan,
    attempt: &VerificationAttempt,
) -> ControlReceipt {
    ControlReceipt::new_case_execution(
        case,
        plan,
        attempt,
        true,
        json!({
            "target_key": case.region.target_key,
            "replay_recipe": ["replay:temporal"],
        }),
    )
}

#[test]
fn adjudication_outputs_retain_policy_decision_reference_ids() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, false);
    let control = build_control_receipt(&case, &plan, &attempt);
    let policy = evaluate_policy(
        &build_policy(ApprovalRequirement::None),
        &case,
        &plan,
        &[],
        false,
        false,
    );
    let scheduler = schedule_check_plan(
        &case,
        &plan,
        BudgetLineage {
            budget_family: "verification".into(),
            retry_family: case.attempt_id.clone(),
            queue_hop_count: 0,
            max_time_budget_ms: Some(30_000),
            remaining_time_budget_ms: Some(30_000),
            max_cost_budget_units: Some(1_000),
            remaining_cost_budget_units: Some(1_000),
            exhausted: false,
        },
        vec![],
        vec![QueueHop {
            hop_index: 0,
            from_queue: "verification".into(),
            to_queue: "adjudication".into(),
            enqueued_at: "2026-03-17T00:00:06Z".into(),
            dequeued_at: Some("2026-03-17T00:00:07Z".into()),
        }],
    );
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-17T00:00:08Z",
        true,
        true,
        500_000,
        125_000,
        vec![],
    );

    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control,
        &policy,
        &calibration,
        false,
        scheduler.budget_lineage.exhausted,
        false,
    );

    assert_eq!(
        result.promotion_decision.policy_decision_id,
        policy.decision_id
    );
    assert_eq!(
        result.refutation_decision.policy_decision_id,
        policy.decision_id
    );
    assert_eq!(result.rollback_plan.policy_decision_id, policy.decision_id);
    assert_eq!(result.promotion_decision.citation, policy.citation);
    assert_eq!(result.refutation_decision.citation, policy.citation);
    assert_eq!(result.rollback_plan.citation, policy.citation);
}

#[test]
fn reduced_promotion_path_for_advisory_only_returns_obvious_disposition() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, true);
    let control = build_control_receipt(&case, &plan, &attempt);
    let policy = evaluate_policy(
        &build_policy(ApprovalRequirement::None),
        &case,
        &plan,
        &[],
        true,
        false,
    );
    let scheduler = schedule_check_plan(
        &case,
        &plan,
        BudgetLineage {
            budget_family: "verification".into(),
            retry_family: case.attempt_id.clone(),
            queue_hop_count: 1,
            max_time_budget_ms: Some(10_000),
            remaining_time_budget_ms: Some(1_000),
            max_cost_budget_units: Some(1_000),
            remaining_cost_budget_units: Some(500),
            exhausted: true,
        },
        vec![DegradationMarker {
            kind: "test".into(),
            reason: "budget degraded".into(),
            blocks_promotion: true,
        }],
        vec![],
    );
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-17T00:00:08Z",
        false,
        true,
        500_000,
        125_000,
        vec![],
    );
    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control,
        &policy,
        &calibration,
        false,
        scheduler.budget_lineage.exhausted,
        false,
    );
    assert!(matches!(
        result.disposition,
        VerificationDisposition::AdvisoryOnly
            | VerificationDisposition::DegradedNoPromotion
            | VerificationDisposition::BudgetExhausted
    ));
}
