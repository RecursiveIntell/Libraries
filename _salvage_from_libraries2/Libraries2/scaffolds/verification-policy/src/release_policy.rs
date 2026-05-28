use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReleasePolicyProfileV1 {
    pub schema_version: String,
    pub release_policy_profile_id: String,
    pub required_assurance_sections: Vec<String>,
    pub required_monitor_classes: Vec<String>,
    pub block_on_open_obligations: bool,
}
