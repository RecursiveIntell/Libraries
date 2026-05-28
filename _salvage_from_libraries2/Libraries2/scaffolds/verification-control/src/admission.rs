use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ArtifactAdmissionPolicyV1 {
    pub schema_version: String,
    pub artifact_admission_policy_id: String,
    pub allowed_artifact_families: Vec<String>,
    pub required_trust_root_sets: Vec<String>,
    pub required_transparency_obligations: Vec<String>,
    pub required_replayability_class: String,
    pub disclosure_constraints: Vec<String>,
    pub downgrade_behavior: String,
    pub disqualifying_failures: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RemoteOracleLeaseV1 {
    pub schema_version: String,
    pub remote_oracle_lease_id: String,
    pub oracle_identity: String,
    pub allowed_artifact_families: Vec<String>,
    pub allowed_graph_or_slice_kinds: Vec<String>,
    pub exactness_class_ceiling: String,
    pub budget_ceiling: String,
    pub disclosure_ceiling: String,
    pub replay_obligation: String,
    pub lease_expiry: String,
    pub policy_owner_refs: Vec<String>,
}
