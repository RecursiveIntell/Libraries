use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApplicabilityContextId, AssuranceCaseId, AuthorityLeaseId, CompiledObligationSetId,
    CompositionConflictSetId, ContinuityExceptionId, DeploymentProfileId, EffectIntentId,
    EffectWindowId, EffectiveConstitutionId, ErrorBudgetLedgerId, IncidentCaseId,
    PolicyImpactDiffId, ProfileSetId, RecoveryPlanId, ReleaseReadinessDecisionId,
};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectRuntimeViewV1 {
    pub schema_version: String,
    pub effect_intent_id: EffectIntentId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effect_window_id: Option<EffectWindowId>,
    pub reversibility_class: String,
    pub run_mode: String,
    pub publication_status: String,
    #[serde(default)]
    pub degradation_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationRuntimeViewV1 {
    pub schema_version: String,
    pub authority_lease_id: AuthorityLeaseId,
    pub validity_state: String,
    pub advisory_only: bool,
    #[serde(default)]
    pub revocation_refs: Vec<String>,
    #[serde(default)]
    pub missing_backpointers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DeployabilityRuntimeViewV1 {
    pub schema_version: String,
    pub deployment_profile_id: DeploymentProfileId,
    pub assurance_case_id: AssuranceCaseId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub release_readiness_decision_id: Option<ReleaseReadinessDecisionId>,
    pub readiness_state: String,
    pub advisory_only: bool,
    #[serde(default)]
    pub open_gaps: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityRuntimeViewV1 {
    pub schema_version: String,
    pub incident_case_id: IncidentCaseId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub recovery_plan_id: Option<RecoveryPlanId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuity_exception_id: Option<ContinuityExceptionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error_budget_ledger_id: Option<ErrorBudgetLedgerId>,
    pub replay_completeness: String,
    #[serde(default)]
    pub degradation_markers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectiveConstitutionViewV1 {
    pub schema_version: String,
    pub effective_constitution_id: EffectiveConstitutionId,
    pub applicability_context_id: ApplicabilityContextId,
    pub profile_set_id: ProfileSetId,
    pub mode_classification: String,
    pub blocking: bool,
    #[serde(default)]
    pub approval_requirements: Vec<String>,
    #[serde(default)]
    pub hard_block_summary: Vec<String>,
    #[serde(default)]
    pub active_warning_summary: Vec<String>,
    pub valid_as_of: String,
    pub recorded_as_of: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledObligationRuntimeViewV1 {
    pub schema_version: String,
    pub compiled_obligation_set_id: CompiledObligationSetId,
    pub effective_constitution_id: EffectiveConstitutionId,
    #[serde(default)]
    pub required_approvals: Vec<String>,
    #[serde(default)]
    pub required_checks: Vec<String>,
    #[serde(default)]
    pub required_monitors: Vec<String>,
    #[serde(default)]
    pub required_disclosure_obligations: Vec<String>,
    #[serde(default)]
    pub required_post_hoc_review_obligations: Vec<String>,
    #[serde(default)]
    pub unresolved_dependency_refs: Vec<String>,
    pub effect_eligibility_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompositionConflictRuntimeViewV1 {
    pub schema_version: String,
    pub composition_conflict_set_id: CompositionConflictSetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_constitution_id: Option<EffectiveConstitutionId>,
    pub blocking: bool,
    #[serde(default)]
    pub conflict_keys: Vec<String>,
    #[serde(default)]
    pub review_paths: Vec<String>,
    #[serde(default)]
    pub admissible_exception_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyImpactDiffRuntimeViewV1 {
    pub schema_version: String,
    pub policy_impact_diff_id: PolicyImpactDiffId,
    pub from_effective_constitution_id: EffectiveConstitutionId,
    pub to_effective_constitution_id: EffectiveConstitutionId,
    #[serde(default)]
    pub changed_obligation_families: Vec<String>,
    #[serde(default)]
    pub newly_blocked_paths: Vec<String>,
    #[serde(default)]
    pub newly_admitted_paths: Vec<String>,
    #[serde(default)]
    pub migration_consequences: Vec<String>,
    pub simulation_only: bool,
}
