//! Gram-table scoring for FibQuant fixed-rate codes.

use crate::MemoryError;

#[cfg(feature = "fib-quant-codec")]
use fib_quant::{bitpack, FibCodeV1, FibQuantizer};

/// Precomputed query-to-codebook dot products, arranged by block then codeword.
#[cfg(feature = "fib-quant-codec")]
#[derive(Debug, Clone)]
pub struct FibGramScorer {
    gram_table: Vec<Vec<f32>>, // block_count × codebook_size (materialized per query)
    codebook: Vec<f32>,        // row-major codebook_size × block_dim
    block_dim: usize,
    block_count: usize,
    codebook_size: usize,
}

/// A query prepared for repeated FibGramScorer scoring.
#[cfg(feature = "fib-quant-codec")]
#[derive(Debug, Clone)]
pub struct PreparedFibQuery {
    pub(crate) gram_rows: Vec<Vec<f32>>,
}

#[cfg(feature = "fib-quant-codec")]
fn fib_error(err: fib_quant::FibQuantError) -> MemoryError {
    MemoryError::QuantizationError(format!("fib-quant: {err}"))
}

#[cfg(feature = "fib-quant-codec")]
impl FibGramScorer {
    /// Build the query-independent Gram table from a quantizer's codebook.
    pub fn from_quantizer(quantizer: &FibQuantizer) -> Result<Self, MemoryError> {
        let profile = quantizer.profile();
        let block_dim = profile.block_dim as usize;
        let block_count = profile.block_count() as usize;
        let codebook_size = profile.codebook_size as usize;
        let expected = codebook_size
            .checked_mul(block_dim)
            .ok_or_else(|| MemoryError::QuantizationError("Fib codebook size overflow".into()))?;
        if quantizer.codebook().codewords.len() != expected {
            return Err(MemoryError::QuantizationError(format!(
                "Fib codebook has {} values, expected {}",
                quantizer.codebook().codewords.len(),
                expected
            )));
        }
        Ok(Self {
            gram_table: vec![vec![0.0; codebook_size]; block_count],
            codebook: quantizer.codebook().codewords.clone(),
            block_dim,
            block_count,
            codebook_size,
        })
    }

    /// Prepare a query by computing its dot product with every codeword per block.
    pub fn prepare_query(&self, query: &[f32]) -> Result<PreparedFibQuery, MemoryError> {
        if query.len() != self.block_count * self.block_dim {
            return Err(MemoryError::DimensionMismatch {
                expected: self.block_count * self.block_dim,
                actual: query.len(),
            });
        }
        if query.iter().any(|value| !value.is_finite()) {
            return Err(MemoryError::QuantizationError(
                "Fib query contains non-finite value".into(),
            ));
        }
        let mut gram_rows = self.gram_table.clone();
        for (block, query_block) in query.chunks_exact(self.block_dim).enumerate() {
            for index in 0..self.codebook_size {
                let base = index * self.block_dim;
                gram_rows[block][index] = query_block
                    .iter()
                    .zip(&self.codebook[base..base + self.block_dim])
                    .map(|(q, c)| q * c)
                    .sum();
            }
        }
        Ok(PreparedFibQuery { gram_rows })
    }

    pub fn score(&self, prepared: &PreparedFibQuery, code: &FibCodeV1) -> Result<f32, MemoryError> {
        if prepared.gram_rows.len() != self.block_count
            || code.block_count as usize != self.block_count
            || code.block_dim as usize != self.block_dim
            || code.wire_index_bits == 0
        {
            return Err(MemoryError::QuantizationError(
                "Fib code/query shape mismatch".into(),
            ));
        }
        let indices =
            bitpack::unpack_indices(&code.indices, self.block_count, code.wire_index_bits)
                .map_err(fib_error)?;
        let mut total = 0.0;
        for (block, index) in indices.into_iter().enumerate() {
            let index = index as usize;
            if index >= self.codebook_size {
                return Err(MemoryError::QuantizationError(format!(
                    "Fib codebook index {index} out of range"
                )));
            }
            total += prepared.gram_rows[block][index];
        }
        Ok(total)
    }

    pub fn score_batch(
        &self,
        prepared: &PreparedFibQuery,
        codes: &[FibCodeV1],
    ) -> Result<Vec<f32>, MemoryError> {
        codes
            .iter()
            .map(|code| self.score(prepared, code))
            .collect()
    }
}
