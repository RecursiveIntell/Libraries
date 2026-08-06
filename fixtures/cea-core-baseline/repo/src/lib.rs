//! Causal edit attribution for structured code patches.
//!
//! This crate converts a [`typed_patch::StructuredPatch`] plus a
//! [`check_runner::CheckResult`] into weighted cause/effect attribution triples,
//! learns edge weights in a causal graph, and predicts future edit risk without
//! requiring heavyweight ML dependencies.
//!
//! Key properties:
//! - Deterministic attribution and run hashing.
//! - Multi-cause proportional attribution instead of nearest-neighbor collapse.
//! - Beta-distribution confidence tracking for edge reliability.
//! - Compact graph persistence with versioned binary snapshots.

mod attribution;
mod calibration;
mod error;
mod graph;
mod predict;
mod scope;
mod types;

pub use attribution::{
    attribute_effects, attribute_effects_with_config, attribution_score,
    attribution_score_with_confidence, build_edit_op_signature, compute_run_hash, edit_op_node_id,
    effect_node_id, global_pass_effect_signature, AttributedRunResult, AttributionConfig,
    AttributionTriple,
};
pub use calibration::{
    advisory_confidence, conservative_reliability, effective_sample_size, posterior_mean,
    posterior_variance, sample_factor, sample_sufficiency_factor, ConfidenceEnvelope,
    MIN_SAMPLES_PER_SIGNATURE,
};
pub use error::CeaCoreError;
pub use graph::{CausalEdge, CausalGraph, CausalNode, CoverageSummary, EdgeStats};
pub use predict::{predict, predict_with_config, PredictionConfig};
pub use types::{
    AnchorKind, CausalPrediction, EditOpKind, EditOpSignature, FileIndex, OpIndex, RiskFlag,
    ScopeTag,
};

#[cfg(test)]
mod tests;
