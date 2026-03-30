use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RequirementControlMappingEntryV1 {
    pub requirement: String,
    pub control: String,
    pub evidence_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceClassRequirementV1 {
    pub evidence_class: String,
    pub minimum_family: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RegulatoryRegimeProfileV1 {
    pub schema_version: String,
    pub regulatory_regime_profile_id: String,
    pub regime_name: String,
    pub regime_version: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_products: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mandatory_control_families: Vec<String>,
    pub audit_cycle_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RequirementControlMapV1 {
    pub schema_version: String,
    pub requirement_control_map_id: String,
    pub regulatory_regime_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mappings: Vec<RequirementControlMappingEntryV1>,
    pub gap_classification_default: String,
    pub owner_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct EvidenceCollectionPlanV1 {
    pub schema_version: String,
    pub evidence_collection_plan_id: String,
    pub regulatory_regime_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_evidence_classes: Vec<EvidenceClassRequirementV1>,
    pub collection_cadence: String,
    pub retention_class: String,
    pub owner_ref: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RecertificationScheduleV1 {
    pub schema_version: String,
    pub recertification_schedule_id: String,
    pub regulatory_regime_profile_id: String,
    pub review_interval_days: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trigger_classes: Vec<String>,
    pub grace_window_days: i64,
    pub blocked_state_on_expiry: String,
}
