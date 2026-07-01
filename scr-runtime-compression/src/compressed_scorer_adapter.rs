//! `CompressedScorerAdapter` — compressed-domain candidate scoring.
//!
//! This adapter is the sibling of `ExactFallbackAdapter`: instead of decoding
//! every candidate to f32, it prepares the query once and scores compressed
//! payloads directly through the shared `compressed-scorer` trait.

use compressed_scorer::CompressedScorer;

use crate::{CodecId, CompressionError};

/// Candidate with caller-owned item identity and approximate compressed score.
#[derive(Debug, Clone, PartialEq)]
pub struct ScoredCompressedCandidate<K> {
    pub key: K,
    pub score: f32,
    pub source_rank: usize,
}

/// Codec-agnostic scorer adapter over an implementation of `CompressedScorer`.
pub struct CompressedScorerAdapter<S: CompressedScorer> {
    codec_id: CodecId,
    scorer: S,
}

impl<S: CompressedScorer> CompressedScorerAdapter<S> {
    /// Wrap a concrete scorer implementation.
    pub fn new(codec_id: CodecId, scorer: S) -> Self {
        Self { codec_id, scorer }
    }

    /// Return the codec routed by this adapter.
    pub fn codec_id(&self) -> CodecId {
        self.codec_id
    }

    /// Borrow the underlying scorer.
    pub fn scorer(&self) -> &S {
        &self.scorer
    }

    /// Score compressed candidates without decoding them.
    pub fn score_candidates<K: Clone>(
        &self,
        query: &[f32],
        candidates: &[(K, S::Compressed)],
        min_score: f32,
        limit: usize,
    ) -> Result<Vec<ScoredCompressedCandidate<K>>, CompressionError> {
        if limit == 0 || candidates.is_empty() {
            return Ok(Vec::new());
        }
        let prepared = self
            .scorer
            .prepare_query(query)
            .map_err(|e| CompressionError::ScoringFailed(e.to_string()))?;
        let mut scored = Vec::new();
        for (seq, (key, compressed)) in candidates.iter().enumerate() {
            let score = self
                .scorer
                .score_prepared(&prepared, compressed)
                .map_err(|e| CompressionError::ScoringFailed(e.to_string()))?;
            if score.is_finite() && score >= min_score {
                scored.push(ScoredCompressedCandidate {
                    key: key.clone(),
                    score,
                    source_rank: seq + 1,
                });
            }
        }
        scored.sort_by(|a, b| {
            b.score
                .total_cmp(&a.score)
                .then_with(|| a.source_rank.cmp(&b.source_rank))
        });
        scored.truncate(limit);
        Ok(scored)
    }
}

#[cfg(feature = "turbo")]
impl CompressedScorerAdapter<compressed_scorer::turbo_impl::TurboScorerAdapter> {
    /// Build a TurboQuant compressed scorer adapter.
    pub fn turbo_quant(
        dim: usize,
        bits: u8,
        projections: usize,
        seed: u64,
    ) -> Result<Self, CompressionError> {
        let scorer = compressed_scorer::turbo_impl::TurboScorerAdapter::new(
            dim,
            bits,
            projections,
            seed,
        )
        .map_err(|e| CompressionError::ScoringFailed(e.to_string()))?;
        Ok(Self::new(CodecId::TurboQuant, scorer))
    }
}

#[cfg(feature = "fib")]
impl CompressedScorerAdapter<compressed_scorer::fib_impl::FibScorerAdapter> {
    /// Build a FibQuant compressed scorer adapter from an existing quantizer.
    pub fn fib_quant(quantizer: fib_quant::FibQuantizer) -> Result<Self, CompressionError> {
        let scorer = compressed_scorer::fib_impl::FibScorerAdapter::new(quantizer)
            .map_err(|e| CompressionError::ScoringFailed(e.to_string()))?;
        Ok(Self::new(CodecId::FibQuant, scorer))
    }
}

impl CompressedScorerAdapter<compressed_scorer::PerDimScorer> {
    /// Build a per-dimension quantized scorer adapter.
    pub fn per_dim(dim: usize, bits: u32) -> Result<Self, CompressionError> {
        let scorer = compressed_scorer::PerDimScorer::new(dim, bits)
            .map_err(|e| CompressionError::ScoringFailed(e.to_string()))?;
        Ok(Self::new(CodecId::PerDim, scorer))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use compressed_scorer::{CompressedScorer, PreparedQuery, ScorerResult};

    #[derive(Clone)]
    struct Prepared(Vec<f32>);
    impl PreparedQuery for Prepared {
        fn dim(&self) -> usize { self.0.len() }
    }

    struct ToyScorer;
    impl CompressedScorer for ToyScorer {
        type Prepared = Prepared;
        type Compressed = Vec<f32>;
        fn prepare_query(&self, query: &[f32]) -> ScorerResult<Self::Prepared> { Ok(Prepared(query.to_vec())) }
        fn score_prepared(&self, prepared: &Self::Prepared, compressed: &Self::Compressed) -> ScorerResult<f32> {
            Ok(prepared.0.iter().zip(compressed).map(|(a,b)| a*b).sum())
        }
        fn decode(&self, compressed: &Self::Compressed) -> ScorerResult<Vec<f32>> { Ok(compressed.clone()) }
        fn dim(&self) -> usize { 2 }
        fn codec_name(&self) -> &'static str { "toy" }
        fn internal_bytes(&self) -> usize { 0 }
    }

    #[test]
    fn adapter_scores_and_sorts_without_decode() {
        let adapter = CompressedScorerAdapter::new(CodecId::Uncompressed, ToyScorer);
        let candidates = vec![
            ("low", vec![0.0, 1.0]),
            ("high", vec![1.0, 0.0]),
            ("mid", vec![0.5, 0.5]),
        ];
        let scored = adapter.score_candidates(&[1.0, 0.0], &candidates, -1.0, 2).unwrap();
        assert_eq!(scored.len(), 2);
        assert_eq!(scored[0].key, "high");
        assert_eq!(scored[1].key, "mid");
    }
}
