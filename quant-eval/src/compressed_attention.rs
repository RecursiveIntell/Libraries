use compressed_scorer::{AttentionCache, PerDimScorer};
use serde::{Deserialize, Serialize};

use crate::QuantEvalError;

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompressedAttentionConfig {
    pub bits: u32,
    pub top_k: usize,
    pub min_mean_output_cosine: f32,
    pub max_mean_output_mse: f32,
    pub min_top_k_overlap: f32,
}

impl Default for CompressedAttentionConfig {
    fn default() -> Self {
        Self {
            bits: 8,
            top_k: 8,
            min_mean_output_cosine: 0.95,
            max_mean_output_mse: 0.05,
            min_top_k_overlap: 0.80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedAttentionReceipt {
    pub schema: String,
    pub scoring_path: String,
    pub config: CompressedAttentionConfig,
    pub query_count: usize,
    pub cache_len: usize,
    pub dim: usize,
    pub top_k: usize,
    pub mean_output_cosine: f32,
    pub mean_output_mse: f32,
    pub mean_top_k_overlap: f32,
    pub decompressed_value_count: usize,
    pub passed: bool,
    pub blockers: Vec<String>,
    pub verdict: String,
    pub claim_boundary: String,
}

pub fn run_compressed_attention_eval(
    keys: &[Vec<f32>],
    values: &[Vec<f32>],
    queries: &[Vec<f32>],
    config: &CompressedAttentionConfig,
) -> Result<CompressedAttentionReceipt, QuantEvalError> {
    let dim = validate_attention_inputs(keys, values, queries, config)?;
    let mut scorer = PerDimScorer::new(dim, config.bits)
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
    let fit_vectors = keys
        .iter()
        .chain(values.iter())
        .map(Vec::as_slice)
        .collect::<Vec<_>>();
    scorer
        .fit(&fit_vectors)
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;

    let mut cache = AttentionCache::new(scorer.clone());
    for (key, value) in keys.iter().zip(values.iter()) {
        let compressed_key = scorer
            .compress(key)
            .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
        let compressed_value = scorer
            .compress(value)
            .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
        cache.push_compressed(compressed_key, compressed_value);
    }

    let mut cosine_sum = 0.0f32;
    let mut mse_sum = 0.0f32;
    let mut overlap_sum = 0.0f32;
    let mut decompressed_value_count = 0usize;
    let top_k = config.top_k.min(keys.len());

    for query in queries {
        let exact = exact_attention(query, keys, values, top_k);
        let compressed = cache
            .attention_topk(query, top_k)
            .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
        cosine_sum += cosine_similarity(&exact.output, &compressed.output);
        mse_sum += mse(&exact.output, &compressed.output);
        overlap_sum += top_k_overlap(&exact.top_k_indices, &compressed.top_k_indices, top_k);
        decompressed_value_count += compressed.decompression_count;
    }

    let denom = queries.len() as f32;
    let mean_output_cosine = cosine_sum / denom;
    let mean_output_mse = mse_sum / denom;
    let mean_top_k_overlap = overlap_sum / denom;
    let mut blockers = Vec::new();
    if mean_output_cosine < config.min_mean_output_cosine {
        blockers.push(format!(
            "mean_output_cosine {:.4} < min {:.4}",
            mean_output_cosine, config.min_mean_output_cosine
        ));
    }
    if mean_output_mse > config.max_mean_output_mse {
        blockers.push(format!(
            "mean_output_mse {:.6} > max {:.6}",
            mean_output_mse, config.max_mean_output_mse
        ));
    }
    if mean_top_k_overlap < config.min_top_k_overlap {
        blockers.push(format!(
            "mean_top_k_overlap {:.4} < min {:.4}",
            mean_top_k_overlap, config.min_top_k_overlap
        ));
    }
    let passed = blockers.is_empty();
    let verdict = if passed {
        "compressed attention fixture passed declared top-k decode gate".to_string()
    } else {
        "compressed attention fixture failed declared gate; do not promote to KV-cache claims"
            .to_string()
    };

    Ok(CompressedAttentionReceipt {
        schema: "compressed-attention-eval-v1".to_string(),
        scoring_path: "compressed_key_logits_topk_value_decode".to_string(),
        config: *config,
        query_count: queries.len(),
        cache_len: keys.len(),
        dim,
        top_k,
        mean_output_cosine,
        mean_output_mse,
        mean_top_k_overlap,
        decompressed_value_count,
        passed,
        blockers,
        verdict,
        claim_boundary: "attention fixture evidence only; not model-quality, perplexity, latency, or production KV-cache preservation evidence".to_string(),
    })
}

fn validate_attention_inputs(
    keys: &[Vec<f32>],
    values: &[Vec<f32>],
    queries: &[Vec<f32>],
    config: &CompressedAttentionConfig,
) -> Result<usize, QuantEvalError> {
    if keys.is_empty() {
        return Err(QuantEvalError::InvalidCorpus(
            "attention eval requires at least one key".to_string(),
        ));
    }
    if keys.len() != values.len() {
        return Err(QuantEvalError::InvalidCorpus(
            "attention keys and values must have the same length".to_string(),
        ));
    }
    if queries.is_empty() {
        return Err(QuantEvalError::InvalidCorpus(
            "attention eval requires at least one query".to_string(),
        ));
    }
    if config.bits == 0 || config.bits > 8 {
        return Err(QuantEvalError::InvalidCorpus(
            "compressed attention bits must be in 1..=8".to_string(),
        ));
    }
    if config.top_k == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "compressed attention top_k must be > 0".to_string(),
        ));
    }
    let dim = keys[0].len();
    if dim == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "attention vector dimension must be > 0".to_string(),
        ));
    }
    for (label, vectors) in [("key", keys), ("value", values), ("query", queries)] {
        for (idx, vector) in vectors.iter().enumerate() {
            if vector.len() != dim {
                return Err(QuantEvalError::InvalidCorpus(format!(
                    "attention {label} {idx} has dimension {}, expected {dim}",
                    vector.len()
                )));
            }
            if vector.iter().any(|v| !v.is_finite()) {
                return Err(QuantEvalError::InvalidCorpus(format!(
                    "attention {label} {idx} contains non-finite values"
                )));
            }
        }
    }
    Ok(dim)
}

struct ExactAttentionOutput {
    output: Vec<f32>,
    top_k_indices: Vec<usize>,
}

fn exact_attention(
    query: &[f32],
    keys: &[Vec<f32>],
    values: &[Vec<f32>],
    top_k: usize,
) -> ExactAttentionOutput {
    let scale = (query.len() as f32).sqrt();
    let logits = keys
        .iter()
        .map(|key| dot(query, key) / scale)
        .collect::<Vec<_>>();
    let probabilities = softmax(&logits);
    let top_k_indices = topk_indices_by_probability(&probabilities, top_k);
    let mut output = vec![0.0f32; query.len()];
    for &idx in &top_k_indices {
        let prob = probabilities[idx];
        for (acc, value) in output.iter_mut().zip(values[idx].iter()) {
            *acc += prob * *value;
        }
    }
    ExactAttentionOutput {
        output,
        top_k_indices,
    }
}

fn dot(a: &[f32], b: &[f32]) -> f32 {
    a.iter().zip(b.iter()).map(|(x, y)| x * y).sum()
}

fn softmax(logits: &[f32]) -> Vec<f32> {
    let max = logits.iter().copied().fold(f32::NEG_INFINITY, f32::max);
    let exps = logits
        .iter()
        .map(|logit| (*logit - max).exp())
        .collect::<Vec<_>>();
    let sum: f32 = exps.iter().sum();
    exps.into_iter().map(|value| value / sum).collect()
}

fn topk_indices_by_probability(probabilities: &[f32], k: usize) -> Vec<usize> {
    let mut indexed = probabilities
        .iter()
        .copied()
        .enumerate()
        .collect::<Vec<_>>();
    indexed.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));
    indexed.into_iter().take(k).map(|(idx, _)| idx).collect()
}

fn cosine_similarity(a: &[f32], b: &[f32]) -> f32 {
    let dot_product = dot(a, b);
    let norm_a = dot(a, a).sqrt();
    let norm_b = dot(b, b).sqrt();
    if norm_a == 0.0 || norm_b == 0.0 {
        return 0.0;
    }
    dot_product / (norm_a * norm_b)
}

fn mse(a: &[f32], b: &[f32]) -> f32 {
    a.iter()
        .zip(b.iter())
        .map(|(x, y)| {
            let err = x - y;
            err * err
        })
        .sum::<f32>()
        / a.len() as f32
}

fn top_k_overlap(a: &[usize], b: &[usize], k: usize) -> f32 {
    let k = k.min(a.len()).min(b.len());
    if k == 0 {
        return 0.0;
    }
    let matches = a
        .iter()
        .take(k)
        .filter(|idx| b.iter().take(k).any(|other| other == *idx))
        .count();
    matches as f32 / k as f32
}
