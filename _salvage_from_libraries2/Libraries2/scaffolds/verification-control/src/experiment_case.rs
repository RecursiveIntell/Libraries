use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ExperimentCaseV1 {
    pub schema_version: String,
    pub experiment_case_id: String,
    pub intervention_id: String,
    pub outcome_schema_id: String,
    pub cohort_contract_id: String,
    pub comparability_matrix_id: String,
    pub refuter_suite_id: String,
    pub budget_class: String,
    pub risk_class: String,
    pub lifecycle_state: String,
    pub final_disposition: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ComparabilityMatrixV1 {
    pub schema_version: String,
    pub comparability_matrix_id: String,
    pub workload_comparability: String,
    pub environment_comparability: String,
    pub config_comparability: String,
    pub time_window_comparability: String,
    pub retry_replay_comparability: String,
    pub missingness_markers: Vec<String>,
    pub admissibility_judgment: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RefuterResultV1 {
    pub schema_version: String,
    pub refuter_result_id: String,
    pub refuter_kind: String,
    pub counterfactual_slice_id: String,
    pub result_summary: String,
    pub state: String,
    pub replay_linkage: String,
    pub promotion_impact: String,
}
