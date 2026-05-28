use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentClassRuleV1 {
    pub incident_class: String,
    pub default_severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentRouteRuleV1 {
    pub incident_class: String,
    pub route: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityRuleV1 {
    pub condition: String,
    pub severity: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityResponseClockV1 {
    pub severity: String,
    pub minutes: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityPostmortemClockV1 {
    pub severity: String,
    pub hours: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct IncidentTaxonomyV1 {
    pub schema_version: String,
    pub incident_taxonomy_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub incident_classes: Vec<IncidentClassRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub default_routes: Vec<IncidentRouteRuleV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_artifact_families: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SeverityMatrixV1 {
    pub schema_version: String,
    pub severity_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub severity_rules: Vec<SeverityRuleV1>,
    pub customer_impact_rubric: String,
    pub internal_impact_rubric: String,
    pub override_rule: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PagerRouteProfileV1 {
    pub schema_version: String,
    pub pager_route_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub rotation_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub handoff_rules: Vec<String>,
    pub ack_timeout_minutes: i64,
    pub max_levels: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EscalationClockPolicyV1 {
    pub schema_version: String,
    pub escalation_clock_policy_id: String,
    pub severity_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub response_clock_minutes: Vec<SeverityResponseClockV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub postmortem_clock_hours: Vec<SeverityPostmortemClockV1>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub pause_rules: Vec<String>,
    pub exception_path: String,
}
