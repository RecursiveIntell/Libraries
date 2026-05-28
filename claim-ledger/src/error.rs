//! Error types for claim-ledger operations.

use thiserror::Error;

/// Errors returned by claim-ledger operations.
#[derive(Debug, Error)]
pub enum ClaimLedgerError {
    /// No matching claim found in the bundle.
    #[error("claim not found: {0}")]
    ClaimNotFound(String),

    /// No matching support judgment found.
    #[error("support judgment not found: {0}")]
    SupportJudgmentNotFound(String),

    /// No matching evidence bundle found.
    #[error("evidence bundle not found: {0}")]
    EvidenceBundleNotFound(String),

    /// No matching contradiction record found.
    #[error("contradiction not found: {0}")]
    ContradictionNotFound(String),

    /// Unsupported or invalid support admission state transition.
    #[error("invalid support state transition: {0}")]
    InvalidSupportTransition(String),

    /// Supported admissions require proof payload or explicit proof-debt waiver.
    #[error("supported admission requires proof payload or proof debt waiver")]
    MissingProofPayload,

    /// The ledger is corrupt or verification failed.
    #[error("ledger verification failed: {0}")]
    LedgerCorrupt(String),

    /// Serialization or deserialization failure.
    #[error("serialization error: {0}")]
    SerializationError(String),

    /// A required input reference was not provided.
    #[error("missing required input reference: {0}")]
    MissingInputRef(String),
}

impl ClaimLedgerError {
    /// Stable string discriminant for programmatic matching.
    pub fn kind(&self) -> &'static str {
        match self {
            Self::ClaimNotFound(_) => "claim_not_found",
            Self::SupportJudgmentNotFound(_) => "support_judgment_not_found",
            Self::EvidenceBundleNotFound(_) => "evidence_bundle_not_found",
            Self::ContradictionNotFound(_) => "contradiction_not_found",
            Self::InvalidSupportTransition(_) => "invalid_support_transition",
            Self::MissingProofPayload => "missing_proof_payload",
            Self::LedgerCorrupt(_) => "ledger_corrupt",
            Self::SerializationError(_) => "serialization_error",
            Self::MissingInputRef(_) => "missing_input_ref",
        }
    }
}