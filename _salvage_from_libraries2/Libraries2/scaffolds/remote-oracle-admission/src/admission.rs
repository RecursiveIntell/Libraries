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
