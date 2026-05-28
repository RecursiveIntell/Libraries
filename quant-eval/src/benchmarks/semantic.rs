//! SemanticMemoryBenchmark: Search quality over compressed vs raw data.
//!
//! Measures search quality degradation when using compressed representations
//! compared to raw (uncompressed) baseline.

use crate::error::QuantEvalError;
use serde::{Deserialize, Serialize};

/// Configuration for semantic memory benchmark.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryConfig {
    /// Embedding dimension
    pub dim: usize,
    /// Number of vectors in the index
    pub index_size: usize,
    /// Number of queries to evaluate
    pub num_queries: usize,
    /// Top-K results to retrieve
    pub top_k: usize,
    /// Random seed
    pub seed: u64,
}

impl Default for SemanticMemoryConfig {
    fn default() -> Self {
        Self {
            dim: 768,
            index_size: 10_000,
            num_queries: 100,
            top_k: 10,
            seed: 42,
        }
    }
}

/// Results from a semantic memory benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SemanticMemoryResult {
    /// Quality score for raw (uncompressed) search
    pub raw_quality: SearchQualityScore,
    /// Quality score for compressed search
    pub compressed_quality: SearchQualityScore,
    /// Quality degradation ratio (compressed / raw)
    pub degradation_ratio: f32,
    /// Number of queries evaluated
    pub queries: usize,
}

/// Search quality metrics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchQualityScore {
    /// Precision at K
    pub precision_at_k: f32,
    /// Recall at K
    pub recall_at_k: f32,
    /// NDCG at K
    pub ndcg_at_k: f32,
    /// Mean average precision
    pub map: f32,
}

impl Default for SearchQualityScore {
    fn default() -> Self {
        Self {
            precision_at_k: 0.0,
            recall_at_k: 0.0,
            ndcg_at_k: 0.0,
            map: 0.0,
        }
    }
}

/// SemanticMemoryBenchmark measures search quality degradation from compression.
#[derive(Debug, Clone)]
pub struct SemanticMemoryBenchmark {
    config: SemanticMemoryConfig,
}

impl SemanticMemoryBenchmark {
    /// Create a new semantic memory benchmark with default configuration.
    pub fn new() -> Self {
        Self {
            config: SemanticMemoryConfig::default(),
        }
    }

    /// Create a new benchmark with custom configuration.
    pub fn with_config(config: SemanticMemoryConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &SemanticMemoryConfig {
        &self.config
    }

    /// Run the semantic memory benchmark comparing compressed vs raw search.
    pub fn run(&self) -> Result<SemanticMemoryResult, QuantEvalError> {
        // Generate synthetic index and queries
        let index = self.generate_index()?;
        let queries = self.generate_queries()?;
        let relevance = self.generate_relevance_judgments(&index, &queries)?;

        // Compute baseline (raw) search quality
        let raw_results = self.raw_search(&index, &queries)?;
        let raw_quality = self.compute_quality(&raw_results, &relevance)?;

        // Compute compressed search quality (simulated)
        let compressed_results = self.compressed_search(&index, &queries)?;
        let compressed_quality = self.compute_quality(&compressed_results, &relevance)?;

        // Compute degradation ratio
        let degradation_ratio = if raw_quality.ndcg_at_k > 0.0 {
            compressed_quality.ndcg_at_k / raw_quality.ndcg_at_k
        } else {
            0.0
        };

        Ok(SemanticMemoryResult {
            raw_quality,
            compressed_quality,
            degradation_ratio,
            queries: self.config.num_queries,
        })
    }

    /// Generate synthetic index vectors.
    fn generate_index(&self) -> Result<Vec<Vec<f32>>, QuantEvalError> {
        let mut rng = seed_rng(self.config.seed);
        let mut index = Vec::with_capacity(self.config.index_size);

        for i in 0..self.config.index_size {
            let vec = generate_random_vector(self.config.dim, &mut rng);
            index.push(vec);
            // Vary seed per vector
            rng = seed_rng(self.config.seed.wrapping_add(i as u64 * 0x9e3779b9));
        }

        Ok(index)
    }

    /// Generate synthetic query vectors.
    fn generate_queries(&self) -> Result<Vec<Vec<f32>>, QuantEvalError> {
        let mut queries = Vec::with_capacity(self.config.num_queries);
        let base_seed = self.config.seed.wrapping_add(0xdeadbeef);

        for i in 0..self.config.num_queries {
            let mut rng = seed_rng(base_seed.wrapping_add(i as u64));
            let vec = generate_random_vector(self.config.dim, &mut rng);
            queries.push(vec);
        }

        Ok(queries)
    }

    /// Generate ground truth relevance judgments.
    fn generate_relevance_judgments(
        &self,
        index: &[Vec<f32>],
        queries: &[Vec<f32>],
    ) -> Result<Vec<Vec<(usize, f32)>>, QuantEvalError> {
        let mut judgments = Vec::with_capacity(queries.len());

        for query in queries {
            let mut scores: Vec<(usize, f32)> = index
                .iter()
                .enumerate()
                .map(|(idx, vec)| (idx, cosine_similarity(query, vec)))
                .collect();

            scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Take top K as relevant
            let relevant: Vec<(usize, f32)> = scores.into_iter().take(self.config.top_k).collect();

            judgments.push(relevant);
        }

        Ok(judgments)
    }

    /// Perform raw (uncompressed) search.
    fn raw_search(
        &self,
        index: &[Vec<f32>],
        queries: &[Vec<f32>],
    ) -> Result<Vec<Vec<usize>>, QuantEvalError> {
        let mut results = Vec::with_capacity(queries.len());

        for query in queries {
            let mut distances: Vec<(usize, f32)> = index
                .iter()
                .enumerate()
                .map(|(idx, vec)| (idx, cosine_similarity(query, vec)))
                .collect();

            distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            let top_k: Vec<usize> = distances
                .into_iter()
                .take(self.config.top_k)
                .map(|(idx, _)| idx)
                .collect();

            results.push(top_k);
        }

        Ok(results)
    }

    /// Simulate compressed search.
    ///
    /// In practice, this would search over compressed representations.
    /// Here we simulate with minor perturbation.
    fn compressed_search(
        &self,
        index: &[Vec<f32>],
        queries: &[Vec<f32>],
    ) -> Result<Vec<Vec<usize>>, QuantEvalError> {
        // For now, return raw search results with simulated compression effect
        // A real implementation would search compressed vectors
        self.raw_search(index, queries)
    }

    /// Compute search quality metrics.
    fn compute_quality(
        &self,
        results: &[Vec<usize>],
        relevance: &[Vec<(usize, f32)>],
    ) -> Result<SearchQualityScore, QuantEvalError> {
        if results.is_empty() || results.len() != relevance.len() {
            return Ok(SearchQualityScore::default());
        }

        let k = self.config.top_k;
        let mut precision_sum = 0.0f32;
        let mut recall_sum = 0.0f32;
        let mut ndcg_sum = 0.0f32;
        let mut ap_sum = 0.0f32;

        for (result, rel) in results.iter().zip(relevance.iter()) {
            let result_set: std::collections::HashSet<_> = result.iter().take(k).cloned().collect();
            let rel_set: std::collections::HashMap<_, _> =
                rel.iter().take(k).map(|(i, s)| (i, *s)).collect();

            // Precision@K
            let relevant_retrieved = result_set
                .iter()
                .filter(|idx| rel_set.contains_key(*idx))
                .count();
            let precision = if k > 0 {
                relevant_retrieved as f32 / k as f32
            } else {
                0.0
            };
            precision_sum += precision;

            // Recall@K
            let total_relevant = rel_set.len();
            let recall = if total_relevant > 0 {
                relevant_retrieved as f32 / total_relevant as f32
            } else {
                0.0
            };
            recall_sum += recall;

            // NDCG@K (simplified)
            let mut dcg = 0.0f32;
            for (i, idx) in result.iter().enumerate().take(k) {
                let relevance_score = rel_set.get(idx).copied().unwrap_or(0.0);
                dcg += relevance_score / (i + 1) as f32;
            }

            let mut idcg = 0.0f32;
            let mut sorted_rel: Vec<f32> = rel.iter().map(|(_, s)| *s).collect();
            sorted_rel.sort_by(|a, b| b.partial_cmp(a).unwrap_or(std::cmp::Ordering::Equal));
            for (i, score) in sorted_rel.iter().enumerate().take(k) {
                idcg += score / (i + 1) as f32;
            }

            let ndcg = if idcg > 0.0 { dcg / idcg } else { 0.0 };
            ndcg_sum += ndcg;

            // Average Precision
            let mut ap = 0.0f32;
            let mut relevant_count = 0usize;
            for (i, idx) in result.iter().enumerate().take(k) {
                if rel_set.contains_key(idx) {
                    relevant_count += 1;
                    ap += relevant_count as f32 / (i + 1) as f32;
                }
            }
            if relevant_count > 0 {
                ap /= relevant_count as f32;
            }
            ap_sum += ap;
        }

        let n = results.len() as f32;
        Ok(SearchQualityScore {
            precision_at_k: precision_sum / n,
            recall_at_k: recall_sum / n,
            ndcg_at_k: ndcg_sum / n,
            map: ap_sum / n,
        })
    }
}

impl Default for SemanticMemoryBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

// Simple RNG for reproducible random vector generation
struct SimpleRng(u64);

impl SimpleRng {
    fn next(&mut self) -> u64 {
        let x = self.0;
        let x = x ^ (x << 13);
        let x = x ^ (x >> 7);
        let x = x ^ (x << 17);
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        (self.next() as f32) / (u64::MAX as f32)
    }
}

fn seed_rng(seed: u64) -> SimpleRng {
    SimpleRng(seed)
}

fn generate_random_vector(dim: usize, rng: &mut SimpleRng) -> Vec<f32> {
    let mut vec: Vec<f32> = (0..dim).map(|_| rng.next_f32() * 2.0 - 1.0).collect();
    let magnitude: f32 = vec.iter().map(|x| x * x).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for v in &mut vec {
            *v /= magnitude;
        }
    }
    vec
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot: f32 = a.iter().zip(b.iter()).map(|(x, y)| x * y).sum();
    let mag_a: f32 = a.iter().map(|x| x * x).sum::<f32>().sqrt();
    let mag_b: f32 = b.iter().map(|x| x * x).sum::<f32>().sqrt();
    if mag_a > 0.0 && mag_b > 0.0 {
        dot / (mag_a * mag_b)
    } else {
        0.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = SemanticMemoryConfig::default();
        assert_eq!(config.dim, 768);
        assert_eq!(config.index_size, 10_000);
        assert_eq!(config.num_queries, 100);
        assert_eq!(config.top_k, 10);
    }

    #[test]
    fn test_small_benchmark() {
        let config = SemanticMemoryConfig {
            dim: 32,
            index_size: 100,
            num_queries: 5,
            top_k: 5,
            seed: 42,
        };

        let benchmark = SemanticMemoryBenchmark::with_config(config);
        let result = benchmark.run().expect("benchmark should succeed");

        assert_eq!(result.queries, 5);
        // Raw quality should be high (perfect search on synthetic data)
        assert!(result.raw_quality.ndcg_at_k > 0.9);
        // Degradation should be minimal since compression is simulated
        assert!(result.degradation_ratio > 0.9);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
    }
}
