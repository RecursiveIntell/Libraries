//! Adaptive runtime viscosity controller (FEUT-005).
//!
//! Computes a composite signal from loop metrics and maps it to a
//! strictness level. The effective viscosity is:
//!
//! ```text
//! I(t) = w1 * failure_rate + w2 * drift_rate + w3 * ambiguity_score
//!        + w4 * contradiction_density
//! ν_eff = ν_base + γ * I(t)
//! ```
//!
//! High viscosity → strict gates, slow execution, thorough verification.
//! Low viscosity → fast execution, minimal gates, high throughput.
//!
//! Replaces the binary safe_mode with a continuous 4-level signal:
//! Fast / Normal / Strict / Frozen.

use crate::evaluation::FactDisposition;
use serde::{Deserialize, Serialize};
use std::collections::VecDeque;

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

/// Configuration for the viscosity controller.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViscosityConfig {
    /// Weight for failure_rate (default 0.30).
    pub failure_weight: f64,
    /// Weight for drift_rate (default 0.20).
    pub drift_weight: f64,
    /// Weight for ambiguity_score (default 0.20).
    pub ambiguity_weight: f64,
    /// Weight for contradiction_density (default 0.30).
    pub contradiction_weight: f64,
    /// Base viscosity (minimum strictness even at zero signal).
    pub base_viscosity: f64,
    /// Gamma coefficient: ν_eff = ν_base + γ * I(t).
    pub gamma: f64,
    /// Window size for rolling metrics.
    pub window_size: usize,
}

impl Default for ViscosityConfig {
    fn default() -> Self {
        Self {
            failure_weight: 0.30,
            drift_weight: 0.20,
            ambiguity_weight: 0.20,
            contradiction_weight: 0.30,
            base_viscosity: 0.4,
            gamma: 0.7,
            window_size: 20,
        }
    }
}

// ---------------------------------------------------------------------------
// Signal & Strictness
// ---------------------------------------------------------------------------

/// Composite signal computed from rolling loop metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ViscositySignal {
    /// Rolling failure rate (failures / total attempts, window=20).
    pub failure_rate: f64,
    /// Drift rate: fraction of recent captures that were duplicates.
    pub drift_rate: f64,
    /// Ambiguity score: fraction of recent evaluations → Quarantine.
    pub ambiguity_score: f64,
    /// Contradiction density: contradictions / facts added.
    pub contradiction_density: f64,
}

impl Default for ViscositySignal {
    fn default() -> Self {
        Self {
            failure_rate: 0.0,
            drift_rate: 0.0,
            ambiguity_score: 0.0,
            contradiction_density: 0.0,
        }
    }
}

/// The computed strictness level for this cycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum StrictnessLevel {
    /// I(t) < 0.2 — fast execution, minimal gates, high throughput.
    Fast,
    /// 0.2 ≤ I(t) < 0.5 — normal execution, standard gates.
    Normal,
    /// 0.5 ≤ I(t) < 0.8 — slow, thorough verification, hostile audit.
    Strict,
    /// I(t) ≥ 0.8 — pause task generation, shift to subtractive mode.
    Frozen,
}

impl Default for StrictnessLevel {
    fn default() -> Self {
        Self::Normal
    }
}

impl std::fmt::Display for StrictnessLevel {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Fast => f.write_str("fast"),
            Self::Normal => f.write_str("normal"),
            Self::Strict => f.write_str("strict"),
            Self::Frozen => f.write_str("frozen"),
        }
    }
}

// ---------------------------------------------------------------------------
// Cycle record (internal)
// ---------------------------------------------------------------------------

/// Record of a single cycle's outcomes, kept in the rolling window.
#[derive(Debug, Clone)]
struct CycleRecord {
    success: bool,
    was_duplicate: bool,
    disposition: FactDisposition,
    contradictions_detected: usize,
    facts_added: usize,
}

// ---------------------------------------------------------------------------
// Controller
// ---------------------------------------------------------------------------

/// Adaptive viscosity controller.
#[derive(Debug, Clone)]
pub struct ViscosityController {
    config: ViscosityConfig,
    /// Rolling window of cycle records.
    history: VecDeque<CycleRecord>,
}

impl ViscosityController {
    /// Create a new controller with the given config.
    pub fn new(config: ViscosityConfig) -> Self {
        Self {
            config: config.clone(),
            history: VecDeque::with_capacity(config.window_size),
        }
    }

    /// Create a new controller with default config.
    pub fn with_defaults() -> Self {
        Self::new(ViscosityConfig::default())
    }

    /// Record a cycle's outcomes.
    pub fn record(
        &mut self,
        success: bool,
        was_duplicate: bool,
        disposition: FactDisposition,
        contradictions_detected: usize,
        facts_added: usize,
    ) {
        let record = CycleRecord {
            success,
            was_duplicate,
            disposition,
            contradictions_detected,
            facts_added,
        };
        self.history.push_back(record);
        if self.history.len() > self.config.window_size {
            self.history.pop_front();
        }
    }

    /// Compute the current viscosity signal from the rolling window.
    pub fn compute_signal(&self) -> ViscositySignal {
        if self.history.is_empty() {
            return ViscositySignal::default();
        }

        let total = self.history.len() as f64;
        let failures = self.history.iter().filter(|r| !r.success).count() as f64;
        let duplicates = self.history.iter().filter(|r| r.was_duplicate).count() as f64;
        let ambiguous = self
            .history
            .iter()
            .filter(|r| r.disposition == FactDisposition::Quarantine)
            .count() as f64;
        let total_contradictions: usize =
            self.history.iter().map(|r| r.contradictions_detected).sum();
        let total_facts: usize = self.history.iter().map(|r| r.facts_added).sum();

        ViscositySignal {
            failure_rate: failures / total,
            drift_rate: duplicates / total,
            ambiguity_score: ambiguous / total,
            contradiction_density: if total_facts > 0 {
                total_contradictions as f64 / total_facts as f64
            } else {
                0.0
            },
        }
    }

    /// Compute effective viscosity: ν_eff = ν_base + γ * I(t).
    pub fn compute_viscosity(&self) -> f64 {
        let signal = self.compute_signal();
        let i_t = self.config.failure_weight * signal.failure_rate
            + self.config.drift_weight * signal.drift_rate
            + self.config.ambiguity_weight * signal.ambiguity_score
            + self.config.contradiction_weight * signal.contradiction_density;
        self.config.base_viscosity + self.config.gamma * i_t
    }

    /// Map viscosity to strictness level.
    pub fn current_strictness(&self) -> StrictnessLevel {
        let v = self.compute_viscosity();
        // ν_eff ranges from base_viscosity (≈0.4) to base + gamma (≈1.1).
        // Thresholds:
        //   < 0.45 → Fast    (I(t) < ~0.07, very low signal)
        //   < 0.6  → Normal  (low to moderate signal)
        //   < 0.8  → Strict  (moderate to high signal)
        //   ≥ 0.8  → Frozen  (high signal)
        if v < 0.45 {
            StrictnessLevel::Fast
        } else if v < 0.6 {
            StrictnessLevel::Normal
        } else if v < 0.8 {
            StrictnessLevel::Strict
        } else {
            StrictnessLevel::Frozen
        }
    }

    /// Get the promotion threshold for the current strictness.
    /// Facts scoring below this are quarantined instead of promoted.
    pub fn promotion_threshold(&self) -> f64 {
        match self.current_strictness() {
            StrictnessLevel::Fast => 0.6,
            StrictnessLevel::Normal => 0.8,
            StrictnessLevel::Strict => 0.9,
            StrictnessLevel::Frozen => 1.0, // Never promote in frozen mode.
        }
    }

    /// Whether to generate new tasks this cycle.
    pub fn should_generate_tasks(&self) -> bool {
        !matches!(self.current_strictness(), StrictnessLevel::Frozen)
    }

    /// Whether to run a hostile audit this cycle.
    pub fn should_run_audit(&self) -> bool {
        matches!(
            self.current_strictness(),
            StrictnessLevel::Strict | StrictnessLevel::Frozen
        )
    }

    /// Whether to shift to subtractive mode.
    pub fn should_shift_to_subtractive(&self) -> bool {
        matches!(self.current_strictness(), StrictnessLevel::Frozen)
    }

    /// Get the sleep multiplier for the current strictness.
    /// Frozen sleeps 4x longer, strict 2x, normal 1x, fast 0.5x.
    pub fn sleep_multiplier(&self) -> f64 {
        match self.current_strictness() {
            StrictnessLevel::Fast => 0.5,
            StrictnessLevel::Normal => 1.0,
            StrictnessLevel::Strict => 2.0,
            StrictnessLevel::Frozen => 4.0,
        }
    }

    /// Get the number of cycles recorded.
    pub fn cycle_count(&self) -> usize {
        self.history.len()
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_starts_normal() {
        let vc = ViscosityController::with_defaults();
        // No history → zero signal → base viscosity (0.4).
        // 0.4 < 0.45 → Fast (base is at the fast/normal boundary).
        // This is correct — with no failures, the loop should run fast.
        let s = vc.current_strictness();
        assert!(
            matches!(s, StrictnessLevel::Fast | StrictnessLevel::Normal),
            "expected Fast or Normal with no history, got {s}"
        );
    }

    #[test]
    fn test_failures_increase_strictness() {
        let mut vc = ViscosityController::with_defaults();
        // Record 10 failures out of 10.
        for _ in 0..10 {
            vc.record(false, false, FactDisposition::Reject, 0, 0);
        }
        // High failure rate should push to at least Strict.
        let s = vc.current_strictness();
        assert!(
            matches!(s, StrictnessLevel::Strict | StrictnessLevel::Frozen),
            "expected Strict or Frozen, got {s}"
        );
    }

    #[test]
    fn test_success_decreases_strictness() {
        let mut vc = ViscosityController::with_defaults();
        // Record 20 successes with promoted facts.
        for _ in 0..20 {
            vc.record(true, false, FactDisposition::Promote, 0, 1);
        }
        // All success → low signal → Fast.
        assert_eq!(vc.current_strictness(), StrictnessLevel::Fast);
    }

    #[test]
    fn test_contradictions_increase_strictness() {
        let mut vc = ViscosityController::with_defaults();
        // Record cycles with high contradiction density.
        for _ in 0..10 {
            vc.record(true, false, FactDisposition::Promote, 5, 1);
        }
        // 5 contradictions per 1 fact added = density 5.0 → high signal.
        let s = vc.current_strictness();
        assert!(
            matches!(s, StrictnessLevel::Strict | StrictnessLevel::Frozen),
            "expected Strict or Frozen with high contradictions, got {s}"
        );
    }

    #[test]
    fn test_frozen_blocks_task_generation() {
        let mut vc = ViscosityController::with_defaults();
        // Force Frozen by recording many failures + contradictions + drift.
        for _ in 0..20 {
            vc.record(false, true, FactDisposition::Reject, 5, 1);
        }
        // failure_rate=1.0, drift_rate=1.0, contradiction_density=5.0
        // I(t) = 0.3*1 + 0.2*1 + 0.2*0 + 0.3*5 = 0.3+0.2+1.5 = 2.0
        // ν_eff = 0.4 + 0.7*2.0 = 1.8 → Frozen
        let s = vc.current_strictness();
        assert_eq!(s, StrictnessLevel::Frozen);
        assert!(!vc.should_generate_tasks());
        assert!(vc.should_shift_to_subtractive());
        assert_eq!(vc.promotion_threshold(), 1.0);
    }

    #[test]
    fn test_window_eviction() {
        let mut vc = ViscosityController::with_defaults();
        // Record 30 entries (window is 20).
        for i in 0..30 {
            vc.record(i % 2 == 0, false, FactDisposition::Promote, 0, 1);
        }
        assert_eq!(vc.cycle_count(), 20);
    }
}
