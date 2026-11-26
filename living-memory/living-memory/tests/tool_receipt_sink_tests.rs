use forge_engine::{ForgeStore, ForgeToolReceiptSink};
use llm_tool_runtime::semantic_memory::ToolObservationMemory;
use llm_tool_runtime::{
    ToolApprovalState, ToolBackendKind, ToolBudgetContext, ToolPlannerStage, ToolReceipt,
    ToolReceiptSink, ToolRetryOwner,
};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use semantic_memory_forge::ForgeToolReceiptV2;
use stack_ids::{AttemptId, ContentDigest, TraceCtx, TrialId};
use std::sync::Arc;
use tempfile::TempDir;

fn sample_receipt() -> ToolReceipt {
    ToolReceipt {
        receipt_id: "receipt-1".into(),
        tool_run_id: "tool-run-1".into(),
        tool_name: "run_verification".into(),
        tool_version: "1".into(),
        backend_kind: ToolBackendKind::LocalFunction,
        input_digest: ContentDigest::compute_str("input"),
        output_digest_or_refs: serde_json::json!({ "job": { "job_id": "job-1" } }),
        policy_hash: ContentDigest::compute_str("policy"),
        approval_state: ToolApprovalState::Approved,
        host_identity: "host-1".into(),
        started_at: "2026-03-09T00:00:00Z".into(),
        finished_at: "2026-03-09T00:00:01Z".into(),
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::new("attempt-1"),
        trial_id: TrialId::new("trial-1"),
        planner_stage: ToolPlannerStage::Execution,
        deadline: Some("2026-03-12T00:00:05Z".into()),
        workload_class: Some("verification".into()),
        budget_context: Some(ToolBudgetContext {
            budget_kind: Some("control_plane".into()),
            max_steps: Some(4),
            time_budget_ms: Some(2_000),
            cost_budget_units: Some(8),
        }),
        parent_receipt_id: Some("dispatch-1".into()),
        family_receipt_id: Some("attempt-family-1".into()),
        replay_parent_receipt_id: Some("retry-0".into()),
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        error_class: None,
        retry_owner: ToolRetryOwner::LlmPipeline,
        replay_link: Some("replay://tool-run-1".into()),
        provider_call_id: Some("provider-call-1".into()),
    }
}

#[tokio::test]
async fn forge_tool_receipt_sink_persists_raw_receipt() {
    let dir = TempDir::new().unwrap();
    let store = Arc::new(ForgeStore::open(&dir.path().join("forge.db")).unwrap());
    let sink = ForgeToolReceiptSink::new(store.clone());
    let receipt = sample_receipt();

    sink.persist(&receipt).await.unwrap();

    let stored = store.get_tool_receipt("receipt-1").unwrap().unwrap();
    assert_eq!(stored.tool_name, "run_verification");
    assert_eq!(stored.attempt_id, "attempt-1");
    assert_eq!(stored.trial_id, "trial-1");
    assert_eq!(stored.provider_call_id.as_deref(), Some("provider-call-1"));
    assert_eq!(stored.trace_id, receipt.trace_ctx.trace_id);
    assert!(stored.raw_payload_json.contains("run_verification"));

    let raw: ForgeToolReceiptV2 = serde_json::from_str(&stored.raw_payload_json).unwrap();
    assert_eq!(raw.deadline.as_deref(), Some("2026-03-12T00:00:05Z"));
    assert_eq!(raw.workload_class.as_deref(), Some("verification"));
    assert_eq!(
        raw.budget_context
            .as_ref()
            .and_then(|budget| budget.max_steps),
        Some(4)
    );
    assert_eq!(raw.parent_receipt_id.as_deref(), Some("dispatch-1"));
    assert_eq!(raw.family_receipt_id.as_deref(), Some("attempt-family-1"));
    assert_eq!(raw.replay_parent_receipt_id.as_deref(), Some("retry-0"));
    assert_eq!(raw.planner_stage, "Execution");
}

#[tokio::test]
async fn receipt_retries_preserve_first_write_and_reject_conflicts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("forge.db");
    let store = Arc::new(ForgeStore::open(&path).unwrap());
    let retry_store = ForgeStore::open(&path).unwrap();
    let sink = ForgeToolReceiptSink::new(store.clone());
    let mut receipt = sample_receipt();
    sink.persist(&receipt).await.unwrap();

    let first = store.get_tool_receipt("receipt-1").unwrap().unwrap();
    let mut replay = store.get_tool_receipt("receipt-1").unwrap().unwrap();
    replay.recorded_at = "2030-01-01T00:00:00Z".into();
    assert!(!retry_store.insert_tool_receipt(&replay).unwrap());
    sink.persist(&receipt).await.unwrap();

    replay.error_class = Some(String::new());
    let error = retry_store.insert_tool_receipt(&replay).unwrap_err();
    assert!(matches!(
        error,
        forge_engine::ForgeError::ToolReceiptConflict { .. }
    ));

    receipt.input_digest = ContentDigest::compute_str("conflicting input");
    let error = sink.persist(&receipt).await.unwrap_err();
    assert_eq!(
        error.class,
        llm_tool_runtime::ToolErrorClass::ReceiptPersistence
    );
    let stored = store.get_tool_receipt("receipt-1").unwrap().unwrap();
    assert_eq!(stored, first);
}

#[tokio::test]
async fn projection_reopens_only_committed_scoped_receipts() {
    let dir = TempDir::new().unwrap();
    let path = dir.path().join("forge.db");
    let memory_config = MemoryConfig {
        base_dir: dir.path().join("mem"),
        ..Default::default()
    };
    let open_memory = || {
        MemoryStore::open_with_embedder(
            memory_config.clone(),
            Box::new(MockEmbedder::new(memory_config.embedding.dimensions)),
        )
        .unwrap()
    };
    let memory = open_memory();
    let store = Arc::new(ForgeStore::open(&path).unwrap());
    let sink =
        ForgeToolReceiptSink::with_observation_scope(store.clone(), "run_verification", "pilot")
            .unwrap();
    let wrong_scope =
        ForgeToolReceiptSink::with_observation_scope(store.clone(), "run_verification", "other")
            .unwrap();
    let mut receipt = sample_receipt();
    assert!(sink
        .project_observation(&memory, &receipt.receipt_id)
        .await
        .is_err());
    sink.persist(&receipt).await.unwrap();
    let first = store
        .get_tool_receipt(&receipt.receipt_id)
        .unwrap()
        .unwrap();
    assert!(wrong_scope
        .project_observation(&memory, &receipt.receipt_id)
        .await
        .is_err());
    assert!(wrong_scope.persist(&receipt).await.is_err());
    receipt.receipt_id = "legacy-receipt".into();
    ForgeToolReceiptSink::new(store.clone())
        .persist(&receipt)
        .await
        .unwrap();
    assert!(sink
        .project_observation(&memory, &receipt.receipt_id)
        .await
        .is_err());
    receipt.receipt_id = "failed-receipt".into();
    receipt.error_class = Some(llm_tool_runtime::ToolErrorClass::Execution);
    sink.persist(&receipt).await.unwrap();
    drop(wrong_scope);
    drop(sink);
    drop(store);
    let reopened = Arc::new(ForgeStore::open(&path).unwrap());
    let projector =
        ForgeToolReceiptSink::with_observation_scope(reopened.clone(), "run_verification", "pilot")
            .unwrap();
    for id in ["receipt-1", "receipt-1", "failed-receipt"] {
        projector.project_observation(&memory, id).await.unwrap();
    }
    assert_eq!(reopened.get_tool_receipt("receipt-1").unwrap(), Some(first));
    drop(memory);
    let memory = open_memory();
    let mut results = memory
        .find_similar_tool_observations("run_verification", "pilot", 10)
        .await
        .unwrap()
        .observations;
    results.sort_by(|a, b| a.receipt_id.cmp(&b.receipt_id));
    assert_eq!(results.len(), 2);
    assert_eq!(results[0].summary, "Tool run_verification failed");
    assert_eq!(results[1].summary, "Tool run_verification succeeded");
    assert_eq!(results[1].receipt_id.as_deref(), Some("receipt-1"));
    assert!(memory
        .find_similar_tool_observations("run_verification", "other", 10)
        .await
        .unwrap()
        .observations
        .is_empty());
}
