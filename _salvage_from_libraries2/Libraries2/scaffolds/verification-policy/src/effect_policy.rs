use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EffectPolicyProfileV1 {
    pub schema_version: String,
    pub effect_policy_profile_id: String,
    pub allowed_run_modes: Vec<String>,
    pub required_preflight_checks: Vec<String>,
    pub required_observation_classes: Vec<String>,
    pub requires_compensation_plan_for: Vec<String>,
}
