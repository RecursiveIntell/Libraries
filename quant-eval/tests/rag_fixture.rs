use quant_eval::{evaluate_rag_fixture, RagQueryFixture, RagRetrievedDoc};

fn fixture(relevant_doc_ids: &[&str]) -> RagQueryFixture {
    RagQueryFixture {
        query_id: "q1".to_string(),
        query: "local fixture query".to_string(),
        relevant_doc_ids: relevant_doc_ids.iter().map(|id| id.to_string()).collect(),
    }
}

fn retrieved(doc_ids: &[&str]) -> Vec<RagRetrievedDoc> {
    doc_ids
        .iter()
        .enumerate()
        .map(|(index, doc_id)| RagRetrievedDoc {
            doc_id: doc_id.to_string(),
            score: 1.0 - index as f32 * 0.01,
        })
        .collect()
}

#[test]
fn perfect_rag_retrieval_scores_one() {
    let fixture = fixture(&["doc-a", "doc-b"]);
    let retrieved = retrieved(&["doc-a", "doc-b"]);

    let result = evaluate_rag_fixture(&fixture, &retrieved, 2);

    assert_eq!(result.recall_at_k, 1.0);
    assert_eq!(result.ndcg_at_k, 1.0);
    assert_eq!(result.exact_rerank_recovery, 1.0);
}

#[test]
fn partial_rag_retrieval_has_recall_below_one() {
    let fixture = fixture(&["doc-a", "doc-b", "doc-c"]);
    let retrieved = retrieved(&["doc-a", "doc-x"]);

    let result = evaluate_rag_fixture(&fixture, &retrieved, 2);

    assert!(result.recall_at_k < 1.0);
    assert_eq!(result.recall_at_k, 1.0 / 3.0);
}

#[test]
fn irrelevant_rag_top_one_has_no_exact_recovery() {
    let fixture = fixture(&["doc-a"]);
    let retrieved = retrieved(&["doc-x", "doc-a"]);

    let result = evaluate_rag_fixture(&fixture, &retrieved, 2);

    assert_eq!(result.exact_rerank_recovery, 0.0);
}

#[test]
fn empty_rag_relevant_docs_returns_zero_metrics() {
    let fixture = fixture(&[]);
    let retrieved = retrieved(&["doc-a"]);

    let result = evaluate_rag_fixture(&fixture, &retrieved, 1);

    assert_eq!(result.recall_at_k, 0.0);
    assert_eq!(result.ndcg_at_k, 0.0);
    assert_eq!(result.exact_rerank_recovery, 0.0);
}

#[test]
fn rag_k_truncates_ranked_list() {
    let fixture = fixture(&["doc-a", "doc-b"]);
    let retrieved = retrieved(&["doc-a", "doc-b"]);

    let result = evaluate_rag_fixture(&fixture, &retrieved, 1);

    assert_eq!(result.recall_at_k, 0.5);
    assert_eq!(result.ndcg_at_k, 1.0);
}
