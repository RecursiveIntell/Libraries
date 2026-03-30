use forge_engine::{ForgeStore, ForgeToolReceiptSink};
use llm_tool_runtime::{
    ToolApprovalState, ToolBackendKind, ToolBudgetContext, ToolPlannerStage, ToolReceipt,
    ToolReceiptSink, ToolRetryOwner,
};
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
