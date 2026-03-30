//! Objective-driven scoring policy.

use serde::{Deserialize, Serialize};

/// What kind of objective the patch is trying to achieve.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ObjectiveKind {
    BugFix,
    Refactor,
    SafetyHardening,
    Performance,
    Exploration,
}

/// Policy for how scores are combined for a given objective.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObjectivePolicy {
    pub kind: ObjectiveKind,
    /// Weight for correctness in the objective.
    pub correctness_weight: f64,
    /// Weight for novelty (auxiliary, explainable).
    pub novelty_weight: f64,
    /// Weight for stability (may be absent if trials insufficient).
    pub stability_weight: f64,
    /// Whether scalarization into weighted_total is allowed.
    pub allow_scalarization: bool,
    /// Minimum trial count for stability to be included.
    pub min_trials_for_stability: u32,
}

impl ObjectivePolicy {
    /// Create a policy for a bug fix objective.
    pub fn bug_fix() -> Self {
        Self {
            kind: ObjectiveKind::BugFix,
            correctness_weight: 0.85,
            novelty_weight: 0.05,
            stability_weight: 0.10,
            allow_scalarization: true,
            min_trials_for_stability: 3,
        }
    }

    /// Create a policy for a refactoring objective.
    pub fn refactor() -> Self {
        Self {
            kind: ObjectiveKind::Refactor,
            correctness_weight: 0.70,
            novelty_weight: 0.20,
            stability_weight: 0.10,
            allow_scalarization: true,
            min_trials_for_stability: 3,
        }
    }

    /// Create a policy for a safety hardening objective.
    pub fn safety() -> Self {
        Self {
            kind: ObjectiveKind::SafetyHardening,
            correctness_weight: 0.90,
            novelty_weight: 0.0,
            stability_weight: 0.10,
            allow_scalarization: true,
            min_trials_for_stability: 5,
        }
    }

    /// Create a policy for a performance objective.
    pub fn performance() -> Self {
        Self {
            kind: ObjectiveKind::Performance,
            correctness_weight: 0.60,
            novelty_weight: 0.10,
            stability_weight: 0.30,
            allow_scalarization: false, // timing needs statistical backing
            min_trials_for_stability: 5,
        }
    }

    /// Create a policy for exploratory work.
    pub fn exploration() -> Self {
        Self {
            kind: ObjectiveKind::Exploration,
            correctness_weight: 0.50,
            novelty_weight: 0.40,
            stability_weight: 0.10,
            allow_scalarization: false, // exploratory, not production-grade
            min_trials_for_stability: 1,
        }
    }

    /// Compute weighted total if scalarization is allowed and stability is valid.
    pub fn compute_weighted_total(
        &self,
        correctness: f64,
        novelty: f64,
        stability: Option<f64>,
        trial_count: u32,
    ) -> Option<f64> {
        if !self.allow_scalarization {
            return None;
        }

        let stability_val = if trial_count >= self.min_trials_for_stability {
            stability.unwrap_or(0.0)
        } else {
            0.0 // insufficient trials, stability excluded
        };

        Some(
            self.correctness_weight * correctness
                + self.novelty_weight * novelty
                + self.stability_weight * stability_val,
        )
    }
}

impl Default for ObjectivePolicy {
    fn default() -> Self {
        Self::refactor()
    }
}

/// Check comparability classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ComparabilityClass {
    /// Core checks whose results are directly comparable between baseline and patched.
    ComparableCore,
    /// Diagnostic checks only meaningful in the patched context.
    PatchOnlyDiagnostic,
    /// Informational-only, never feeds metrics.
    Informational,
}

/// A classified check in an execution plan.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PlannedCheck {
    pub check_kind: crate::exec::backend::CheckKind,
    pub comparability: ComparabilityClass,
    /// Override from task fixture config, if any.
    pub config_override: bool,
}

/// Frozen execution plan with classified checks.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatchExecutionPlan {
    pub checks: Vec<PlannedCheck>,
    pub frozen: bool,
}

impl PatchExecutionPlan {
    /// Create a default Cargo-based execution plan.
    pub fn cargo_default() -> Self {
        Self {
            checks: vec![
                PlannedCheck {
                    check_kind: crate::exec::backend::CheckKind::Fmt,
                    comparability: ComparabilityClass::ComparableCore,
                    config_override: false,
                },
                PlannedCheck {
                    check_kind: crate::exec::backend::CheckKind::Clippy,
                    comparability: ComparabilityClass::ComparableCore,
                    config_override: false,
                },
                PlannedCheck {
                    check_kind: crate::exec::backend::CheckKind::Test,
                    comparability: ComparabilityClass::ComparableCore,
                    config_override: false,
                },
            ],
            frozen: true,
        }
    }

    /// Get only ComparableCore checks.
    pub fn comparable_checks(&self) -> Vec<&PlannedCheck> {
        self.checks
            .iter()
            .filter(|c| c.comparability == ComparabilityClass::ComparableCore)
            .collect()
    }
}
