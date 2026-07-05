use quant_eval::{
    run_hyperquant_real_corpus_eval, HyperQuantRealCorpus, HyperQuantRealCorpusConfig,
    RealCorpusDocument, RealCorpusQuery,
};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let corpus = HyperQuantRealCorpus {
        corpus_id: "tiny-semantic-fixture-v1".to_string(),
        embedding_model: "hand-authored-unit-vectors".to_string(),
        metadata: Some(serde_json::json!({
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
    };
    let config = HyperQuantRealCorpusConfig {
        top_k: 2,
        candidate_k: 3,
        scale: 16.0,
        min_top_k_overlap: 0.5,
        min_exact_rerank_recovery_at_1: 1.0,
    };
    let receipt = run_hyperquant_real_corpus_eval(&corpus, &config)?;
    println!("{}", serde_json::to_string_pretty(&receipt)?);
    Ok(())
}
