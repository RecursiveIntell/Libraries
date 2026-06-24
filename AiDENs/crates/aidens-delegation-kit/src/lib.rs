//! Delegation status surface.
//!
//! Phase 02 removed the local duplicate attestation and settlement artifact
//! types from `aidens-contracts`. The prior helper policy in this crate was
//! coupled to those local shadow fields, so it is intentionally not preserved
//! as a compatibility adapter.

pub mod federation;

use thiserror::Error;

#[derive(Debug, Error, PartialEq, Eq)]
pub enum DelegationError {
    #[error("delegation helper semantics are quarantined pending canonical owner wiring")]
    CanonicalOwnerRequired,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct DelegationKitStatus {
    pub enabled: bool,
    pub note: String,
}

impl Default for DelegationKitStatus {
    fn default() -> Self {
        Self {
            enabled: false,
            note: "delegation helpers are quarantined after canonical attestation and settlement ownership collapse".into(),
        }
    }
}
