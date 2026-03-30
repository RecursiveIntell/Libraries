//! V25 constitutional citation and obligation reference types shared across effect artifacts.
//!
//! Re-exports [`V25ConstitutionCitation`] from `stack-ids` and defines the
//! obligation reference grouping used by preflight reports and commit decisions.

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use stack_ids::V25ConstitutionCitation;

/// Categorized obligation references attached to effect artifacts for governance traceability.
///
/// Flattened into parent structs via `#[serde(flatten)]` to keep the wire
/// format flat while grouping obligation ref categories in code.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct V25ObligationRefs {
    #[serde(default)]
    pub required_obligation_refs: Vec<String>,
    #[serde(default)]
    pub blocking_obligation_refs: Vec<String>,
    #[serde(default)]
    pub monitoring_obligation_refs: Vec<String>,
    #[serde(default)]
    pub decision_basis_obligation_refs: Vec<String>,
}
