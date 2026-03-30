use rand::Rng;
use serde::{Deserialize, Serialize};

/// AlgebraSpec — a candidate's parameter specification for MindState compilation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlgebraSpec {
    /// The primary evidence retrieval parameter.
    pub k: f64,
    /// Operator weights for combining signals.
    pub operator_weights: OperatorWeights,
    /// Delta amplitudes for stabilization phases.
    pub delta_amp_default: f64,
    pub delta_amp_stabilize1: f64,
    pub delta_amp_stabilize2: f64,
    pub delta_amp_clamp: f64,
    /// Evidence budget.
    pub evidence_budget: usize,
    /// Orthogonality target for novelty.
    pub orthogonality_target: f64,
    /// Token budget for MindState.
    pub token_budget: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OperatorWeights {
    pub correctness: f64,
    pub novelty: f64,
    pub stability: f64,
}

impl Default for AlgebraSpec {
    fn default() -> Self {
        Self {
            k: 5.0,
            operator_weights: OperatorWeights {
                correctness: 0.7,
                novelty: 0.2,
                stability: 0.1,
            },
            delta_amp_default: 0.7,
            delta_amp_stabilize1: 0.2,
            delta_amp_stabilize2: 0.1,
            delta_amp_clamp: 0.0,
            evidence_budget: 8,
            orthogonality_target: 0.10,
            token_budget: 1800,
        }
    }
}

/// E1: Param mutation — perturb numeric parameters.
/// `perturbation_scale` controls magnitude; `rng` must be seeded by caller for reproducibility.
pub fn mutate_params<R: Rng>(
    parent: &AlgebraSpec,
    perturbation_scale: f64,
    rng: &mut R,
) -> AlgebraSpec {
    let mut child = parent.clone();

    child.k = clamp_positive(child.k * (1.0 + perturbation_scale * signed(rng)));
    child.delta_amp_default =
        (child.delta_amp_default + perturbation_scale * signed(rng)).clamp(0.0, 1.0);
    child.delta_amp_stabilize1 =
        (child.delta_amp_stabilize1 + perturbation_scale * signed(rng)).clamp(0.0, 1.0);
    child.delta_amp_stabilize2 =
        (child.delta_amp_stabilize2 + perturbation_scale * signed(rng)).clamp(0.0, 1.0);
    child.orthogonality_target =
        (child.orthogonality_target + perturbation_scale * 0.1 * signed(rng)).clamp(0.0, 1.0);
    child.evidence_budget = ((child.evidence_budget as f64
        + perturbation_scale * 2.0 * signed(rng))
    .round()
    .max(1.0)) as usize;

    // Perturb operator weights then normalize to sum to 1.0
    child.operator_weights.correctness =
        (child.operator_weights.correctness + perturbation_scale * 0.1 * signed(rng)).max(0.01);
    child.operator_weights.novelty =
        (child.operator_weights.novelty + perturbation_scale * 0.1 * signed(rng)).max(0.01);
    child.operator_weights.stability =
        (child.operator_weights.stability + perturbation_scale * 0.1 * signed(rng)).max(0.01);
    normalize_weights(&mut child.operator_weights);

    child
}

/// E2: Crossover — merge two parent specs.
pub fn crossover(parent_a: &AlgebraSpec, parent_b: &AlgebraSpec) -> AlgebraSpec {
    let mut child = AlgebraSpec {
        // From parent A
        k: parent_a.k,
        token_budget: parent_a.token_budget,
        evidence_budget: parent_a.evidence_budget,

        // From parent B (delta policy)
        delta_amp_default: parent_b.delta_amp_default,
        delta_amp_stabilize1: parent_b.delta_amp_stabilize1,
        delta_amp_stabilize2: parent_b.delta_amp_stabilize2,
        delta_amp_clamp: parent_b.delta_amp_clamp,

        // Average other numeric parameters
        orthogonality_target: (parent_a.orthogonality_target + parent_b.orthogonality_target) / 2.0,
        operator_weights: OperatorWeights {
            correctness: (parent_a.operator_weights.correctness
                + parent_b.operator_weights.correctness)
                / 2.0,
            novelty: (parent_a.operator_weights.novelty + parent_b.operator_weights.novelty) / 2.0,
            stability: (parent_a.operator_weights.stability + parent_b.operator_weights.stability)
                / 2.0,
        },
    };
    normalize_weights(&mut child.operator_weights);
    child
}

/// Normalize operator weights so they sum to 1.0.
fn normalize_weights(w: &mut OperatorWeights) {
    let sum = w.correctness + w.novelty + w.stability;
    if sum > 0.0 {
        w.correctness /= sum;
        w.novelty /= sum;
        w.stability /= sum;
    }
}

fn clamp_positive(v: f64) -> f64 {
    if v < 0.1 {
        0.1
    } else {
        v
    }
}

/// Signed random direction: -1.0 or +1.0.
fn signed<R: Rng>(rng: &mut R) -> f64 {
    if rng.random::<bool>() {
        1.0
    } else {
        -1.0
    }
}
