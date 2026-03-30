use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const TENANT_BOUNDARY_PROFILE_V1_SCHEMA: &str = "TenantBoundaryProfileV1";
pub const CROSS_BOUNDARY_TRANSFER_CLASS_V1_SCHEMA: &str = "CrossBoundaryTransferClassV1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrossBoundaryDowngradeBehaviorV1 {
    SummaryOnlyOrBlock,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TenantKeyKindV1 {
    AccountId,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TenantIsolationClassV1 {
    HardLogicalIsolation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrossTenantQueryDefaultV1 {
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrossBoundaryAttestationRequirementV1 {
    SignedExportEnvelope,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DisclosurePolicyClassV1 {
    RedactedVendorExport,
}

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
    pub tenant_key_kind: TenantKeyKindV1,
    pub isolation_class: TenantIsolationClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub shared_service_allowances: Vec<String>,
    pub cross_tenant_query_default: CrossTenantQueryDefaultV1,
    pub key_management_segregation: String,
}

impl TenantBoundaryProfileV1 {
    /// Validates tenant-boundary metadata and required segregation guidance.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != TENANT_BOUNDARY_PROFILE_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.tenant_boundary_profile_id.trim().is_empty() {
            return Err("tenant_boundary_profile_id");
        }
        if self.key_management_segregation.trim().is_empty() {
            return Err("key_management_segregation");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct CrossBoundaryTransferClassV1 {
    pub schema_version: String,
    pub cross_boundary_transfer_class_id: String,
    pub source_class: String,
    pub destination_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    pub required_attestation: CrossBoundaryAttestationRequirementV1,
    pub required_disclosure_policy_class: DisclosurePolicyClassV1,
    pub downgrade_behavior: CrossBoundaryDowngradeBehaviorV1,
}

impl CrossBoundaryTransferClassV1 {
    /// Validates cross-boundary transfer metadata and required policy refs.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != CROSS_BOUNDARY_TRANSFER_CLASS_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.cross_boundary_transfer_class_id.trim().is_empty() {
            return Err("cross_boundary_transfer_class_id");
        }
        if self.source_class.trim().is_empty() {
            return Err("source_class");
        }
        if self.destination_class.trim().is_empty() {
            return Err("destination_class");
        }
        Ok(())
    }
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
