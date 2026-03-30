use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub const PRIVACY_RETENTION_PROFILE_V1_SCHEMA: &str = "PrivacyRetentionProfileV1";
pub const REDACTION_RULE_SET_V1_SCHEMA: &str = "RedactionRuleSetV1";
pub const ACCESS_PURPOSE_MATRIX_V1_SCHEMA: &str = "AccessPurposeMatrixV1";
pub const AUDIT_EXTRACTION_POLICY_V1_SCHEMA: &str = "AuditExtractionPolicyV1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum PrivacyRetentionClassV1 {
    #[serde(rename = "customer_sensitive_365d")]
    CustomerSensitive365d,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ArchiveRestoreExpectationV1 {
    RestoreBeforeAuditExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CrossBorderTransferDefaultV1 {
    Blocked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedactionReversibilityClassV1 {
    OperatorReversibleOnly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum RedactionApprovalRequirementV1 {
    #[serde(rename = "privacy-review")]
    PrivacyReview,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum DisclosureBudgetClassV1 {
    LimitedSupportExport,
    RegulatorExport,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AccessDefaultDecisionV1 {
    Deny,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuditExportPackageFormatV1 {
    AttestedBundle,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RedactionFieldActionV1 {
    pub field: String,
    pub action: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccessPurposeRuleV1 {
    pub actor_class: String,
    pub purpose: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct PrivacyRetentionProfileV1 {
    pub schema_version: String,
    pub privacy_retention_profile_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub applicable_namespaces: Vec<String>,
    pub default_retention_class: PrivacyRetentionClassV1,
    pub archive_restore_expectation: ArchiveRestoreExpectationV1,
    pub cross_border_transfer_default: CrossBorderTransferDefaultV1,
    pub default_redaction_rule_set_id: String,
    pub compaction_requires_receipt: bool,
}

impl PrivacyRetentionProfileV1 {
    /// Validates privacy-retention metadata and required default policy links.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != PRIVACY_RETENTION_PROFILE_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.privacy_retention_profile_id.trim().is_empty() {
            return Err("privacy_retention_profile_id");
        }
        if self.default_redaction_rule_set_id.trim().is_empty() {
            return Err("default_redaction_rule_set_id");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RedactionRuleSetV1 {
    pub schema_version: String,
    pub redaction_rule_set_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_actions: Vec<RedactionFieldActionV1>,
    pub reversibility_class: RedactionReversibilityClassV1,
    pub approval_requirement: RedactionApprovalRequirementV1,
    pub default_disclosure_budget_class: DisclosureBudgetClassV1,
}

impl RedactionRuleSetV1 {
    /// Validates redaction-rule-set metadata and required policy references.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != REDACTION_RULE_SET_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.redaction_rule_set_id.trim().is_empty() {
            return Err("redaction_rule_set_id");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccessPurposeMatrixV1 {
    pub schema_version: String,
    pub access_purpose_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actor_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub purpose_rules: Vec<AccessPurposeRuleV1>,
    pub default_decision: AccessDefaultDecisionV1,
    pub elevation_path: String,
    pub audit_logging_required: bool,
}

impl AccessPurposeMatrixV1 {
    /// Validates access-purpose matrix metadata and its review path.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != ACCESS_PURPOSE_MATRIX_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.access_purpose_matrix_id.trim().is_empty() {
            return Err("access_purpose_matrix_id");
        }
        if self.elevation_path.trim().is_empty() {
            return Err("elevation_path");
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuditExtractionPolicyV1 {
    pub schema_version: String,
    pub audit_extraction_policy_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    pub required_redaction_rule_set_id: String,
    pub disclosure_budget_class: DisclosureBudgetClassV1,
    pub export_package_format: AuditExportPackageFormatV1,
    pub expiry_hours: i64,
    pub evidence_preservation_required: bool,
}

impl AuditExtractionPolicyV1 {
    /// Validates audit-extraction metadata and required redaction linkage.
    pub fn validate(&self) -> Result<(), &'static str> {
        if self.schema_version != AUDIT_EXTRACTION_POLICY_V1_SCHEMA {
            return Err("schema_version");
        }
        if self.audit_extraction_policy_id.trim().is_empty() {
            return Err("audit_extraction_policy_id");
        }
        if self.required_redaction_rule_set_id.trim().is_empty() {
            return Err("required_redaction_rule_set_id");
        }
        if self.expiry_hours <= 0 {
            return Err("expiry_hours");
        }
        Ok(())
    }
}
