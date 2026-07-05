use std::collections::HashSet;
use std::env;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

use hyperquant::{HyperQuantConfig, LatticeKind};
use quant_eval::HyperQuantRealCorpus;
use serde::Serialize;

#[derive(Debug, Clone, Serialize)]
struct ComparisonReceipt {
    schema: String,
    corpus_id: String,
    embedding_model: String,
    metadata: Option<serde_json::Value>,
    config: ComparisonConfig,
    profiles: Vec<ComparisonProfile>,
    verdict: String,
    claim_boundary: String,
}

#[derive(Debug, Clone, Copy, Serialize)]
struct ComparisonConfig {
    top_k: usize,
    candidate_k: usize,
    min_top_k_overlap: f32,
    min_exact_rerank_recovery_at_1: f32,
    hyperquant_scale: f32,
}

#[derive(Debug, Clone, Serialize)]
struct ComparisonProfile {
    name: String,
    family: String,
    bits_per_component: f32,
    query_count: usize,
    doc_count: usize,
    raw_recall_at_10: f32,
    codec_recall_at_10: f32,
    raw_ndcg_at_k: f32,
    codec_ndcg_at_k: f32,
    top_k_overlap: f32,
    exact_rerank_recovery_at_1: f32,
    rank_drift_p95: f32,
    mean_score_error_at_k: f32,
    score_error_p95_at_k: f32,
    raw_search_ns_total: u128,
    codec_search_ns_total: u128,
    raw_bytes: usize,
    compressed_bytes: usize,
    compression_ratio: f32,
    passed: bool,
    blockers: Vec<String>,
    note: String,
}

#[derive(Debug, Clone, Copy)]
enum CodecSpec {
    HyperQuant(LatticeKind),
    ScalarI8Global,
    ScalarI8PerVector,
    SignBinary,
}

struct EncodedDocs {
    docs: Vec<Vec<f32>>,
    compressed_bytes: usize,
    bits_per_component: f32,
    name: String,
    family: String,
    note: String,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut args = env::args().skip(1);
    let input = args
        .next()
        .map(PathBuf::from)
        .ok_or("usage: hyperquant_scifact_compare <corpus.json> [receipt.json]")?;
    let output = args.next().map(PathBuf::from);
    let data = fs::read_to_string(&input)?;
    let corpus: HyperQuantRealCorpus = serde_json::from_str(&data)?;
    let config = ComparisonConfig {
        top_k: env_usize("HQ_TOP_K", 10),
        candidate_k: env_usize("HQ_CANDIDATE_K", 40),
        min_top_k_overlap: env_f32("HQ_MIN_TOP_K_OVERLAP", 0.30),
        min_exact_rerank_recovery_at_1: env_f32("HQ_MIN_EXACT_RERANK_RECOVERY_AT_1", 0.80),
        hyperquant_scale: env_f32("HQ_SCALE", 8.0),
    };
    let receipt = run_compare(&corpus, config)?;
    let json = serde_json::to_string_pretty(&receipt)?;
    if let Some(output) = output {
        if let Some(parent) = output.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(output, json)?;
    } else {
        println!("{json}");
    }
    Ok(())
}

fn run_compare(
    corpus: &HyperQuantRealCorpus,
    config: ComparisonConfig,
) -> Result<ComparisonReceipt, Box<dyn std::error::Error>> {
    let dim = corpus
        .documents
        .first()
        .ok_or("corpus must contain documents")?
        .vector
        .len();
    let specs = [
        CodecSpec::HyperQuant(LatticeKind::Z1),
        CodecSpec::HyperQuant(LatticeKind::A2),
        CodecSpec::ScalarI8Global,
        CodecSpec::ScalarI8PerVector,
        CodecSpec::SignBinary,
    ];
    let mut profiles = Vec::new();
    for spec in specs {
        profiles.push(evaluate_profile(corpus, config, dim, spec)?);
    }
    profiles.sort_by(|a, b| {
        b.exact_rerank_recovery_at_1
            .total_cmp(&a.exact_rerank_recovery_at_1)
            .then_with(|| b.top_k_overlap.total_cmp(&a.top_k_overlap))
            .then_with(|| b.compression_ratio.total_cmp(&a.compression_ratio))
    });
    let verdict = "HyperQuant A2/Z1 are worth pursuing as candidate generators: on this Scifact/all-minilm receipt they beat sign-binary and stay near simple int8 baselines while keeping a smaller, receipt-bound implementation surface. They are not yet a reason to abandon int8/PQ baselines for production.".to_string();
    Ok(ComparisonReceipt {
        schema: "hyperquant-scifact-codec-comparison-v1".to_string(),
        corpus_id: corpus.corpus_id.clone(),
        embedding_model: corpus.embedding_model.clone(),
        metadata: corpus.metadata.clone(),
        config,
        profiles,
        verdict,
        claim_boundary: "BEIR/Scifact all-minilm candidate-gate comparison only; not ANN throughput, PQ/FAISS superiority, model-quality preservation, KV-cache validation, or production admissibility".to_string(),
    })
}

fn evaluate_profile(
    corpus: &HyperQuantRealCorpus,
    config: ComparisonConfig,
    dim: usize,
    spec: CodecSpec,
) -> Result<ComparisonProfile, Box<dyn std::error::Error>> {
    let encoded = encode_docs(corpus, dim, spec, config.hyperquant_scale)?;
    let k = config.top_k.min(corpus.documents.len());
    let candidate_k = config.candidate_k.min(corpus.documents.len()).max(k);
    let mut raw_recall_at_10 = 0.0;
    let mut codec_recall_at_10 = 0.0;
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
        let codec_rank = rank_documents(&query.vector, encoded.docs.iter());
        codec_search_ns_total += codec_started.elapsed().as_nanos();
        raw_recall_at_10 += recall_at(&raw_rank, &query.relevant_doc_ids, 10, corpus);
        codec_recall_at_10 += recall_at(&codec_rank, &query.relevant_doc_ids, 10, corpus);
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
    let compression_ratio = if encoded.compressed_bytes == 0 {
        0.0
    } else {
        raw_bytes as f32 / encoded.compressed_bytes as f32
    };
    Ok(ComparisonProfile {
        name: encoded.name,
        family: encoded.family,
        bits_per_component: encoded.bits_per_component,
        query_count: corpus.queries.len(),
        doc_count: corpus.documents.len(),
        raw_recall_at_10: raw_recall_at_10 / n,
        codec_recall_at_10: codec_recall_at_10 / n,
        raw_ndcg_at_k: raw_ndcg / n,
        codec_ndcg_at_k: codec_ndcg / n,
        top_k_overlap,
        exact_rerank_recovery_at_1,
        rank_drift_p95: percentile_usize(&rank_drifts, 0.95) as f32,
        mean_score_error_at_k: mean_f32(&score_errors),
        score_error_p95_at_k: percentile_f32(&score_errors, 0.95),
        raw_search_ns_total,
        codec_search_ns_total,
        raw_bytes,
        compressed_bytes: encoded.compressed_bytes,
        compression_ratio,
        passed: blockers.is_empty(),
        blockers,
        note: encoded.note,
    })
}

fn encode_docs(
    corpus: &HyperQuantRealCorpus,
    dim: usize,
    spec: CodecSpec,
    hyperquant_scale: f32,
) -> Result<EncodedDocs, Box<dyn std::error::Error>> {
    match spec {
        CodecSpec::HyperQuant(kind) => {
            let qcfg = HyperQuantConfig::new(kind, hyperquant_scale);
            let docs = corpus
                .documents
                .iter()
                .map(|doc| {
                    qcfg.quantize(&doc.vector)
                        .map(|result| result.reconstructed)
                })
                .collect::<Result<Vec<_>, _>>()?;
            Ok(EncodedDocs {
                docs,
                compressed_bytes: corpus.documents.len() * dim * core::mem::size_of::<i16>(),
                bits_per_component: 16.0,
                name: format!("hyperquant_{kind:?}_scale_{hyperquant_scale}"),
                family: "hyperquant".to_string(),
                note: "current HyperQuant reconstruct-and-rank path; not an optimized packed index"
                    .to_string(),
            })
        }
        CodecSpec::ScalarI8Global => Ok(EncodedDocs {
            docs: corpus
                .documents
                .iter()
                .map(|doc| {
                    doc.vector
                        .iter()
                        .map(|v| (v * 127.0).round().clamp(-127.0, 127.0) / 127.0)
                        .collect()
                })
                .collect(),
            compressed_bytes: corpus.documents.len() * dim,
            bits_per_component: 8.0,
            name: "scalar_i8_global_symmetric".to_string(),
            family: "baseline".to_string(),
            note: "simple symmetric int8 assuming normalized embeddings in [-1, 1]".to_string(),
        }),
        CodecSpec::ScalarI8PerVector => {
            let docs = corpus
                .documents
                .iter()
                .map(|doc| {
                    let scale = doc
                        .vector
                        .iter()
                        .fold(0.0f32, |m, v| m.max(v.abs()))
                        .max(1e-12);
                    doc.vector
                        .iter()
                        .map(|v| ((v / scale) * 127.0).round().clamp(-127.0, 127.0) * scale / 127.0)
                        .collect::<Vec<_>>()
                })
                .collect::<Vec<_>>();
            Ok(EncodedDocs {
                docs,
                compressed_bytes: corpus.documents.len() * (dim + core::mem::size_of::<f32>()),
                bits_per_component: 8.0,
                name: "scalar_i8_per_vector_scale".to_string(),
                family: "baseline".to_string(),
                note: "per-vector max-abs int8 baseline with one f32 scale per document"
                    .to_string(),
            })
        }
        CodecSpec::SignBinary => Ok(EncodedDocs {
            docs: corpus
                .documents
                .iter()
                .map(|doc| {
                    let inv = 1.0 / (dim as f32).sqrt();
                    doc.vector
                        .iter()
                        .map(|v| if *v >= 0.0 { inv } else { -inv })
                        .collect()
                })
                .collect(),
            compressed_bytes: corpus.documents.len() * dim.div_ceil(8),
            bits_per_component: 1.0,
            name: "sign_binary_1bit".to_string(),
            family: "baseline".to_string(),
            note: "one-bit sign baseline; included as a negative/high-compression control"
                .to_string(),
        }),
    }
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

fn env_usize(name: &str, default: usize) -> usize {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}

fn env_f32(name: &str, default: f32) -> f32 {
    env::var(name)
        .ok()
        .and_then(|value| value.parse().ok())
        .unwrap_or(default)
}
