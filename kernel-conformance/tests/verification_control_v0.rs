use forge_pilot::TargetKind;
use serde_json::json;
use stack_ids::{AttemptId, ClaimVersionId, ScopeKey, TraceCtx, TrialId};
use verification_adjudication::{adjudicate_case, VerificationDisposition};
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    replay_case, schedule_check_plan, BudgetLineage, CaseRegion, CheckMethod, CheckPlan,
    ControlReceipt, DegradationMarker, LedgerEntry, LedgerEvent, PromotionClass, QueueHop,
    ReversibilityClass, TerminalDisposition, VerificationAttempt, VerificationAttemptState,
    VerificationCase, VerificationCaseClass, VerificationWorkloadClass,
};
use verification_policy::{
    evaluate_policy, ApprovalRequirement, AutonomyCeiling, MethodPolicy, PolicySnapshot,
    POLICY_SNAPSHOT_V1_SCHEMA,
};

#[test]
fn forge_pilot_targets_map_to_canonical_case_classes() {
    let target = TargetKind::UnverifiedClaimVersion {
        claim_version_id: ClaimVersionId::new("claim-v1"),
    };

    assert_eq!(
        target.case_class(),
        VerificationCaseClass::UnverifiedClaimVersion
    );
    assert_eq!(
        target.primary_claim_version_id(),
        Some(ClaimVersionId::new("claim-v1"))
    );

    let refutation_target = TargetKind::RefutationGap {
        target_node_id: "node-1".into(),
        claim_version_id: Some(ClaimVersionId::new("claim-v2")),
    };
    assert_eq!(
        refutation_target.primary_claim_version_id(),
        Some(ClaimVersionId::new("claim-v2"))
    );
}

#[test]
fn control_ledger_replay_rebuilds_closed_case() {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "unverified:claim-v1".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v1")),
            as_of_recorded_at: Some("2026-03-12T00:00:00Z".into()),
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["kernel_oracle".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "exact bounded oracle",
        json!({"oracle_slice_id": "slice-1"}),
    );
    let attempt = VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        case.attempt_id.clone(),
        Some(TrialId::new("trial-1")),
        VerificationAttemptState::Succeeded,
        false,
        false,
        "2026-03-12T00:00:00Z",
        "2026-03-12T00:00:01Z",
        Some("supported".into()),
    );
    let receipt = ControlReceipt::new_case_execution(
        &case,
        &plan,
        &attempt,
        true,
        json!({"action_family": "oracle"}),
    );

    let replayed = replay_case(&[
        LedgerEntry::new(
            case.case_id.clone(),
            1,
            LedgerEvent::CaseOpened { case: case.clone() },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            2,
            LedgerEvent::PlanAdopted { plan: plan.clone() },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            3,
            LedgerEvent::AttemptRecorded {
                attempt: attempt.clone(),
            },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            4,
            LedgerEvent::ReceiptAppended {
                receipt: Box::new(receipt.clone()),
            },
        ),
        LedgerEntry::new(
            case.case_id.clone(),
            5,
            LedgerEvent::CaseClosed {
                case_id: case.case_id.clone(),
                disposition: TerminalDisposition::EligibleForPromotion,
            },
        ),
    ])
    .unwrap();

    assert_eq!(replayed.plan_count, 1);
    assert_eq!(replayed.attempt_count, 1);
    assert_eq!(replayed.receipt_count, 1);
    assert_eq!(
        replayed.terminal_disposition,
        Some(TerminalDisposition::EligibleForPromotion)
    );
}

#[test]
fn denied_methods_are_blocked_before_execution() {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "thin_export".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        true,
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
        "paired patch",
        json!({}),
    );
    let policy = PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-1".into(),
        effective_from: "2026-03-01T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method: CheckMethod::PairedPatch,
            allowed: false,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::None,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: verification_policy::V25CitationContext::missing(),
        obligation_refs: verification_policy::V25ControlObligationRefs::missing(),
    };

    let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    assert!(decision.issue_execution_permit(&case, &plan, &[]).is_err());
}

#[test]
fn calibration_downgrades_promotion_to_advisory_only() {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "unverified:claim-v1".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v1")),
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        false,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["kernel_oracle".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "oracle",
        json!({}),
    );
    let attempt = VerificationAttempt::completed(
        case.case_id.clone(),
        plan.plan_id.clone(),
        case.attempt_id.clone(),
        Some(TrialId::new("trial-1")),
        VerificationAttemptState::Succeeded,
        false,
        false,
        "2026-03-12T00:00:00Z",
        "2026-03-12T00:00:01Z",
        Some("supported".into()),
    );
    let receipt = ControlReceipt::new_case_execution(&case, &plan, &attempt, false, json!({}));
    let policy = PolicySnapshot::permissive("policy-1", "2026-03-01T00:00:00Z");
    let policy_decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
    let calibration = CalibrationSnapshot::evaluate(
        case.case_id.clone(),
        "2026-03-12T00:00:00Z",
        false,
        true,
        500_000,
        100_000,
        vec![],
    );

    let adjudication = adjudicate_case(
        &case,
        &plan,
        &attempt,
        &receipt,
        &policy_decision,
        &calibration,
        false,
        false,
        false,
    );
    assert_eq!(
        adjudication.disposition,
        VerificationDisposition::AdvisoryOnly
    );
}

#[test]
fn degraded_scheduler_paths_cannot_silently_promote() {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "thin_export".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-1"),
        "2026-03-12T00:00:00Z",
        true,
        false,
    );
    let plan = CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ConservativeOracle,
        vec!["kernel_oracle".into()],
        PromotionClass::P0,
        ReversibilityClass::ReversibleLocal,
        true,
        false,
        true,
        "degraded oracle",
        json!({}),
    );

    let decision = schedule_check_plan(
        &case,
        &plan,
        BudgetLineage {
            budget_family: "pilot".into(),
            retry_family: case.attempt_id.clone(),
            queue_hop_count: 1,
            max_time_budget_ms: Some(1000),
            remaining_time_budget_ms: Some(0),
            max_cost_budget_units: None,
            remaining_cost_budget_units: None,
            exhausted: true,
        },
        vec![DegradationMarker {
            kind: "thin_export".into(),
            reason: "missing semantics".into(),
            blocks_promotion: true,
        }],
        vec![QueueHop {
            hop_index: 0,
            from_queue: "observe".into(),
            to_queue: "verify".into(),
            enqueued_at: "2026-03-12T00:00:00Z".into(),
            dequeued_at: Some("2026-03-12T00:00:01Z".into()),
        }],
    );

    assert_eq!(decision.workload_class, VerificationWorkloadClass::Oracle);
    assert!(decision.promotion_blocked);
}
