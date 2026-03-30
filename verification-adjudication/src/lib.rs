use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    EffectAdjudicationReceiptId, EffectExecutionReceiptId, EffectObservationBundleId,
    IncidentCaseId, PolicyDecisionId, PromotionDecisionId, RefutationDecisionId,
    ReleaseReadinessDecisionId, ReleaseRollbackDecisionId, RollbackPlanId,
};
use verification_calibration::CalibrationSnapshot;
use verification_control::{
    interpret_promotion_eligibility, interpret_rollback_requirement, CheckPlan, ControlReceipt,
    PromotionClass, ReversibilityClass, TerminalDisposition, VerificationAttempt, VerificationCase,
};
use verification_policy::{PolicyDecision, V25CitationContext};

pub mod v14;
pub use v14::{
    RollbackDecisionV1, RolloutDecisionV1, ROLLBACK_DECISION_V1_SCHEMA, ROLLOUT_DECISION_V1_SCHEMA,
};

pub const VERIFICATION_DISPOSITION_V1_SCHEMA: &str = "verification_disposition_v1";
pub const PROMOTION_DECISION_V1_SCHEMA: &str = "promotion_decision_v1";
pub const REFUTATION_DECISION_V1_SCHEMA: &str = "refutation_decision_v1";
pub const ROLLBACK_PLAN_V1_SCHEMA: &str = "rollback_plan_v1";
pub const EFFECT_ADJUDICATION_RECEIPT_V1_SCHEMA: &str = "effect_adjudication_receipt_v1";
pub const RELEASE_ROLLBACK_DECISION_V1_SCHEMA: &str = "release_rollback_decision_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectAdjudicationReceiptV1 {
    pub schema_version: String,
    pub effect_adjudication_receipt_id: EffectAdjudicationReceiptId,
    pub effect_execution_receipt_id: EffectExecutionReceiptId,
    pub observation_bundle_id: EffectObservationBundleId,
    pub policy_decision_id: PolicyDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    pub adjudicated_state: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseRollbackDecisionV1 {
    pub schema_version: String,
    pub release_rollback_decision_id: ReleaseRollbackDecisionId,
    pub release_readiness_decision_id: ReleaseReadinessDecisionId,
    pub incident_case_id: IncidentCaseId,
    pub policy_decision_id: PolicyDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    pub rollback_required: bool,
    pub generated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VerificationDisposition {
    EligibleForPromotion,
    AdvisoryOnly,
    BlockedByPolicy,
    PendingApproval,
    Refuted,
    DegradedNoPromotion,
    BudgetExhausted,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PromotionDecision {
    pub schema_version: String,
    pub decision_id: PromotionDecisionId,
    pub case_id: stack_ids::VerificationCaseId,
    pub disposition: VerificationDisposition,
    pub promotion_class: PromotionClass,
    pub reversibility_class: ReversibilityClass,
    pub policy_decision_id: PolicyDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    pub promotable: bool,
    pub evidence_sufficiency_summary: String,
    pub approval_basis: String,
    pub rollback_scope: RollbackScopeV1,
    #[serde(default)]
    pub reasons: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub control_receipt_id: Option<stack_ids::ControlReceiptId>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RefutationDecision {
    pub schema_version: String,
    pub decision_id: RefutationDecisionId,
    pub case_id: stack_ids::VerificationCaseId,
    pub disposition: VerificationDisposition,
    pub refuted: bool,
    pub refutation_class: RefutationClassV1,
    pub policy_decision_id: PolicyDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(default)]
    pub violated_constraints: Vec<String>,
    #[serde(default)]
    pub witness_refs: Vec<String>,
    pub follow_up_required: bool,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RollbackPlan {
    pub schema_version: String,
    pub rollback_plan_id: RollbackPlanId,
    pub case_id: stack_ids::VerificationCaseId,
    pub policy_decision_id: PolicyDecisionId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    pub required: bool,
    pub rollback_scope: RollbackScopeV1,
    pub invalidation_radius: InvalidationRadiusV1,
    #[serde(default)]
    pub affected_artifacts: Vec<String>,
    #[serde(default)]
    pub stale_surfaces: Vec<String>,
    #[serde(default)]
    pub recomputation_requirements: Vec<String>,
    #[serde(default)]
    pub quarantine_targets: Vec<String>,
    #[serde(default)]
    pub operator_actions: Vec<String>,
    #[serde(default)]
    pub completion_criteria: Vec<String>,
    pub reason: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RollbackScopeV1 {
    None,
    LocalDerivedOnly,
    ProjectionScope,
    AuthorityEscalationRequired,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub enum RefutationClassV1 {
    #[serde(rename = "none")]
    None,
    #[serde(rename = "causal_refuted")]
    CausalRefuted,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum InvalidationRadiusV1 {
    None,
    Local,
    ClaimLineage,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AdjudicationResult {
    pub schema_version: String,
    pub case_id: stack_ids::VerificationCaseId,
    pub disposition: VerificationDisposition,
    pub terminal_disposition: TerminalDisposition,
    pub promotion_decision: PromotionDecision,
    pub refutation_decision: RefutationDecision,
    pub rollback_plan: RollbackPlan,
}

impl PromotionDecision {
    /// Rejects promotion decisions whose promotability and rollback vocabulary drift apart.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != PROMOTION_DECISION_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{PROMOTION_DECISION_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.promotable != (self.disposition == VerificationDisposition::EligibleForPromotion) {
            return Err("promotion decision promotable flag must match disposition".into());
        }
        Ok(())
    }
}

impl RefutationDecision {
    /// Rejects refutation decisions whose boolean and refutation vocabulary disagree.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != REFUTATION_DECISION_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{REFUTATION_DECISION_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        match (self.refuted, self.refutation_class) {
            (true, RefutationClassV1::CausalRefuted) | (false, RefutationClassV1::None) => Ok(()),
            (true, RefutationClassV1::None) => {
                Err("refuted decision cannot carry refutation_class `none`".into())
            }
            (false, RefutationClassV1::CausalRefuted) => {
                Err("non-refuted decision cannot carry causal_refuted classification".into())
            }
        }
    }
}

impl RollbackPlan {
    /// Rejects rollback plans whose required flag conflicts with their rollback geometry.
    pub fn validate(&self) -> Result<(), String> {
        if self.schema_version != ROLLBACK_PLAN_V1_SCHEMA {
            return Err(format!(
                "expected schema_version `{ROLLBACK_PLAN_V1_SCHEMA}`, found `{}`",
                self.schema_version
            ));
        }
        if self.required {
            if matches!(self.rollback_scope, RollbackScopeV1::None) {
                return Err("required rollback plan cannot use rollback_scope `none`".into());
            }
            if matches!(self.invalidation_radius, InvalidationRadiusV1::None) {
                return Err("required rollback plan cannot use invalidation_radius `none`".into());
            }
        } else if !matches!(self.rollback_scope, RollbackScopeV1::None)
            || !matches!(self.invalidation_radius, InvalidationRadiusV1::None)
        {
            return Err("non-required rollback plan must use `none` rollback geometry".into());
        }
        Ok(())
    }
}

#[allow(clippy::too_many_arguments)]
pub fn adjudicate_case(
    case: &VerificationCase,
    plan: &CheckPlan,
    attempt: &VerificationAttempt,
    control_receipt: &ControlReceipt,
    policy_decision: &PolicyDecision,
    calibration_snapshot: &CalibrationSnapshot,
    refuted: bool,
    budget_exhausted: bool,
    currently_promoted: bool,
) -> AdjudicationResult {
    let promotion_eligible = interpret_promotion_eligibility(
        policy_decision.promotion_allowed,
        calibration_snapshot.forces_advisory_only,
        attempt.degraded,
        budget_exhausted,
        matches!(
            attempt.state,
            verification_control::VerificationAttemptState::Succeeded
        ),
    );

    let (disposition, terminal_disposition, reason) =
        if !policy_decision.method_allowed || !policy_decision.autonomy_allowed {
            (
                VerificationDisposition::BlockedByPolicy,
                TerminalDisposition::Blocked,
                "blocked by policy".to_string(),
            )
        } else if policy_decision.approval_required && !policy_decision.approval_satisfied {
            (
                VerificationDisposition::PendingApproval,
                TerminalDisposition::Blocked,
                "approval required".to_string(),
            )
        } else if refuted {
            (
                VerificationDisposition::Refuted,
                TerminalDisposition::Refuted,
                "refutation evidence prevents promotion".to_string(),
            )
        } else if budget_exhausted {
            (
                VerificationDisposition::BudgetExhausted,
                TerminalDisposition::DegradedNoPromotion,
                "budget exhausted".to_string(),
            )
        } else if attempt.degraded
            || calibration_snapshot.forces_advisory_only
            || !policy_decision.promotion_allowed
        {
            (
                VerificationDisposition::AdvisoryOnly,
                TerminalDisposition::DegradedNoPromotion,
                "degraded or poorly calibrated path is advisory only".to_string(),
            )
        } else if promotion_eligible {
            (
                VerificationDisposition::EligibleForPromotion,
                TerminalDisposition::EligibleForPromotion,
                "all required gates satisfied".to_string(),
            )
        } else {
            (
                VerificationDisposition::Failed,
                TerminalDisposition::Exhausted,
                "attempt did not succeed".to_string(),
            )
        };

    let promotion_decision = PromotionDecision {
        schema_version: PROMOTION_DECISION_V1_SCHEMA.into(),
        decision_id: PromotionDecisionId::generate(),
        case_id: case.case_id.clone(),
        policy_decision_id: policy_decision.decision_id.clone(),
        citation: policy_decision.citation.clone(),
        disposition,
        promotion_class: plan.promotion_class,
        reversibility_class: plan.reversibility_class,
        promotable: disposition == VerificationDisposition::EligibleForPromotion,
        evidence_sufficiency_summary: if disposition
            == VerificationDisposition::EligibleForPromotion
        {
            format!(
                "method {:?} succeeded with replayable receipt lineage",
                plan.method
            )
        } else {
            format!(
                "promotion class {:?} was not satisfied",
                plan.promotion_class
            )
        },
        approval_basis: if policy_decision.approval_required {
            if policy_decision.approval_satisfied {
                "typed approval record present".into()
            } else {
                "required approval record absent".into()
            }
        } else {
            "approval not required".into()
        },
        rollback_scope: match plan.promotion_class {
            PromotionClass::P0 => RollbackScopeV1::None,
            PromotionClass::P1 => RollbackScopeV1::LocalDerivedOnly,
            PromotionClass::P2 => RollbackScopeV1::ProjectionScope,
            PromotionClass::P3 => RollbackScopeV1::AuthorityEscalationRequired,
        },
        reasons: policy_decision.reasons.clone(),
        control_receipt_id: Some(control_receipt.receipt_id.clone()),
    };
    debug_assert!(promotion_decision.validate().is_ok());
    let refutation_decision = RefutationDecision {
        schema_version: REFUTATION_DECISION_V1_SCHEMA.into(),
        decision_id: RefutationDecisionId::generate(),
        case_id: case.case_id.clone(),
        policy_decision_id: policy_decision.decision_id.clone(),
        citation: policy_decision.citation.clone(),
        disposition,
        refuted,
        refutation_class: if refuted {
            RefutationClassV1::CausalRefuted
        } else {
            RefutationClassV1::None
        },
        violated_constraints: if refuted {
            vec!["promotion eligibility".into()]
        } else {
            Vec::new()
        },
        witness_refs: control_receipt
            .details
            .get("target_key")
            .and_then(|value| value.as_str())
            .map(|target| vec![format!("target:{target}")])
            .unwrap_or_default(),
        follow_up_required: refuted && !currently_promoted,
        reason: if refuted {
            "refutation witness present".into()
        } else {
            "no refutation witness triggered".into()
        },
    };
    debug_assert!(refutation_decision.validate().is_ok());
    let rollback_required = interpret_rollback_requirement(refuted, currently_promoted);
    let rollback_plan = RollbackPlan {
        schema_version: ROLLBACK_PLAN_V1_SCHEMA.into(),
        rollback_plan_id: RollbackPlanId::generate(),
        case_id: case.case_id.clone(),
        policy_decision_id: policy_decision.decision_id.clone(),
        citation: policy_decision.citation.clone(),
        required: rollback_required,
        rollback_scope: if rollback_required {
            match plan.promotion_class {
                PromotionClass::P0 | PromotionClass::P1 => RollbackScopeV1::LocalDerivedOnly,
                PromotionClass::P2 => RollbackScopeV1::ProjectionScope,
                PromotionClass::P3 => RollbackScopeV1::AuthorityEscalationRequired,
            }
        } else {
            RollbackScopeV1::None
        },
        invalidation_radius: if rollback_required {
            InvalidationRadiusV1::ClaimLineage
        } else {
            InvalidationRadiusV1::None
        },
        affected_artifacts: if rollback_required {
            vec![case.region.target_key.clone()]
        } else {
            Vec::new()
        },
        stale_surfaces: if rollback_required {
            vec!["semantic_memory_projection".into()]
        } else {
            Vec::new()
        },
        recomputation_requirements: if rollback_required {
            vec!["invalidate bounded derivations".into()]
        } else {
            Vec::new()
        },
        quarantine_targets: if refuted {
            vec![case.region.target_key.clone()]
        } else {
            Vec::new()
        },
        operator_actions: if rollback_required {
            vec!["block automatic promotion until recomputation completes".into()]
        } else {
            Vec::new()
        },
        completion_criteria: if rollback_required {
            vec![
                "derived state invalidated".into(),
                "stale surfaces marked".into(),
            ]
        } else {
            Vec::new()
        },
        reason,
    };
    debug_assert!(rollback_plan.validate().is_ok());

    AdjudicationResult {
        schema_version: VERIFICATION_DISPOSITION_V1_SCHEMA.into(),
        case_id: case.case_id.clone(),
        disposition,
        terminal_disposition,
        promotion_decision,
        refutation_decision,
        rollback_plan,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_ids::{AttemptId, ScopeKey, TraceCtx, TrialId};
    use verification_calibration::CalibrationSnapshot;
    use verification_control::{
        CaseRegion, CheckMethod, CheckPlan, ControlReceipt, PromotionClass, ReversibilityClass,
        VerificationAttempt, VerificationAttemptState, VerificationCase, VerificationCaseClass,
    };
    use verification_policy::{
        evaluate_policy, ApprovalRequirement, AutonomyCeiling, MethodPolicy, PolicySnapshot,
    };

    #[test]
    fn calibration_can_force_advisory_only_disposition() {
        let case = VerificationCase::new(
            VerificationCaseClass::UnverifiedClaimVersion,
            CaseRegion {
                namespace: "demo".into(),
                scope_key: Some(ScopeKey::namespace_only("demo")),
                target_key: "unverified:claim-v1".into(),
                region_id: None,
                region_digest_id: None,
                claim_version_id: Some(stack_ids::ClaimVersionId::new("claim-v1")),
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
            serde_json::json!({}),
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
        let control_receipt =
            ControlReceipt::new_case_execution(&case, &plan, &attempt, true, serde_json::json!({}));
        let policy = PolicySnapshot {
            schema_version: verification_policy::POLICY_SNAPSHOT_V1_SCHEMA.into(),
            policy_version: "policy-1".into(),
            effective_from: "2026-03-01T00:00:00Z".into(),
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
        assert_eq!(result.disposition, VerificationDisposition::AdvisoryOnly);
    }

    #[test]
    fn v21_v24_adjudication_receipts_roundtrip() {
        let effect = EffectAdjudicationReceiptV1 {
            schema_version: EFFECT_ADJUDICATION_RECEIPT_V1_SCHEMA.into(),
            effect_adjudication_receipt_id: stack_ids::EffectAdjudicationReceiptId::new(
                "effect-adjudication-receipt-1",
            ),
            effect_execution_receipt_id: stack_ids::EffectExecutionReceiptId::new(
                "effect-execution-receipt-1",
            ),
            observation_bundle_id: stack_ids::EffectObservationBundleId::new(
                "effect-observation-bundle-1",
            ),
            policy_decision_id: stack_ids::PolicyDecisionId::new("policy-decision-1"),
            citation: V25CitationContext {
                applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                    "applicability-context-1",
                )),
                profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-1")),
                composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                    "composition-receipt-1",
                )),
                effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                    "effective-constitution-1",
                )),
                compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                    "compiled-obligation-set-1",
                )),
                composition_conflict_set_id: None,
                profile_exception_bundle_ids: vec![stack_ids::ProfileExceptionBundleId::new(
                    "profile-exception-bundle-1",
                )],
            },
            adjudicated_state: "closed_successfully".into(),
            generated_at: "2026-03-15T13:00:00Z".into(),
        };
        let release = ReleaseRollbackDecisionV1 {
            schema_version: RELEASE_ROLLBACK_DECISION_V1_SCHEMA.into(),
            release_rollback_decision_id: stack_ids::ReleaseRollbackDecisionId::new(
                "release-rollback-decision-1",
            ),
            release_readiness_decision_id: stack_ids::ReleaseReadinessDecisionId::new(
                "release-readiness-decision-1",
            ),
            incident_case_id: stack_ids::IncidentCaseId::new("incident-case-1"),
            policy_decision_id: stack_ids::PolicyDecisionId::new("policy-decision-1"),
            citation: V25CitationContext {
                applicability_context_id: Some(stack_ids::ApplicabilityContextId::new(
                    "applicability-context-1",
                )),
                profile_set_id: Some(stack_ids::ProfileSetId::new("profile-set-1")),
                composition_receipt_id: Some(stack_ids::CompositionReceiptId::new(
                    "composition-receipt-1",
                )),
                effective_constitution_id: Some(stack_ids::EffectiveConstitutionId::new(
                    "effective-constitution-1",
                )),
                compiled_obligation_set_id: Some(stack_ids::CompiledObligationSetId::new(
                    "compiled-obligation-set-1",
                )),
                composition_conflict_set_id: None,
                profile_exception_bundle_ids: vec![stack_ids::ProfileExceptionBundleId::new(
                    "profile-exception-bundle-1",
                )],
            },
            rollback_required: true,
            generated_at: "2026-03-15T13:01:00Z".into(),
        };

        let json = serde_json::to_string(&(effect, release)).expect("serialize receipts");
        let _: (EffectAdjudicationReceiptV1, ReleaseRollbackDecisionV1) =
            serde_json::from_str(&json).expect("deserialize receipts");
    }
}
