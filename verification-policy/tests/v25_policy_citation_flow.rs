use serde_json::json;
use stack_ids::{AttemptId, ClaimVersionId, ScopeKey, TraceCtx};
use verification_control::ConstitutionalContextStatus;
use verification_control::{
    CheckMethod, PromotionClass, ReversibilityClass, VerificationCase, VerificationCaseClass,
};
use verification_policy::{
    evaluate_policy, ApprovalRecord, ApprovalRequirement, ApprovalScope, AutonomyCeiling,
    MethodPolicy, PermitIssuanceError, PolicyDecision, PolicySnapshot, POLICY_DECISION_V1_SCHEMA,
    POLICY_SNAPSHOT_V1_SCHEMA,
};

fn complete_citation() -> verification_policy::V25CitationContext {
    verification_policy::V25CitationContext {
        applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
            "applicability-context-v25",
        )),
        profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-v25")),
        composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
            "composition-receipt-v25",
        )),
        effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
            "effective-constitution-v25",
        )),
        compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
            "compiled-obligation-set-v25",
        )),
        composition_conflict_set_id: Some(stack_ids::CompositionConflictSetId::new(
            "composition-conflict-set-v25",
        )),
        profile_exception_bundle_ids: vec![stack_ids::ProfileExceptionBundleId::new(
            "profile-exception-bundle-v25",
        )],
    }
}

fn complete_obligation_refs() -> verification_policy::V25ControlObligationRefs {
    verification_policy::V25ControlObligationRefs {
        required_obligation_refs: vec!["obligation:required:v25".into()],
        blocking_obligation_refs: vec!["obligation:blocking:v25".into()],
        monitoring_obligation_refs: vec!["obligation:monitoring:v25".into()],
    }
}

#[test]
fn policy_decision_carries_v25_citation_shape() {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        verification_control::CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "policy-case-v25".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v25-1")),
            as_of_recorded_at: Some("2026-03-16T00:00:00Z".into()),
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-v25"),
        "2026-03-16T00:00:00Z",
        false,
        false,
    );

    let plan = verification_control::CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["policy-check".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "policy lane check",
        json!({"target_key": case.region.target_key}),
    );

    let snapshot = PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-25".into(),
        effective_from: "2026-03-16T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method: CheckMethod::ExactBoundedOracle,
            allowed: true,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::None,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: verification_policy::V25CitationContext::missing(),
        obligation_refs: verification_policy::V25ControlObligationRefs::missing(),
    };

    let decision = evaluate_policy(&snapshot, &case, &plan, &[], false, false);
    assert_eq!(
        decision.citation_status,
        ConstitutionalContextStatus::Missing
    );
    assert_eq!(
        decision.obligation_refs_status,
        ConstitutionalContextStatus::Missing
    );
    assert_eq!(decision.schema_version, POLICY_DECISION_V1_SCHEMA);
    assert!(!decision.promotion_allowed);
    assert!(decision
        .reasons
        .iter()
        .any(|reason| reason.contains("constitutional citation context")));
    assert!(decision
        .reasons
        .iter()
        .any(|reason| reason.contains("constitutional obligation refs")));

    let decision_from_example: PolicyDecision = serde_json::from_str(
        &std::fs::read_to_string(format!(
            "{}/../examples/policy-decision-v1.example.json",
            env!("CARGO_MANIFEST_DIR")
        ))
        .unwrap(),
    )
    .unwrap();

    assert_eq!(
        decision_from_example
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        decision_from_example.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        decision_from_example.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );
}

#[test]
fn approval_records_influence_admission_requirement() {
    let case = VerificationCase::new(
        VerificationCaseClass::UnverifiedClaimVersion,
        verification_control::CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "policy-case-v25-approve".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: Some(ClaimVersionId::new("claim-v25-2")),
            as_of_recorded_at: Some("2026-03-16T00:00:00Z".into()),
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-v25-2"),
        "2026-03-16T00:00:00Z",
        false,
        false,
    );
    let plan = verification_control::CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::ExactBoundedOracle,
        vec!["policy-check".into()],
        PromotionClass::P2,
        ReversibilityClass::ReversibleScoped,
        true,
        false,
        false,
        "policy lane check",
        json!({"target_key": case.region.target_key}),
    );
    let snapshot = PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-25".into(),
        effective_from: "2026-03-16T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method: CheckMethod::ExactBoundedOracle,
            allowed: true,
            max_autonomy: AutonomyCeiling::PromotionEligible,
            approval_requirement: ApprovalRequirement::HumanReview,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: complete_citation(),
        obligation_refs: complete_obligation_refs(),
    };
    let approval = ApprovalRecord::new(
        "policy-25",
        None,
        None,
        ApprovalScope {
            namespace: "demo".into(),
            target_key: Some(case.region.target_key.clone()),
            method: Some(CheckMethod::ExactBoundedOracle),
            promotion_class: Some(PromotionClass::P2),
            expires_at: None,
            reusable: true,
        },
        "operator",
        "2026-03-16T00:00:00Z",
        vec!["artifact:policy-check".into()],
        "approved for promotion",
    );

    let decision = evaluate_policy(
        &snapshot,
        &case,
        &plan,
        std::slice::from_ref(&approval),
        false,
        false,
    );
    assert!(decision.approval_required);
    assert!(decision.approval_satisfied);
    assert!(decision.promotion_allowed);
    assert!(decision
        .issue_execution_permit(&case, &plan, &[approval])
        .is_ok());
    assert_eq!(
        decision.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        decision.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );
}

#[test]
fn advisory_only_plan_cannot_mint_execution_permit() {
    let case = VerificationCase::new(
        VerificationCaseClass::ThinExport,
        verification_control::CaseRegion {
            namespace: "demo".into(),
            scope_key: Some(ScopeKey::namespace_only("demo")),
            target_key: "policy-case-v25-advisory".into(),
            region_id: None,
            region_digest_id: None,
            claim_version_id: None,
            as_of_recorded_at: None,
        },
        TraceCtx::generate(),
        AttemptId::new("attempt-v25-advisory"),
        "2026-03-16T00:00:00Z",
        false,
        true,
    );
    let plan = verification_control::CheckPlan::new(
        case.case_id.clone(),
        CheckMethod::AdvisoryOnly,
        vec!["policy-check".into()],
        PromotionClass::P1,
        ReversibilityClass::ReversibleScoped,
        false,
        true,
        false,
        "advisory lane check",
        json!({"target_key": case.region.target_key}),
    );
    let snapshot = PolicySnapshot {
        schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
        policy_version: "policy-25".into(),
        effective_from: "2026-03-16T00:00:00Z".into(),
        effective_to: None,
        autonomy_ceiling: AutonomyCeiling::PromotionEligible,
        method_rules: vec![MethodPolicy {
            method: CheckMethod::AdvisoryOnly,
            allowed: true,
            max_autonomy: AutonomyCeiling::AdvisoryOnly,
            approval_requirement: ApprovalRequirement::None,
        }],
        blocked_promotions_when_degraded: true,
        blocked_promotions_when_budget_exhausted: true,
        citation: complete_citation(),
        obligation_refs: complete_obligation_refs(),
    };

    let decision = evaluate_policy(&snapshot, &case, &plan, &[], false, false);
    assert_eq!(
        decision.issue_execution_permit(&case, &plan, &[]),
        Err(PermitIssuanceError::PlanNotPromotionCapable)
    );
}
