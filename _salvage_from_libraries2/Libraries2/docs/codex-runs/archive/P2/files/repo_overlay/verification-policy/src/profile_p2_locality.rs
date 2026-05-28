use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ResidencyPolicyProfileV1 {
    pub schema_version: String,
    pub residency_policy_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_storage_regions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_execution_regions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_replay_regions: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_transfer_classes: Vec<String>,
    pub default_exception_path: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TenantBoundaryProfileV1 {
    pub schema_version: String,
    pub tenant_boundary_profile_id: String,
    pub tenant_key_kind: String,
    pub isolation_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_service_allowances: Vec<String>,
    pub cross_tenant_query_default: String,
    pub key_management_segregation: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossBoundaryTransferClassV1 {
    pub schema_version: String,
    pub cross_boundary_transfer_class_id: String,
    pub source_class: String,
    pub destination_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    pub required_attestation: String,
    pub required_disclosure_policy_class: String,
    pub downgrade_behavior: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct LocalityExceptionV1 {
    pub schema_version: String,
    pub locality_exception_id: String,
    pub residency_policy_profile_id: String,
    pub reason: String,
    pub scope: String,
    pub expires_at: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approved_by: Vec<String>,
    pub post_hoc_review_required: bool,
}
