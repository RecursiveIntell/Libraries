//! HyperQuant evaluation harness.
//!
//! This module is intentionally local and fixture-driven. It measures the
//! current `hyperquant` crate as an experimental primitive; it does not claim
//! paper parity, model-quality preservation, or production admissibility.

use crate::QuantEvalError;
use hyperquant::{
    estimate_best_rice_profile, quantize_a2, quantize_d4, quantize_z1, HyperQuantError, LatticeKind,
};
use quant_governor::{
    AdmissibilityClass, CodecProfile as GovernorCodecProfile, ContentType, GovernancePolicy,
    GovernanceRequest,
};
use serde::{Deserialize, Serialize};

/// Configuration for deterministic HyperQuant fixture evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantEvalConfig {
    /// Vector dimension.
    pub dim: usize,
    /// Number of vectors in the synthetic fixture.
    pub vectors: usize,
    /// Deterministic fixture seed.
    pub seed: u64,
    /// Quantization scale passed to HyperQuant.
    pub scale: f32,
}

impl HyperQuantEvalConfig {
    /// A small fixture where points lie on the A2 triangular basis.
    pub fn triangular_fixture() -> Self {
        Self {
            dim: 2,
            vectors: 12,
            seed: 0xA2,
            scale: 1.0,
        }
    }
}

impl Default for HyperQuantEvalConfig {
    fn default() -> Self {
        Self {
            dim: 16,
            vectors: 64,
            seed: 42,
            scale: 8.0,
        }
    }
}

/// Per-lattice profile result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantProfileEval {
    pub kind: LatticeKind,
    pub mean_mse: f32,
    pub max_mse: f32,
    pub mean_bytes_per_vector: f32,
    pub estimated_raw_bytes_per_vector: usize,
    pub estimated_compressed_bytes_per_vector: usize,
    pub rejected_vectors: usize,
    pub receipt_count: usize,
}

/// Full HyperQuant evaluation result.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantEvalResult {
    pub config: HyperQuantEvalConfig,
    pub profiles: Vec<HyperQuantProfileEval>,
    pub claim_boundary: String,
}

impl HyperQuantEvalResult {
    /// Return a profile by lattice kind.
    pub fn profile(&self, kind: LatticeKind) -> Option<&HyperQuantProfileEval> {
        self.profiles.iter().find(|profile| profile.kind == kind)
    }
}

/// Governance policy preset for governed HyperQuant evaluation.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum GovernedHyperQuantPolicyPreset {
    /// Default quant-governor policy.
    Default,
    /// Storage-efficient quant-governor policy.
    StorageEfficient,
    /// Low-latency quant-governor policy.
    LowLatency,
    /// Accuracy-oriented quant-governor policy.
    AccuracyOriented,
    /// Strict custom budget that blocks HyperQuant and Q4.
    CustomStrict,
}

impl GovernedHyperQuantPolicyPreset {
    fn policy(self) -> GovernancePolicy {
        match self {
            GovernedHyperQuantPolicyPreset::Default => GovernancePolicy::default(),
            GovernedHyperQuantPolicyPreset::StorageEfficient => {
                GovernancePolicy::storage_efficient()
            }
            GovernedHyperQuantPolicyPreset::LowLatency => GovernancePolicy::low_latency(),
            GovernedHyperQuantPolicyPreset::AccuracyOriented => {
                GovernancePolicy::accuracy_oriented()
            }
            GovernedHyperQuantPolicyPreset::CustomStrict => GovernancePolicy::new(0.06, 64, 0.999),
        }
    }
}

/// Configuration for a governed HyperQuant evaluation receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernedHyperQuantEvalConfig {
    /// HyperQuant fixture configuration to evaluate.
    pub fixture: HyperQuantEvalConfig,
    /// Policy preset used for governor routing.
    pub policy_preset: GovernedHyperQuantPolicyPreset,
    /// Embedding payload size presented to the governor.
    pub size_bytes: u64,
    /// Required accuracy presented to the governor.
    pub accuracy_requirement: f64,
    /// Latency tolerance presented to the governor.
    pub latency_tolerance_ms: u64,
    /// Admissibility class presented to the governor.
    pub admissibility: AdmissibilityClass,
}

/// String-backed policy trace captured from quant-governor.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernedHyperQuantDecisionTrace {
    /// Selected codec profile.
    pub selected_codec: String,
    /// Policy name used by quant-governor.
    pub policy_name: String,
    /// Content type routed by quant-governor.
    pub content_type: String,
    /// Admissibility class used by quant-governor.
    pub admissibility: String,
    /// Human-readable routing rationale.
    pub rationale: String,
    /// Profiles blocked by the routing branch.
    pub blocked_profiles: Vec<String>,
    /// Profiles considered by the routing branch.
    pub candidate_profiles: Vec<String>,
}

/// Baseline byte accounting for a governed HyperQuant candidate.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernedHyperQuantBaseline {
    /// Codec label.
    pub codec: String,
    /// Estimated bytes per vector.
    pub estimated_bytes_per_vector: usize,
    /// Raw f32 bytes per vector.
    pub raw_bytes_per_vector: usize,
    /// Raw/compressed byte ratio.
    pub compression_ratio: f32,
}

/// Full governed HyperQuant evaluation receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GovernedHyperQuantEvalReceipt {
    /// Configuration used to generate this receipt.
    pub config: GovernedHyperQuantEvalConfig,
    /// Governor decision trace.
    pub decision: GovernedHyperQuantDecisionTrace,
    /// True when the governor selected HyperQuant.
    pub admitted: bool,
    /// Admission or rejection reason.
    pub admission_reason: String,
    /// Measured HyperQuant fixture evaluation.
    pub hyperquant_eval: HyperQuantEvalResult,
    /// Baseline byte estimates for Q8, Q4, and HyperQuant.
    pub baselines: Vec<GovernedHyperQuantBaseline>,
    /// Selected measured HyperQuant profile for ROI accounting.
    pub selected_hyperquant_profile: Option<HyperQuantProfileEval>,
    /// Narrow claim boundary for this receipt.
    pub claim_boundary: String,
}

/// Run a governed HyperQuant fixture evaluation and produce an audit receipt.
pub fn run_governed_hyperquant_eval(
    config: &GovernedHyperQuantEvalConfig,
) -> Result<GovernedHyperQuantEvalReceipt, QuantEvalError> {
    let policy = config.policy_preset.policy();
    let request = GovernanceRequest {
        content_type: ContentType::Embedding,
        size_bytes: config.size_bytes,
        accuracy_requirement: config.accuracy_requirement,
        latency_tolerance_ms: config.latency_tolerance_ms,
        admissibility: config.admissibility.clone(),
    };
    let decision = policy.evaluate(request).map_err(|err| {
        QuantEvalError::InvalidCorpus(format!("quant-governor evaluation failed: {err}"))
    })?;

    let trace = decision_trace(&decision, &policy);
    let hyperquant_eval = run_hyperquant_eval(&config.fixture)?;
    let selected_profile = select_hyperquant_profile(&hyperquant_eval).cloned();
    let baselines = build_baselines(config.fixture.dim, selected_profile.as_ref());
    let admitted = decision.codec == GovernorCodecProfile::Hyperquant;

    Ok(GovernedHyperQuantEvalReceipt {
        config: config.clone(),
        admission_reason: trace.rationale.clone(),
        decision: trace,
        admitted,
        hyperquant_eval,
        baselines,
        selected_hyperquant_profile: selected_profile,
        claim_boundary: "fixture-level governed HyperQuant ROI receipt only; not production, paper parity, E8, or retrieval-quality evidence".to_string(),
    })
}

/// Run deterministic fixture evaluation for HyperQuant Z1, A2, and D4.
pub fn run_hyperquant_eval(
    config: &HyperQuantEvalConfig,
) -> Result<HyperQuantEvalResult, QuantEvalError> {
    validate_config(config)?;
    let vectors = generate_fixture_vectors(config);
    let profiles = vec![
        evaluate_profile(LatticeKind::Z1, config.scale, &vectors),
        evaluate_profile(LatticeKind::A2, config.scale, &vectors),
        evaluate_profile(LatticeKind::D4, config.scale, &vectors),
    ];

    Ok(HyperQuantEvalResult {
        config: *config,
        profiles,
        claim_boundary: "experimental primitive only; not paper parity or model-quality evidence"
            .to_string(),
    })
}

fn validate_config(config: &HyperQuantEvalConfig) -> Result<(), QuantEvalError> {
    if config.dim == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "hyperquant eval dim must be > 0".to_string(),
        ));
    }
    if config.vectors == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "hyperquant eval vectors must be > 0".to_string(),
        ));
    }
    Ok(())
}

fn decision_trace(
    decision: &quant_governor::CodecDecision,
    policy: &GovernancePolicy,
) -> GovernedHyperQuantDecisionTrace {
    match &decision.receipt {
        quant_governor::decision::CodecReceipt::Governance { governance, .. } => {
            GovernedHyperQuantDecisionTrace {
                selected_codec: decision.codec.to_string(),
                policy_name: governance.policy_name.clone(),
                content_type: governance.content_type.clone(),
                admissibility: governance.admissibility.clone(),
                rationale: governance.rationale.clone(),
                blocked_profiles: governance
                    .blocked_profiles
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
                candidate_profiles: governance
                    .candidate_profiles
                    .iter()
                    .map(ToString::to_string)
                    .collect(),
            }
        }
        _ => GovernedHyperQuantDecisionTrace {
            selected_codec: decision.codec.to_string(),
            policy_name: policy.name().to_string(),
            content_type: ContentType::Embedding.to_string(),
            admissibility: "unknown".to_string(),
            rationale: "quant-governor returned a direct decision without governance trace"
                .to_string(),
            blocked_profiles: Vec::new(),
            candidate_profiles: vec![decision.codec.to_string()],
        },
    }
}

fn select_hyperquant_profile(result: &HyperQuantEvalResult) -> Option<&HyperQuantProfileEval> {
    result
        .profile(LatticeKind::D4)
        .or_else(|| result.profile(LatticeKind::A2))
        .or_else(|| result.profile(LatticeKind::Z1))
}

fn build_baselines(
    dim: usize,
    selected_profile: Option<&HyperQuantProfileEval>,
) -> Vec<GovernedHyperQuantBaseline> {
    let raw_bytes = dim * core::mem::size_of::<f32>();
    let q8_bytes = dim;
    let q4_bytes = dim.div_ceil(2);
    let hyperquant_bytes = selected_profile
        .map(|profile| profile.estimated_compressed_bytes_per_vector)
        .unwrap_or(raw_bytes);

    vec![
        baseline("q8", raw_bytes, q8_bytes),
        baseline("q4", raw_bytes, q4_bytes),
        baseline("hyperquant", raw_bytes, hyperquant_bytes),
    ]
}

fn baseline(codec: &str, raw_bytes: usize, compressed_bytes: usize) -> GovernedHyperQuantBaseline {
    let estimated_bytes_per_vector = compressed_bytes.max(1);
    GovernedHyperQuantBaseline {
        codec: codec.to_string(),
        estimated_bytes_per_vector,
        raw_bytes_per_vector: raw_bytes,
        compression_ratio: raw_bytes as f32 / estimated_bytes_per_vector as f32,
    }
}

fn evaluate_profile(kind: LatticeKind, scale: f32, vectors: &[Vec<f32>]) -> HyperQuantProfileEval {
    let mut mse_values = Vec::with_capacity(vectors.len());
    let mut compressed_bytes_values = Vec::with_capacity(vectors.len());
    let mut rejected_vectors = 0usize;
    let mut receipt_count = 0usize;

    for vector in vectors {
        let result = match kind {
            LatticeKind::Z1 => quantize_z1(vector, scale),
            LatticeKind::A2 => quantize_a2(vector, scale),
            LatticeKind::D4 => quantize_d4(vector, scale),
            LatticeKind::E8 => Err(HyperQuantError::UnsupportedLattice(kind)),
        };
        match result {
            Ok(result) => {
                let receipt = result.receipt();
                if receipt.mse.is_finite() {
                    mse_values.push(receipt.mse);
                    match estimate_best_rice_profile(&result.codes, 0..=8) {
                        Ok(profile) => compressed_bytes_values.push(profile.encoded_bytes),
                        Err(_) => rejected_vectors += 1,
                    }
                    receipt_count += 1;
                } else {
                    rejected_vectors += 1;
                }
            }
            Err(_) => rejected_vectors += 1,
        }
    }

    let mean_mse = if mse_values.is_empty() {
        0.0
    } else {
        mse_values.iter().sum::<f32>() / mse_values.len() as f32
    };
    let max_mse = mse_values.iter().copied().fold(0.0f32, f32::max);
    let dim = vectors.first().map_or(0usize, Vec::len);
    let raw_bytes = dim * core::mem::size_of::<f32>();
    let mean_compressed_bytes = if compressed_bytes_values.is_empty() {
        0.0
    } else {
        compressed_bytes_values.iter().sum::<usize>() as f32 / compressed_bytes_values.len() as f32
    };
    let compressed_bytes = mean_compressed_bytes.ceil() as usize;

    HyperQuantProfileEval {
        kind,
        mean_mse,
        max_mse,
        mean_bytes_per_vector: mean_compressed_bytes,
        estimated_raw_bytes_per_vector: raw_bytes,
        estimated_compressed_bytes_per_vector: compressed_bytes,
        rejected_vectors,
        receipt_count,
    }
}

fn generate_fixture_vectors(config: &HyperQuantEvalConfig) -> Vec<Vec<f32>> {
    if config.dim == 2 && config.scale == 1.0 {
        return triangular_vectors(config.vectors);
    }

    (0..config.vectors)
        .map(|row| {
            (0..config.dim)
                .map(|col| deterministic_value(config.seed, row, col))
                .collect()
        })
        .collect()
}

fn triangular_vectors(count: usize) -> Vec<Vec<f32>> {
    const SQRT_3_OVER_2: f32 = 0.866_025_4;
    (0..count)
        .map(|i| {
            let u = (i % 4) as f32 - 1.0;
            let v = ((i / 4) % 4) as f32 - 1.0;
            vec![u + 0.5 * v, SQRT_3_OVER_2 * v]
        })
        .collect()
}

fn deterministic_value(seed: u64, row: usize, col: usize) -> f32 {
    let mut x = seed
        ^ (row as u64).wrapping_mul(0x9E37_79B9_7F4A_7C15)
        ^ (col as u64).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 30;
    x = x.wrapping_mul(0xBF58_476D_1CE4_E5B9);
    x ^= x >> 27;
    x = x.wrapping_mul(0x94D0_49BB_1331_11EB);
    x ^= x >> 31;
    let unit = (x as f64 / u64::MAX as f64) as f32;
    unit * 2.0 - 1.0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn deterministic_value_is_stable() {
        assert_eq!(deterministic_value(1, 2, 3), deterministic_value(1, 2, 3));
        assert_ne!(deterministic_value(1, 2, 3), deterministic_value(1, 2, 4));
    }

    #[test]
    fn triangular_vectors_are_a2_points() {
        let vectors = triangular_vectors(4);
        let profile = evaluate_profile(LatticeKind::A2, 1.0, &vectors);
        assert_eq!(profile.rejected_vectors, 0);
        assert!(profile.max_mse < 1.0e-6);
    }
}
