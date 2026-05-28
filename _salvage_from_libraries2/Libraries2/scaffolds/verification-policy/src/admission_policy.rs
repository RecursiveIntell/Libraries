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
pub struct DisclosurePolicyV1 {
    pub schema_version: String,
    pub disclosure_policy_id: String,
    pub allowed_consumers: Vec<String>,
    pub redaction_rules: Vec<String>,
    pub replay_visibility: String,
    pub retention_window: String,
    pub downgrade_behavior: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DisclosureBudgetV1 {
    pub schema_version: String,
    pub disclosure_budget_id: String,
    pub allowed_reveal_class: String,
    pub current_spend: String,
    pub redaction_ceiling: String,
    pub escalation_path: String,
}
