//! Typed continuity and incident surface crate for post-v20 artifact families.
//!
//! The crate keeps its compatibility name, but it publishes typed continuity
//! surfaces and bounded helper profiles rather than an incident-command runtime.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   incident case status, containment decision state, and SLO profile error budgets from
//!   semantic-memory projections.
//! - **forge-pilot act phase:** active incident cases and containment decisions gate whether
//!   the loop proceeds with normal execution or enters incident-aware mode.
//! - **verification-control:** incident case and recovery plan case types are affected;
//!   SLO breach events trigger recertification and postmortem verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of incident
//!   case lifecycle and SLO profile budget consumption.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`IncidentCaseV1`] | `IncidentCaseV1` | this crate |
//! | [`ContainmentDecisionV1`] | `ContainmentDecisionV1` | this crate |
//! | [`ForensicFreezeV1`] | `ForensicFreezeV1` | this crate |
//! | [`ContinuityExceptionV1`] | `ContinuityExceptionV1` | this crate |
//! | [`RecoveryPlanV1`] | `RecoveryPlanV1` | this crate |
//! | [`RecoveryReplaySliceV1`] | `RecoveryReplaySliceV1` | this crate |
//! | [`PostmortemBundleV1`] | `PostmortemBundleV1` | this crate |
//! | [`ResilienceExerciseV1`] | `ResilienceExerciseV1` | this crate |
//! | [`ServiceLevelProfileV1`] | `ServiceLevelProfileV1` | this crate |
//! | [`ErrorBudgetLedgerV1`] | `ErrorBudgetLedgerV1` | this crate |

pub mod error;
pub mod incident;
pub mod profile_p7_incident_routing;
pub mod recovery;
pub mod slo;
pub mod validation;
pub mod vocab;

pub use error::*;
pub use incident::*;
pub use profile_p7_incident_routing::*;
pub use recovery::*;
pub use slo::*;
pub use vocab::*;
