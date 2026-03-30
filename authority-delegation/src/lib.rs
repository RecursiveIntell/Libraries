//! Typed delegated-authority surface crate for post-v20 artifact families.
//!
//! The crate keeps its compatibility name, but it publishes typed authority
//! surfaces and bounded helper profiles rather than a general access-control runtime.
//!
//! ## Integration Points
//!
//! - **forge-pilot observe phase:** `governance_gate.rs` (`#[cfg(feature = "governance")]`) reads
//!   authority lease validity, delegation chain state, and break glass grant status from
//!   semantic-memory projections.
//! - **forge-pilot act phase:** capability class blast radius ceilings and authority lease
//!   scope constrain which effects the loop is permitted to execute.
//! - **verification-control:** authority chain validity and delegation revocation case types
//!   are affected; separation of duties policy violations trigger verification cases.
//! - **Stack Arena scenarios:** governed lane exercises governance observation of authority
//!   lease boundaries and delegation chain integrity.
//!
//! ## Artifact Families
//!
//! | Artifact | Schema Version | Owner |
//! |----------|---------------|-------|
//! | [`CapabilityClassV1`] | `CapabilityClassV1` | this crate |
//! | [`AuthorityLeaseV1`] | `AuthorityLeaseV1` | this crate |
//! | [`DelegationBundleV1`] | `DelegationBundleV1` | this crate |
//! | [`AuthorityChainV1`] | `AuthorityChainV1` | this crate |
//! | [`BreakGlassGrantV1`] | `BreakGlassGrantV1` | this crate |
//! | [`DelegationRevocationV1`] | `DelegationRevocationV1` | this crate |
//! | [`ActingOnBehalfReceiptV1`] | `ActingOnBehalfReceiptV1` | this crate |
//! | [`SeparationOfDutiesPolicyV1`] | `SeparationOfDutiesPolicyV1` | this crate |
//! | [`DualControlApprovalV1`] | `DualControlApprovalV1` | this crate |
//! | [`ConflictDisclosureV1`] | `ConflictDisclosureV1` | this crate |

pub mod capability;
pub mod emergency;
pub mod error;
pub mod profile_p3_roles;
pub mod sod;

pub use capability::*;
pub use emergency::*;
pub use error::*;
pub use profile_p3_roles::*;
pub use sod::*;
