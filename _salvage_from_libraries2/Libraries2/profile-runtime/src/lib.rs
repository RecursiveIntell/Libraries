//! Canonical v25 composition lane.
//!
//! `profile-runtime` owns the effective constitution surface for concrete,
//! profile-sensitive decision contexts. Family-specific profile content remains
//! in existing owner crates; this crate composes those overlays into one
//! replayable answer with typed conflicts and explicit exceptions.

pub mod adapters;
pub mod applicability;
pub mod compose;
pub mod constitution;
pub mod exception;
pub mod profile_set;
pub mod rules;

pub use adapters::*;
pub use applicability::*;
pub use compose::*;
pub use constitution::*;
pub use exception::*;
pub use profile_set::*;
pub use rules::*;
