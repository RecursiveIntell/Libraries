use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ContinuityReviewCaseV1 {
    pub schema_version: String,
    pub continuity_review_case_id: String,
    pub incident_case_id: String,
    pub continuity_exception_id: String,
    pub post_hoc_review_due: bool,
    pub final_state: String,
}
