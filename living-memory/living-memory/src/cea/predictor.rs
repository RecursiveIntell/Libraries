use crate::cea::graph::CausalGraph;
use crate::cea::instrumentation::EditOpSignature;
use crate::config::CeaConfig;
pub use cea_core::{CausalPrediction, CoverageSummary, RiskFlag};

/// Predict correctness from patch topology using the causal graph.
pub fn predict(
    signatures: &[EditOpSignature],
    graph: &CausalGraph,
    config: &CeaConfig,
) -> CausalPrediction {
    cea_core::predict(
        signatures,
        graph,
        config.risk_confidence_threshold,
        config.zero_shot_coverage_threshold,
    )
}
