use std::sync::Arc;

use chrono::Utc;
use llm_pipeline::payload::Payload;
use llm_pipeline::receipts::{ReceiptSigner, ReceiptVerifier};
use llm_pipeline::{ExecCtx, ExecutionOutcome, LlmCall, MockBackend, PipelineExecutionReceiptV1};
use semantic_memory::llm_receipt_ingest::LlmReceiptEvidence;
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use serde_json::json;
use tempfile::tempdir;

#[tokio::test]
async fn llm_receipt_provenance_round_trip() {
    let backend = Arc::new(MockBackend::fixed(r#"{"answer":"provenance-ok"}"#));
    let ctx = ExecCtx::builder("http://mock.invalid")
        .backend(backend)
        .build();
    let call = LlmCall::new("provenance", "Return JSON for {input}")
        .expecting_json()
        .with_model("mock-model");
    let output = call
        .invoke(&ctx, json!("the provenance test"))
        .await
        .unwrap();
    assert_eq!(output.value["answer"], "provenance-ok");

    let response_json = serde_json::to_string(&output.value).unwrap();
    let mut receipt = PipelineExecutionReceiptV1 {
        receipt_version: "1".into(),
        crate_version: env!("CARGO_PKG_VERSION").into(),
        integrity_tag: None,
        previous_receipt_digest: None,
        traceparent: None,
        tracestate: None,
        chain_valid: false,
        receipt_id: "receipt-e2e-001".into(),
        pipeline_id: "provenance-test".into(),
        provider_calls: Vec::new(),
        retry_decisions: Vec::new(),
        budget_debits: Vec::new(),
        response_digest: LlmReceiptEvidence::compute_digest(&response_json),
        outcome: ExecutionOutcome::Success,
        recorded_time: Utc::now(),
    };

    let key = [7_u8; 32];
    ReceiptSigner::new(key).sign(&mut receipt).unwrap();
    let signed_json = serde_json::to_string(&receipt).unwrap();
    let digest = LlmReceiptEvidence::compute_digest(&signed_json);
    let evidence = LlmReceiptEvidence::new(
        &signed_json,
        &digest,
        true,
        receipt.traceparent.clone(),
        &receipt.pipeline_id,
        "mock",
        "mock-model",
    );

    let dir = tempdir().unwrap();
    let mut config = MemoryConfig::default();
    config.base_dir = dir.path().to_path_buf();
    let store = MemoryStore::open_with_embedder(config, Box::new(MockEmbedder::new(768))).unwrap();
    let metadata = serde_json::to_value(&evidence).unwrap();
    let fact_id = store
        .add_fact(
            "llm-executions",
            "LLM provenance receipt completed successfully",
            Some("llm-pipeline"),
            Some(metadata),
        )
        .await
        .unwrap();

    let verification = ReceiptVerifier::new(key).verify(&mut receipt, None);
    assert!(verification.integrity_ok);
    assert!(verification.chain_ok);
    assert!(verification.trace_ok);

    let results = store
        .search(
            "provenance receipt completed",
            Some(5),
            Some(&["llm-executions"]),
            None,
        )
        .await
        .unwrap();
    assert!(results
        .iter()
        .any(|result| result.content.contains("provenance receipt")));
    let fact = store.get_fact(&fact_id).await.unwrap().unwrap();
    let stored: LlmReceiptEvidence = serde_json::from_value(fact.metadata.unwrap()).unwrap();
    assert_eq!(stored.receipt_digest, digest);
    assert_eq!(stored.receipt_json, signed_json);
    assert!(stored.integrity_verified);
}
