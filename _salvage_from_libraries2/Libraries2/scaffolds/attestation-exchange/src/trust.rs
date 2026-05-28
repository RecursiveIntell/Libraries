use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct TrustRootSetV1 {
    pub schema_version: String,
    pub trust_root_set_id: String,
    pub trust_root_identities: Vec<String>,
    pub allowed_signer_classes: Vec<String>,
    pub expiration_policy: String,
    pub rotation_policy: String,
    pub allowed_artifact_families: Vec<String>,
    pub revocation_sources: Vec<String>,
    pub policy_owner_refs: Vec<String>,
}
