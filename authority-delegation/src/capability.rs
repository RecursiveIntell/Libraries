use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use crate::error::{
    require_non_empty, require_non_empty_slice, AuthorityValidationError, AuthorityValidationResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityBlastRadiusCeilingV1 {
    SingleTenant,
    SingleService,
    Regional,
    Global,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityChainValidityStateV1 {
    Valid,
    Expired,
    Revoked,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CapabilityDisclosureCeilingV1 {
    Internal,
    RedactedStructuredOnly,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct CapabilityClassV1 {
    pub schema_version: String,
    pub capability_class_id: String,
    pub governed_effect_families: Vec<String>,
    pub blast_radius_ceiling: CapabilityBlastRadiusCeilingV1,
    pub disclosure_ceiling: CapabilityDisclosureCeilingV1,
    pub required_review_class: String,
    pub dual_control_required: bool,
    pub max_delegation_depth: i64,
}

impl CapabilityClassV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        capability_class_id: impl Into<String>,
        governed_effect_families: Vec<String>,
        blast_radius_ceiling: CapabilityBlastRadiusCeilingV1,
        disclosure_ceiling: CapabilityDisclosureCeilingV1,
        required_review_class: impl Into<String>,
        dual_control_required: bool,
        max_delegation_depth: i64,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "CapabilityClassV1".to_string(),
            capability_class_id: capability_class_id.into(),
            governed_effect_families,
            blast_radius_ceiling,
            disclosure_ceiling,
            required_review_class: required_review_class.into(),
            dual_control_required,
            max_delegation_depth,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.capability_class_id, "capability_class_id")?;
        require_non_empty_slice(&self.governed_effect_families, "governed_effect_families")?;
        require_non_empty(&self.required_review_class, "required_review_class")?;
        if self.max_delegation_depth < 0 {
            return Err(AuthorityValidationError::InvalidState(
                "max_delegation_depth",
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuthorityLeaseV1 {
    pub schema_version: String,
    pub authority_lease_id: String,
    pub capability_class_id: String,
    pub grantor: String,
    pub grantee: String,
    pub scope: String,
    pub time_window: String,
    pub allowed_effect_families: Vec<String>,
    pub hard_ceilings: Vec<String>,
    pub revocation_conditions: Vec<String>,
    pub required_receipts: Vec<String>,
}

impl AuthorityLeaseV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        authority_lease_id: impl Into<String>,
        capability_class_id: impl Into<String>,
        grantor: impl Into<String>,
        grantee: impl Into<String>,
        scope: impl Into<String>,
        time_window: impl Into<String>,
        allowed_effect_families: Vec<String>,
        hard_ceilings: Vec<String>,
        revocation_conditions: Vec<String>,
        required_receipts: Vec<String>,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "AuthorityLeaseV1".to_string(),
            authority_lease_id: authority_lease_id.into(),
            capability_class_id: capability_class_id.into(),
            grantor: grantor.into(),
            grantee: grantee.into(),
            scope: scope.into(),
            time_window: time_window.into(),
            allowed_effect_families,
            hard_ceilings,
            revocation_conditions,
            required_receipts,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.authority_lease_id, "authority_lease_id")?;
        require_non_empty(&self.capability_class_id, "capability_class_id")?;
        require_non_empty(&self.grantor, "grantor")?;
        require_non_empty(&self.grantee, "grantee")?;
        require_non_empty(&self.scope, "scope")?;
        require_non_empty(&self.time_window, "time_window")?;
        require_non_empty_slice(&self.allowed_effect_families, "allowed_effect_families")?;
        require_non_empty_slice(&self.required_receipts, "required_receipts")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct DelegationBundleV1 {
    pub schema_version: String,
    pub delegation_bundle_id: String,
    pub authority_lease_id: String,
    pub delegated_rights: Vec<String>,
    pub reduced_ceilings: Vec<String>,
    pub replay_requirements: Vec<String>,
    pub further_delegation_permitted: bool,
}

impl DelegationBundleV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        delegation_bundle_id: impl Into<String>,
        authority_lease_id: impl Into<String>,
        delegated_rights: Vec<String>,
        reduced_ceilings: Vec<String>,
        replay_requirements: Vec<String>,
        further_delegation_permitted: bool,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "DelegationBundleV1".to_string(),
            delegation_bundle_id: delegation_bundle_id.into(),
            authority_lease_id: authority_lease_id.into(),
            delegated_rights,
            reduced_ceilings,
            replay_requirements,
            further_delegation_permitted,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.delegation_bundle_id, "delegation_bundle_id")?;
        require_non_empty(&self.authority_lease_id, "authority_lease_id")?;
        require_non_empty_slice(&self.delegated_rights, "delegated_rights")?;
        require_non_empty_slice(&self.replay_requirements, "replay_requirements")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct AuthorityChainV1 {
    pub schema_version: String,
    pub authority_chain_id: String,
    pub local_principal: String,
    pub delegated_principals: Vec<String>,
    pub external_principals: Vec<String>,
    pub approval_refs: Vec<String>,
    pub current_validity_state: AuthorityChainValidityStateV1,
}

impl AuthorityChainV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        authority_chain_id: impl Into<String>,
        local_principal: impl Into<String>,
        delegated_principals: Vec<String>,
        external_principals: Vec<String>,
        approval_refs: Vec<String>,
        current_validity_state: AuthorityChainValidityStateV1,
    ) -> Result<Self, AuthorityValidationError> {
        let value = Self {
            schema_version: "AuthorityChainV1".to_string(),
            authority_chain_id: authority_chain_id.into(),
            local_principal: local_principal.into(),
            delegated_principals,
            external_principals,
            approval_refs,
            current_validity_state,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AuthorityValidationResult {
        require_non_empty(&self.authority_chain_id, "authority_chain_id")?;
        require_non_empty(&self.local_principal, "local_principal")?;
        require_non_empty_slice(&self.approval_refs, "approval_refs")?;
        Ok(())
    }
}
