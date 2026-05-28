use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EffectReviewCaseV1 {
    pub schema_version: String,
    pub effect_review_case_id: String,
    pub effect_intent_id: String,
    pub effect_preflight_report_id: String,
    pub required_policy_refs: Vec<String>,
    pub decision_basis: String,
    pub final_state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct EffectBlockReceiptV1 {
    pub schema_version: String,
    pub effect_block_receipt_id: String,
    pub effect_review_case_id: String,
    pub block_reason: String,
    pub generated_at: String,
}
