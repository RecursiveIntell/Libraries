//! Page-level compressed scorer primitives.
//!
//! This module scores many FibQuant compressed candidates from already-decoded
//! scoring ingredients: query codeword indices, stored codeword indices, stored
//! norms, and the Gram table. It does **not** decode vectors.

use crate::{GpuError, Result};

#[derive(Debug, Clone, Copy)]
pub struct FibGramPageScoreInput<'a> {
    pub query_indices: &'a [u32],
    pub stored_indices: &'a [u32],
    pub stored_norms: &'a [f32],
    pub gram: &'a [f32],
    pub query_norm: f32,
    pub n_candidates: usize,
    pub block_count: usize,
    pub n_codewords: usize,
}

impl FibGramPageScoreInput<'_> {
    pub fn validate(&self) -> Result<()> {
        if self.block_count == 0 {
            return Err(GpuError::InvalidConfig("block_count must be > 0".into()));
        }
        if self.n_codewords == 0 {
            return Err(GpuError::InvalidConfig("n_codewords must be > 0".into()));
        }
        if self.query_indices.len() != self.block_count {
            return Err(GpuError::DimensionMismatch {
                expected: self.block_count,
                got: self.query_indices.len(),
            });
        }
        let expected_indices = self
            .n_candidates
            .checked_mul(self.block_count)
            .ok_or_else(|| GpuError::InvalidConfig("stored index length overflow".into()))?;
        if self.stored_indices.len() != expected_indices {
            return Err(GpuError::DimensionMismatch {
                expected: expected_indices,
                got: self.stored_indices.len(),
            });
        }
        if self.stored_norms.len() != self.n_candidates {
            return Err(GpuError::DimensionMismatch {
                expected: self.n_candidates,
                got: self.stored_norms.len(),
            });
        }
        let expected_gram = self
            .n_codewords
            .checked_mul(self.n_codewords)
            .ok_or_else(|| GpuError::InvalidConfig("gram length overflow".into()))?;
        if self.gram.len() != expected_gram {
            return Err(GpuError::DimensionMismatch {
                expected: expected_gram,
                got: self.gram.len(),
            });
        }
        if !self.query_norm.is_finite() {
            return Err(GpuError::InvalidConfig("query_norm must be finite".into()));
        }
        for &idx in self.query_indices {
            if idx as usize >= self.n_codewords {
                return Err(GpuError::InvalidConfig(format!(
                    "query index {idx} >= n_codewords {}",
                    self.n_codewords
                )));
            }
        }
        for &idx in self.stored_indices {
            if idx as usize >= self.n_codewords {
                return Err(GpuError::InvalidConfig(format!(
                    "stored index {idx} >= n_codewords {}",
                    self.n_codewords
                )));
            }
        }
        if self.stored_norms.iter().any(|v| !v.is_finite()) {
            return Err(GpuError::InvalidConfig(
                "stored_norms contain non-finite values".into(),
            ));
        }
        Ok(())
    }
}

pub fn score_fib_gram_pages_cpu(input: FibGramPageScoreInput<'_>) -> Result<Vec<f32>> {
    input.validate()?;
    let mut scores = vec![0.0f32; input.n_candidates];
    for (candidate, score) in scores.iter_mut().enumerate() {
        let base = candidate * input.block_count;
        let mut sum = 0.0f32;
        for block in 0..input.block_count {
            let qi = input.query_indices[block] as usize;
            let si = input.stored_indices[base + block] as usize;
            sum += input.gram[qi * input.n_codewords + si];
        }
        *score = sum * input.query_norm * input.stored_norms[candidate];
    }
    Ok(scores)
}

pub fn score_fib_gram_pages(input: FibGramPageScoreInput<'_>) -> Result<Vec<f32>> {
    input.validate()?;
    #[cfg(feature = "gpu")]
    {
        if let Some(ctx) = crate::GpuContext::init() {
            if input.n_candidates >= crate::GpuContext::GPU_MIN_BATCH_SIZE {
                return crate::cuda::fib_gram_page_score_gpu(ctx, input);
            }
        }
    }
    score_fib_gram_pages_cpu(input)
}

pub fn topk_indices_desc(scores: &[f32], top_k: usize) -> Vec<usize> {
    let mut indices: Vec<usize> = (0..scores.len()).collect();
    indices.sort_by(|&a, &b| {
        scores[b]
            .partial_cmp(&scores[a])
            .unwrap_or(core::cmp::Ordering::Equal)
            .then_with(|| a.cmp(&b))
    });
    indices.truncate(top_k.min(indices.len()));
    indices
}
