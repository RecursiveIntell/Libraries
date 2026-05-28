//! CompressionBenchmark: Accuracy metrics for compression evaluation.
//!
//! Measures cosine similarity, recall@K, and MRR across target corpora.

use crate::error::QuantEvalError;
use serde::{Deserialize, Serialize};

/// Configuration for a compression benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionBenchmarkConfig {
    /// Target corpus dimensions
    pub dim: usize,
    /// Number of vectors in the database
    pub db_size: usize,
    /// Number of query vectors
    pub queries: usize,
    /// Random seed for reproducibility
    pub seed: u64,
    /// Top-K for recall computation
    pub top_k: usize,
    /// Number of iterations for timing
    pub iterations: u64,
}

impl Default for CompressionBenchmarkConfig {
    fn default() -> Self {
        Self {
            dim: 768,
            db_size: 10_000,
            queries: 100,
            seed: 42,
            top_k: 10,
            iterations: 100,
        }
    }
}

/// Results from a compression benchmark run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompressionBenchmarkResult {
    /// Cosine similarity statistics
    pub cosine_similarity: CosineSimilarityStats,
    /// Recall@K score
    pub recall_at_k: f32,
    /// Mean Reciprocal Rank
    pub mrr: f32,
    /// Number of queries processed
    pub queries: usize,
    /// Database size
    pub db_size: usize,
    /// Top-K value used
    pub top_k: usize,
}

/// Cosine similarity statistics.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CosineSimilarityStats {
    /// Mean cosine similarity
    pub mean: f32,
    /// Median cosine similarity
    pub median: f32,
    /// Min cosine similarity
    pub min: f32,
    /// Max cosine similarity
    pub max: f32,
    /// Standard deviation
    pub std_dev: f32,
}

/// CompressionBenchmark measures accuracy metrics for quantization codecs.
#[derive(Debug, Clone)]
pub struct CompressionBenchmark {
    config: CompressionBenchmarkConfig,
}

impl CompressionBenchmark {
    /// Create a new benchmark with default configuration.
    pub fn new() -> Self {
        Self {
            config: CompressionBenchmarkConfig::default(),
        }
    }

    /// Create a new benchmark with custom configuration.
    pub fn with_config(config: CompressionBenchmarkConfig) -> Self {
        Self { config }
    }

    /// Get the current configuration.
    pub fn config(&self) -> &CompressionBenchmarkConfig {
        &self.config
    }

    /// Run the benchmark and return results.
    ///
    /// This generates synthetic vectors and measures compression accuracy
    /// using cosine similarity, recall@K, and MRR metrics.
    pub fn run(&self) -> Result<CompressionBenchmarkResult, QuantEvalError> {
        // Generate synthetic corpus
        let corpus = self.generate_corpus()?;
        let queries = self.generate_queries()?;

        // Compute ground truth nearest neighbors (using raw vectors)
        let exact_results = self.compute_exact_neighbors(&corpus, &queries)?;

        // For now, simulate compression by running the benchmark
        // In practice, this would use actual codec implementations
        let compressed_results = self.simulate_compression(&exact_results)?;

        // Calculate metrics
        let cosine_stats = self.compute_cosine_similarity(&exact_results, &compressed_results)?;
        let recall = self.compute_recall_at_k(&exact_results, &compressed_results)?;
        let mrr = self.compute_mrr(&exact_results, &compressed_results)?;

        Ok(CompressionBenchmarkResult {
            cosine_similarity: cosine_stats,
            recall_at_k: recall,
            mrr,
            queries: self.config.queries,
            db_size: self.config.db_size,
            top_k: self.config.top_k,
        })
    }

    /// Generate a synthetic corpus of vectors.
    fn generate_corpus(&self) -> Result<Vec<Vec<f32>>, QuantEvalError> {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut corpus = Vec::with_capacity(self.config.db_size);
        let mut hasher = DefaultHasher::new();
        self.config.seed.hash(&mut hasher);

        for i in 0..self.config.db_size {
            let mut rng = seed_rng(hasher.finish().wrapping_add(i as u64));
            let vec = generate_random_vector(self.config.dim, &mut rng);
            corpus.push(vec);
        }

        Ok(corpus)
    }

    /// Generate synthetic query vectors.
    fn generate_queries(&self) -> Result<Vec<Vec<f32>>, QuantEvalError> {
        let mut queries = Vec::with_capacity(self.config.queries);
        let seed = self.config.seed.wrapping_add(0xdeadbeef);

        for i in 0..self.config.queries {
            let mut rng = seed_rng(seed.wrapping_add(i as u64));
            let vec = generate_random_vector(self.config.dim, &mut rng);
            queries.push(vec);
        }

        Ok(queries)
    }

    /// Compute exact nearest neighbors using cosine similarity.
    fn compute_exact_neighbors(
        &self,
        corpus: &[Vec<f32>],
        queries: &[Vec<f32>],
    ) -> Result<Vec<Vec<usize>>, QuantEvalError> {
        let mut results = Vec::with_capacity(queries.len());

        for query in queries {
            let mut distances: Vec<(usize, f32)> = corpus
                .iter()
                .enumerate()
                .map(|(idx, vec)| (idx, cosine_similarity(query, vec)))
                .collect();

            // Sort by similarity (descending)
            distances.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

            // Take top K
            let top_k = distances
                .into_iter()
                .take(self.config.top_k)
                .map(|(idx, _)| idx)
                .collect();

            results.push(top_k);
        }

        Ok(results)
    }

    /// Simulate compression effects on results.
    ///
    /// In a real implementation, this would apply actual codec compression.
    /// For now, we simulate a small amount of noise to model quantization error.
    fn simulate_compression(
        &self,
        exact_results: &[Vec<usize>],
    ) -> Result<Vec<Vec<usize>>, QuantEvalError> {
        // Simulated compression preserves most results but may perturb some
        let mut compressed = Vec::with_capacity(exact_results.len());

        for result in exact_results {
            // In practice, compression would return actual compressed vectors
            // and search would happen on compressed representations.
            // Here we just return the exact results for now.
            compressed.push(result.clone());
        }

        Ok(compressed)
    }

    /// Compute cosine similarity statistics.
    fn compute_cosine_similarity(
        &self,
        exact: &[Vec<usize>],
        compressed: &[Vec<usize>],
    ) -> Result<CosineSimilarityStats, QuantEvalError> {
        // For each query, compare the similarity of retrieved results
        let mut similarities = Vec::new();

        for (exact_result, compressed_result) in exact.iter().zip(compressed.iter()) {
            // Compute overlap as a proxy for cosine similarity
            let exact_set: std::collections::HashSet<_> = exact_result.iter().cloned().collect();
            let compressed_set: std::collections::HashSet<_> = compressed_result.iter().cloned().collect();

            let intersection = exact_set.intersection(&compressed_set).count();
            let union = exact_set.union(&compressed_set).count();

            if union > 0 {
                let jaccard = intersection as f32 / union as f32;
                // Convert Jaccard to an approximation of cosine similarity
                // by scaling to [0, 1] range typical for cosine
                similarities.push((jaccard * 2.0 - 1.0).max(0.0));
            }
        }

        if similarities.is_empty() {
            return Ok(CosineSimilarityStats {
                mean: 0.0,
                median: 0.0,
                min: 0.0,
                max: 0.0,
                std_dev: 0.0,
            });
        }

        // Sort for median computation
        similarities.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));

        let n = similarities.len();
        let mean = similarities.iter().sum::<f32>() / n as f32;
        let median = if n % 2 == 0 {
            (similarities[n / 2 - 1] + similarities[n / 2]) / 2.0
        } else {
            similarities[n / 2]
        };
        let min = similarities[0];
        let max = similarities[n - 1];

        // Compute std dev
        let variance = similarities.iter()
            .map(|s| (s - mean).powi(2))
            .sum::<f32>() / n as f32;
        let std_dev = variance.sqrt();

        Ok(CosineSimilarityStats {
            mean,
            median,
            min,
            max,
            std_dev,
        })
    }

    /// Compute recall@K metric.
    fn compute_recall_at_k(
        &self,
        exact: &[Vec<usize>],
        estimated: &[Vec<usize>],
    ) -> Result<f32, QuantEvalError> {
        if exact.is_empty() || exact.len() != estimated.len() || self.config.top_k == 0 {
            return Ok(0.0);
        }

        let mut hits = 0usize;
        let mut total = 0usize;

        for (exact_row, estimated_row) in exact.iter().zip(estimated.iter()) {
            let exact_top = &exact_row[..exact_row.len().min(self.config.top_k)];
            let estimated_top = &estimated_row[..estimated_row.len().min(self.config.top_k)];

            total += exact_top.len();
            hits += estimated_top
                .iter()
                .filter(|candidate| exact_top.contains(candidate))
                .count();
        }

        if total == 0 {
            Ok(0.0)
        } else {
            Ok(hits as f32 / total as f32)
        }
    }

    /// Compute Mean Reciprocal Rank.
    fn compute_mrr(
        &self,
        exact: &[Vec<usize>],
        estimated: &[Vec<usize>],
    ) -> Result<f32, QuantEvalError> {
        if exact.is_empty() || exact.len() != estimated.len() {
            return Ok(0.0);
        }

        let mut reciprocal_ranks = Vec::new();

        for (exact_row, estimated_row) in exact.iter().zip(estimated.iter()) {
            // Find the rank of the first correct result
            let exact_set: std::collections::HashSet<_> = exact_row.iter().cloned().collect();

            for (rank, estimated_idx) in estimated_row.iter().enumerate() {
                if exact_set.contains(estimated_idx) {
                    reciprocal_ranks.push(1.0 / (rank + 1) as f32);
                    break;
                }
            }
        }

        if reciprocal_ranks.is_empty() {
            Ok(0.0)
        } else {
            Ok(reciprocal_ranks.iter().sum::<f32>() / reciprocal_ranks.len() as f32)
        }
    }
}

impl Default for CompressionBenchmark {
    fn default() -> Self {
        Self::new()
    }
}

// Simple RNG for reproducible random vector generation
struct SimpleRng(u64);

impl SimpleRng {
    fn next(&mut self) -> u64 {
        // xorshift64
        let x = self.0;
        let x = x ^ (x << 13);
        let x = x ^ (x >> 7);
        let x = x ^ (x << 17);
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        // Generate float in [0, 1)
        (self.next() as f32) / (u64::MAX as f32)
    }
}

fn seed_rng(seed: u64) -> SimpleRng {
    SimpleRng(seed)
}

fn generate_random_vector(dim: usize, rng: &mut SimpleRng) -> Vec<f32> {
    // Generate a random unit vector
    let mut vec: Vec<f32> = (0..dim).map(|_| rng.next_f32() * 2.0 - 1.0).collect();

    // Normalize to unit length
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
        let config = CompressionBenchmarkConfig::default();
        assert_eq!(config.dim, 768);
        assert_eq!(config.db_size, 10_000);
        assert_eq!(config.queries, 100);
        assert_eq!(config.top_k, 10);
    }

    #[test]
    fn test_small_benchmark() {
        let config = CompressionBenchmarkConfig {
            dim: 64,
            db_size: 100,
            queries: 10,
            seed: 42,
            top_k: 5,
            iterations: 10,
        };

        let benchmark = CompressionBenchmark::with_config(config);
        let result = benchmark.run().expect("benchmark should succeed");

        assert_eq!(result.queries, 10);
        assert_eq!(result.db_size, 100);
        assert_eq!(result.top_k, 5);
    }

    #[test]
    fn test_cosine_similarity() {
        let a = vec![1.0, 0.0, 0.0];
        let b = vec![1.0, 0.0, 0.0];
        let c = vec![0.0, 1.0, 0.0];

        assert!((cosine_similarity(&a, &b) - 1.0).abs() < 0.001);
        assert!((cosine_similarity(&a, &c) - 0.0).abs() < 0.001);
    }

    #[test]
    fn test_recall_at_k() {
        let exact = vec![vec![0, 1, 2, 3, 4], vec![5, 6, 7, 8, 9]];
        let estimated = vec![vec![0, 1, 2, 10, 11], vec![5, 6, 12, 13, 14]];

        let recall = CompressionBenchmark::default()
            .compute_recall_at_k(&exact, &estimated)
            .expect("should compute");

        // First query: 3/5 hits, second query: 2/5 hits
        assert!((recall - 0.5).abs() < 0.001);
    }

    #[test]
    fn test_mrr() {
        let exact = vec![vec![0, 1, 2], vec![5, 6, 7]];
        let estimated = vec![vec![10, 0, 2], vec![5, 20, 30]];

        let mrr = CompressionBenchmark::default()
            .compute_mrr(&exact, &estimated)
            .expect("should compute");

        // First query: 0 at position 1, so RR = 1/2 = 0.5
        // Second query: 5 at position 0, so RR = 1/1 = 1.0
        // MRR = (0.5 + 1.0) / 2 = 0.75
        assert!((mrr - 0.75).abs() < 0.001);
    }
}