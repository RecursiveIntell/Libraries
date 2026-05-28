use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct DelegationReviewCaseV1 {
    pub schema_version: String,
    pub delegation_review_case_id: String,
    pub authority_chain_id: String,
    pub separation_of_duties_policy_id: String,
    pub decision_state: String,
    pub generated_at: String,
}
