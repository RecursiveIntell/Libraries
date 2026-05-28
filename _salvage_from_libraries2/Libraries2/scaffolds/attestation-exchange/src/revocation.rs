use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationRevocationV1 {
    pub schema_version: String,
    pub attestation_revocation_id: String,
    pub affected_refs: Vec<String>,
    pub revocation_reason: String,
    pub effective_time: String,
    pub blast_radius: String,
    pub required_local_invalidation_behavior: String,
    pub dispute_linkage: Option<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationSupersessionV1 {
    pub schema_version: String,
    pub attestation_supersession_id: String,
    pub prior_ref: String,
    pub replacement_ref: String,
    pub semantic_delta_summary: String,
    pub effective_time: String,
    pub replay_impact: String,
    pub requires_re_admission: bool,
}
