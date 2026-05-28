use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DelegationPolicyProfileV1 {
    pub schema_version: String,
    pub delegation_policy_profile_id: String,
    pub max_delegation_depth: i64,
    pub break_glass_requires_post_hoc_review: bool,
    pub forbidden_role_combinations: Vec<String>,
}
