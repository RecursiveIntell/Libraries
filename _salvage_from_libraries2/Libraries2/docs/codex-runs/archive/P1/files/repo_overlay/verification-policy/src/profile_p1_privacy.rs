use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

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
    pub default_retention_class: String,
    pub archive_restore_expectation: String,
    pub cross_border_transfer_default: String,
    pub default_redaction_rule_set_id: String,
    pub compaction_requires_receipt: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RedactionRuleSetV1 {
    pub schema_version: String,
    pub redaction_rule_set_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub target_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub field_actions: Vec<RedactionFieldActionV1>,
    pub reversibility_class: String,
    pub approval_requirement: String,
    pub default_disclosure_budget_class: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AccessPurposeMatrixV1 {
    pub schema_version: String,
    pub access_purpose_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub actor_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub purpose_rules: Vec<AccessPurposeRuleV1>,
    pub default_decision: String,
    pub elevation_path: String,
    pub audit_logging_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AuditExtractionPolicyV1 {
    pub schema_version: String,
    pub audit_extraction_policy_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    pub required_redaction_rule_set_id: String,
    pub disclosure_budget_class: String,
    pub export_package_format: String,
    pub expiry_hours: i64,
    pub evidence_preservation_required: bool,
}
