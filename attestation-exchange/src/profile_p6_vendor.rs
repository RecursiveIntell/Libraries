use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt::{Display, Formatter, Result as FmtResult};

use crate::error::{
    require_non_empty, require_non_empty_slice, AttestationValidationError, AttestationValidationResult,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VendorTranslationModeV1 {
    AttestedAndLossinessDeclared,
}

impl Display for VendorTranslationModeV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::AttestedAndLossinessDeclared => "attested_and_lossiness_declared",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VendorRotationChannelV1 {
    PublishedKeyset,
}

impl Display for VendorRotationChannelV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::PublishedKeyset => "published_keyset",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VendorRevocationChannelV1 {
    SignedRevocationFeed,
}

impl Display for VendorRevocationChannelV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::SignedRevocationFeed => "signed_revocation_feed",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VendorReplayImpactV1 {
    HistoricalReplayRecheckRequired,
}

impl Display for VendorReplayImpactV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::HistoricalReplayRecheckRequired => "historical_replay_recheck_required",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum VendorAdmissionImpactV1 {
    PromotionBlockedUntilReAdmission,
}

impl Display for VendorAdmissionImpactV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::PromotionBlockedUntilReAdmission => "promotion_blocked_until_re_admission",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorCertificationAdapterV1 {
    pub schema_version: String,
    pub vendor_certification_adapter_id: String,
    pub vendor_name: String,
    pub product_surface: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub covered_artifact_families: Vec<String>,
    pub translation_mode: VendorTranslationModeV1,
    pub support_window: String,
}

impl VendorCertificationAdapterV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vendor_certification_adapter_id: impl Into<String>,
        vendor_name: impl Into<String>,
        product_surface: impl Into<String>,
        covered_artifact_families: Vec<String>,
        translation_mode: VendorTranslationModeV1,
        support_window: impl Into<String>,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: "VendorCertificationAdapterV1".to_string(),
            vendor_certification_adapter_id: vendor_certification_adapter_id.into(),
            vendor_name: vendor_name.into(),
            product_surface: product_surface.into(),
            covered_artifact_families,
            translation_mode,
            support_window: support_window.into(),
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            &self.vendor_certification_adapter_id,
            "vendor_certification_adapter_id",
        )?;
        require_non_empty(&self.vendor_name, "vendor_name")?;
        require_non_empty(&self.product_surface, "product_surface")?;
        require_non_empty_slice(&self.covered_artifact_families, "covered_artifact_families")?;
        require_non_empty(&self.support_window, "support_window")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorEvidenceTranslationV1 {
    pub schema_version: String,
    pub vendor_evidence_translation_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub source_shapes: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub canonical_targets: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub lossy_fields: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub required_caveats: Vec<String>,
}

impl VendorEvidenceTranslationV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vendor_evidence_translation_id: impl Into<String>,
        vendor_certification_adapter_id: impl Into<String>,
        source_shapes: Vec<String>,
        canonical_targets: Vec<String>,
        lossy_fields: Vec<String>,
        required_caveats: Vec<String>,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: "VendorEvidenceTranslationV1".to_string(),
            vendor_evidence_translation_id: vendor_evidence_translation_id.into(),
            vendor_certification_adapter_id: vendor_certification_adapter_id.into(),
            source_shapes,
            canonical_targets,
            lossy_fields,
            required_caveats,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            &self.vendor_evidence_translation_id,
            "vendor_evidence_translation_id",
        )?;
        require_non_empty(
            &self.vendor_certification_adapter_id,
            "vendor_certification_adapter_id",
        )?;
        require_non_empty_slice(&self.source_shapes, "source_shapes")?;
        require_non_empty_slice(&self.canonical_targets, "canonical_targets")?;
        require_non_empty_slice(&self.required_caveats, "required_caveats")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorTrustRootBindingV1 {
    pub schema_version: String,
    pub vendor_trust_root_binding_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trust_root_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub signer_classes: Vec<String>,
    pub rotation_channel: VendorRotationChannelV1,
    pub revocation_channel: VendorRevocationChannelV1,
}

impl VendorTrustRootBindingV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vendor_trust_root_binding_id: impl Into<String>,
        vendor_certification_adapter_id: impl Into<String>,
        trust_root_refs: Vec<String>,
        signer_classes: Vec<String>,
        rotation_channel: VendorRotationChannelV1,
        revocation_channel: VendorRevocationChannelV1,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: "VendorTrustRootBindingV1".to_string(),
            vendor_trust_root_binding_id: vendor_trust_root_binding_id.into(),
            vendor_certification_adapter_id: vendor_certification_adapter_id.into(),
            trust_root_refs,
            signer_classes,
            rotation_channel,
            revocation_channel,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            &self.vendor_trust_root_binding_id,
            "vendor_trust_root_binding_id",
        )?;
        require_non_empty(
            &self.vendor_certification_adapter_id,
            "vendor_certification_adapter_id",
        )?;
        require_non_empty_slice(&self.trust_root_refs, "trust_root_refs")?;
        require_non_empty_slice(&self.signer_classes, "signer_classes")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct VendorRevocationHandlingV1 {
    pub schema_version: String,
    pub vendor_revocation_handling_id: String,
    pub vendor_certification_adapter_id: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_inputs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub local_invalidation_actions: Vec<String>,
    pub replay_impact: VendorReplayImpactV1,
    pub admission_impact: VendorAdmissionImpactV1,
}

impl VendorRevocationHandlingV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        vendor_revocation_handling_id: impl Into<String>,
        vendor_certification_adapter_id: impl Into<String>,
        revocation_inputs: Vec<String>,
        local_invalidation_actions: Vec<String>,
        replay_impact: VendorReplayImpactV1,
        admission_impact: VendorAdmissionImpactV1,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: "VendorRevocationHandlingV1".to_string(),
            vendor_revocation_handling_id: vendor_revocation_handling_id.into(),
            vendor_certification_adapter_id: vendor_certification_adapter_id.into(),
            revocation_inputs,
            local_invalidation_actions,
            replay_impact,
            admission_impact,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            &self.vendor_revocation_handling_id,
            "vendor_revocation_handling_id",
        )?;
        require_non_empty(
            &self.vendor_certification_adapter_id,
            "vendor_certification_adapter_id",
        )?;
        require_non_empty_slice(&self.revocation_inputs, "revocation_inputs")?;
        require_non_empty_slice(
            &self.local_invalidation_actions,
            "local_invalidation_actions",
        )?;
        Ok(())
    }
}
