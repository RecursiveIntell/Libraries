use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RoleDefinitionV1 {
    pub role: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub capabilities: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationEdgeV1 {
    pub from_role: String,
    pub to_role: String,
    pub capability: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalRuleV1 {
    pub action_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictClassRuleV1 {
    pub conflict_class: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub disallowed_roles: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct RoleCatalogV1 {
    pub schema_version: String,
    pub role_catalog_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub role_definitions: Vec<RoleDefinitionV1>,
    pub default_autonomy_ceiling: String,
    pub scope_rule: String,
    pub review_cycle_days: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationMatrixV1 {
    pub schema_version: String,
    pub delegation_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_edges: Vec<DelegationEdgeV1>,
    pub max_delegation_depth: i64,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_lease_classes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub forbidden_chain_patterns: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ApprovalMatrixV1 {
    pub schema_version: String,
    pub approval_matrix_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub approval_rules: Vec<ApprovalRuleV1>,
    pub default_quorum: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub requires_independent_review_for: Vec<String>,
    pub break_glass_post_hoc_review_hours: i64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictClassCatalogV1 {
    pub schema_version: String,
    pub conflict_class_catalog_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub conflict_classes: Vec<ConflictClassRuleV1>,
    pub default_recusal_behavior: String,
    pub override_path: String,
    pub disclosure_required: bool,
}
