use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{
    require_non_empty, require_non_empty_slice, AuthorityValidationError, AuthorityValidationResult,
};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct BreakGlassGrantV1 {
    pub schema_version: String,
    pub break_glass_grant_id: String,
    pub authority_lease_id: String,
    pub emergency_reason: String,
    pub approved_by: Vec<String>,
    pub expires_at: String,
    pub post_hoc_review_required: bool,
}

impl BreakGlassGrantV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        break_glass_grant_id: impl Into<String>,
        authority_lease_id: impl Into<String>,
        emergency_reason: impl Into<String>,
        approved_by: Vec<String>,
        expires_at: impl Into<String>,
        post_hoc_review_required: bool,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "BreakGlassGrantV1".to_string(),
            break_glass_grant_id: break_glass_grant_id.into(),
            authority_lease_id: authority_lease_id.into(),
            emergency_reason: emergency_reason.into(),
            approved_by,
            expires_at: expires_at.into(),
            post_hoc_review_required,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.break_glass_grant_id, "break_glass_grant_id")?;
        require_non_empty(&self.authority_lease_id, "authority_lease_id")?;
        require_non_empty(&self.emergency_reason, "emergency_reason")?;
        require_non_empty_slice(&self.approved_by, "approved_by")?;
        require_non_empty(&self.expires_at, "expires_at")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationRevocationV1 {
    pub schema_version: String,
    pub delegation_revocation_id: String,
    pub authority_lease_id: String,
    pub revoked_by: String,
    pub revocation_reason: String,
    pub effective_at: String,
    pub supersedes_authority_chain_id: String,
}

impl DelegationRevocationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        delegation_revocation_id: impl Into<String>,
        authority_lease_id: impl Into<String>,
        revoked_by: impl Into<String>,
        revocation_reason: impl Into<String>,
        effective_at: impl Into<String>,
        supersedes_authority_chain_id: impl Into<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "DelegationRevocationV1".to_string(),
            delegation_revocation_id: delegation_revocation_id.into(),
            authority_lease_id: authority_lease_id.into(),
            revoked_by: revoked_by.into(),
            revocation_reason: revocation_reason.into(),
            effective_at: effective_at.into(),
            supersedes_authority_chain_id: supersedes_authority_chain_id.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.delegation_revocation_id, "delegation_revocation_id")?;
        require_non_empty(&self.authority_lease_id, "authority_lease_id")?;
        require_non_empty(&self.revoked_by, "revoked_by")?;
        require_non_empty(&self.revocation_reason, "revocation_reason")?;
        require_non_empty(&self.effective_at, "effective_at")?;
        require_non_empty(
            &self.supersedes_authority_chain_id,
            "supersedes_authority_chain_id",
        )?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ActingOnBehalfReceiptV1 {
    pub schema_version: String,
    pub acting_on_behalf_receipt_id: String,
    pub authority_chain_id: String,
    pub effect_execution_receipt_id: String,
    pub delegated_action: String,
    pub within_lease: bool,
    pub generated_at: String,
}

impl ActingOnBehalfReceiptV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        acting_on_behalf_receipt_id: impl Into<String>,
        authority_chain_id: impl Into<String>,
        effect_execution_receipt_id: impl Into<String>,
        delegated_action: impl Into<String>,
        within_lease: bool,
        generated_at: impl Into<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "ActingOnBehalfReceiptV1".to_string(),
            acting_on_behalf_receipt_id: acting_on_behalf_receipt_id.into(),
            authority_chain_id: authority_chain_id.into(),
            effect_execution_receipt_id: effect_execution_receipt_id.into(),
            delegated_action: delegated_action.into(),
            within_lease,
            generated_at: generated_at.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(
            &self.acting_on_behalf_receipt_id,
            "acting_on_behalf_receipt_id",
        )?;
        require_non_empty(&self.authority_chain_id, "authority_chain_id")?;
        require_non_empty(
            &self.effect_execution_receipt_id,
            "effect_execution_receipt_id",
        )?;
        require_non_empty(&self.delegated_action, "delegated_action")?;
        require_non_empty(&self.generated_at, "generated_at")?;
        Ok(())
    }
}
