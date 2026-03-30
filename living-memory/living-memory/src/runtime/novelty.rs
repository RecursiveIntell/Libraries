use std::collections::BTreeSet;

use crate::runtime::patch::types::*;

pub type StrategyTag = stabilizer_core::StrategyTag;
pub use stabilizer_core::AttemptPhase;

#[derive(Debug, Clone)]
pub struct DeltaPolicy(stabilizer_core::DeltaPolicy);

/// Extract strategy tags from a StructuredPatch's topology.
pub fn extract_strategy_tags(patch: &StructuredPatch) -> Vec<StrategyTag> {
    stabilizer_core::extract_strategy_tags(patch)
}

/// Compute novelty score from strategy tags vs. recent traces.
pub fn compute_tag_novelty(current_tags: &[String], recent_tags_union: &BTreeSet<String>) -> f64 {
    stabilizer_core::compute_tag_novelty(current_tags, recent_tags_union)
}

impl DeltaPolicy {
    pub fn from_config(config: &crate::config::NoveltyConfig) -> Self {
        Self(stabilizer_core::DeltaPolicy::new(
            config.delta_amp_default,
            config.delta_amp_stabilize1,
            config.delta_amp_stabilize2,
            config.delta_amp_clamp,
        ))
    }

    pub fn amplitude_for_phase(&self, phase: AttemptPhase) -> f64 {
        self.0.amplitude_for_phase(phase)
    }
}

/// Determine approach family from strategy tags (for MAP-Elites cell key).
pub fn determine_approach_family(tags: &[String]) -> &'static str {
    stabilizer_core::determine_approach_family(tags)
}
