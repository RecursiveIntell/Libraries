use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct ToolEffectDispatchReceiptV1 {
    pub schema_version: String,
    pub tool_effect_dispatch_receipt_id: String,
    pub effect_commit_decision_id: String,
    pub provider_route: Vec<String>,
    pub dispatch_state: String,
    pub deadline_at_dispatch: String,
    pub cancellation_reason: String,
}
