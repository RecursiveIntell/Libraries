use quant_eval::{
    run_compressed_scorer_real_corpus_eval, CompressedScorerRealCorpusConfig, HyperQuantRealCorpus,
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
fn compressed_scorer_real_corpus_eval_emits_candidate_receipt() {
    let config = CompressedScorerRealCorpusConfig {
        top_k: 2,
        candidate_k: 3,
        bits: 8,
        min_top_k_overlap: 0.5,
        min_exact_rerank_recovery_at_1: 1.0,
    };

    let receipt = run_compressed_scorer_real_corpus_eval(&tiny_semantic_corpus(), &config)
        .expect("compressed scorer real corpus eval succeeds");

    assert_eq!(receipt.schema, "compressed-scorer-real-corpus-eval-v1");
    assert_eq!(receipt.corpus_id, "tiny-semantic-fixture-v1");
    assert!(receipt
        .claim_boundary
        .contains("candidate-gate evidence only"));
    assert_eq!(receipt.profiles.len(), 1);

    let profile = &receipt.profiles[0];
    assert_eq!(profile.name, "per_dim_8bit");
    assert_eq!(profile.family, "compressed-scorer");
    assert_eq!(
        profile.scoring_path,
        "lookup_table_compressed_domain_score_then_exact_f32_rerank"
    );
    assert_eq!(profile.query_count, 3);
    assert_eq!(profile.doc_count, 4);
    assert_eq!(profile.decoded_doc_count, 0);
    assert!(profile.exact_rerank_count > 0);
    assert!(profile.raw_recall_at_1 >= 0.99);
    assert!(profile.codec_recall_at_1 >= 0.99);
    assert!(profile.top_k_overlap >= 0.5);
    assert!(profile.exact_rerank_recovery_at_1 >= 1.0);
    assert!(profile.raw_search_ns_total > 0);
    assert!(profile.codec_search_ns_total > 0);
    assert!(profile.compression_ratio > 1.0);
    assert!(profile.passed, "blockers: {:?}", profile.blockers);
}

#[test]
fn compressed_scorer_real_corpus_eval_rejects_dimension_mismatch() {
    let mut corpus = tiny_semantic_corpus();
    corpus.documents[0].vector.pop();

    let err = run_compressed_scorer_real_corpus_eval(
        &corpus,
        &CompressedScorerRealCorpusConfig::default(),
    )
    .expect_err("dimension mismatch must fail");

    assert!(err.to_string().contains("dimension"));
}
