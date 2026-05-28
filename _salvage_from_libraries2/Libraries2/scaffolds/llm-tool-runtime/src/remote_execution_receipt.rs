use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, JsonSchema)]
pub struct RemoteExecutionReceiptV1 {
    pub schema_version: String,
    pub receipt_id: String,
    pub provider_route: String,
    pub remote_run_ref: String,
    pub attempt_family_id: String,
    pub trace_ctx: String,
    pub workload_class: String,
    pub budget_lineage: Vec<String>,
    pub replayability_class: String,
    pub degradation_markers: Vec<String>,
}
