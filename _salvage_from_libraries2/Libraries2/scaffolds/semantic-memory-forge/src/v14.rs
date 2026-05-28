//! Draft v14 artifact family scaffold.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct InterventionBundleV1 {
    pub schema_version: String,
    pub intervention_id: String,
    pub episode_id: String,
    pub unit_of_analysis: String,
    pub treatment_definition: String,
    pub baseline_treatment: String,
    pub start_condition: String,
    pub stop_condition: String,
    pub allowed_scope: String,
    pub valid_from: String,
    pub valid_to: Option<String>,
    pub recorded_at: String,
    pub related_claim_ids: Vec<String>,
    pub policy_refs: Vec<String>,
    pub approval_refs: Vec<String>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct OutcomeSchemaV1 {
    pub schema_version: String,
    pub outcome_schema_id: String,
    pub name: String,
    pub measurement_definition: String,
    pub time_window: String,
    pub aggregation_rule: String,
    pub degradation_behavior: String,
    pub exactness_requirement: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CohortContractV1 {
    pub schema_version: String,
    pub cohort_contract_id: String,
    pub unit_definition: String,
    pub inclusion_logic: Vec<String>,
    pub exclusion_logic: Vec<String>,
    pub scope_namespace: String,
    pub workload_class: String,
    pub environment_constraints: Vec<String>,
    pub replay_linkage: String,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize, schemars::JsonSchema)]
pub struct CounterfactualSliceV1 {
    pub schema_version: String,
    pub counterfactual_slice_id: String,
    pub intervention_id: String,
    pub baseline_treatment: String,
    pub cohort_contract_id: String,
    pub as_of_valid_time: String,
    pub as_of_recorded_time: String,
    pub replayable_data_slice: String,
    pub exactness_target: String,
    pub modeling_assumptions: Vec<String>,
    pub expected_failure_modes: Vec<String>,
}
