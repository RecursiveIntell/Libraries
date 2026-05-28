use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct DecisionTraceV1 {
    pub schema_version: String,
    pub decision_trace_id: String,
    pub triggering_artifact_refs: Vec<String>,
    pub intervention_id: String,
    pub counterfactual_slice_id: String,
    pub refuters_satisfied: Vec<String>,
    pub refuters_failed: Vec<String>,
    pub refuters_skipped: Vec<String>,
    pub exactness_spend: String,
    pub policy_basis: Vec<String>,
    pub selected_decision: String,
    pub rollback_prerequisites: Vec<String>,
    pub cheap_next_checks: Vec<String>,
}
