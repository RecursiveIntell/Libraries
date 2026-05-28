use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MonitorDefinitionV1 {
    pub monitor: String,
    pub threshold: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HazardLibraryV1 {
    pub schema_version: String,
    pub hazard_library_id: String,
    pub sector: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hazard_families: Vec<String>,
    pub scoring_model_ref: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub linked_operating_envelopes: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct HazardScenarioV1 {
    pub schema_version: String,
    pub hazard_scenario_id: String,
    pub hazard_library_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_conditions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub affected_surfaces: Vec<String>,
    pub severity_baseline: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_monitor_refs: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MonitorCatalogV1 {
    pub schema_version: String,
    pub monitor_catalog_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub monitor_definitions: Vec<MonitorDefinitionV1>,
    pub evaluation_cadence: String,
    pub false_positive_budget: String,
    pub owner_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct MitigationPlaybookV1 {
    pub schema_version: String,
    pub mitigation_playbook_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub hazard_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub containment_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub recovery_steps: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub success_criteria: Vec<String>,
}
