use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApprovalRecordId, ContinuityPolicyProfileId, DelegationPolicyProfileId,
    EffectPolicyProfileId, PolicyDecisionId, ReleasePolicyProfileId,
};
use verification_control::{CheckMethod, CheckPlan, PromotionClass, VerificationCase};

pub mod v14;
pub use v14::{
    ArtifactAdmissionPolicyV1, DisclosureBudgetV1, DisclosurePolicyV1, ExperimentBudgetV1,
    RefuterSuiteV1, ARTIFACT_ADMISSION_POLICY_V1_SCHEMA, DISCLOSURE_BUDGET_V1_SCHEMA,
    DISCLOSURE_POLICY_V1_SCHEMA, EXPERIMENT_BUDGET_V1_SCHEMA, REFUTER_SUITE_V1_SCHEMA,
};

pub mod profile_p1_privacy;
pub mod profile_p2_locality;
pub use profile_p1_privacy::*;
pub use profile_p2_locality::*;

pub const POLICY_SNAPSHOT_V1_SCHEMA: &str = "policy_snapshot_v1";
pub const APPROVAL_RECORD_V1_SCHEMA: &str = "approval_record_v1";
pub const POLICY_DECISION_V1_SCHEMA: &str = "policy_decision_v1";
pub const EFFECT_POLICY_PROFILE_V1_SCHEMA: &str = "effect_policy_profile_v1";
pub const DELEGATION_POLICY_PROFILE_V1_SCHEMA: &str = "delegation_policy_profile_v1";
pub const RELEASE_POLICY_PROFILE_V1_SCHEMA: &str = "release_policy_profile_v1";
pub const CONTINUITY_POLICY_PROFILE_V1_SCHEMA: &str = "continuity_policy_profile_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectPolicyProfileV1 {
    pub schema_version: String,
    pub effect_policy_profile_id: EffectPolicyProfileId,
    #[serde(default)]
    pub allowed_run_modes: Vec<String>,
    #[serde(default)]
    pub required_preflight_checks: Vec<String>,
    #[serde(default)]
    pub required_observation_classes: Vec<String>,
    #[serde(default)]
    pub requires_compensation_plan_for: Vec<String>,
    pub block_live_without_commit: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationPolicyProfileV1 {
    pub schema_version: String,
    pub delegation_policy_profile_id: DelegationPolicyProfileId,
    pub max_delegation_depth: i64,
    pub break_glass_requires_post_hoc_review: bool,
    #[serde(default)]
    pub forbidden_role_combinations: Vec<String>,
    pub require_typed_authority_chain: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleasePolicyProfileV1 {
    pub schema_version: String,
    pub release_policy_profile_id: ReleasePolicyProfileId,
    #[serde(default)]
    pub required_assurance_sections: Vec<String>,
    #[serde(default)]
    pub required_monitor_classes: Vec<String>,
    pub block_on_open_obligations: bool,
    pub forbid_score_only_gate: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityPolicyProfileV1 {
    pub schema_version: String,
    pub continuity_policy_profile_id: ContinuityPolicyProfileId,
    #[serde(default)]
    pub required_forensic_freeze_surfaces: Vec<String>,
    pub continuity_exception_ttl_minutes: i64,
    #[serde(default)]
    pub requires_postmortem_for_severity: Vec<String>,
    pub require_error_budget_linkage: bool,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum AutonomyCeiling {
    AdvisoryOnly,
    VerificationOnly,
    PromotionEligible,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ApprovalRequirement {
    None,
    HumanReview,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalScope {
    pub namespace: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub target_key: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<CheckMethod>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub promotion_class: Option<PromotionClass>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expires_at: Option<String>,
    pub reusable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MethodPolicy {
    pub method: CheckMethod,
    pub allowed: bool,
    pub max_autonomy: AutonomyCeiling,
    pub approval_requirement: ApprovalRequirement,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicySnapshot {
    pub schema_version: String,
    pub policy_version: String,
    pub effective_from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_to: Option<String>,
    pub autonomy_ceiling: AutonomyCeiling,
    #[serde(default)]
    pub method_rules: Vec<MethodPolicy>,
    #[serde(default)]
    pub blocked_promotions_when_degraded: bool,
    #[serde(default)]
    pub blocked_promotions_when_budget_exhausted: bool,
}

impl PolicySnapshot {
    pub fn permissive(
        policy_version: impl Into<String>,
        effective_from: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
            policy_version: policy_version.into(),
            effective_from: effective_from.into(),
            effective_to: None,
            autonomy_ceiling: AutonomyCeiling::PromotionEligible,
            method_rules: vec![
                CheckMethod::ExactBoundedOracle,
                CheckMethod::ConservativeOracle,
                CheckMethod::DeltaParityOracle,
                CheckMethod::TemporalReplayOracle,
                CheckMethod::CausalRefuter,
                CheckMethod::MinimalPerturbationOracle,
                CheckMethod::PairedPatch,
                CheckMethod::AdvisoryOnly,
            ]
            .into_iter()
            .map(|method| MethodPolicy {
                method,
                allowed: true,
                max_autonomy: AutonomyCeiling::PromotionEligible,
                approval_requirement: ApprovalRequirement::None,
            })
            .collect(),
            blocked_promotions_when_degraded: true,
            blocked_promotions_when_budget_exhausted: true,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRecord {
    pub schema_version: String,
    pub approval_record_id: ApprovalRecordId,
    pub policy_version: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub case_id: Option<stack_ids::VerificationCaseId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub plan_id: Option<stack_ids::CheckPlanId>,
    pub scope: ApprovalScope,
    pub approver: String,
    pub approved_at: String,
    #[serde(default)]
    pub reviewed_artifact_refs: Vec<String>,
    pub rationale: String,
}

impl ApprovalRecord {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        policy_version: impl Into<String>,
        case_id: Option<stack_ids::VerificationCaseId>,
        plan_id: Option<stack_ids::CheckPlanId>,
        scope: ApprovalScope,
        approver: impl Into<String>,
        approved_at: impl Into<String>,
        reviewed_artifact_refs: Vec<String>,
        rationale: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: APPROVAL_RECORD_V1_SCHEMA.into(),
            approval_record_id: ApprovalRecordId::generate(),
            policy_version: policy_version.into(),
            case_id,
            plan_id,
            scope,
            approver: approver.into(),
            approved_at: approved_at.into(),
            reviewed_artifact_refs,
            rationale: rationale.into(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyDecision {
    pub schema_version: String,
    pub decision_id: PolicyDecisionId,
    pub policy_version: String,
    pub case_id: stack_ids::VerificationCaseId,
    pub plan_id: stack_ids::CheckPlanId,
    pub evaluated_at: String,
    pub method_allowed: bool,
    pub autonomy_allowed: bool,
    pub promotion_allowed: bool,
    pub approval_required: bool,
    pub approval_satisfied: bool,
    #[serde(default)]
    pub reasons: Vec<String>,
}

impl PolicyDecision {
    pub fn allows_execution(&self) -> bool {
        self.method_allowed
            && self.autonomy_allowed
            && (!self.approval_required || self.approval_satisfied)
    }
}

pub fn approval_matches(
    approval: &ApprovalRecord,
    case: &VerificationCase,
    plan: &CheckPlan,
    evaluated_at: &str,
) -> bool {
    if approval
        .case_id
        .as_ref()
        .is_some_and(|case_id| *case_id != case.case_id)
    {
        return false;
    }
    if approval
        .plan_id
        .as_ref()
        .is_some_and(|plan_id| *plan_id != plan.plan_id)
    {
        return false;
    }
    if approval.scope.namespace != case.region.namespace {
        return false;
    }
    if approval
        .scope
        .target_key
        .as_ref()
        .is_some_and(|target_key| target_key != &case.region.target_key)
    {
        return false;
    }
    if approval
        .scope
        .method
        .is_some_and(|method| method != plan.method)
    {
        return false;
    }
    if approval
        .scope
        .promotion_class
        .is_some_and(|promotion_class| promotion_class != plan.promotion_class)
    {
        return false;
    }
    if approval
        .scope
        .expires_at
        .as_ref()
        .is_some_and(|expires_at| expires_at.as_str() <= evaluated_at)
    {
        return false;
    }

    true
}

pub fn policy_as_of<'a>(policies: &'a [PolicySnapshot], as_of: &str) -> Option<&'a PolicySnapshot> {
    policies
        .iter()
        .filter(|policy| {
            policy.effective_from.as_str() <= as_of
                && policy
                    .effective_to
                    .as_ref()
                    .map_or(true, |effective_to| effective_to.as_str() > as_of)
        })
        .max_by(|left, right| left.effective_from.cmp(&right.effective_from))
}

pub fn evaluate_policy(
    snapshot: &PolicySnapshot,
    case: &VerificationCase,
    plan: &CheckPlan,
    approvals: &[ApprovalRecord],
    degraded: bool,
    budget_exhausted: bool,
) -> PolicyDecision {
    let rule = snapshot
        .method_rules
        .iter()
        .find(|rule| rule.method == plan.method);
    let mut reasons = Vec::new();
    let method_allowed = rule.is_some_and(|rule| rule.allowed);
    if !method_allowed {
        reasons.push(format!("method {:?} is denied by policy", plan.method));
    }

    let required_autonomy = if plan.advisory_only {
        AutonomyCeiling::AdvisoryOnly
    } else if plan.promotable_if_completed {
        AutonomyCeiling::PromotionEligible
    } else {
        AutonomyCeiling::VerificationOnly
    };
    let max_autonomy = rule
        .map(|rule| rule.max_autonomy)
        .unwrap_or(AutonomyCeiling::AdvisoryOnly);
    let autonomy_ceiling = std::cmp::min(snapshot.autonomy_ceiling, max_autonomy);
    let autonomy_allowed = required_autonomy <= autonomy_ceiling;
    if !autonomy_allowed {
        reasons.push(format!(
            "required autonomy {:?} exceeds ceiling {:?}",
            required_autonomy, autonomy_ceiling
        ));
    }

    let approval_required =
        rule.is_some_and(|rule| rule.approval_requirement == ApprovalRequirement::HumanReview);
    let approval_satisfied = !approval_required
        || approvals.iter().any(|approval| {
            approval.policy_version == snapshot.policy_version
                && approval_matches(approval, case, plan, case.opened_at.as_str())
        });
    if approval_required && !approval_satisfied {
        reasons.push("human approval is required before execution".into());
    }

    let mut promotion_allowed = method_allowed && autonomy_allowed;
    if degraded && snapshot.blocked_promotions_when_degraded {
        promotion_allowed = false;
        reasons.push("degraded path cannot promote".into());
    }
    if budget_exhausted && snapshot.blocked_promotions_when_budget_exhausted {
        promotion_allowed = false;
        reasons.push("budget-exhausted path cannot promote".into());
    }

    PolicyDecision {
        schema_version: POLICY_DECISION_V1_SCHEMA.into(),
        decision_id: PolicyDecisionId::generate(),
        policy_version: snapshot.policy_version.clone(),
        case_id: case.case_id.clone(),
        plan_id: plan.plan_id.clone(),
        evaluated_at: case.opened_at.clone(),
        method_allowed,
        autonomy_allowed,
        promotion_allowed,
        approval_required,
        approval_satisfied,
        reasons,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use stack_ids::{AttemptId, ScopeKey, TraceCtx};
    use verification_control::{
        CaseRegion, CheckPlan, PromotionClass, ReversibilityClass, VerificationCase,
        VerificationCaseClass,
    };

    #[test]
    fn denied_method_cannot_execute() {
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
            true,
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
            "patch",
            serde_json::json!({}),
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
        };

        let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
        assert!(!decision.allows_execution());
    }

    #[test]
    fn scoped_approval_can_satisfy_runtime_generated_case_ids() {
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
        let policy = PolicySnapshot {
            schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
            policy_version: "policy-1".into(),
            effective_from: "2026-03-01T00:00:00Z".into(),
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
        };
        let approval = ApprovalRecord::new(
            "policy-1",
            None,
            None,
            ApprovalScope {
                namespace: "demo".into(),
                target_key: Some("unverified:claim-v1".into()),
                method: Some(CheckMethod::ExactBoundedOracle),
                promotion_class: Some(PromotionClass::P2),
                expires_at: None,
                reusable: true,
            },
            "operator",
            "2026-03-12T00:00:00Z",
            vec!["claim-v1".into()],
            "reviewed exact oracle promotion",
        );

        let decision = evaluate_policy(&policy, &case, &plan, &[approval], false, false);
        assert!(decision.approval_required);
        assert!(decision.approval_satisfied);
        assert!(decision.allows_execution());
    }

    #[test]
    fn v21_v24_policy_profiles_roundtrip() {
        let effect = EffectPolicyProfileV1 {
            schema_version: EFFECT_POLICY_PROFILE_V1_SCHEMA.into(),
            effect_policy_profile_id: stack_ids::EffectPolicyProfileId::new(
                "effect-policy-profile-1",
            ),
            allowed_run_modes: vec!["dry_run".into(), "live".into()],
            required_preflight_checks: vec!["admission".into(), "budget".into()],
            required_observation_classes: vec!["monitoring".into()],
            requires_compensation_plan_for: vec!["external_write".into()],
            block_live_without_commit: true,
        };
        let delegation = DelegationPolicyProfileV1 {
            schema_version: DELEGATION_POLICY_PROFILE_V1_SCHEMA.into(),
            delegation_policy_profile_id: stack_ids::DelegationPolicyProfileId::new(
                "delegation-policy-profile-1",
            ),
            max_delegation_depth: 1,
            break_glass_requires_post_hoc_review: true,
            forbidden_role_combinations: vec!["requester+approver".into()],
            require_typed_authority_chain: true,
        };
        let release = ReleasePolicyProfileV1 {
            schema_version: RELEASE_POLICY_PROFILE_V1_SCHEMA.into(),
            release_policy_profile_id: stack_ids::ReleasePolicyProfileId::new(
                "release-policy-profile-1",
            ),
            required_assurance_sections: vec!["hazards".into(), "controls".into()],
            required_monitor_classes: vec!["error_budget".into()],
            block_on_open_obligations: true,
            forbid_score_only_gate: true,
        };
        let continuity = ContinuityPolicyProfileV1 {
            schema_version: CONTINUITY_POLICY_PROFILE_V1_SCHEMA.into(),
            continuity_policy_profile_id: stack_ids::ContinuityPolicyProfileId::new(
                "continuity-policy-profile-1",
            ),
            required_forensic_freeze_surfaces: vec!["logs".into(), "receipts".into()],
            continuity_exception_ttl_minutes: 30,
            requires_postmortem_for_severity: vec!["sev1".into(), "sev2".into()],
            require_error_budget_linkage: true,
        };

        let json = serde_json::to_string(&(effect, delegation, release, continuity))
            .expect("serialize policy profiles");
        let _: (
            EffectPolicyProfileV1,
            DelegationPolicyProfileV1,
            ReleasePolicyProfileV1,
            ContinuityPolicyProfileV1,
        ) = serde_json::from_str(&json).expect("deserialize policy profiles");
    }
}
