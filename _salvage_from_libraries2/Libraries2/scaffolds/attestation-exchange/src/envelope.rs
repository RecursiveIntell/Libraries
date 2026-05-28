use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct AttestationEnvelopeV1 {
    pub schema_version: String,
    pub attestation_envelope_id: String,
    pub artifact_family: String,
    pub artifact_version: String,
    pub content_digest: String,
    pub schema_identity: String,
    pub signer_identity: String,
    pub signing_time: String,
    pub trust_root_set_id: String,
    pub provenance_summary: String,
    pub disclosure_policy_id: String,
    pub replayability_class: String,
    pub revocation_refs: Vec<String>,
    pub supersession_refs: Vec<String>,
}
