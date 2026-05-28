use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ClaimId, ClaimStateId, ClaimVersionId, ContentDigest, ContradictionWitnessId,
    RetractionRecordId, SemanticsProfileId, SupportSetId,
};

use crate::{DegradationKindV1, ExactnessLevelV1, EvidenceAdmissibilityV1, SemanticViewV1};

pub const BILATTICE_TRUTH_V1_SCHEMA: &str = "bilattice_truth_v1";
pub const SUPPORT_SET_V1_SCHEMA: &str = "support_set_v1";
pub const CONTRADICTION_WITNESS_V1_SCHEMA: &str = "contradiction_witness_v1";
pub const RETRACTION_RECORD_V1_SCHEMA: &str = "retraction_record_v1";
pub const CLAIM_STATE_V13_SCHEMA: &str = "claim_state_v13";

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BilatticeTruthV1 {
    Unknown,
    TrueOnly,
    FalseOnly,
    Both,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportPolarityV1 {
    Supports,
    Refutes,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum SupportProvenanceKindV1 {
    EvidenceRef,
    ClaimVersion,
    RelationVersion,
    Episode,
    Receipt,
    External,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SupportTokenV1 {
    pub token_id: String,
    pub kind: SupportProvenanceKindV1,
    pub reference: String,
    pub polarity: SupportPolarityV1,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(tag = "op", rename_all = "snake_case")]
pub enum SupportExprV1 {
    Token { token_id: String },
    AnyOf { children: Vec<SupportExprV1> },
    AllOf { children: Vec<SupportExprV1> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SupportSetV1 {
    pub schema_version: String,
    pub support_set_id: SupportSetId,
    pub claim_id: ClaimId,
    pub semantics_profile_id: SemanticsProfileId,
    pub support_tokens: Vec<SupportTokenV1>,
    pub support_expr: SupportExprV1,
    pub content_digest: ContentDigest,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QualityVectorV1 {
    pub exactness: ExactnessLevelV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub degradation: Vec<DegradationKindV1>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub freshness: Option<String>,
    pub replay_limited: bool,
    pub execution_contaminated: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ContradictionWitnessV1 {
    pub schema_version: String,
    pub contradiction_witness_id: ContradictionWitnessId,
    pub claim_id: ClaimId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflicting_token_ids: Vec<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RetractionRecordV1 {
    pub schema_version: String,
    pub retraction_record_id: RetractionRecordId,
    pub claim_id: ClaimId,
    pub retracted_claim_version_id: ClaimVersionId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub superseded_by_claim_version_id: Option<ClaimVersionId>,
    pub effective_recorded_at: String,
    pub reason: String,
    pub cascade_required: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub delta_summary: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ClaimStateV13 {
    pub schema_version: String,
    pub claim_state_id: ClaimStateId,
    pub claim_id: ClaimId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub claim_version_id: Option<ClaimVersionId>,
    pub semantics_profile_id: SemanticsProfileId,
    pub view: SemanticViewV1,
    pub bilattice_truth: BilatticeTruthV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_set_id: Option<SupportSetId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub support_set_digest: Option<ContentDigest>,
    pub quality_vector: QualityVectorV1,
    pub evidence_admissibility: EvidenceAdmissibilityV1,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub contradiction_witness_id: Option<ContradictionWitnessId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_from: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub valid_to: Option<String>,
    pub tx_from: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tx_to: Option<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub proof_obligations_remaining: Vec<String>,
    pub policy_action_allowed: bool,
}
