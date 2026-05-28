use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteSliceRequestV1 {
    pub schema_version: String,
    pub remote_slice_request_id: String,
    pub requested_slice_definition: String,
    pub required_artifact_refs: Vec<String>,
    pub allowed_disclosure_policy: String,
    pub exactness_target: String,
    pub trust_root_set_id: String,
    pub challenge_expectations: Vec<String>,
    pub remote_oracle_lease_id: String,
}
