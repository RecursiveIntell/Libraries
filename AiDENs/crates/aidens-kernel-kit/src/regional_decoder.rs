//! P11 — Regional Decoder Kernel types.
//!
//! Bounded execution region definitions, residual envelopes, syndrome
//! envelopes, and convergence reports. These types enable iterative runtime
//! convergence with stop rules and oscillation/degradation detection.

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Bounded execution region definition. Defines a subgraph that can be
/// iterated independently and checked for convergence.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionContractV1 {
    /// Unique region identifier.
    pub region_id: String,
    /// Content digest of the region's compiled subgraph.
    pub region_digest_id: String,
    /// Node IDs within this region.
    pub node_ids: Vec<String>,
    /// Hyperedge IDs within this region.
    pub hyperedge_ids: Vec<String>,
    /// Constraint IDs applicable to this region.
    pub constraint_ids: Vec<String>,
    /// Whether this region is a bounded default unit of work.
    pub bounded_default_unit_of_work: bool,
    /// Maximum iterations before forced stop.
    pub max_iterations: u32,
    /// Convergence tolerance (change below this = converged).
    pub convergence_tolerance: f64,
}

/// Expected-vs-observed mismatch envelope. Captures the residual
/// between what the kernel expected and what it observed.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResidualEnvelopeV1 {
    /// Region this residual applies to.
    pub region_id: String,
    /// Expected values (node_id -> expected output).
    pub expected: HashMap<String, f64>,
    /// Observed values (node_id -> actual output).
    pub observed: HashMap<String, f64>,
    /// Computed residuals (node_id -> |expected - observed|).
    pub residuals: HashMap<String, f64>,
    /// Max residual across all nodes.
    pub max_residual: f64,
    /// Whether the residual is within tolerance.
    pub within_tolerance: bool,
}

/// Violated constraint bundle. Captures which constraints
/// were violated during execution and why.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyndromeEnvelopeV1 {
    /// Region this syndrome was detected in.
    pub region_id: String,
    /// Constraint IDs that were violated.
    pub violated_constraint_ids: Vec<String>,
    /// Human-readable descriptions of each violation.
    pub violation_descriptions: Vec<String>,
    /// Whether the syndrome is recoverable (retryable) or terminal.
    pub recoverable: bool,
}

/// Iterative runtime convergence evidence. Reports whether
/// a region's iterative execution converged, oscillated, or degraded.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegionConvergenceReportV1 {
    /// Region that was iterated.
    pub region_id: String,
    /// Number of iterations executed.
    pub iterations: u32,
    /// Final convergence state.
    pub convergence_state: ConvergenceState,
    /// Residual history (iteration -> max_residual).
    pub residual_history: Vec<f64>,
    /// Stop reason.
    pub stop_reason: ConvergenceStopReason,
    /// Time elapsed in milliseconds.
    pub elapsed_ms: u64,
}

/// Convergence state after iterative execution.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergenceState {
    /// Residuals decreased below tolerance.
    Converged,
    /// Residuals oscillated without converging.
    Oscillating,
    /// Residuals increased (diverged).
    Diverging,
    /// Hit max iterations before converging.
    MaxIterationsReached,
    /// Detected a constraint violation (syndrome).
    SyndromeDetected,
}

/// Why the iteration stopped.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConvergenceStopReason {
    /// Converged within tolerance.
    Converged,
    /// Max iterations reached.
    MaxIterations,
    /// Detected oscillation.
    OscillationDetected,
    /// Detected divergence.
    DivergenceDetected,
    /// Syndrome (constraint violation) detected.
    Syndrome,
    /// Budget exhausted.
    BudgetExhausted,
}

impl RegionConvergenceReportV1 {
    /// Whether the region converged successfully.
    pub fn is_converged(&self) -> bool {
        matches!(self.convergence_state, ConvergenceState::Converged)
    }

    /// Whether the region exhibited a problematic state.
    pub fn is_problematic(&self) -> bool {
        matches!(
            self.convergence_state,
            ConvergenceState::Oscillating
                | ConvergenceState::Diverging
                | ConvergenceState::SyndromeDetected
        )
    }
}

/// Detect convergence from a residual history.
pub fn classify_convergence(
    residual_history: &[f64],
    tolerance: f64,
    max_iterations: u32,
) -> (ConvergenceState, ConvergenceStopReason) {
    if residual_history.is_empty() {
        return (
            ConvergenceState::MaxIterationsReached,
            ConvergenceStopReason::MaxIterations,
        );
    }

    let last = *residual_history.last().unwrap();

    if last < tolerance {
        return (
            ConvergenceState::Converged,
            ConvergenceStopReason::Converged,
        );
    }

    if residual_history.len() >= max_iterations as usize {
        // Check for oscillation (last 3 values alternate)
        if residual_history.len() >= 4 {
            let n = residual_history.len();
            let a = residual_history[n - 4];
            let b = residual_history[n - 3];
            let c = residual_history[n - 2];
            let d = residual_history[n - 1];
            if (a > b && b < c && c > d) || (a < b && b > c && c < d) {
                return (
                    ConvergenceState::Oscillating,
                    ConvergenceStopReason::OscillationDetected,
                );
            }
        }

        // Check for divergence (residuals increasing)
        if residual_history.len() >= 3 {
            let n = residual_history.len();
            if residual_history[n - 3] < residual_history[n - 2]
                && residual_history[n - 2] < residual_history[n - 1]
            {
                return (
                    ConvergenceState::Diverging,
                    ConvergenceStopReason::DivergenceDetected,
                );
            }
        }

        return (
            ConvergenceState::MaxIterationsReached,
            ConvergenceStopReason::MaxIterations,
        );
    }

    (
        ConvergenceState::MaxIterationsReached,
        ConvergenceStopReason::MaxIterations,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn converged_state_is_converged() {
        let report = RegionConvergenceReportV1 {
            region_id: "r1".into(),
            iterations: 5,
            convergence_state: ConvergenceState::Converged,
            residual_history: vec![1.0, 0.5, 0.1, 0.01, 0.001],
            stop_reason: ConvergenceStopReason::Converged,
            elapsed_ms: 42,
        };
        assert!(report.is_converged());
        assert!(!report.is_problematic());
    }

    #[test]
    fn oscillating_state_is_problematic() {
        let report = RegionConvergenceReportV1 {
            region_id: "r1".into(),
            iterations: 10,
            convergence_state: ConvergenceState::Oscillating,
            residual_history: vec![0.5, 0.1, 0.5, 0.1, 0.5],
            stop_reason: ConvergenceStopReason::OscillationDetected,
            elapsed_ms: 100,
        };
        assert!(!report.is_converged());
        assert!(report.is_problematic());
    }

    #[test]
    fn classify_convergence_detects_converged() {
        let history = vec![1.0, 0.5, 0.1, 0.001];
        let (state, reason) = classify_convergence(&history, 0.01, 100);
        assert_eq!(state, ConvergenceState::Converged);
        assert_eq!(reason, ConvergenceStopReason::Converged);
    }

    #[test]
    fn classify_convergence_detects_oscillation() {
        let history = vec![0.5, 0.1, 0.5, 0.1, 0.5];
        let (state, reason) = classify_convergence(&history, 0.01, 4);
        assert_eq!(state, ConvergenceState::Oscillating);
        assert_eq!(reason, ConvergenceStopReason::OscillationDetected);
    }

    #[test]
    fn classify_convergence_detects_divergence() {
        let history = vec![0.1, 0.2, 0.5, 0.8];
        let (state, reason) = classify_convergence(&history, 0.01, 3);
        assert_eq!(state, ConvergenceState::Diverging);
        assert_eq!(reason, ConvergenceStopReason::DivergenceDetected);
    }

    #[test]
    fn classify_convergence_detects_max_iterations() {
        let history = vec![0.5, 0.4, 0.3, 0.25, 0.2];
        let (state, reason) = classify_convergence(&history, 0.01, 4);
        assert_eq!(state, ConvergenceState::MaxIterationsReached);
        assert_eq!(reason, ConvergenceStopReason::MaxIterations);
    }

    #[test]
    fn region_contract_serializes() {
        let contract = RegionContractV1 {
            region_id: "region-01".into(),
            region_digest_id: "digest-01".into(),
            node_ids: vec!["node-a".into(), "node-b".into()],
            hyperedge_ids: vec!["edge-01".into()],
            constraint_ids: vec!["constraint:edge-01".into()],
            bounded_default_unit_of_work: true,
            max_iterations: 100,
            convergence_tolerance: 0.01,
        };
        let json = serde_json::to_string(&contract).unwrap();
        let deserialized: RegionContractV1 = serde_json::from_str(&json).unwrap();
        assert_eq!(contract.region_id, deserialized.region_id);
        assert_eq!(contract.max_iterations, deserialized.max_iterations);
    }

    #[test]
    fn residual_envelope_computes_within_tolerance() {
        let mut expected = HashMap::new();
        expected.insert("a".into(), 1.0);
        expected.insert("b".into(), 2.0);
        let mut observed = HashMap::new();
        observed.insert("a".into(), 1.01);
        observed.insert("b".into(), 2.02);

        let mut residuals = HashMap::new();
        residuals.insert("a".into(), 0.01);
        residuals.insert("b".into(), 0.02);

        let envelope = ResidualEnvelopeV1 {
            region_id: "r1".into(),
            expected,
            observed,
            residuals,
            max_residual: 0.02,
            within_tolerance: true,
        };
        assert!(envelope.within_tolerance);
    }

    #[test]
    fn syndrome_envelope_marks_recoverable() {
        let syndrome = SyndromeEnvelopeV1 {
            region_id: "r1".into(),
            violated_constraint_ids: vec!["c1".into()],
            violation_descriptions: vec!["constraint c1 violated".into()],
            recoverable: true,
        };
        assert!(syndrome.recoverable);
    }
}
