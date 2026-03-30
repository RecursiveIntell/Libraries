//! Scaffold owner crate for post-v20 final-pass surfaces.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   effect intent publication status, preflight report dispositions, and commit decision state
//!   from semantic-memory projections.
//! - **forge-pilot act phase:** effect commit decisions (authorized vs. denied) and execution
//!   permit validity directly gate whether the loop executes planned effects.
//! - **verification-control:** effect preflight report and commit decision case types are
//!   affected; compensation plan execution feeds into post-effect verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of effect
//!   lifecycle from intent through execution receipt and observation bundle closure.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`EffectWindowV1`] | `EffectWindowV1` | this crate |
//! | [`EffectIntentV1`] | `EffectIntentV1` | this crate |
//! | [`EffectPreflightReportV1`] | `EffectPreflightReportV1` | this crate |
//! | [`EffectCommitDecisionV1`] | `EffectCommitDecisionV1` | this crate |
//! | [`EffectExecutionReceiptV1`] | `EffectExecutionReceiptV1` | this crate |
//! | [`EffectObservationBundleV1`] | `EffectObservationBundleV1` | this crate |
//! | [`ExternalEffectLedgerEntryV1`] | `ExternalEffectLedgerEntryV1` | this crate |
//! | [`CompensationPlanV1`] | `CompensationPlanV1` | this crate |
//! | [`CompensationExecutionReceiptV1`] | `CompensationExecutionReceiptV1` | this crate |

pub mod compensation;
pub mod effect;
pub mod error;
pub mod observation;
pub mod v25;
pub mod vocab;

pub use compensation::*;
pub use effect::*;
pub use error::*;
pub use observation::*;
pub use v25::*;
pub use vocab::*;
