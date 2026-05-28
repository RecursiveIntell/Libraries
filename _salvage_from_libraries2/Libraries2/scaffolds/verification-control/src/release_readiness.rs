use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseGateCaseV1 {
    pub schema_version: String,
    pub release_gate_case_id: String,
    pub deployment_profile_id: String,
    pub assurance_case_id: String,
    pub release_readiness_decision_id: String,
    pub final_state: String,
}
