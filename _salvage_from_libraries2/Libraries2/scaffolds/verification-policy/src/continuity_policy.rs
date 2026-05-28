use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityPolicyProfileV1 {
    pub schema_version: String,
    pub continuity_policy_profile_id: String,
    pub required_forensic_freeze_surfaces: Vec<String>,
    pub continuity_exception_ttl_minutes: i64,
    pub requires_postmortem_for_severity: Vec<String>,
}
