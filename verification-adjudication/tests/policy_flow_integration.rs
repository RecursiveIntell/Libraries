use serde_json::json;
use stack_ids::{AttemptId, ClaimVersionId, ScopeKey, TraceCtx, TrialId};
use verification_adjudication::{adjudicate_case, RollbackScopeV1, VerificationDisposition};
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    schedule_check_plan, BudgetLineage, CaseRegion, CheckMethod, CheckPlan, ControlReceipt,
    DegradationMarker, PromotionClass, QueueHop, ReversibilityClass, VerificationAttempt,
    VerificationAttemptState, VerificationCase, VerificationCaseClass,
};
use verification_policy::{
    evaluate_policy, ApprovalRecord, ApprovalRequirement, ApprovalScope, AutonomyCeiling,
    MethodPolicy, PolicySnapshot,
};

fn complete_citation() -> verification_policy::V25CitationContext {
    verification_policy::V25CitationContext {
        applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
            "applicability-context-alpha",
        )),
        profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-alpha")),
        composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
            "composition-receipt-alpha",
        )),
        effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
            "effective-constitution-alpha",
        )),
        compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
            "compiled-obligation-set-alpha",
        )),
        composition_conflict_set_id: None,
        profile_exception_bundle_ids: vec![stack_ids::ProfileExceptionBundleId::new(
            "profile-exception-bundle-alpha",
        )],
    }
}

fn complete_obligation_refs() -> verification_policy::V25ControlObligationRefs {
    verification_policy::V25ControlObligationRefs {
        required_obligation_refs: vec!["obligation:required:alpha".into()],
        blocking_obligation_refs: Vec::new(),
        monitoring_obligation_refs: vec!["obligation:monitoring:alpha".into()],
    }
}

fn build_case() -> VerificationCase {
    VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "claim:alpha".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-alpha-v1")),
            as_of_recorded_at: Some("2026-03-12T00:00:00Z".into()),
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-alpha"),
        "2026-03-12T00:00:00Z",
        false,
        false,
    )
}

fn build_plan(case: &VerificationCase) -> CheckPlan {
    CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec![
            "temporal_replay".into(),
            "bounded_oracle_parity".into(),
            "minimal_falsifier".into(),
        ],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "exact bounded oracle is the cheapest adequate verification path",
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
        Some(TrialId::new("trial-alpha")),
        VerificationAttemptState::Succeeded,
        false,
        degraded,
        "2026-03-12T00:00:02Z",
        "2026-03-12T00:00:05Z",
        Some("oracle:match".into()),
    )
}

fn build_policy(approval_requirement: ApprovalRequirement) -> PolicySnapshot {
    PolicySnapshot {
        schema_version: verification_policy::POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-1".into(),
        effective_from: "2026-03-01T00:00:00Z".into(),
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
        citation: complete_citation(),
        obligation_refs: complete_obligation_refs(),
    }
}

fn build_approval(case: &VerificationCase, plan: &CheckPlan) -> ApprovalRecord {
    ApprovalRecord::new(
        "policy-1",
        None,
        None,
        ApprovalScope {
            namespace: case.region.namespace.clone(),
            target_key: Some(case.region.target_key.clone()),
            method: Some(plan.method),
            promotion_class: Some(plan.promotion_class),
            expires_at: None,
            reusable: true,
        },
        "operator",
        "2026-03-12T00:00:01Z",
        vec![
            "artifact:claim-alpha".into(),
            "artifact:oracle-snapshot".into(),
        ],
        "bounded oracle reviewed for promotion eligibility",
    )
}

fn budget_lineage(exhausted: bool) -> BudgetLineage {
    BudgetLineage {
        budget_family: "verification".into(),
        retry_family: AttemptId::new("attempt-alpha"),
        queue_hop_count: 1,
        max_time_budget_ms: Some(30_000),
        remaining_time_budget_ms: Some(if exhausted { 0 } else { 24_000 }),
        max_cost_budget_units: Some(100),
        remaining_cost_budget_units: Some(if exhausted { 0 } else { 75 }),
        exhausted,
    }
}

fn queue_hops() -> Vec<QueueHop> {
    vec![QueueHop {
        hop_index: 0,
        from_queue: "pilot".into(),
        to_queue: "verification".into(),
        enqueued_at: "2026-03-12T00:00:01Z".into(),
        dequeued_at: Some("2026-03-12T00:00:02Z".into()),
    }]
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
            "replay_recipe": ["replay:temporal", "oracle:exact_slice"],
        }),
    )
}

#[test]
fn happy_path_is_promotion_eligible() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, false);
    let policy = build_policy(ApprovalRequirement::None);
    let policy_decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    let scheduler = schedule_check_plan(&case, &plan, budget_lineage(false), vec![], queue_hops());
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-12T00:00:05Z",
        true,
        true,
        500_000,
        125_000,
        vec![],
    );
    let control_receipt = build_control_receipt(&case, &plan, &attempt);

    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control_receipt,
        &policy_decision,
        &calibration,
        false,
        scheduler.budget_lineage.exhausted,
        false,
    );

    assert!(policy_decision
        .issue_execution_permit(&case, &plan, &[])
        .is_ok());
    assert!(!scheduler.promotion_blocked);
    assert_eq!(
        result.disposition,
        VerificationDisposition::EligibleForPromotion
    );
    assert!(result.promotion_decision.promotable);
    assert!(!result.rollback_plan.required);
}

#[test]
fn missing_required_approval_blocks_execution_and_promotion() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, false);
    let policy = build_policy(ApprovalRequirement::HumanReview);
    let policy_decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-12T00:00:05Z",
        true,
        true,
        500_000,
        125_000,
        vec![],
    );
    let control_receipt = build_control_receipt(&case, &plan, &attempt);

    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control_receipt,
        &policy_decision,
        &calibration,
        false,
        false,
        false,
    );

    assert!(policy_decision.approval_required);
    assert!(!policy_decision.approval_satisfied);
    assert!(policy_decision
        .issue_execution_permit(&case, &plan, &[])
        .is_err());
    assert_eq!(result.disposition, VerificationDisposition::PendingApproval);
    assert!(!result.promotion_decision.promotable);
}

#[test]
fn calibration_drift_forces_advisory_only_even_with_approval() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, false);
    let policy = build_policy(ApprovalRequirement::HumanReview);
    let approval = build_approval(&case, &plan);
    let policy_decision = evaluate_policy(
        &policy,
        &case,
        &plan,
        std::slice::from_ref(&approval),
        false,
        false,
    );
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-12T00:00:05Z",
        true,
        true,
        500_000,
        125_000,
        vec!["comparability_drift".into()],
    );
    let control_receipt = build_control_receipt(&case, &plan, &attempt);

    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control_receipt,
        &policy_decision,
        &calibration,
        false,
        false,
        false,
    );

    assert!(policy_decision
        .issue_execution_permit(&case, &plan, &[approval])
        .is_ok());
    assert!(calibration.forces_advisory_only);
    assert_eq!(result.disposition, VerificationDisposition::AdvisoryOnly);
    assert!(!result.promotion_decision.promotable);
    assert_eq!(
        result.terminal_disposition,
        verification_control::TerminalDisposition::DegradedNoPromotion
    );
}

#[test]
fn refuted_currently_promoted_case_requires_rollback() {
    let case = build_case();
    let plan = build_plan(&case);
    let attempt = build_attempt(&case, &plan, false);
    let policy = build_policy(ApprovalRequirement::HumanReview);
    let approval = build_approval(&case, &plan);
    let policy_decision = evaluate_policy(
        &policy,
        &case,
        &plan,
        std::slice::from_ref(&approval),
        false,
        false,
    );
    let scheduler = schedule_check_plan(
        &case,
        &plan,
        budget_lineage(false),
        vec![DegradationMarker {
            kind: "refutation_gap".into(),
            reason: "prior promotion lacked a completed falsifier".into(),
            blocks_promotion: true,
        }],
        queue_hops(),
    );
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-12T00:00:05Z",
        true,
        true,
        500_000,
        125_000,
        vec![],
    );
    let control_receipt = build_control_receipt(&case, &plan, &attempt);

    let result = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &control_receipt,
        &policy_decision,
        &calibration,
        true,
        scheduler.budget_lineage.exhausted,
        true,
    );

    assert!(policy_decision
        .issue_execution_permit(&case, &plan, &[approval])
        .is_ok());
    assert!(scheduler.promotion_blocked);
    assert_eq!(result.disposition, VerificationDisposition::Refuted);
    assert!(result.refutation_decision.refuted);
    assert!(result.rollback_plan.required);
    assert_eq!(
        result.rollback_plan.rollback_scope,
        RollbackScopeV1::ProjectionScope
    );
}
