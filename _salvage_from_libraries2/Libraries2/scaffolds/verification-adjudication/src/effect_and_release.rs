use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EffectAdjudicationReceiptV1 {
    pub schema_version: String,
    pub effect_adjudication_receipt_id: String,
    pub effect_execution_receipt_id: String,
    pub observation_bundle_id: String,
    pub adjudicated_state: String,
    pub generated_at: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ReleaseRollbackDecisionV1 {
    pub schema_version: String,
    pub release_rollback_decision_id: String,
    pub release_readiness_decision_id: String,
    pub incident_case_id: String,
    pub rollback_required: bool,
    pub generated_at: String,
}
