//! Real-corpus retrieval evaluation for compressed-scorer.
//!
//! This harness makes `compressed-scorer` the canonical candidate-scoring
//! substrate in `quant-eval`: score compressed vectors, return candidate IDs,
//! and rely on authoritative f32 vectors for exact rerank. It intentionally
//! does not decode compressed documents during candidate scoring.

use std::collections::HashSet;
use std::time::Instant;

use compressed_scorer::{search_topk, CompressedScorer, PerDimScorer};
use serde::{Deserialize, Serialize};

use crate::{HyperQuantRealCorpus, QuantEvalError};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct CompressedScorerRealCorpusConfig {
    pub top_k: usize,
    pub candidate_k: usize,
    pub bits: u32,
    pub min_top_k_overlap: f32,
    pub min_exact_rerank_recovery_at_1: f32,
}

impl Default for CompressedScorerRealCorpusConfig {
    fn default() -> Self {
        Self {
            top_k: 10,
            candidate_k: 40,
            bits: 8,
            min_top_k_overlap: 0.30,
            min_exact_rerank_recovery_at_1: 0.80,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedScorerRealCorpusProfile {
    pub name: String,
    pub family: String,
    pub scoring_path: String,
    pub query_count: usize,
    pub doc_count: usize,
    pub bits_per_component: f32,
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
    pub scorer_internal_bytes: usize,
    pub compression_ratio: f32,
    pub decoded_doc_count: usize,
    pub exact_rerank_count: usize,
    pub passed: bool,
    pub blockers: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct CompressedScorerRealCorpusReceipt {
    pub schema: String,
    pub corpus_id: String,
    pub embedding_model: String,
    pub metadata: Option<serde_json::Value>,
    pub config: CompressedScorerRealCorpusConfig,
    pub profiles: Vec<CompressedScorerRealCorpusProfile>,
    pub verdict: String,
    pub claim_boundary: String,
}

pub fn run_compressed_scorer_real_corpus_eval(
    corpus: &HyperQuantRealCorpus,
    config: &CompressedScorerRealCorpusConfig,
) -> Result<CompressedScorerRealCorpusReceipt, QuantEvalError> {
    let dim = validate_corpus(corpus, config)?;
    let profile = evaluate_per_dim_profile(corpus, config, dim)?;
    let verdict = if profile.passed {
        "compressed-scorer per-dim candidate scoring passed the declared real-corpus gate; keep exact f32 rerank mandatory".to_string()
    } else {
        "compressed-scorer per-dim candidate scoring failed the declared gate; keep fallback/default raw path".to_string()
    };
    Ok(CompressedScorerRealCorpusReceipt {
        schema: "compressed-scorer-real-corpus-eval-v1".to_string(),
        corpus_id: corpus.corpus_id.clone(),
        embedding_model: corpus.embedding_model.clone(),
        metadata: corpus.metadata.clone(),
        config: *config,
        profiles: vec![profile],
        verdict,
        claim_boundary: "candidate-gate evidence only; compressed candidates are not authoritative results and must be exact-f32 reranked before semantic-memory/product use".to_string(),
    })
}

fn validate_corpus(
    corpus: &HyperQuantRealCorpus,
    config: &CompressedScorerRealCorpusConfig,
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
    if config.bits == 0 || config.bits > 8 {
        return Err(QuantEvalError::InvalidCorpus(
            "compressed-scorer bits must be in 1..=8".to_string(),
        ));
    }
    let dim = corpus.documents[0].vector.len();
    if dim == 0 {
        return Err(QuantEvalError::InvalidCorpus(
            "document vector dimension must be > 0".to_string(),
        ));
    }
    for doc in &corpus.documents {
        validate_vector(&doc.vector, dim, &format!("document '{}'", doc.doc_id))?;
    }
    let doc_ids = corpus
        .documents
        .iter()
        .map(|doc| doc.doc_id.as_str())
        .collect::<HashSet<_>>();
    for query in &corpus.queries {
        validate_vector(&query.vector, dim, &format!("query '{}'", query.query_id))?;
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

fn validate_vector(vector: &[f32], expected_dim: usize, label: &str) -> Result<(), QuantEvalError> {
    if vector.len() != expected_dim {
        return Err(QuantEvalError::InvalidCorpus(format!(
            "{label} has dimension {}, expected {expected_dim}",
            vector.len()
        )));
    }
    if vector.iter().any(|v| !v.is_finite()) {
        return Err(QuantEvalError::InvalidCorpus(format!(
            "{label} contains non-finite values"
        )));
    }
    Ok(())
}

fn evaluate_per_dim_profile(
    corpus: &HyperQuantRealCorpus,
    config: &CompressedScorerRealCorpusConfig,
    dim: usize,
) -> Result<CompressedScorerRealCorpusProfile, QuantEvalError> {
    let mut scorer = PerDimScorer::new(dim, config.bits)
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
    let fit_docs = corpus
        .documents
        .iter()
        .map(|doc| doc.vector.as_slice())
        .collect::<Vec<_>>();
    scorer
        .fit(&fit_docs)
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
    let compressed_docs = corpus
        .documents
        .iter()
        .map(|doc| scorer.compress(&doc.vector))
        .collect::<Result<Vec<_>, _>>()
        .map_err(|err| QuantEvalError::Codec(err.to_string()))?;

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
    let mut exact_rerank_count = 0usize;

    for query in &corpus.queries {
        let raw_started = Instant::now();
        let raw_rank = rank_documents(
            &query.vector,
            corpus.documents.iter().map(|doc| &doc.vector),
        );
        raw_search_ns_total += raw_started.elapsed().as_nanos();

        let codec_started = Instant::now();
        let candidates = search_topk(&scorer, &query.vector, &compressed_docs, candidate_k)
            .map_err(|err| QuantEvalError::Codec(err.to_string()))?;
        codec_search_ns_total += codec_started.elapsed().as_nanos();
        let codec_rank = candidates
            .iter()
            .map(|candidate| (candidate.idx, candidate.score))
            .collect::<Vec<_>>();
        exact_rerank_count += codec_rank.len();

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
    let compressed_bytes = compressed_docs
        .iter()
        .map(|doc| doc.codes.len() + core::mem::size_of::<f32>())
        .sum::<usize>();
    let scorer_internal_bytes = scorer.internal_bytes();
    let compression_ratio = if compressed_bytes == 0 {
        0.0
    } else {
        raw_bytes as f32 / compressed_bytes as f32
    };
    let top_k_overlap = overlap / n;
    let exact_rerank_recovery_at_1 = recovery / n;
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

    Ok(CompressedScorerRealCorpusProfile {
        name: format!("per_dim_{}bit", config.bits),
        family: "compressed-scorer".to_string(),
        scoring_path: "lookup_table_compressed_domain_score_then_exact_f32_rerank".to_string(),
        query_count: corpus.queries.len(),
        doc_count: corpus.documents.len(),
        bits_per_component: config.bits as f32,
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
        rank_drift_mean: mean_usize(&rank_drifts),
        rank_drift_p95: percentile_usize(&rank_drifts, 0.95) as f32,
        rank_drift_max: rank_drifts.iter().copied().max().unwrap_or(0),
        mean_score_error_at_k: mean_f32(&score_errors),
        score_error_p95_at_k: percentile_f32(&score_errors, 0.95),
        score_error_max_at_k: score_errors.iter().copied().fold(0.0f32, f32::max),
        raw_search_ns_total,
        codec_search_ns_total,
        raw_bytes,
        compressed_bytes,
        scorer_internal_bytes,
        compression_ratio,
        decoded_doc_count: 0,
        exact_rerank_count,
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
    a.intersection(&b).count() as f32 / a.union(&b).count() as f32
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
    sorted[percentile_index(sorted.len(), percentile)]
}

fn percentile_f32(values: &[f32], percentile: f32) -> f32 {
    if values.is_empty() {
        return 0.0;
    }
    let mut sorted = values.to_vec();
    sorted.sort_by(f32::total_cmp);
    sorted[percentile_index(sorted.len(), percentile)]
}

fn percentile_index(len: usize, percentile: f32) -> usize {
    ((len.saturating_sub(1)) as f32 * percentile.clamp(0.0, 1.0)).ceil() as usize
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
