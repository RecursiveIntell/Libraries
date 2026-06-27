//! HyperQuant evaluation harness.
//!
//! This module is intentionally local and fixture-driven. It measures the
//! current `hyperquant` crate as an experimental primitive; it does not claim
//! paper parity, model-quality preservation, or production admissibility.

use crate::QuantEvalError;
use hyperquant::{quantize_a2, quantize_z1, HyperQuantError, LatticeKind};
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

/// Run deterministic fixture evaluation for HyperQuant Z1 and A2.
pub fn run_hyperquant_eval(
    config: &HyperQuantEvalConfig,
) -> Result<HyperQuantEvalResult, QuantEvalError> {
    validate_config(config)?;
    let vectors = generate_fixture_vectors(config);
    let profiles = vec![
        evaluate_profile(LatticeKind::Z1, config.scale, &vectors),
        evaluate_profile(LatticeKind::A2, config.scale, &vectors),
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

fn evaluate_profile(kind: LatticeKind, scale: f32, vectors: &[Vec<f32>]) -> HyperQuantProfileEval {
    let mut mse_values = Vec::with_capacity(vectors.len());
    let mut rejected_vectors = 0usize;
    let mut receipt_count = 0usize;

    for vector in vectors {
        let result = match kind {
            LatticeKind::Z1 => quantize_z1(vector, scale),
            LatticeKind::A2 => quantize_a2(vector, scale),
            LatticeKind::D4 | LatticeKind::E8 => Err(HyperQuantError::UnsupportedLattice(kind)),
        };
        match result {
            Ok(result) => {
                let receipt = result.receipt();
                if receipt.mse.is_finite() {
                    mse_values.push(receipt.mse);
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
    let compressed_bytes = dim * core::mem::size_of::<i16>();

    HyperQuantProfileEval {
        kind,
        mean_mse,
        max_mse,
        mean_bytes_per_vector: compressed_bytes as f32,
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
