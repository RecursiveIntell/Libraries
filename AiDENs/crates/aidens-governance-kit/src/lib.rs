//! Thin governance facade over canonical verification and policy crates.

pub mod canonical_stack {
    pub use assurance_runtime;
    pub use authority_delegation::{
        ActingOnBehalfReceiptV1, AuthorityChainV1, AuthorityLeaseV1, DelegationBundleV1,
    };
    pub use stack_ids::{AttemptId, ClaimVersionId, ScopeKey, TraceCtx};
    pub use verification_adjudication;
    pub use verification_adjudication::{
        AdjudicationResult, PromotionDecision, VerificationDisposition,
    };
    pub use verification_calibration::CalibrationSnapshot;
    pub use verification_control::{
        CaseRegion, CheckMethod, CheckPlan, ControlReceipt, PromotionClass, ReversibilityClass,
        TerminalDisposition, VerificationAttempt, VerificationAttemptState, VerificationCase,
        VerificationCaseClass,
    };
    pub use verification_policy::{
        ApprovalRecord, ApprovalRequirement, ApprovalScope, AutonomyCeiling,
        DelegationPolicyProfileV1, EffectPolicyProfileV1, ExecutionPermit, MethodPolicy,
        PolicyDecision, PolicySnapshot, ReleasePolicyProfileV1,
    };
}

pub use canonical_stack::{CheckPlan, VerificationCase};

#[derive(Debug, Clone, Copy, Default)]
pub struct CanonicalGovernanceAdapter;

impl CanonicalGovernanceAdapter {
    #[allow(clippy::too_many_arguments)]
    pub fn verification_case(
        &self,
        class: canonical_stack::VerificationCaseClass,
        namespace: impl Into<String>,
        target_key: impl Into<String>,
        trace_ctx: canonical_stack::TraceCtx,
        attempt_id: canonical_stack::AttemptId,
        opened_at: impl Into<String>,
        degraded: bool,
        advisory_only: bool,
    ) -> canonical_stack::VerificationCase {
        let namespace = namespace.into();
        canonical_stack::VerificationCase::new(
            class,
            canonical_stack::CaseRegion {
                scope_key: Some(canonical_stack::ScopeKey::namespace_only(namespace.clone())),
                namespace,
                target_key: target_key.into(),
                region_id: None,
                region_digest_id: None,
                claim_version_id: None,
                as_of_recorded_at: None,
            },
            trace_ctx,
            attempt_id,
            opened_at,
            degraded,
            advisory_only,
        )
    }

    pub fn claim_version_case(
        &self,
        namespace: impl Into<String>,
        claim_version_id: canonical_stack::ClaimVersionId,
        trace_ctx: canonical_stack::TraceCtx,
        attempt_id: canonical_stack::AttemptId,
        opened_at: impl Into<String>,
    ) -> canonical_stack::VerificationCase {
        let namespace = namespace.into();
        canonical_stack::VerificationCase::new(
            canonical_stack::VerificationCaseClass::UnverifiedClaimVersion,
            canonical_stack::CaseRegion {
                scope_key: Some(canonical_stack::ScopeKey::namespace_only(namespace.clone())),
                namespace,
                target_key: claim_version_id.as_str().to_owned(),
                region_id: None,
                region_digest_id: None,
                claim_version_id: Some(claim_version_id),
                as_of_recorded_at: None,
            },
            trace_ctx,
            attempt_id,
            opened_at,
            false,
            false,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn check_plan(
        &self,
        case: &canonical_stack::VerificationCase,
        method: canonical_stack::CheckMethod,
        check_names: Vec<String>,
        promotion_class: canonical_stack::PromotionClass,
        reversibility_class: canonical_stack::ReversibilityClass,
        promotable_if_completed: bool,
        advisory_only: bool,
        degraded: bool,
        rationale: impl Into<String>,
        input: serde_json::Value,
    ) -> canonical_stack::CheckPlan {
        canonical_stack::CheckPlan::new(
            case.case_id.clone(),
            method,
            check_names,
            promotion_class,
            reversibility_class,
            promotable_if_completed,
            advisory_only,
            degraded,
            rationale,
            input,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn completed_attempt(
        &self,
        case: &canonical_stack::VerificationCase,
        plan: &canonical_stack::CheckPlan,
        state: canonical_stack::VerificationAttemptState,
        started_at: impl Into<String>,
        finished_at: impl Into<String>,
        outcome_signature: Option<String>,
    ) -> canonical_stack::VerificationAttempt {
        canonical_stack::VerificationAttempt::completed(
            case.case_id.clone(),
            plan.plan_id.clone(),
            case.attempt_id.clone(),
            None,
            state,
            plan.advisory_only,
            plan.degraded,
            started_at,
            finished_at,
            outcome_signature,
        )
    }

    pub fn control_receipt_for_attempt(
        &self,
        case: &canonical_stack::VerificationCase,
        plan: &canonical_stack::CheckPlan,
        attempt: &canonical_stack::VerificationAttempt,
        promotable: bool,
        details: serde_json::Value,
    ) -> canonical_stack::ControlReceipt {
        canonical_stack::ControlReceipt::new_case_execution(
            case, plan, attempt, promotable, details,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn approval_record(
        &self,
        policy_version: impl Into<String>,
        case: Option<&canonical_stack::VerificationCase>,
        plan: Option<&canonical_stack::CheckPlan>,
        scope: canonical_stack::ApprovalScope,
        approver: impl Into<String>,
        approved_at: impl Into<String>,
        reviewed_artifact_refs: Vec<String>,
        rationale: impl Into<String>,
    ) -> canonical_stack::ApprovalRecord {
        canonical_stack::ApprovalRecord::new(
            policy_version,
            case.map(|case| case.case_id.clone()),
            plan.map(|plan| plan.plan_id.clone()),
            scope,
            approver,
            approved_at,
            reviewed_artifact_refs,
            rationale,
        )
    }

    pub fn evaluate_policy(
        &self,
        snapshot: &canonical_stack::PolicySnapshot,
        case: &canonical_stack::VerificationCase,
        plan: &canonical_stack::CheckPlan,
        approvals: &[canonical_stack::ApprovalRecord],
        degraded: bool,
        budget_exhausted: bool,
    ) -> canonical_stack::PolicyDecision {
        verification_policy::evaluate_policy(
            snapshot,
            case,
            plan,
            approvals,
            degraded,
            budget_exhausted,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn calibration_snapshot(
        &self,
        case: &canonical_stack::VerificationCase,
        recorded_at: impl Into<String>,
        comparability_available: bool,
        oracle_calibrated: bool,
        abstention_threshold_micros: u64,
        observed_risk_micros: u64,
        drift_markers: Vec<String>,
    ) -> canonical_stack::CalibrationSnapshot {
        canonical_stack::CalibrationSnapshot::evaluate(
            case.case_id.clone(),
            recorded_at,
            comparability_available,
            oracle_calibrated,
            abstention_threshold_micros,
            observed_risk_micros,
            drift_markers,
        )
    }

    #[allow(clippy::too_many_arguments)]
    pub fn adjudicate_case(
        &self,
        case: &canonical_stack::VerificationCase,
        plan: &canonical_stack::CheckPlan,
        attempt: &canonical_stack::VerificationAttempt,
        control_receipt: &canonical_stack::ControlReceipt,
        policy_decision: &canonical_stack::PolicyDecision,
        calibration_snapshot: &canonical_stack::CalibrationSnapshot,
        refuted: bool,
        budget_exhausted: bool,
        currently_promoted: bool,
    ) -> canonical_stack::AdjudicationResult {
        verification_adjudication::adjudicate_case(
            case,
            plan,
            attempt,
            control_receipt,
            policy_decision,
            calibration_snapshot,
            refuted,
            budget_exhausted,
            currently_promoted,
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn creates_canonical_control_receipt() {
        let adapter = CanonicalGovernanceAdapter;
        let case = adapter.claim_version_case(
            "aidens",
            canonical_stack::ClaimVersionId::new("claim-version:governance"),
            canonical_stack::TraceCtx::from_trace_id("trace:governance"),
            canonical_stack::AttemptId::new("attempt:governance"),
            "2026-04-28T00:00:00Z",
        );
        let plan = adapter.check_plan(
            &case,
            canonical_stack::CheckMethod::CausalRefuter,
            vec!["causal-refuter".into()],
            canonical_stack::PromotionClass::P1,
            canonical_stack::ReversibilityClass::RequiresSupersession,
            true,
            false,
            false,
            "canonical verification-control check plan",
            serde_json::json!({"source": "aidens-governance-kit"}),
        );
        let attempt = adapter.completed_attempt(
            &case,
            &plan,
            canonical_stack::VerificationAttemptState::Succeeded,
            "2026-04-28T00:00:00Z",
            "2026-04-28T00:00:01Z",
            Some("ok".into()),
        );
        let receipt = adapter.control_receipt_for_attempt(
            &case,
            &plan,
            &attempt,
            true,
            serde_json::json!({"promotable": true}),
        );

        assert_eq!(
            receipt.schema_version,
            verification_control::CONTROL_RECEIPT_V1_SCHEMA
        );
        assert_eq!(receipt.case_id, Some(case.case_id));
        assert_eq!(receipt.plan_id, Some(plan.plan_id));
    }
}
