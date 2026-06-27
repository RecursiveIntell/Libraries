//! Local fixture evaluation for RAG-style retrieval experiments.

use std::collections::HashSet;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RagQueryFixture {
    pub query_id: String,
    pub query: String,
    pub relevant_doc_ids: Vec<String>,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RagRetrievedDoc {
    pub doc_id: String,
    pub score: f32,
}

#[derive(Debug, Clone, PartialEq)]
pub struct RagEvalResult {
    pub recall_at_k: f32,
    pub ndcg_at_k: f32,
    pub exact_rerank_recovery: f32,
}

pub fn evaluate_rag_fixture(
    fixture: &RagQueryFixture,
    retrieved: &[RagRetrievedDoc],
    k: usize,
) -> RagEvalResult {
    let relevant_doc_ids: HashSet<&str> = fixture
        .relevant_doc_ids
        .iter()
        .map(String::as_str)
        .collect();

    if relevant_doc_ids.is_empty() {
        return RagEvalResult {
            recall_at_k: 0.0,
            ndcg_at_k: 0.0,
            exact_rerank_recovery: 0.0,
        };
    }

    let exact_rerank_recovery = retrieved
        .first()
        .filter(|doc| relevant_doc_ids.contains(doc.doc_id.as_str()))
        .map(|_| 1.0)
        .unwrap_or(0.0);

    let mut seen_doc_ids = HashSet::new();
    let mut relevant_found = HashSet::new();
    let mut dcg = 0.0f32;

    for (index, doc) in retrieved.iter().take(k).enumerate() {
        if !seen_doc_ids.insert(doc.doc_id.as_str()) {
            continue;
        }

        if relevant_doc_ids.contains(doc.doc_id.as_str()) {
            relevant_found.insert(doc.doc_id.as_str());
            dcg += discounted_gain(index + 1);
        }
    }

    let ideal_len = relevant_doc_ids.len().min(k);
    let idcg = (1..=ideal_len).map(discounted_gain).sum::<f32>();
    let ndcg_at_k = if idcg > 0.0 { dcg / idcg } else { 0.0 };

    RagEvalResult {
        recall_at_k: relevant_found.len() as f32 / relevant_doc_ids.len() as f32,
        ndcg_at_k,
        exact_rerank_recovery,
    }
}

fn discounted_gain(rank: usize) -> f32 {
    if rank == 1 {
        1.0
    } else {
        1.0 / ((rank + 1) as f32).log2()
    }
}
