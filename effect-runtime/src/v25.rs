use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

pub use stack_ids::V25ConstitutionCitation;

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
