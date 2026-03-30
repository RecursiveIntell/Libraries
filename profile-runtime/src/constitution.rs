use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ApplicabilityContextId, CompiledObligationSetId, CompositionConflictSetId,
    CompositionReceiptId, CompositionRuleSetId, ContentDigest, EffectiveConstitutionId,
    PolicyImpactDiffId, ProfileExceptionBundleId, ProfileSetId,
};

use crate::rules::{CompiledObligationKindV1, FoldClassV1};

pub const COMPOSITION_RECEIPT_V1_SCHEMA: &str = "composition_receipt_v1";
pub const EFFECTIVE_CONSTITUTION_V1_SCHEMA: &str = "effective_constitution_v1";
pub const COMPILED_OBLIGATION_SET_V1_SCHEMA: &str = "compiled_obligation_set_v1";
pub const COMPOSITION_CONFLICT_SET_V1_SCHEMA: &str = "composition_conflict_set_v1";
pub const POLICY_IMPACT_DIFF_V1_SCHEMA: &str = "policy_impact_diff_v1";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompositionReceiptV1 {
    pub schema_version: String,
    pub composition_receipt_id: CompositionReceiptId,
    pub applicability_context_ref: ApplicabilityContextId,
    pub profile_set_ref: ProfileSetId,
    pub composition_rule_set_ref: CompositionRuleSetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_constitution_ref: Option<EffectiveConstitutionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub compiled_obligation_set_ref: Option<CompiledObligationSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_conflict_set_ref: Option<CompositionConflictSetId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation_markers: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub execution_context_ref: Option<String>,
    pub replay_handle: String,
    pub normalized_input_digest: ContentDigest,
    pub generated_at: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ConstitutionModeV1 {
    Normal,
    Incident,
    AdvisoryOnly,
    Blocked,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EffectiveConstitutionV1 {
    pub schema_version: String,
    pub effective_constitution_id: EffectiveConstitutionId,
    pub applicability_context_ref: ApplicabilityContextId,
    pub profile_set_ref: ProfileSetId,
    pub composition_rule_set_ref: CompositionRuleSetId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub base_law_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub doctrine_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted_profile_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admitted_exception_refs: Vec<ProfileExceptionBundleId>,
    pub current_mode_classification: ConstitutionModeV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hard_block_summary: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_warning_summary: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_requirements: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disclosure_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub residency_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tenancy_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub assurance_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub monitoring_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub continuity_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub effect_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub delegation_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub replay_obligations: Vec<String>,
    pub valid_as_of: String,
    pub recorded_as_of: String,
    pub explanation_summary: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledObligationEntryV1 {
    pub obligation_entry_id: String,
    pub obligation_family: String,
    pub obligation_key: String,
    pub output_kind: CompiledObligationKindV1,
    pub fold_class: FoldClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub string_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub numeric_value: Option<i64>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub expiry_at: Option<String>,
    pub blocking: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_profile_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_exception_refs: Vec<ProfileExceptionBundleId>,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct BlockEntryV1 {
    pub block_key: String,
    pub reason: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_profile_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissible_exception_classes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompiledObligationSetV1 {
    pub schema_version: String,
    pub compiled_obligation_set_id: CompiledObligationSetId,
    pub effective_constitution_ref: EffectiveConstitutionId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub obligation_entries: Vec<CompiledObligationEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub block_entries: Vec<BlockEntryV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_approvals: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_checks: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_monitors: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_evidence_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_disclosure_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_rollback_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_compensation_obligations: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_post_hoc_review_obligations: Vec<String>,
    pub promotion_eligibility_summary: String,
    pub release_eligibility_summary: String,
    pub effect_eligibility_summary: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_dependency_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompositionConflictV1 {
    pub obligation_family: String,
    pub obligation_key: String,
    pub conflict_class: String,
    pub blocking: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_profile_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub implicated_values: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub admissible_exception_classes: Vec<String>,
    pub review_path: String,
    pub explanation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CompositionConflictSetV1 {
    pub schema_version: String,
    pub composition_conflict_set_id: CompositionConflictSetId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub effective_constitution_ref: Option<EffectiveConstitutionId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub composition_receipt_ref: Option<CompositionReceiptId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicts: Vec<CompositionConflictV1>,
    pub blocking: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PolicyImpactDiffV1 {
    pub schema_version: String,
    pub policy_impact_diff_id: PolicyImpactDiffId,
    pub from_effective_constitution_ref: EffectiveConstitutionId,
    pub to_effective_constitution_ref: EffectiveConstitutionId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_obligation_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unchanged_obligation_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub newly_blocked_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub newly_admitted_paths: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_approval_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_monitoring_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_disclosure_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_residency_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub changed_incident_behavior: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub migration_consequences: Vec<String>,
    pub simulation_only: bool,
    pub explanation_summary: String,
}
