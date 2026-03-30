use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{
    require_non_empty, require_non_empty_slice, AuthorityValidationError, AuthorityValidationResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct SeparationOfDutiesPolicyV1 {
    pub schema_version: String,
    pub separation_of_duties_policy_id: String,
    pub prohibited_role_combinations: Vec<String>,
    pub required_reviewer_independence: String,
    pub required_dual_control_classes: Vec<String>,
    pub emergency_exceptions: Vec<String>,
    pub violation_consequences: Vec<String>,
}

impl SeparationOfDutiesPolicyV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        separation_of_duties_policy_id: impl Into<String>,
        prohibited_role_combinations: Vec<String>,
        required_reviewer_independence: impl Into<String>,
        required_dual_control_classes: Vec<String>,
        emergency_exceptions: Vec<String>,
        violation_consequences: Vec<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "SeparationOfDutiesPolicyV1".to_string(),
            separation_of_duties_policy_id: separation_of_duties_policy_id.into(),
            prohibited_role_combinations,
            required_reviewer_independence: required_reviewer_independence.into(),
            required_dual_control_classes,
            emergency_exceptions,
            violation_consequences,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(
            &self.separation_of_duties_policy_id,
            "separation_of_duties_policy_id",
        )?;
        require_non_empty_slice(
            &self.prohibited_role_combinations,
            "prohibited_role_combinations",
        )?;
        require_non_empty(
            &self.required_reviewer_independence,
            "required_reviewer_independence",
        )?;
        require_non_empty_slice(
            &self.required_dual_control_classes,
            "required_dual_control_classes",
        )?;
        require_non_empty_slice(&self.violation_consequences, "violation_consequences")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DualControlApprovalV1 {
    pub schema_version: String,
    pub dual_control_approval_id: String,
    pub approver_refs: Vec<String>,
    pub approval_basis: String,
    pub independence_verified: bool,
    pub approved_at: String,
}

impl DualControlApprovalV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        dual_control_approval_id: impl Into<String>,
        approver_refs: Vec<String>,
        approval_basis: impl Into<String>,
        independence_verified: bool,
        approved_at: impl Into<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "DualControlApprovalV1".to_string(),
            dual_control_approval_id: dual_control_approval_id.into(),
            approver_refs,
            approval_basis: approval_basis.into(),
            independence_verified,
            approved_at: approved_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.dual_control_approval_id, "dual_control_approval_id")?;
        require_non_empty_slice(&self.approver_refs, "approver_refs")?;
        if self.approver_refs.len() < 2 {
            return Err(AuthorityValidationError::InvalidState("approver_refs"));
        }
        require_non_empty(&self.approval_basis, "approval_basis")?;
        require_non_empty(&self.approved_at, "approved_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ConflictDisclosureV1 {
    pub schema_version: String,
    pub conflict_disclosure_id: String,
    pub principal_ref: String,
    pub declared_conflict: String,
    pub mitigation: String,
    pub declared_at: String,
}

impl ConflictDisclosureV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        conflict_disclosure_id: impl Into<String>,
        principal_ref: impl Into<String>,
        declared_conflict: impl Into<String>,
        mitigation: impl Into<String>,
        declared_at: impl Into<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "ConflictDisclosureV1".to_string(),
            conflict_disclosure_id: conflict_disclosure_id.into(),
            principal_ref: principal_ref.into(),
            declared_conflict: declared_conflict.into(),
            mitigation: mitigation.into(),
            declared_at: declared_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.conflict_disclosure_id, "conflict_disclosure_id")?;
        require_non_empty(&self.principal_ref, "principal_ref")?;
        require_non_empty(&self.declared_conflict, "declared_conflict")?;
        require_non_empty(&self.mitigation, "mitigation")?;
        require_non_empty(&self.declared_at, "declared_at")?;
        Ok(())
    }
}
