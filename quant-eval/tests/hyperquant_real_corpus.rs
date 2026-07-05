use hyperquant::LatticeKind;
use quant_eval::{
    run_hyperquant_real_corpus_eval, HyperQuantRealCorpus, HyperQuantRealCorpusConfig,
    RealCorpusDocument, RealCorpusQuery,
};
use serde_json::json;

fn tiny_semantic_corpus() -> HyperQuantRealCorpus {
    HyperQuantRealCorpus {
        corpus_id: "tiny-semantic-fixture-v1".to_string(),
        embedding_model: "hand-authored-unit-vectors".to_string(),
        metadata: Some(json!({
            "source_url": "fixture://tiny-semantic",
            "qrels_digest": "sha256:test",
        })),
        documents: vec![
            RealCorpusDocument {
                doc_id: "rust-memory".to_string(),
                vector: vec![1.0, 0.0, 0.0, 0.0],
            },
            RealCorpusDocument {
                doc_id: "python-http".to_string(),
                vector: vec![0.0, 1.0, 0.0, 0.0],
            },
            RealCorpusDocument {
                doc_id: "esp32-policy".to_string(),
                vector: vec![0.0, 0.0, 1.0, 0.0],
            },
            RealCorpusDocument {
                doc_id: "ui-design".to_string(),
                vector: vec![0.0, 0.0, 0.0, 1.0],
            },
        ],
        queries: vec![
            RealCorpusQuery {
                query_id: "q-rust".to_string(),
                vector: vec![0.92, 0.06, 0.01, 0.01],
                relevant_doc_ids: vec!["rust-memory".to_string()],
            },
            RealCorpusQuery {
                query_id: "q-esp".to_string(),
                vector: vec![0.02, 0.05, 0.9, 0.03],
                relevant_doc_ids: vec!["esp32-policy".to_string()],
            },
            RealCorpusQuery {
                query_id: "q-ui".to_string(),
                vector: vec![0.0, 0.03, 0.04, 0.93],
                relevant_doc_ids: vec!["ui-design".to_string()],
            },
        ],
    }
}

#[test]
fn hyperquant_real_corpus_eval_emits_retrieval_quality_and_gate_receipt() {
    let config = HyperQuantRealCorpusConfig {
        top_k: 2,
        candidate_k: 3,
        scale: 16.0,
        min_top_k_overlap: 0.5,
        min_exact_rerank_recovery_at_1: 1.0,
    };

    let receipt = run_hyperquant_real_corpus_eval(&tiny_semantic_corpus(), &config)
        .expect("real corpus eval succeeds");

    assert_eq!(receipt.schema, "hyperquant-real-corpus-eval-v1");
    assert_eq!(receipt.corpus_id, "tiny-semantic-fixture-v1");
    assert_eq!(receipt.embedding_model, "hand-authored-unit-vectors");
    assert_eq!(
        receipt
            .metadata
            .as_ref()
            .and_then(|m| m.get("qrels_digest"))
            .and_then(|v| v.as_str()),
        Some("sha256:test")
    );
    assert!(receipt
        .claim_boundary
        .contains("real corpus retrieval fixture"));
    assert_eq!(receipt.profiles.len(), 2);

    for profile in &receipt.profiles {
        assert!(matches!(profile.kind, LatticeKind::Z1 | LatticeKind::A2));
        assert_eq!(profile.query_count, 3);
        assert!(profile.raw_recall_at_1 >= 0.99);
        assert!(profile.raw_recall_at_5 >= profile.raw_recall_at_1);
        assert!(profile.raw_recall_at_10 >= profile.raw_recall_at_5);
        assert!(profile.codec_recall_at_1 >= 0.99);
        assert!(profile.codec_recall_at_5 >= profile.codec_recall_at_1);
        assert!(profile.codec_recall_at_10 >= profile.codec_recall_at_5);
        assert!(profile.raw_recall_at_k >= 0.99);
        assert!(profile.codec_recall_at_k >= 0.99);
        assert!(profile.top_k_overlap >= 0.5);
        assert!(profile.exact_rerank_recovery_at_1 >= 1.0);
        assert_eq!(profile.rank_drift_mean, 0.0);
        assert_eq!(profile.rank_drift_p95, 0.0);
        assert_eq!(profile.rank_drift_max, 0);
        assert!(profile.mean_score_error_at_k >= 0.0);
        assert!(profile.score_error_p95_at_k >= profile.mean_score_error_at_k);
        assert!(profile.score_error_max_at_k >= profile.score_error_p95_at_k);
        assert!(profile.raw_search_ns_total > 0);
        assert!(profile.codec_search_ns_total > 0);
        assert!(profile.compression_ratio > 1.0);
        assert!(
            profile.passed,
            "profile {:?} blockers: {:?}",
            profile.kind, profile.blockers
        );
    }
}

#[test]
fn hyperquant_real_corpus_eval_rejects_dimension_mismatch() {
    let mut corpus = tiny_semantic_corpus();
    corpus.documents[0].vector.pop();

    let err = run_hyperquant_real_corpus_eval(&corpus, &HyperQuantRealCorpusConfig::default())
        .expect_err("dimension mismatch must fail");

    assert!(err.to_string().contains("dimension"));
}
