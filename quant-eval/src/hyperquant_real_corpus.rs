//! Real-corpus retrieval evaluation for HyperQuant.
//!
//! This harness accepts caller-supplied document/query embeddings and qrels. It
//! is intentionally small: compare exact f32 retrieval against retrieval after
//! HyperQuant reconstruction, emit gate metrics, and keep the claim boundary
//! narrow. A toy in-tree fixture is still a fixture; external BEIR/Scifact
//! evidence should feed this same API with real embeddings.

use std::collections::HashSet;
use std::time::Instant;

use crate::QuantEvalError;
use hyperquant::{HyperQuantConfig, LatticeKind};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealCorpusDocument {
    pub doc_id: String,
    pub vector: Vec<f32>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RealCorpusQuery {
    pub query_id: String,
    pub vector: Vec<f32>,
    pub relevant_doc_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRealCorpus {
    pub corpus_id: String,
    pub embedding_model: String,
    #[serde(default)]
    pub metadata: Option<serde_json::Value>,
    pub documents: Vec<RealCorpusDocument>,
    pub queries: Vec<RealCorpusQuery>,
}

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRealCorpusConfig {
    pub top_k: usize,
    pub candidate_k: usize,
    pub scale: f32,
    pub min_top_k_overlap: f32,
    pub min_exact_rerank_recovery_at_1: f32,
}

impl Default for HyperQuantRealCorpusConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            candidate_k: 40,
            scale: 8.0,
            min_top_k_overlap: 0.30,
            min_exact_rerank_recovery_at_1: 0.80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRealCorpusProfile {
    pub kind: LatticeKind,
    pub query_count: usize,
    pub doc_count: usize,
    pub raw_recall_at_1: f32,
    pub raw_recall_at_5: f32,
    pub raw_recall_at_10: f32,
    pub raw_recall_at_k: f32,
    pub codec_recall_at_1: f32,
    pub codec_recall_at_5: f32,
    pub codec_recall_at_10: f32,
    pub codec_recall_at_k: f32,
    pub raw_ndcg_at_k: f32,
    pub codec_ndcg_at_k: f32,
    pub top_k_overlap: f32,
    pub exact_rerank_recovery_at_1: f32,
    pub rank_drift_mean: f32,
    pub rank_drift_p95: f32,
    pub rank_drift_max: usize,
    pub mean_score_error_at_k: f32,
    pub score_error_p95_at_k: f32,
    pub score_error_max_at_k: f32,
    pub raw_search_ns_total: u128,
    pub codec_search_ns_total: u128,
    pub raw_bytes: usize,
    pub compressed_bytes: usize,
    pub compression_ratio: f32,
    pub passed: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HyperQuantRealCorpusReceipt {
    pub schema: String,
    pub corpus_id: String,
    pub embedding_model: String,
    pub metadata: Option<serde_json::Value>,
    pub config: HyperQuantRealCorpusConfig,
    pub profiles: Vec<HyperQuantRealCorpusProfile>,
    pub claim_boundary: String,
}

pub fn run_hyperquant_real_corpus_eval(
    corpus: &HyperQuantRealCorpus,
    config: &HyperQuantRealCorpusConfig,
) -> Result<HyperQuantRealCorpusReceipt, QuantEvalError> {
    let dim = validate_corpus(corpus, config)?;
    let profiles = [LatticeKind::Z1, LatticeKind::A2]
        .into_iter()
        .map(|kind| evaluate_profile(corpus, config, dim, kind))
        .collect::<Result<Vec<_>, _>>()?;

    Ok(HyperQuantRealCorpusReceipt {
        schema: "hyperquant-real-corpus-eval-v1".to_string(),
        corpus_id: corpus.corpus_id.clone(),
        embedding_model: corpus.embedding_model.clone(),
        metadata: corpus.metadata.clone(),
        config: *config,
        profiles,
        claim_boundary: "real corpus retrieval fixture evidence only; not BEIR superiority, model-quality preservation, or production admissibility unless caller supplies external corpus receipts".to_string(),
    })
}

fn validate_corpus(
    corpus: &HyperQuantRealCorpus,
    config: &HyperQuantRealCorpusConfig,
) -> Result<usize, QuantEvalError> {
    if corpus.documents.is_empty() {
        return Err(QuantEvalError::InvalidCorpus(
            "real corpus must contain at least one document".to_string(),
        ));
    }
    if corpus.queries.is_empty() {
        return Err(QuantEvalError::InvalidCorpus(
            "real corpus must contain at least one query".to_string(),
        ));
    }
    if config.top_k == 0 || config.candidate_k == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "top_k and candidate_k must be > 0".to_string(),
        ));
    }
    let dim = corpus.documents[0].vector.len();
    if dim == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "document vector dimension must be > 0".to_string(),
        ));
    }
    for doc in &corpus.documents {
        if doc.vector.len() != dim {
            return Err(QuantEvalError::InvalidCorpus(format!(
                "document '{}' has dimension {}, expected {}",
                doc.doc_id,
                doc.vector.len(),
                dim
            )));
        }
        if doc.vector.iter().any(|v| !v.is_finite()) {
            return Err(QuantEvalError::InvalidCorpus(format!(
                "document '{}' contains non-finite values",
                doc.doc_id
            )));
        }
    }
    let doc_ids = corpus
        .documents
        .iter()
        .map(|doc| doc.doc_id.as_str())
        .collect::<HashSet<_>>();
    for query in &corpus.queries {
        if query.vector.len() != dim {
            return Err(QuantEvalError::InvalidCorpus(format!(
                "query '{}' has dimension {}, expected {}",
                query.query_id,
                query.vector.len(),
                dim
            )));
        }
        if query.vector.iter().any(|v| !v.is_finite()) {
            return Err(QuantEvalError::InvalidCorpus(format!(
                "query '{}' contains non-finite values",
                query.query_id
            )));
        }
        if query.relevant_doc_ids.is_empty() {
            return Err(QuantEvalError::InvalidCorpus(format!(
                "query '{}' has no qrels",
                query.query_id
            )));
        }
        for relevant in &query.relevant_doc_ids {
            if !doc_ids.contains(relevant.as_str()) {
                return Err(QuantEvalError::InvalidCorpus(format!(
                    "query '{}' references missing document '{}'",
                    query.query_id, relevant
                )));
            }
        }
    }
    Ok(dim)
}

fn evaluate_profile(
    corpus: &HyperQuantRealCorpus,
    config: &HyperQuantRealCorpusConfig,
    dim: usize,
    kind: LatticeKind,
) -> Result<HyperQuantRealCorpusProfile, QuantEvalError> {
    let qcfg = HyperQuantConfig::new(kind, config.scale);
    let quantized_docs = corpus
        .documents
        .iter()
        .map(|doc| {
            qcfg.quantize(&doc.vector)
                .map(|result| result.reconstructed)
                .map_err(|err| QuantEvalError::Codec(err.to_string()))
        })
        .collect::<Result<Vec<_>, _>>()?;

    let k = config.top_k.min(corpus.documents.len());
    let candidate_k = config.candidate_k.min(corpus.documents.len()).max(k);
    let mut raw_recall_at_1 = 0.0;
    let mut raw_recall_at_5 = 0.0;
    let mut raw_recall_at_10 = 0.0;
    let mut raw_recall = 0.0;
    let mut codec_recall_at_1 = 0.0;
    let mut codec_recall_at_5 = 0.0;
    let mut codec_recall_at_10 = 0.0;
    let mut codec_recall = 0.0;
    let mut raw_ndcg = 0.0;
    let mut codec_ndcg = 0.0;
    let mut overlap = 0.0;
    let mut recovery = 0.0;
    let mut score_errors = Vec::new();
    let mut rank_drifts = Vec::new();
    let mut raw_search_ns_total = 0u128;
    let mut codec_search_ns_total = 0u128;

    for query in &corpus.queries {
        let raw_started = Instant::now();
        let raw_rank = rank_documents(
            &query.vector,
            corpus.documents.iter().map(|doc| &doc.vector),
        );
        raw_search_ns_total += raw_started.elapsed().as_nanos();
        let codec_started = Instant::now();
        let codec_rank = rank_documents(&query.vector, quantized_docs.iter());
        codec_search_ns_total += codec_started.elapsed().as_nanos();
        raw_recall_at_1 += recall_at(&raw_rank, &query.relevant_doc_ids, 1, corpus);
        raw_recall_at_5 += recall_at(&raw_rank, &query.relevant_doc_ids, 5, corpus);
        raw_recall_at_10 += recall_at(&raw_rank, &query.relevant_doc_ids, 10, corpus);
        raw_recall += recall_at(&raw_rank, &query.relevant_doc_ids, k, corpus);
        codec_recall_at_1 += recall_at(&codec_rank, &query.relevant_doc_ids, 1, corpus);
        codec_recall_at_5 += recall_at(&codec_rank, &query.relevant_doc_ids, 5, corpus);
        codec_recall_at_10 += recall_at(&codec_rank, &query.relevant_doc_ids, 10, corpus);
        codec_recall += recall_at(&codec_rank, &query.relevant_doc_ids, k, corpus);
        raw_ndcg += ndcg_at(&raw_rank, &query.relevant_doc_ids, k, corpus);
        codec_ndcg += ndcg_at(&codec_rank, &query.relevant_doc_ids, k, corpus);
        overlap += top_k_overlap(&raw_rank, &codec_rank, k);
        recovery += exact_rerank_recovery(
            &raw_rank,
            &codec_rank,
            &query.relevant_doc_ids,
            candidate_k,
            corpus,
        );
        rank_drifts.push(rank_drift(
            &raw_rank,
            &codec_rank,
            &query.relevant_doc_ids,
            corpus,
        ));
        for &(doc_idx, codec_score) in codec_rank.iter().take(k) {
            let exact_score = cosine(&query.vector, &corpus.documents[doc_idx].vector);
            score_errors.push((codec_score - exact_score).abs());
        }
    }

    let n = corpus.queries.len() as f32;
    let raw_bytes = corpus.documents.len() * dim * core::mem::size_of::<f32>();
    let compressed_bytes = corpus.documents.len() * dim * core::mem::size_of::<i16>();
    let compression_ratio = if compressed_bytes == 0 {
        0.0
    } else {
        raw_bytes as f32 / compressed_bytes as f32
    };
    let top_k_overlap = overlap / n;
    let exact_rerank_recovery_at_1 = recovery / n;
    let rank_drift_mean = mean_usize(&rank_drifts);
    let rank_drift_p95 = percentile_usize(&rank_drifts, 0.95) as f32;
    let rank_drift_max = rank_drifts.iter().copied().max().unwrap_or(0);
    let mean_score_error_at_k = mean_f32(&score_errors);
    let score_error_p95_at_k = percentile_f32(&score_errors, 0.95);
    let score_error_max_at_k = score_errors.iter().copied().fold(0.0f32, f32::max);
    let mut blockers = Vec::new();
    if top_k_overlap < config.min_top_k_overlap {
        blockers.push(format!(
            "top_k_overlap {:.4} < threshold {:.4}",
            top_k_overlap, config.min_top_k_overlap
        ));
    }
    if exact_rerank_recovery_at_1 < config.min_exact_rerank_recovery_at_1 {
        blockers.push(format!(
            "exact_rerank_recovery_at_1 {:.4} < threshold {:.4}",
            exact_rerank_recovery_at_1, config.min_exact_rerank_recovery_at_1
        ));
    }

    Ok(HyperQuantRealCorpusProfile {
        kind,
        query_count: corpus.queries.len(),
        doc_count: corpus.documents.len(),
        raw_recall_at_1: raw_recall_at_1 / n,
        raw_recall_at_5: raw_recall_at_5 / n,
        raw_recall_at_10: raw_recall_at_10 / n,
        raw_recall_at_k: raw_recall / n,
        codec_recall_at_1: codec_recall_at_1 / n,
        codec_recall_at_5: codec_recall_at_5 / n,
        codec_recall_at_10: codec_recall_at_10 / n,
        codec_recall_at_k: codec_recall / n,
        raw_ndcg_at_k: raw_ndcg / n,
        codec_ndcg_at_k: codec_ndcg / n,
        top_k_overlap,
        exact_rerank_recovery_at_1,
        rank_drift_mean,
        rank_drift_p95,
        rank_drift_max,
        mean_score_error_at_k,
        score_error_p95_at_k,
        score_error_max_at_k,
        raw_search_ns_total,
        codec_search_ns_total,
        raw_bytes,
        compressed_bytes,
        compression_ratio,
        passed: blockers.is_empty(),
        blockers,
    })
}

fn rank_documents<'a, I>(query: &[f32], docs: I) -> Vec<(usize, f32)>
where
    I: IntoIterator<Item = &'a Vec<f32>>,
{
    let mut scored = docs
        .into_iter()
        .enumerate()
        .map(|(idx, vector)| (idx, cosine(query, vector)))
        .collect::<Vec<_>>();
    scored.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(&b.0)));
    scored
}

fn recall_at(
    ranking: &[(usize, f32)],
    relevant_doc_ids: &[String],
    k: usize,
    corpus: &HyperQuantRealCorpus,
) -> f32 {
    let relevant = relevant_doc_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let hits = ranking
        .iter()
        .take(k)
        .filter(|(idx, _)| relevant.contains(corpus.documents[*idx].doc_id.as_str()))
        .count();
    hits as f32 / relevant.len() as f32
}

fn ndcg_at(
    ranking: &[(usize, f32)],
    relevant_doc_ids: &[String],
    k: usize,
    corpus: &HyperQuantRealCorpus,
) -> f32 {
    let relevant = relevant_doc_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let dcg = ranking
        .iter()
        .take(k)
        .enumerate()
        .filter(|(_, (idx, _))| relevant.contains(corpus.documents[*idx].doc_id.as_str()))
        .map(|(rank, _)| discounted_gain(rank + 1))
        .sum::<f32>();
    let ideal_len = relevant.len().min(k);
    let idcg = (1..=ideal_len).map(discounted_gain).sum::<f32>();
    if idcg > 0.0 {
        dcg / idcg
    } else {
        0.0
    }
}

fn top_k_overlap(left: &[(usize, f32)], right: &[(usize, f32)], k: usize) -> f32 {
    let a = left
        .iter()
        .take(k)
        .map(|(idx, _)| *idx)
        .collect::<HashSet<_>>();
    let b = right
        .iter()
        .take(k)
        .map(|(idx, _)| *idx)
        .collect::<HashSet<_>>();
    if a.is_empty() && b.is_empty() {
        return 1.0;
    }
    let intersection = a.intersection(&b).count();
    let union = a.union(&b).count();
    intersection as f32 / union as f32
}

fn exact_rerank_recovery(
    raw_rank: &[(usize, f32)],
    codec_rank: &[(usize, f32)],
    relevant_doc_ids: &[String],
    candidate_k: usize,
    corpus: &HyperQuantRealCorpus,
) -> f32 {
    let relevant = relevant_doc_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let Some((best_relevant, _)) = raw_rank
        .iter()
        .find(|(idx, _)| relevant.contains(corpus.documents[*idx].doc_id.as_str()))
    else {
        return 0.0;
    };
    if codec_rank
        .iter()
        .take(candidate_k)
        .any(|(idx, _)| idx == best_relevant)
    {
        1.0
    } else {
        0.0
    }
}

fn rank_drift(
    raw_rank: &[(usize, f32)],
    codec_rank: &[(usize, f32)],
    relevant_doc_ids: &[String],
    corpus: &HyperQuantRealCorpus,
) -> usize {
    let relevant = relevant_doc_ids
        .iter()
        .map(String::as_str)
        .collect::<HashSet<_>>();
    let raw_pos = raw_rank
        .iter()
        .position(|(idx, _)| relevant.contains(corpus.documents[*idx].doc_id.as_str()));
    let codec_pos = codec_rank
        .iter()
        .position(|(idx, _)| relevant.contains(corpus.documents[*idx].doc_id.as_str()));
    match (raw_pos, codec_pos) {
        (Some(raw), Some(codec)) => raw.abs_diff(codec),
        (Some(raw), None) => corpus.documents.len().saturating_sub(raw),
        (None, Some(codec)) => corpus.documents.len().saturating_sub(codec),
        (None, None) => corpus.documents.len(),
    }
}

fn mean_usize(values: &[usize]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<usize>() as f32 / values.len() as f32
    }
}

fn mean_f32(values: &[f32]) -> f32 {
    if values.is_empty() {
        0.0
    } else {
        values.iter().sum::<f32>() / values.len() as f32
    }
}

fn percentile_usize(values: &[usize], percentile: f32) -> usize {
    if values.is_empty() {
        return 0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_unstable();
    let idx = percentile_index(sorted.len(), percentile);
    sorted[idx]
}

fn percentile_f32(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    let idx = percentile_index(sorted.len(), percentile);
    sorted[idx]
}

fn percentile_index(len: usize, percentile: f32) -> usize {
    let clamped = percentile.clamp(0.0, 1.0);
    ((len.saturating_sub(1)) as f32 * clamped).ceil() as usize
}

fn cosine(a: &[f32], b: &[f32]) -> f32 {
    let (mut dot, mut aa, mut bb) = (0.0f32, 0.0f32, 0.0f32);
    for (&x, &y) in a.iter().zip(b.iter()) {
        dot += x * y;
        aa += x * x;
        bb += y * y;
    }
    if aa == 0.0 || bb == 0.0 {
        0.0
    } else {
        dot / (aa.sqrt() * bb.sqrt())
    }
}

fn discounted_gain(rank: usize) -> f32 {
    if rank == 1 {
        1.0
    } else {
        1.0 / ((rank + 1) as f32).log2()
    }
}
