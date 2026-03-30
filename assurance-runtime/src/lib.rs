//! Typed deployability and certification surface crate for post-v20 artifact families.
//!
//! The crate keeps its compatibility name, but it publishes typed assurance
//! surfaces and bounded helper profiles rather than a standalone release runtime.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   assurance case readiness and control mapping coverage from semantic-memory projections.
//! - **forge-pilot act phase:** deployment profile publication status and release readiness
//!   decisions gate whether execution proceeds or is blocked.
//! - **verification-control:** certification bundle and recertification trigger case types
//!   are affected; release readiness decisions feed into verification case adjudication.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of assurance
//!   case completeness and control mapping coverage.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`AssuranceCaseV1`] | `AssuranceCaseV1` | this crate |
//! | [`ControlMappingV1`] | `ControlMappingV1` | this crate |
//! | [`DeploymentProfileV1`] | `DeploymentProfileV1` | this crate |
//! | [`OperatingEnvelopeV1`] | `OperatingEnvelopeV1` | this crate |
//! | [`HazardRegisterV1`] | `HazardRegisterV1` | this crate |
//! | [`CertificationBundleV1`] | `CertificationBundleV1` | this crate |
//! | [`FieldMonitoringPlanV1`] | `FieldMonitoringPlanV1` | this crate |
//! | [`RecertificationTriggerV1`] | `RecertificationTriggerV1` | this crate |
//! | [`ResidualRiskAcceptanceV1`] | `ResidualRiskAcceptanceV1` | this crate |
//! | [`ReleaseReadinessDecisionV1`] | `ReleaseReadinessDecisionV1` | this crate |

pub mod assurance;
pub mod certification;
pub mod error;
pub mod profile;
pub mod profile_p4_regulated;
pub mod profile_p5_hazard;

pub use assurance::*;
pub use certification::*;
pub use error::*;
pub use profile::*;
pub use profile_p4_regulated::*;
pub use profile_p5_hazard::*;
