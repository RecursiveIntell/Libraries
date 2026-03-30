//! Typed attestation exchange surface crate for vendor and trust-root artifacts.
//!
//! The crate keeps the compatibility name, but it publishes typed exchange
//! contracts and bounded profile helpers rather than a transport runtime.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   attestation envelope validity and trust root set state from semantic-memory projections.
//! - **forge-pilot act phase:** transparency receipt admissibility judgments gate whether
//!   artifact provenance is considered trustworthy for execution.
//! - **verification-control:** attestation envelope and trust root set case types are affected;
//!   transparency receipts feed into provenance verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of attestation
//!   envelope integrity and trust root validity.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`AttestationEnvelopeV1`] | `attestation_envelope_v1` | this crate |
//! | [`TrustRootSetV1`] | `trust_root_set_v1` | this crate |
//! | [`TransparencyReceiptV1`] | `transparency_receipt_v1` | this crate |

pub mod error;
pub mod profile_p6_vendor;

pub use error::*;
pub use profile_p6_vendor::*;

use crate::error::{require_non_empty, require_non_empty_slice};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use stack_ids::{
    ArtifactAdmissionPolicyId, AttestationEnvelopeId, ContentDigest, DisclosurePolicyId,
    TransparencyReceiptId, TrustRootSetId,
};
use std::fmt::{Display, Formatter, Result as FmtResult};

pub const ATTESTATION_ENVELOPE_V1_SCHEMA: &str = "attestation_envelope_v1";
pub const TRUST_ROOT_SET_V1_SCHEMA: &str = "trust_root_set_v1";
pub const TRANSPARENCY_RECEIPT_V1_SCHEMA: &str = "transparency_receipt_v1";

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum AttestationReplayabilityClassV1 {
    Replayable,
    ReplayableWithTicket,
}

impl Display for AttestationReplayabilityClassV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::Replayable => "replayable",
            Self::ReplayableWithTicket => "replayable_with_ticket",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustRootExpirationPolicyV1 {
    #[serde(rename = "365d")]
    Days365,
    #[serde(rename = "rotate_every_90_days")]
    RotateEvery90Days,
}

impl Display for TrustRootExpirationPolicyV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::Days365 => "365d",
            Self::RotateEvery90Days => "rotate_every_90_days",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TrustRootRotationPolicyV1 {
    PublishedKeyset,
    Overlap14DaysAndEmitSupersession,
}

impl Display for TrustRootRotationPolicyV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::PublishedKeyset => "published_keyset",
            Self::Overlap14DaysAndEmitSupersession => "overlap_14_days_and_emit_supersession",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TransparencyAdmissibilityJudgmentV1 {
    Admitted,
    LoggedAndAdmissible,
    AdmissibleWithNotedCaveat,
}

impl Display for TransparencyAdmissibilityJudgmentV1 {
    fn fmt(&self, f: &mut Formatter<'_>) -> FmtResult {
        let value = match self {
            Self::Admitted => "admitted",
            Self::LoggedAndAdmissible => "logged_and_admissible",
            Self::AdmissibleWithNotedCaveat => "admissible_with_noted_caveat",
        };
        f.write_str(value)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct AttestationEnvelopeV1 {
    pub schema_version: String,
    pub attestation_envelope_id: AttestationEnvelopeId,
    pub artifact_family: String,
    pub artifact_version: String,
    pub content_digest: ContentDigest,
    pub schema_identity: String,
    pub signer_identity: String,
    pub signing_time: String,
    pub trust_root_set_id: TrustRootSetId,
    pub provenance_summary: String,
    pub disclosure_policy_id: DisclosurePolicyId,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub artifact_admission_policy_id: Option<ArtifactAdmissionPolicyId>,
    pub replayability_class: AttestationReplayabilityClassV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_refs: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub supersession_refs: Vec<String>,
}

impl AttestationEnvelopeV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        attestation_envelope_id: AttestationEnvelopeId,
        artifact_family: impl Into<String>,
        artifact_version: impl Into<String>,
        content_digest: ContentDigest,
        schema_identity: impl Into<String>,
        signer_identity: impl Into<String>,
        signing_time: impl Into<String>,
        trust_root_set_id: TrustRootSetId,
        provenance_summary: impl Into<String>,
        disclosure_policy_id: DisclosurePolicyId,
        artifact_admission_policy_id: Option<ArtifactAdmissionPolicyId>,
        replayability_class: AttestationReplayabilityClassV1,
        revocation_refs: Vec<String>,
        supersession_refs: Vec<String>,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: ATTESTATION_ENVELOPE_V1_SCHEMA.to_string(),
            attestation_envelope_id,
            artifact_family: artifact_family.into(),
            artifact_version: artifact_version.into(),
            content_digest,
            schema_identity: schema_identity.into(),
            signer_identity: signer_identity.into(),
            signing_time: signing_time.into(),
            trust_root_set_id,
            provenance_summary: provenance_summary.into(),
            disclosure_policy_id,
            artifact_admission_policy_id,
            replayability_class,
            revocation_refs,
            supersession_refs,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            self.attestation_envelope_id.as_str(),
            "attestation_envelope_id",
        )?;
        require_non_empty(&self.artifact_family, "artifact_family")?;
        require_non_empty(&self.artifact_version, "artifact_version")?;
        require_non_empty(&self.content_digest.0, "content_digest")?;
        require_non_empty(&self.schema_identity, "schema_identity")?;
        require_non_empty(&self.signer_identity, "signer_identity")?;
        require_non_empty(&self.signing_time, "signing_time")?;
        require_non_empty(self.trust_root_set_id.as_str(), "trust_root_set_id")?;
        require_non_empty(&self.provenance_summary, "provenance_summary")?;
        require_non_empty(self.disclosure_policy_id.as_str(), "disclosure_policy_id")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TrustRootSetV1 {
    pub schema_version: String,
    pub trust_root_set_id: TrustRootSetId,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub trust_root_identities: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_signer_classes: Vec<String>,
    pub expiration_policy: TrustRootExpirationPolicyV1,
    pub rotation_policy: TrustRootRotationPolicyV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub allowed_artifact_families: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub revocation_sources: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub policy_owner_refs: Vec<String>,
}

impl TrustRootSetV1 {
    #[allow(clippy::too_many_arguments)]
    pub fn new(
        trust_root_set_id: TrustRootSetId,
        trust_root_identities: Vec<String>,
        allowed_signer_classes: Vec<String>,
        expiration_policy: TrustRootExpirationPolicyV1,
        rotation_policy: TrustRootRotationPolicyV1,
        allowed_artifact_families: Vec<String>,
        revocation_sources: Vec<String>,
        policy_owner_refs: Vec<String>,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: TRUST_ROOT_SET_V1_SCHEMA.to_string(),
            trust_root_set_id,
            trust_root_identities,
            allowed_signer_classes,
            expiration_policy,
            rotation_policy,
            allowed_artifact_families,
            revocation_sources,
            policy_owner_refs,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(self.trust_root_set_id.as_str(), "trust_root_set_id")?;
        require_non_empty_slice(&self.trust_root_identities, "trust_root_identities")?;
        require_non_empty_slice(&self.allowed_signer_classes, "allowed_signer_classes")?;
        require_non_empty_slice(&self.allowed_artifact_families, "allowed_artifact_families")?;
        require_non_empty_slice(&self.revocation_sources, "revocation_sources")?;
        require_non_empty_slice(&self.policy_owner_refs, "policy_owner_refs")?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct TransparencyReceiptV1 {
    pub schema_version: String,
    pub transparency_receipt_id: TransparencyReceiptId,
    pub attestation_envelope_id: AttestationEnvelopeId,
    pub registry_identity: String,
    pub inclusion_material: String,
    pub recorded_time: String,
    pub admissibility_judgment: TransparencyAdmissibilityJudgmentV1,
}

impl TransparencyReceiptV1 {
    pub fn new(
        transparency_receipt_id: TransparencyReceiptId,
        attestation_envelope_id: AttestationEnvelopeId,
        registry_identity: impl Into<String>,
        inclusion_material: impl Into<String>,
        recorded_time: impl Into<String>,
        admissibility_judgment: TransparencyAdmissibilityJudgmentV1,
    ) -> Result<Self, AttestationValidationError> {
        let value = Self {
            schema_version: TRANSPARENCY_RECEIPT_V1_SCHEMA.to_string(),
            transparency_receipt_id,
            attestation_envelope_id,
            registry_identity: registry_identity.into(),
            inclusion_material: inclusion_material.into(),
            recorded_time: recorded_time.into(),
            admissibility_judgment,
        };
        value.validate()?;
        Ok(value)
    }

    pub fn validate(&self) -> AttestationValidationResult {
        require_non_empty(
            self.transparency_receipt_id.as_str(),
            "transparency_receipt_id",
        )?;
        require_non_empty(
            self.attestation_envelope_id.as_str(),
            "attestation_envelope_id",
        )?;
        require_non_empty(&self.registry_identity, "registry_identity")?;
        require_non_empty(&self.inclusion_material, "inclusion_material")?;
        require_non_empty(&self.recorded_time, "recorded_time")?;
        Ok(())
    }
}
