#![cfg_attr(test, allow(clippy::expect_used))]
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApplicabilityContextId, ApprovalRecordId, CompiledObligationSetId, CompositionReceiptId,
    ContinuityPolicyProfileId, DelegationPolicyProfileId, EffectPolicyProfileId,
    EffectiveConstitutionId, PolicyDecisionId, ProfileSetId, ReleasePolicyProfileId,
};
use std::collections::BTreeSet;
use verification_control::{CheckMethod, CheckPlan, PromotionClass, VerificationCase};

pub mod v14;
pub use v14::{
    ArtifactAdmissionPolicyV1, DisclosureBudgetV1, DisclosurePolicyV1, ExperimentBudgetV1,
    RefuterSuiteV1, ARTIFACT_ADMISSION_POLICY_V1_SCHEMA, DISCLOSURE_BUDGET_V1_SCHEMA,
    DISCLOSURE_POLICY_V1_SCHEMA, EXPERIMENT_BUDGET_V1_SCHEMA, REFUTER_SUITE_V1_SCHEMA,
};

pub mod permit;
pub mod profile_p1_privacy;
pub mod profile_p2_locality;
pub use permit::{CommitToken, ExecutionPermit, ExecutionPermitScope, PermitIssuanceError};
pub use profile_p1_privacy::*;
pub use profile_p2_locality::*;

pub const POLICY_SNAPSHOT_V1_SCHEMA: &str = "policy_snapshot_v1";
pub const APPROVAL_RECORD_V1_SCHEMA: &str = "approval_record_v1";
pub const POLICY_DECISION_V1_SCHEMA: &str = "policy_decision_v1";
pub const EFFECT_POLICY_PROFILE_V1_SCHEMA: &str = "effect_policy_profile_v1";
pub const DELEGATION_POLICY_PROFILE_V1_SCHEMA: &str = "delegation_policy_profile_v1";
pub const RELEASE_POLICY_PROFILE_V1_SCHEMA: &str = "release_policy_profile_v1";
pub const CONTINUITY_POLICY_PROFILE_V1_SCHEMA: &str = "continuity_policy_profile_v1";

pub use verification_control::{
    ConstitutionalContextStatus, V25CitationContext, V25ControlObligationRefs,
};

fn require_schema_version(
    found: &str,
    expected: &'static str,
    field: &'static str,
) -> Result<(), &'static str> {
    if found != expected {
        return Err(field);
    }
    Ok(())
}

fn require_non_empty(value: &str, field: &'static str) -> Result<(), &'static str> {
    if value.trim().is_empty() {
        return Err(field);
    }
    Ok(())
}

fn require_unique<T>(values: &[T], field: &'static str) -> Result<(), &'static str>
where
    T: Copy + Ord,
{
    let mut seen = BTreeSet::new();
    for value in values {
        if !seen.insert(*value) {
            return Err(field);
        }
    }
    Ok(())
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EffectPolicyRunModeV1 {
    Shadow,
    DryRun,
    Live,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EffectPreflightCheckV1 {
    Integrity,
    Admission,
    Reachability,
    Budget,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum EffectObservationClassV1 {
    Latency,
    ErrorBudget,
    Compensation,
    Monitoring,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum CompensationPolicyTriggerV1 {
    Mutation,
    ExternalSideEffect,
    ExternalWrite,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum DelegationRoleCombinationV1 {
    #[serde(rename = "requester+approver")]
    RequesterApprover,
    #[serde(rename = "commander+auditor")]
    CommanderAuditor,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseAssuranceSectionV1 {
    HazardSummary,
    ControlEvidence,
    RollbackPlan,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ReleaseMonitorClassV1 {
    ErrorBudget,
    LatencyP95,
    Saturation,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum ForensicFreezeSurfaceV1 {
    AuditLog,
    EffectLedger,
    DeploymentConfig,
}

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize, JsonSchema,
)]
#[serde(rename_all = "snake_case")]
pub enum PolicySeverityV1 {
    Sev1,
    Sev2,
    Sev3,
    Sev4,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectPolicyProfileV1 {
    pub schema_version: String,
    pub effect_policy_profile_id: EffectPolicyProfileId,
    #[serde(default)]
    pub allowed_run_modes: Vec<EffectPolicyRunModeV1>,
    #[serde(default)]
    pub required_preflight_checks: Vec<EffectPreflightCheckV1>,
    #[serde(default)]
    pub required_observation_classes: Vec<EffectObservationClassV1>,
    #[serde(default)]
    pub requires_compensation_plan_for: Vec<CompensationPolicyTriggerV1>,
    pub block_live_without_commit: bool,
}

impl EffectPolicyProfileV1 {
    /// Builds an effect policy profile with typed run, preflight, observation,
    /// and compensation requirements.
    pub fn new(
        effect_policy_profile_id: EffectPolicyProfileId,
        allowed_run_modes: Vec<EffectPolicyRunModeV1>,
        required_preflight_checks: Vec<EffectPreflightCheckV1>,
        required_observation_classes: Vec<EffectObservationClassV1>,
        requires_compensation_plan_for: Vec<CompensationPolicyTriggerV1>,
        block_live_without_commit: bool,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: EFFECT_POLICY_PROFILE_V1_SCHEMA.to_string(),
            effect_policy_profile_id,
            allowed_run_modes,
            required_preflight_checks,
            required_observation_classes,
            requires_compensation_plan_for,
            block_live_without_commit,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates schema identity and rejects duplicate typed policy vocab.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            EFFECT_POLICY_PROFILE_V1_SCHEMA,
            "schema_version",
        )?;
        require_unique(&self.allowed_run_modes, "allowed_run_modes")?;
        require_unique(&self.required_preflight_checks, "required_preflight_checks")?;
        require_unique(
            &self.required_observation_classes,
            "required_observation_classes",
        )?;
        require_unique(
            &self.requires_compensation_plan_for,
            "requires_compensation_plan_for",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationPolicyProfileV1 {
    pub schema_version: String,
    pub delegation_policy_profile_id: DelegationPolicyProfileId,
    pub max_delegation_depth: i64,
    pub break_glass_requires_post_hoc_review: bool,
    #[serde(default)]
    pub forbidden_role_combinations: Vec<DelegationRoleCombinationV1>,
    pub require_typed_authority_chain: bool,
}

impl DelegationPolicyProfileV1 {
    /// Builds a delegation policy profile with typed role-combination bans.
    pub fn new(
        delegation_policy_profile_id: DelegationPolicyProfileId,
        max_delegation_depth: i64,
        break_glass_requires_post_hoc_review: bool,
        forbidden_role_combinations: Vec<DelegationRoleCombinationV1>,
        require_typed_authority_chain: bool,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: DELEGATION_POLICY_PROFILE_V1_SCHEMA.to_string(),
            delegation_policy_profile_id,
            max_delegation_depth,
            break_glass_requires_post_hoc_review,
            forbidden_role_combinations,
            require_typed_authority_chain,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates schema identity, delegation depth, and duplicate role bans.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            DELEGATION_POLICY_PROFILE_V1_SCHEMA,
            "schema_version",
        )?;
        if self.max_delegation_depth < 0 {
            return Err("max_delegation_depth");
        }
        require_unique(
            &self.forbidden_role_combinations,
            "forbidden_role_combinations",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ReleasePolicyProfileV1 {
    pub schema_version: String,
    pub release_policy_profile_id: ReleasePolicyProfileId,
    #[serde(default)]
    pub required_assurance_sections: Vec<ReleaseAssuranceSectionV1>,
    #[serde(default)]
    pub required_monitor_classes: Vec<ReleaseMonitorClassV1>,
    pub block_on_open_obligations: bool,
    pub forbid_score_only_gate: bool,
}

impl ReleasePolicyProfileV1 {
    /// Builds a release policy profile with typed assurance and monitoring
    /// requirements.
    pub fn new(
        release_policy_profile_id: ReleasePolicyProfileId,
        required_assurance_sections: Vec<ReleaseAssuranceSectionV1>,
        required_monitor_classes: Vec<ReleaseMonitorClassV1>,
        block_on_open_obligations: bool,
        forbid_score_only_gate: bool,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: RELEASE_POLICY_PROFILE_V1_SCHEMA.to_string(),
            release_policy_profile_id,
            required_assurance_sections,
            required_monitor_classes,
            block_on_open_obligations,
            forbid_score_only_gate,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates schema identity and rejects duplicate release requirements.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            RELEASE_POLICY_PROFILE_V1_SCHEMA,
            "schema_version",
        )?;
        require_unique(
            &self.required_assurance_sections,
            "required_assurance_sections",
        )?;
        require_unique(&self.required_monitor_classes, "required_monitor_classes")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityPolicyProfileV1 {
    pub schema_version: String,
    pub continuity_policy_profile_id: ContinuityPolicyProfileId,
    #[serde(default)]
    pub required_forensic_freeze_surfaces: Vec<ForensicFreezeSurfaceV1>,
    pub continuity_exception_ttl_minutes: i64,
    #[serde(default)]
    pub requires_postmortem_for_severity: Vec<PolicySeverityV1>,
    pub require_error_budget_linkage: bool,
}

impl ContinuityPolicyProfileV1 {
    /// Builds a continuity policy profile with typed freeze-surface and
    /// severity requirements.
    pub fn new(
        continuity_policy_profile_id: ContinuityPolicyProfileId,
        required_forensic_freeze_surfaces: Vec<ForensicFreezeSurfaceV1>,
        continuity_exception_ttl_minutes: i64,
        requires_postmortem_for_severity: Vec<PolicySeverityV1>,
        require_error_budget_linkage: bool,
    ) -> Result<Self, &'static str> {
        let value = Self {
            schema_version: CONTINUITY_POLICY_PROFILE_V1_SCHEMA.to_string(),
            continuity_policy_profile_id,
            required_forensic_freeze_surfaces,
            continuity_exception_ttl_minutes,
            requires_postmortem_for_severity,
            require_error_budget_linkage,
        };
        value.validate()?;
        Ok(value)
    }

    /// Validates schema identity, positive TTL, and duplicate continuity
    /// vocab entries.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            CONTINUITY_POLICY_PROFILE_V1_SCHEMA,
            "schema_version",
        )?;
        if self.continuity_exception_ttl_minutes <= 0 {
            return Err("continuity_exception_ttl_minutes");
        }
        require_unique(
            &self.required_forensic_freeze_surfaces,
            "required_forensic_freeze_surfaces",
        )?;
        require_unique(
            &self.requires_postmortem_for_severity,
            "requires_postmortem_for_severity",
        )?;
        Ok(())
    }
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
    /// Carries ProfileExceptionBundleId through the shared v25 citation context when present.
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
}

impl PolicySnapshot {
    /// Builds a permissive reference policy snapshot for tests and compatibility fixtures.
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
            citation: V25CitationContext {
                applicability_context_id: Some(ApplicabilityContextId::new(
                    "permissive-applicability-context",
                )),
                profile_set_id: Some(ProfileSetId::new("permissive-profile-set")),
                composition_receipt_id: Some(CompositionReceiptId::new(
                    "permissive-composition-receipt",
                )),
                effective_constitution_id: Some(EffectiveConstitutionId::new(
                    "permissive-effective-constitution",
                )),
                compiled_obligation_set_id: Some(CompiledObligationSetId::new(
                    "permissive-compiled-obligation-set",
                )),
                composition_conflict_set_id: None,
                profile_exception_bundle_ids: Vec::new(),
            },
            obligation_refs: V25ControlObligationRefs {
                required_obligation_refs: vec!["obligation:permissive-required".into()],
                blocking_obligation_refs: vec!["obligation:permissive-blocking".into()],
                monitoring_obligation_refs: vec!["obligation:permissive-monitoring".into()],
            },
        }
    }

    /// Builds a deny-default policy snapshot that blocks all methods and
    /// promotions. Used as a fail-closed fallback when governance state is
    /// unavailable (AUTH-001 fix).
    pub fn deny(
        policy_version: impl Into<String>,
        effective_from: impl Into<String>,
    ) -> Self {
        Self {
            schema_version: POLICY_SNAPSHOT_V1_SCHEMA.into(),
            policy_version: policy_version.into(),
            effective_from: effective_from.into(),
            effective_to: None,
            autonomy_ceiling: AutonomyCeiling::AdvisoryOnly,
            method_rules: vec![],
            blocked_promotions_when_degraded: true,
            blocked_promotions_when_budget_exhausted: true,
            citation: V25CitationContext {
                applicability_context_id: Some(ApplicabilityContextId::new(
                    "deny-applicability-context",
                )),
                profile_set_id: Some(ProfileSetId::new("deny-profile-set")),
                composition_receipt_id: Some(CompositionReceiptId::new(
                    "deny-composition-receipt",
                )),
                effective_constitution_id: Some(EffectiveConstitutionId::new(
                    "deny-effective-constitution",
                )),
                compiled_obligation_set_id: Some(CompiledObligationSetId::new(
                    "deny-compiled-obligation-set",
                )),
                composition_conflict_set_id: None,
                profile_exception_bundle_ids: Vec::new(),
            },
            obligation_refs: V25ControlObligationRefs {
                required_obligation_refs: vec![],
                blocking_obligation_refs: vec!["obligation:deny-all".into()],
                monitoring_obligation_refs: vec![],
            },
        }
    }

    /// Validates snapshot metadata needed to evaluate a policy version.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            POLICY_SNAPSHOT_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.policy_version, "policy_version")?;
        require_non_empty(&self.effective_from, "effective_from")?;
        Ok(())
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
    /// Creates an approval record bound to a policy version, optional case, and optional plan.
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

    /// Validates approval-record metadata and its scoped approval target.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            APPROVAL_RECORD_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.policy_version, "policy_version")?;
        self.scope.validate()?;
        require_non_empty(&self.approver, "approver")?;
        require_non_empty(&self.approved_at, "approved_at")?;
        require_non_empty(&self.rationale, "rationale")?;
        Ok(())
    }
}

impl ApprovalScope {
    /// Validates the namespace-scoped approval target and optional expiry.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_non_empty(&self.namespace, "namespace")?;
        if self
            .target_key
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err("target_key");
        }
        if self
            .expires_at
            .as_ref()
            .is_some_and(|value| value.trim().is_empty())
        {
            return Err("expires_at");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyDecision {
    pub schema_version: String,
    pub decision_id: PolicyDecisionId,
    pub policy_version: String,
    pub case_id: stack_ids::VerificationCaseId,
    pub plan_id: stack_ids::CheckPlanId,
    #[serde(flatten)]
    pub citation: V25CitationContext,
    #[serde(flatten)]
    pub obligation_refs: V25ControlObligationRefs,
    pub citation_status: ConstitutionalContextStatus,
    pub obligation_refs_status: ConstitutionalContextStatus,
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
    /// Validates the emitted policy decision envelope metadata.
    pub fn validate(&self) -> Result<(), &'static str> {
        require_schema_version(
            &self.schema_version,
            POLICY_DECISION_V1_SCHEMA,
            "schema_version",
        )?;
        require_non_empty(&self.policy_version, "policy_version")?;
        require_non_empty(&self.evaluated_at, "evaluated_at")?;
        Ok(())
    }
}

/// Returns true when an approval still covers the evaluated case and plan scope.
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

/// Selects the newest policy snapshot that is effective for the requested timestamp.
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

/// Evaluates one verification case against policy and emits a promotion-aware decision record.
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

    let citation = snapshot.citation.clone();
    let citation_status = citation.context_status();
    let obligation_refs = snapshot.obligation_refs.clone();
    let obligation_refs_status = obligation_refs.context_status();

    let promotion_requested = plan.promotable_if_completed && !plan.advisory_only;
    let mut promotion_allowed = promotion_requested && method_allowed && autonomy_allowed;
    if degraded && snapshot.blocked_promotions_when_degraded {
        promotion_allowed = false;
        reasons.push("degraded path cannot promote".into());
    }
    if budget_exhausted && snapshot.blocked_promotions_when_budget_exhausted {
        promotion_allowed = false;
        reasons.push("budget-exhausted path cannot promote".into());
    }
    if !citation_status.is_complete() {
        promotion_allowed = false;
        reasons.push("constitutional citation context is missing or partial".into());
    }
    if !obligation_refs_status.is_complete() {
        promotion_allowed = false;
        reasons.push("constitutional obligation refs are missing".into());
    }

    PolicyDecision {
        schema_version: POLICY_DECISION_V1_SCHEMA.into(),
        decision_id: PolicyDecisionId::generate(),
        policy_version: snapshot.policy_version.clone(),
        case_id: case.case_id.clone(),
        plan_id: plan.plan_id.clone(),
        citation,
        obligation_refs,
        citation_status,
        obligation_refs_status,
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

    fn complete_citation() -> V25CitationContext {
        V25CitationContext {
            applicability_context_id: Some(ApplicabilityContextId::new("test-applicability")),
            profile_set_id: Some(ProfileSetId::new("test-profile-set")),
            composition_receipt_id: Some(CompositionReceiptId::new("test-composition")),
            effective_constitution_id: Some(EffectiveConstitutionId::new(
                "test-effective-constitution",
            )),
            compiled_obligation_set_id: Some(CompiledObligationSetId::new(
                "test-compiled-obligations",
            )),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: Vec::new(),
        }
    }

    fn complete_obligation_refs() -> V25ControlObligationRefs {
        V25ControlObligationRefs {
            required_obligation_refs: vec!["obligation:test-required".into()],
            blocking_obligation_refs: vec!["obligation:test-blocking".into()],
            monitoring_obligation_refs: vec!["obligation:test-monitoring".into()],
        }
    }

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
            citation: complete_citation(),
            obligation_refs: complete_obligation_refs(),
        };

        let decision = evaluate_policy(&policy, &case, &plan, &[], false, false);
        assert_eq!(
            decision.issue_execution_permit(&case, &plan, &[]),
            Err(PermitIssuanceError::MethodDenied)
        );
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
            citation: complete_citation(),
            obligation_refs: complete_obligation_refs(),
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

        let decision = evaluate_policy(
            &policy,
            &case,
            &plan,
            std::slice::from_ref(&approval),
            false,
            false,
        );
        assert!(decision.approval_required);
        assert!(decision.approval_satisfied);
        let permit = decision
            .issue_execution_permit(&case, &plan, &[approval])
            .expect("permit should mint for approved promotable plan");
        assert_eq!(permit.scope().namespace(), "demo");
        assert_eq!(permit.scope().target_key(), "unverified:claim-v1");
    }

    #[test]
    fn v21_v24_policy_profiles_roundtrip() {
        let effect = EffectPolicyProfileV1 {
            schema_version: EFFECT_POLICY_PROFILE_V1_SCHEMA.into(),
            effect_policy_profile_id: stack_ids::EffectPolicyProfileId::new(
                "effect-policy-profile-1",
            ),
            allowed_run_modes: vec![EffectPolicyRunModeV1::DryRun, EffectPolicyRunModeV1::Live],
            required_preflight_checks: vec![
                EffectPreflightCheckV1::Admission,
                EffectPreflightCheckV1::Budget,
            ],
            required_observation_classes: vec![EffectObservationClassV1::Monitoring],
            requires_compensation_plan_for: vec![CompensationPolicyTriggerV1::ExternalWrite],
            block_live_without_commit: true,
        };
        let delegation = DelegationPolicyProfileV1 {
            schema_version: DELEGATION_POLICY_PROFILE_V1_SCHEMA.into(),
            delegation_policy_profile_id: stack_ids::DelegationPolicyProfileId::new(
                "delegation-policy-profile-1",
            ),
            max_delegation_depth: 1,
            break_glass_requires_post_hoc_review: true,
            forbidden_role_combinations: vec![DelegationRoleCombinationV1::RequesterApprover],
            require_typed_authority_chain: true,
        };
        let release = ReleasePolicyProfileV1 {
            schema_version: RELEASE_POLICY_PROFILE_V1_SCHEMA.into(),
            release_policy_profile_id: stack_ids::ReleasePolicyProfileId::new(
                "release-policy-profile-1",
            ),
            required_assurance_sections: vec![
                ReleaseAssuranceSectionV1::HazardSummary,
                ReleaseAssuranceSectionV1::ControlEvidence,
            ],
            required_monitor_classes: vec![ReleaseMonitorClassV1::ErrorBudget],
            block_on_open_obligations: true,
            forbid_score_only_gate: true,
        };
        let continuity = ContinuityPolicyProfileV1 {
            schema_version: CONTINUITY_POLICY_PROFILE_V1_SCHEMA.into(),
            continuity_policy_profile_id: stack_ids::ContinuityPolicyProfileId::new(
                "continuity-policy-profile-1",
            ),
            required_forensic_freeze_surfaces: vec![
                ForensicFreezeSurfaceV1::AuditLog,
                ForensicFreezeSurfaceV1::EffectLedger,
            ],
            continuity_exception_ttl_minutes: 30,
            requires_postmortem_for_severity: vec![PolicySeverityV1::Sev1, PolicySeverityV1::Sev2],
            require_error_budget_linkage: true,
        };

        effect.validate().expect("effect profile validates");
        delegation.validate().expect("delegation profile validates");
        release.validate().expect("release profile validates");
        continuity.validate().expect("continuity profile validates");

        let json = serde_json::to_string(&(effect, delegation, release, continuity))
            .expect("serialize policy profiles");
        let _: (
            EffectPolicyProfileV1,
            DelegationPolicyProfileV1,
            ReleasePolicyProfileV1,
            ContinuityPolicyProfileV1,
        ) = serde_json::from_str(&json).expect("deserialize policy profiles");
    }

    #[test]
    fn duplicate_policy_vocab_is_rejected() {
        let effect = EffectPolicyProfileV1 {
            schema_version: EFFECT_POLICY_PROFILE_V1_SCHEMA.into(),
            effect_policy_profile_id: stack_ids::EffectPolicyProfileId::new(
                "effect-policy-profile-dup",
            ),
            allowed_run_modes: vec![EffectPolicyRunModeV1::Live, EffectPolicyRunModeV1::Live],
            required_preflight_checks: vec![EffectPreflightCheckV1::Integrity],
            required_observation_classes: vec![EffectObservationClassV1::Latency],
            requires_compensation_plan_for: vec![CompensationPolicyTriggerV1::Mutation],
            block_live_without_commit: true,
        };

        assert_eq!(effect.validate(), Err("allowed_run_modes"));
    }
}
