use crate::{scalar, HyperQuantConfig, HyperQuantResult, LatticeKind};
use serde::{Deserialize, Serialize};

/// Conservative claim boundary attached to receipts.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ClaimBoundary {
    /// The result is a local primitive receipt, not a model-quality or paper-parity claim.
    ExperimentalPrimitiveOnly,
}

/// Receipt for a single HyperQuant operation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantReceiptV1 {
    pub kind: LatticeKind,
    pub input_len: usize,
    pub code_len: usize,
    pub effective_scale: f32,
    pub mse: f32,
    pub input_digest: String,
    pub config_digest: String,
    pub claim_boundary: ClaimBoundary,
}

impl HyperQuantReceiptV1 {
    pub(crate) fn from_result(result: &HyperQuantResult) -> Self {
        Self {
            kind: result.kind,
            input_len: result.input_len,
            code_len: result.codes.len(),
            effective_scale: result.effective_scale,
            mse: result.mse,
            input_digest: result.input_digest.clone(),
            config_digest: result.config_digest.clone(),
            claim_boundary: ClaimBoundary::ExperimentalPrimitiveOnly,
        }
    }
}

pub(crate) fn input_digest(input: &[f32]) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"hyperquant-input-v1");
    hasher.update(&(input.len() as u64).to_le_bytes());
    for value in input {
        hasher.update(&value.to_bits().to_le_bytes());
    }
    format!("blake3:{}", scalar::hex(hasher.finalize().as_bytes()))
}

pub(crate) fn config_digest(config: &HyperQuantConfig) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"hyperquant-config-v1");
    hasher.update(&[config.kind as u8]);
    hasher.update(&config.effective_scale().to_bits().to_le_bytes());
    format!("blake3:{}", scalar::hex(hasher.finalize().as_bytes()))
}
