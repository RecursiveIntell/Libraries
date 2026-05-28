use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RolloutDecisionV1 {
    pub schema_version: String,
    pub rollout_decision_id: String,
    pub intervention_id: String,
    pub decision_class: String,
    pub allowed_blast_radius: String,
    pub observability_obligations: Vec<String>,
    pub rollback_trigger_conditions: Vec<String>,
    pub quarantine_trigger_conditions: Vec<String>,
    pub policy_basis: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RollbackDecisionV1 {
    pub schema_version: String,
    pub rollback_decision_id: String,
    pub triggering_evidence: Vec<String>,
    pub affected_surfaces: Vec<String>,
    pub experiment_case_id: String,
    pub counterfactual_slice_id: String,
    pub rollback_class: String,
    pub remaining_uncertainty: String,
    pub motivation: String,
}
