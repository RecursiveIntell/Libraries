use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteSliceResultV1 {
    pub schema_version: String,
    pub remote_slice_result_id: String,
    pub remote_slice_request_id: String,
    pub returned_artifact_refs: Vec<String>,
    pub exactness_class: String,
    pub remote_execution_evidence: String,
    pub disclosure_markers: Vec<String>,
    pub replay_handle: String,
    pub local_admission_recommendation: String,
    pub attestation_envelope_id: String,
}
