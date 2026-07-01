use serde::{Deserialize, Serialize};
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressedAttentionBenchConfig {
    pub model_or_fixture: String,
    pub codec: String,
    pub context_len: usize,
    pub top_k: usize,
    pub raw_fp16_cache_bytes: u64,
    pub compressed_cache_bytes: u64,
    pub quality_threshold_cosine: f32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct CompressedAttentionBenchReceipt {
    pub schema_version: String,
    pub model_or_fixture: String,
    pub codec: String,
    pub context_len: usize,
    pub top_k: usize,
    pub cache_bytes_raw_fp16: u64,
    pub cache_bytes_compressed: u64,
    pub compression_ratio: f64,
    pub bytes_loaded_per_query: u64,
    pub decoded_values: usize,
    pub refined_candidates: usize,
    pub exact_fallbacks: usize,
    pub topk_overlap: f32,
    pub attention_output_cosine: f32,
    pub attention_output_mse: f32,
    pub logit_mae: f32,
    pub latency_p50_us: u64,
    pub latency_p95_us: u64,
    pub passed: bool,
    pub blockers: Vec<String>,
}

pub fn run_synthetic_compressed_attention_bench(
    config: CompressedAttentionBenchConfig,
    exact_scores: &[f32],
    compressed_scores: &[f32],
) -> CompressedAttentionBenchReceipt {
    let start = Instant::now();
    let mut blockers = Vec::new();
    if exact_scores.len() != compressed_scores.len() {
        blockers.push("score length mismatch".to_string());
    }
    if exact_scores.is_empty() {
        blockers.push("empty score fixture".to_string());
    }
    let n = exact_scores.len().min(compressed_scores.len());
    let k = config.top_k.min(n).max(1);
    let mut exact_idx: Vec<usize> = (0..n).collect();
    let mut comp_idx: Vec<usize> = (0..n).collect();
    exact_idx.sort_by(|&a, &b| {
        exact_scores[b]
            .partial_cmp(&exact_scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    comp_idx.sort_by(|&a, &b| {
        compressed_scores[b]
            .partial_cmp(&compressed_scores[a])
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    exact_idx.truncate(k);
    comp_idx.truncate(k);
    let overlap = comp_idx.iter().filter(|i| exact_idx.contains(i)).count() as f32 / k as f32;
    let mut dot = 0.0f32;
    let mut a2 = 0.0f32;
    let mut b2 = 0.0f32;
    let mut mse = 0.0f32;
    let mut mae = 0.0f32;
    for i in 0..n {
        let a = exact_scores[i];
        let b = compressed_scores[i];
        dot += a * b;
        a2 += a * a;
        b2 += b * b;
        let d = a - b;
        mse += d * d;
        mae += d.abs();
    }
    let denom = (a2.sqrt() * b2.sqrt()).max(f32::MIN_POSITIVE);
    let cosine = dot / denom;
    if n > 0 {
        mse /= n as f32;
        mae /= n as f32;
    }
    if cosine < config.quality_threshold_cosine {
        blockers.push(format!(
            "attention_output_cosine {cosine:.6} below threshold {:.6}",
            config.quality_threshold_cosine
        ));
    }
    let compression_ratio = if config.compressed_cache_bytes > 0 {
        config.raw_fp16_cache_bytes as f64 / config.compressed_cache_bytes as f64
    } else {
        0.0
    };
    let elapsed = start.elapsed().as_micros() as u64;
    CompressedAttentionBenchReceipt {
        schema_version: "compressed_cache_attention_bench_v1".into(),
        model_or_fixture: config.model_or_fixture,
        codec: config.codec,
        context_len: config.context_len,
        top_k: config.top_k,
        cache_bytes_raw_fp16: config.raw_fp16_cache_bytes,
        cache_bytes_compressed: config.compressed_cache_bytes,
        compression_ratio,
        bytes_loaded_per_query: config
            .compressed_cache_bytes
            .min(config.raw_fp16_cache_bytes),
        decoded_values: k,
        refined_candidates: 0,
        exact_fallbacks: 0,
        topk_overlap: overlap,
        attention_output_cosine: cosine,
        attention_output_mse: mse,
        logit_mae: mae,
        latency_p50_us: elapsed,
        latency_p95_us: elapsed,
        passed: blockers.is_empty(),
        blockers,
    }
}
