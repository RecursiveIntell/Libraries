use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::runtime::novelty::AttemptPhase;

pub struct Stabilizer {
    inner: stabilizer_core::Stabilizer,
}

/// Per-attempt overrides produced by the stabilizer.
#[derive(Debug, Clone)]
pub struct AttemptOverrides {
    pub phase: AttemptPhase,
    pub delta_amplitude: f64,
    pub force_family: Option<String>,
    pub force_minimal_diff: bool,
    pub weight_factor: f64,
}

impl Stabilizer {
    pub fn new(config: &ForgeConfig) -> Self {
        let delta_policy = stabilizer_core::DeltaPolicy::new(
            config.novelty.delta_amp_default,
            config.novelty.delta_amp_stabilize1,
            config.novelty.delta_amp_stabilize2,
            config.novelty.delta_amp_clamp,
        );
        Self {
            inner: stabilizer_core::Stabilizer::new(
                delta_policy,
                stabilizer_core::StabilizerConfig {
                    force_family: config.stabilization.stabilize1_force_family.clone(),
                    force_minimal_diff: config.stabilization.stabilize2_force_minimal_diff,
                    stabilize_weight_factor: config.stabilization.increase_stabilize_weight_factor,
                },
            ),
        }
    }

    /// Get the next attempt's overrides.
    ///
    /// Panics in debug builds if called out of order.
    /// Returns error in release builds.
    pub fn next_attempt(&mut self) -> ForgeResult<AttemptOverrides> {
        let next = self.inner.next_attempt()?;
        Ok(AttemptOverrides {
            phase: next.phase,
            delta_amplitude: next.delta_amplitude,
            force_family: next.force_family,
            force_minimal_diff: next.force_minimal_diff,
            weight_factor: next.weight_factor,
        })
    }

    /// Check if more attempts remain.
    pub fn has_next(&self) -> bool {
        self.inner.has_next()
    }

    /// Reset the stabilizer for a new run.
    pub fn reset(&mut self) {
        self.inner.reset();
    }

    /// Get the current phase index (for logging).
    pub fn current_index(&self) -> usize {
        self.inner.current_index()
    }
}
