//! HyperQuant retrieval-quality and latency benchmark.
//!
//! This harness is intentionally synthetic and receipt-backed. It measures the
//! current HyperQuant primitive on clustered embeddings, comparing exact raw
//! search against search over reconstructed HyperQuant vectors. It is not BEIR,
//! TREC RAG, production retrieval evidence, or model-quality proof.

use crate::QuantEvalError;
use hyperquant::{estimate_best_rice_profile, HyperQuantConfig, LatticeKind};
use serde::{Deserialize, Serialize};
use std::time::Instant;

/// Configuration for synthetic HyperQuant retrieval benchmarking.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRetrievalBenchmarkConfig {
    /// Embedding dimension.
    pub dim: usize,
    /// Number of document vectors.
    pub docs: usize,
    /// Number of query vectors.
    pub queries: usize,
    /// Number of synthetic clusters.
    pub clusters: usize,
    /// Top-K retrieval depth.
    pub top_k: usize,
    /// Deterministic fixture seed.
    pub seed: u64,
    /// HyperQuant scale.
    pub scale: f32,
    /// HyperQuant lattice kind.
    pub lattice: LatticeKind,
}

impl Default for HyperQuantRetrievalBenchmarkConfig {
    fn default() -> Self {
        Self {
            dim: 64,
            docs: 512,
            queries: 32,
            clusters: 16,
            top_k: 10,
            seed: 42,
            scale: 16.0,
            lattice: LatticeKind::D4,
        }
    }
}

/// Latency percentile summary in nanoseconds.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct LatencySummaryNs {
    /// 50th percentile latency.
    pub p50: u128,
    /// 95th percentile latency.
    pub p95: u128,
    /// Maximum latency.
    pub max: u128,
    /// Mean latency.
    pub mean: f64,
}

/// Retrieval-quality metrics for HyperQuant vs exact raw retrieval.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRetrievalQuality {
    /// Mean fraction of exact top-K recovered by HyperQuant top-K.
    pub recall_at_k: f32,
    /// Mean Jaccard overlap between exact top-K and HyperQuant top-K.
    pub top_k_overlap: f32,
    /// NDCG@K using exact raw ranking as graded relevance.
    pub ndcg_at_k: f32,
    /// Fraction of queries whose exact top-1 appears in HyperQuant top-K.
    pub exact_rerank_recovery_at_1: f32,
}

/// Floating-point error summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ErrorSummary {
    /// Mean absolute error.
    pub mean: f32,
    /// 95th percentile absolute error.
    pub p95: f32,
    /// Maximum absolute error.
    pub max: f32,
}

/// Rank drift summary.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RankDriftSummary {
    /// Mean drift of exact top-1 in HyperQuant ranking.
    pub mean: f32,
    /// 95th percentile drift.
    pub p95: f32,
    /// Maximum drift.
    pub max: usize,
}

/// Full synthetic HyperQuant retrieval benchmark receipt.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRetrievalBenchmarkReceipt {
    /// Benchmark config.
    pub config: HyperQuantRetrievalBenchmarkConfig,
    /// Lattice used by HyperQuant.
    pub lattice: LatticeKind,
    /// Query count.
    pub queries: usize,
    /// Document count.
    pub docs: usize,
    /// Top-K retrieval depth.
    pub top_k: usize,
    /// Documents rejected during quantization.
    pub rejected_docs: usize,
    /// Queries rejected during quantization.
    pub rejected_queries: usize,
    /// One-time document quantization/build latency.
    pub build_latency_ns: u128,
    /// Exact raw-search latency summary.
    pub raw_latency_ns: LatencySummaryNs,
    /// HyperQuant reconstructed-vector search latency summary.
    pub hyperquant_latency_ns: LatencySummaryNs,
    /// Retrieval quality metrics.
    pub quality: HyperQuantRetrievalQuality,
    /// Score error over HyperQuant top-K candidates.
    pub score_error: ErrorSummary,
    /// Rank drift for exact top-1 in HyperQuant ranking.
    pub rank_drift: RankDriftSummary,
    /// Raw f32 document bytes.
    pub raw_bytes: usize,
    /// Estimated HyperQuant document bytes using Rice accounting.
    pub hyperquant_estimated_bytes: usize,
    /// Raw / HyperQuant estimated byte ratio.
    pub compression_ratio: f32,
    /// Pass/fail thresholds recorded with the receipt.
    pub thresholds: HyperQuantRetrievalThresholds,
    /// Whether this synthetic benchmark passes the configured thresholds.
    pub passed: bool,
    /// Human-readable blockers when thresholds are not met.
    pub blockers: Vec<String>,
    /// Explicit claim boundary.
    pub claim_boundary: String,
}

/// Pass/fail thresholds for the synthetic benchmark.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRetrievalThresholds {
    /// Minimum top-K overlap.
    pub min_top_k_overlap: f32,
    /// Minimum exact top-1 recovery in HyperQuant top-K.
    pub min_exact_rerank_recovery_at_1: f32,
}

impl Default for HyperQuantRetrievalThresholds {
    fn default() -> Self {
        Self {
            min_top_k_overlap: 0.30,
            min_exact_rerank_recovery_at_1: 0.80,
        }
    }
}

#[derive(Debug, Clone)]
struct QuantizedDoc {
    reconstructed: Vec<f32>,
    estimated_bytes: usize,
}

#[derive(Debug, Clone)]
struct RankedQuery {
    ranking: Vec<(usize, f32)>,
    elapsed_ns: u128,
}

#[derive(Debug, Clone)]
struct RetrievalFixture {
    centers: Vec<Vec<f32>>,
    docs: Vec<Vec<f32>>,
    queries: Vec<Vec<f32>>,
}

/// Run the synthetic HyperQuant retrieval-quality and latency benchmark.
pub fn run_hyperquant_retrieval_benchmark(
    config: &HyperQuantRetrievalBenchmarkConfig,
) -> Result<HyperQuantRetrievalBenchmarkReceipt, QuantEvalError> {
    validate_config(config)?;

    let fixture = generate_clustered_fixture(config);
    debug_assert_eq!(fixture.centers.len(), config.clusters);

    let build_start = Instant::now();
    let (quantized_docs, rejected_docs, hyperquant_estimated_bytes) =
        quantize_docs(config, &fixture.docs);
    let build_latency_ns = build_start.elapsed().as_nanos();

    if quantized_docs.len() != fixture.docs.len() {
        return Err(QuantEvalError::Codec(format!(
            "{} docs rejected during quantization",
            rejected_docs
        )));
    }

    let mut raw_rankings = Vec::with_capacity(fixture.queries.len());
    let mut hyperquant_rankings = Vec::with_capacity(fixture.queries.len());
    let mut raw_latencies = Vec::with_capacity(fixture.queries.len());
    let mut hyperquant_latencies = Vec::with_capacity(fixture.queries.len());
    let mut rejected_queries = 0usize;

    for query in &fixture.queries {
        let raw = timed_raw_search(query, &fixture.docs);
        raw_latencies.push(raw.elapsed_ns);
        raw_rankings.push(raw.ranking);

        match timed_hyperquant_search(config, query, &quantized_docs) {
            Ok(hq) => {
                hyperquant_latencies.push(hq.elapsed_ns);
                hyperquant_rankings.push(hq.ranking);
            }
            Err(_) => rejected_queries += 1,
        }
    }

    if rejected_queries > 0 || raw_rankings.len() != hyperquant_rankings.len() {
        return Err(QuantEvalError::Codec(format!(
            "{} queries rejected during quantization",
            rejected_queries
        )));
    }

    let quality = compute_quality(config.top_k, &raw_rankings, &hyperquant_rankings);
    let score_error = compute_score_error(config.top_k, &raw_rankings, &hyperquant_rankings);
    let rank_drift = compute_rank_drift(&raw_rankings, &hyperquant_rankings);
    let raw_bytes = fixture.docs.len() * config.dim * core::mem::size_of::<f32>();
    let compression_ratio = raw_bytes as f32 / hyperquant_estimated_bytes.max(1) as f32;
    let thresholds = HyperQuantRetrievalThresholds::default();
    let mut blockers = Vec::new();
    if quality.top_k_overlap < thresholds.min_top_k_overlap {
        blockers.push(format!(
            "top_k_overlap {} < {}",
            quality.top_k_overlap, thresholds.min_top_k_overlap
        ));
    }
    if quality.exact_rerank_recovery_at_1 < thresholds.min_exact_rerank_recovery_at_1 {
        blockers.push(format!(
            "exact_rerank_recovery_at_1 {} < {}",
            quality.exact_rerank_recovery_at_1, thresholds.min_exact_rerank_recovery_at_1
        ));
    }

    Ok(HyperQuantRetrievalBenchmarkReceipt {
        config: *config,
        lattice: config.lattice,
        queries: config.queries,
        docs: config.docs,
        top_k: config.top_k,
        rejected_docs,
        rejected_queries,
        build_latency_ns,
        raw_latency_ns: summarize_latency(&raw_latencies),
        hyperquant_latency_ns: summarize_latency(&hyperquant_latencies),
        quality,
        score_error,
        rank_drift,
        raw_bytes,
        hyperquant_estimated_bytes,
        compression_ratio,
        thresholds,
        passed: blockers.is_empty(),
        blockers,
        claim_boundary: "synthetic clustered-fixture latency and retrieval-quality evidence only; not BEIR/TREC RAG, production, model-quality, or superiority evidence".to_string(),
    })
}

fn validate_config(config: &HyperQuantRetrievalBenchmarkConfig) -> Result<(), QuantEvalError> {
    if config.dim == 0 {
        return Err(QuantEvalError::InvalidCorpus("dim must be > 0".to_string()));
    }
    if config.docs == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "docs must be > 0".to_string(),
        ));
    }
    if config.queries == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "queries must be > 0".to_string(),
        ));
    }
    if config.clusters == 0 || config.clusters > config.docs {
        return Err(QuantEvalError::InvalidCorpus(
            "clusters must be in 1..=docs".to_string(),
        ));
    }
    if config.top_k == 0 || config.top_k > config.docs {
        return Err(QuantEvalError::InvalidCorpus(
            "top_k must be in 1..=docs".to_string(),
        ));
    }
    Ok(())
}

fn quantize_docs(
    config: &HyperQuantRetrievalBenchmarkConfig,
    docs: &[Vec<f32>],
) -> (Vec<QuantizedDoc>, usize, usize) {
    let hq_config = HyperQuantConfig::new(config.lattice, config.scale);
    let mut quantized = Vec::with_capacity(docs.len());
    let mut rejected = 0usize;
    let mut estimated_bytes = 0usize;

    for doc in docs {
        match hq_config.quantize(doc) {
            Ok(result) => {
                let bytes = estimate_best_rice_profile(&result.codes, 0..=8)
                    .map(|profile| profile.encoded_bytes)
                    .unwrap_or(result.codes.len() * core::mem::size_of::<i16>());
                estimated_bytes += bytes;
                quantized.push(QuantizedDoc {
                    reconstructed: normalize(result.reconstructed),
                    estimated_bytes: bytes,
                });
            }
            Err(_) => rejected += 1,
        }
    }

    let accounted_bytes = quantized
        .iter()
        .map(|doc| doc.estimated_bytes)
        .sum::<usize>();
    debug_assert_eq!(estimated_bytes, accounted_bytes);
    (quantized, rejected, estimated_bytes)
}

fn timed_raw_search(query: &[f32], docs: &[Vec<f32>]) -> RankedQuery {
    let start = Instant::now();
    let ranking = rank_by_score(query, docs.iter().map(Vec::as_slice));
    RankedQuery {
        ranking,
        elapsed_ns: start.elapsed().as_nanos(),
    }
}

fn timed_hyperquant_search(
    config: &HyperQuantRetrievalBenchmarkConfig,
    query: &[f32],
    docs: &[QuantizedDoc],
) -> Result<RankedQuery, QuantEvalError> {
    let hq_config = HyperQuantConfig::new(config.lattice, config.scale);
    let start = Instant::now();
    let query = hq_config
        .quantize(query)
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
    let query = normalize(query.reconstructed);
    let ranking = rank_by_score(&query, docs.iter().map(|doc| doc.reconstructed.as_slice()));
    Ok(RankedQuery {
        ranking,
        elapsed_ns: start.elapsed().as_nanos(),
    })
}

fn rank_by_score<'a, I>(query: &[f32], docs: I) -> Vec<(usize, f32)>
where
    I: IntoIterator<Item = &'a [f32]>,
{
    let mut ranking: Vec<(usize, f32)> = docs
        .into_iter()
        .enumerate()
        .map(|(idx, doc)| (idx, dot(query, doc)))
        .collect();
    ranking.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    ranking
}

fn compute_quality(
    top_k: usize,
    raw_rankings: &[Vec<(usize, f32)>],
    hyperquant_rankings: &[Vec<(usize, f32)>],
) -> HyperQuantRetrievalQuality {
    let mut recall_sum = 0.0f32;
    let mut overlap_sum = 0.0f32;
    let mut ndcg_sum = 0.0f32;
    let mut recovery_hits = 0usize;

    for (raw, hq) in raw_rankings.iter().zip(hyperquant_rankings.iter()) {
        let raw_top: Vec<usize> = raw.iter().take(top_k).map(|(idx, _)| *idx).collect();
        let hq_top: Vec<usize> = hq.iter().take(top_k).map(|(idx, _)| *idx).collect();
        let hits = hq_top.iter().filter(|idx| raw_top.contains(idx)).count();
        recall_sum += hits as f32 / raw_top.len().max(1) as f32;
        let union = raw_top
            .iter()
            .chain(hq_top.iter())
            .copied()
            .collect::<std::collections::HashSet<_>>()
            .len();
        overlap_sum += if union == 0 {
            0.0
        } else {
            hits as f32 / union as f32
        };
        ndcg_sum += ndcg_at_k(top_k, raw, hq);
        if raw_top
            .first()
            .is_some_and(|exact_top_1| hq_top.contains(exact_top_1))
        {
            recovery_hits += 1;
        }
    }

    let n = raw_rankings.len().max(1) as f32;
    HyperQuantRetrievalQuality {
        recall_at_k: recall_sum / n,
        top_k_overlap: overlap_sum / n,
        ndcg_at_k: ndcg_sum / n,
        exact_rerank_recovery_at_1: recovery_hits as f32 / n,
    }
}

fn ndcg_at_k(top_k: usize, raw: &[(usize, f32)], hq: &[(usize, f32)]) -> f32 {
    let relevance = raw
        .iter()
        .take(top_k)
        .enumerate()
        .map(|(rank, (idx, _))| (*idx, (top_k - rank) as f32))
        .collect::<std::collections::HashMap<_, _>>();
    let dcg = hq
        .iter()
        .take(top_k)
        .enumerate()
        .map(|(rank, (idx, _))| {
            relevance.get(idx).copied().unwrap_or(0.0) / ((rank + 2) as f32).log2()
        })
        .sum::<f32>();
    let idcg = (0..top_k)
        .map(|rank| (top_k - rank) as f32 / ((rank + 2) as f32).log2())
        .sum::<f32>();
    if idcg > 0.0 {
        dcg / idcg
    } else {
        0.0
    }
}

fn compute_score_error(
    top_k: usize,
    raw_rankings: &[Vec<(usize, f32)>],
    hyperquant_rankings: &[Vec<(usize, f32)>],
) -> ErrorSummary {
    let mut errors = Vec::new();
    for (raw, hq) in raw_rankings.iter().zip(hyperquant_rankings.iter()) {
        for (doc_id, approx_score) in hq.iter().take(top_k) {
            if let Some((_, exact_score)) = raw.iter().find(|(raw_doc_id, _)| raw_doc_id == doc_id)
            {
                errors.push((approx_score - exact_score).abs());
            }
        }
    }
    summarize_errors(&mut errors)
}

fn compute_rank_drift(
    raw_rankings: &[Vec<(usize, f32)>],
    hyperquant_rankings: &[Vec<(usize, f32)>],
) -> RankDriftSummary {
    let mut drifts = Vec::new();
    for (raw, hq) in raw_rankings.iter().zip(hyperquant_rankings.iter()) {
        if let Some((exact_top_1, _)) = raw.first() {
            let hq_rank = hq
                .iter()
                .position(|(doc_id, _)| doc_id == exact_top_1)
                .unwrap_or(hq.len());
            drifts.push(hq_rank);
        }
    }
    drifts.sort_unstable();
    if drifts.is_empty() {
        return RankDriftSummary {
            mean: 0.0,
            p95: 0.0,
            max: 0,
        };
    }
    let mean = drifts.iter().sum::<usize>() as f32 / drifts.len() as f32;
    let p95 = percentile_usize(&drifts, 0.95) as f32;
    let max = *drifts.last().unwrap_or(&0);
    RankDriftSummary { mean, p95, max }
}

fn summarize_latency(values: &[u128]) -> LatencySummaryNs {
    if values.is_empty() {
        return LatencySummaryNs {
            p50: 0,
            p95: 0,
            max: 0,
            mean: 0.0,
        };
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let mean = sorted.iter().sum::<u128>() as f64 / sorted.len() as f64;
    LatencySummaryNs {
        p50: percentile_u128(&sorted, 0.50),
        p95: percentile_u128(&sorted, 0.95),
        max: *sorted.last().unwrap_or(&0),
        mean,
    }
}

fn summarize_errors(values: &mut [f32]) -> ErrorSummary {
    if values.is_empty() {
        return ErrorSummary {
            mean: 0.0,
            p95: 0.0,
            max: 0.0,
        };
    }
    values.sort_by(|a, b| a.partial_cmp(b).unwrap_or(std::cmp::Ordering::Equal));
    let mean = values.iter().sum::<f32>() / values.len() as f32;
    ErrorSummary {
        mean,
        p95: percentile_f32(values, 0.95),
        max: *values.last().unwrap_or(&0.0),
    }
}

fn percentile_u128(values: &[u128], percentile: f32) -> u128 {
    let idx = percentile_index(values.len(), percentile);
    values[idx]
}

fn percentile_usize(values: &[usize], percentile: f32) -> usize {
    let idx = percentile_index(values.len(), percentile);
    values[idx]
}

fn percentile_f32(values: &[f32], percentile: f32) -> f32 {
    let idx = percentile_index(values.len(), percentile);
    values[idx]
}

fn percentile_index(len: usize, percentile: f32) -> usize {
    (((len.saturating_sub(1)) as f32) * percentile).ceil() as usize
}

fn generate_clustered_fixture(config: &HyperQuantRetrievalBenchmarkConfig) -> RetrievalFixture {
    let mut rng = SimpleRng::new(config.seed);
    let centers: Vec<Vec<f32>> = (0..config.clusters)
        .map(|_| random_unit_vector(config.dim, &mut rng))
        .collect();
    let docs = (0..config.docs)
        .map(|idx| {
            let center = &centers[idx % config.clusters];
            noisy_vector(center, &mut rng, 0.08)
        })
        .collect();
    let queries = (0..config.queries)
        .map(|idx| {
            let center = &centers[idx % config.clusters];
            noisy_vector(center, &mut rng, 0.04)
        })
        .collect();
    RetrievalFixture {
        centers,
        docs,
        queries,
    }
}

fn noisy_vector(center: &[f32], rng: &mut SimpleRng, noise: f32) -> Vec<f32> {
    let vector = center
        .iter()
        .map(|value| value + (rng.next_f32() * 2.0 - 1.0) * noise)
        .collect::<Vec<_>>();
    normalize(vector)
}

fn random_unit_vector(dim: usize, rng: &mut SimpleRng) -> Vec<f32> {
    let vector = (0..dim)
        .map(|_| rng.next_f32() * 2.0 - 1.0)
        .collect::<Vec<_>>();
    normalize(vector)
}

fn normalize(mut vector: Vec<f32>) -> Vec<f32> {
    let magnitude = vector.iter().map(|value| value * value).sum::<f32>().sqrt();
    if magnitude > 0.0 {
        for value in &mut vector {
            *value /= magnitude;
        }
    }
    vector
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

#[derive(Debug, Clone)]
struct SimpleRng(u64);

impl SimpleRng {
    fn new(seed: u64) -> Self {
        Self(seed)
    }

    fn next(&mut self) -> u64 {
        let mut x = self.0;
        x ^= x << 13;
        x ^= x >> 7;
        x ^= x << 17;
        self.0 = x;
        x
    }

    fn next_f32(&mut self) -> f32 {
        self.next() as f32 / u64::MAX as f32
    }
}
