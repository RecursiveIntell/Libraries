use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DisputeBundleV1 {
    pub schema_version: String,
    pub dispute_bundle_id: String,
    pub challenged_artifact_refs: Vec<String>,
    pub basis_of_challenge: String,
    pub counterevidence_refs: Vec<String>,
    pub replay_or_recheck_request: String,
    pub escalation_target: String,
    pub current_disposition: String,
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
