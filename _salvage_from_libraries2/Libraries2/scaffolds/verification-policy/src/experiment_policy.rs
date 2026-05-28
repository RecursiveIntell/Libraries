use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct RefuterSuiteV1 {
    pub schema_version: String,
    pub refuter_suite_id: String,
    pub treatment_definition_check: bool,
    pub negative_control_option: bool,
    pub placebo_option: bool,
    pub dummy_outcome_option: bool,
    pub subset_stability_option: bool,
    pub alternative_comparison_option: bool,
    pub admissibility_notes: Vec<String>,
    pub cost_metadata: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct ExperimentBudgetV1 {
    pub schema_version: String,
    pub experiment_budget_id: String,
    pub budget_class: String,
    pub max_exactness: String,
    pub refuter_allowance: String,
    pub oracle_allowance: String,
    pub replay_allowance: String,
    pub human_review_allowance: String,
    pub exhaustion_behavior: String,
}
