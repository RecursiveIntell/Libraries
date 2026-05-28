use aidens_budget_kit::BudgetV1;
use aidens_contracts::{ProviderRouteKindV1, TurnFinalStateV1};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig, CanonicalEventLogEntry};
use aidens_runner::{AiDENsRunInput, AiDENsRunner};
use aidens_tool_kit::{ToolDispatcher, ToolInvocationError, ToolRegistryV1};
use async_trait::async_trait;
use llm_tool_runtime::{
    McpSurfaceKind, Tool, ToolApprovalKind, ToolBackendKind, ToolBudgetContext, ToolCall, ToolCtx,
    ToolDescriptor, ToolError, ToolErrorClass, ToolExposureMode, ToolExposurePolicy,
    ToolIdempotencyClass, ToolOriginKind, ToolOutputMode, ToolPlannerStage, ToolReceiptPersistence,
    ToolRegistry, ToolResult, ToolRuntime, ToolSideEffectClass,
};
use stack_ids::{AttemptId, ScopeKey, TraceCtx, TrialId};
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};
use verification_control::{ControlActionKind, ControlReceipt, CONTROL_RECEIPT_V1_SCHEMA};

#[tokio::test]
async fn malformed_tool_call_degrades() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-04-malformed-tool-call");
    let repo = root.join("repo");
    let receipts = root.join("receipts");
    std::fs::create_dir_all(&repo)?;
    std::fs::write(repo.join("README.md"), "phase 04 malformed fixture")?;

    let runner = AiDENsRunner::builder()
        .app_id("phase-04-malformed")
        .mock_provider(r#"tool_call: {"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}"#)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&repo)?)
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(&receipts))
        .build()?;

    let output = runner.run(AiDENsRunInput::new("read malformed")).await?;

    assert_eq!(
        output.text,
        "Turn stopped: malformed parser-fallback tool call."
    );
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::ToolFailed
    );
    assert!(output.turn_receipt.degraded);
    assert!(output.turn_receipt.blocked);
    assert!(output.receipt.tool_invocation_receipts.is_empty());
    assert!(output
        .receipt
        .warnings
        .iter()
        .any(|warning| warning.contains("malformed-parser-fallback-tool-call")));
    assert_has_control_record(
        &output.durable_receipt_records,
        "malformed-parser-fallback-tool-call",
    );

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn denied_tool_requires_approval() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-04-denied-tool");
    let repo = root.join("repo");
    let receipts = root.join("receipts");
    std::fs::create_dir_all(&repo)?;
    std::fs::write(repo.join("README.md"), "before\n")?;
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-before\n+after\n";

    let registry = ToolRegistryV1::safe_coding_with_dispatchers(&repo)?;
    let denied = ToolDispatcher::new(registry)
        .invoke("aidens:patch-apply:1", serde_json::json!({ "diff": diff }))
        .await
        .expect_err("patch apply must require approval without scoped permit");
    let denied = denied
        .downcast_ref::<ToolInvocationError>()
        .expect("typed AiDENs invocation error");

    assert!(denied.approval_request().is_some());
    assert!(denied.receipt().approval_request_id.is_some());
    assert!(denied
        .receipt()
        .reason_codes
        .contains(&"permit-required:write".into()));
    assert_eq!(std::fs::read_to_string(repo.join("README.md"))?, "before\n");

    let mut canonical_registry = ToolRegistry::new();
    canonical_registry.register(EffectfulCanonicalTool::new());
    let runtime = ToolRuntime::new(canonical_registry);
    let execution = runtime
        .execute(
            &canonical_tool_ctx(),
            &ToolCall::new(
                "phase04-write",
                "1.0.0",
                serde_json::json!({ "path": "README.md" }),
                ToolOriginKind::Test,
            ),
            None,
            None,
        )
        .await;

    let error = execution.result.expect_err("canonical runtime must deny");
    assert_eq!(error.class, ToolErrorClass::ApprovalRequired);
    assert_eq!(
        execution.receipt.error_class,
        Some(ToolErrorClass::ApprovalRequired)
    );
    assert!(matches!(
        execution.receipt.approval_state,
        llm_tool_runtime::ToolApprovalState::Denied
    ));

    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipts))?;
    let runtime_entry = log.append_runtime_tool_receipt(&execution.receipt)?;
    let control = ControlReceipt::from(&execution.receipt);
    let control_entry = log.append_control_receipt(&control)?;

    assert_eq!(runtime_entry.owner_crate, "llm-tool-runtime");
    assert_eq!(control_entry.owner_crate, "verification-control");
    assert_eq!(control.schema_version, CONTROL_RECEIPT_V1_SCHEMA);
    assert_eq!(control.action_kind, ControlActionKind::ToolExecution);
    assert_eq!(
        control.source_receipt_id.as_deref(),
        Some(execution.receipt.receipt_id.as_str())
    );
    assert!(runtime_entry.verify_digest());
    assert!(control_entry.verify_digest());

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn budget_exhaustion_receipt() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-04-budget-exhaustion");
    let repo = root.join("repo");
    let receipts = root.join("receipts");
    std::fs::create_dir_all(&repo)?;
    std::fs::write(repo.join("README.md"), "phase 04 budget fixture")?;

    let runner = AiDENsRunner::builder()
        .app_id("phase-04-budget")
        .mock_provider(
            r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}"#,
        )
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&repo)?)
        .budget(BudgetV1 {
            max_tool_calls: 0,
            max_retries: 0,
            max_turn_millis: 30_000,
        })
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(&receipts))
        .build()?;

    let output = runner.run(AiDENsRunInput::new("read budget")).await?;

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::BudgetExhausted
    );
    assert!(output.turn_receipt.blocked);
    assert_eq!(output.receipt.budget_exhaustion_receipts.len(), 1);
    assert!(output.receipt.budget_exhaustion_receipts[0]
        .reason_codes
        .contains(&"max-tool-calls-exhausted".into()));
    assert_has_control_record(&output.durable_receipt_records, "max-tool-calls-exhausted");

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

#[tokio::test]
async fn provider_route_unavailable() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-04-provider-unavailable");
    let receipts = root.join("receipts");
    let config = CanonicalEventLogConfig::for_root(&receipts);
    let runner = AiDENsRunner::builder()
        .app_id("phase-04-provider")
        .provider_kind("openai")
        .model("gpt-test")
        .api_key("configured")
        .canonical_receipt_log_config(config.clone())
        .build()?;

    let error = runner
        .run(AiDENsRunInput::new("provider should fail honestly"))
        .await
        .expect_err("unavailable provider must surface an error");

    assert!(error.to_string().contains("provider unavailable"));
    let reports = runner.run_reports().list();
    assert_eq!(reports.len(), 1);
    let route = reports[0].provider_route.as_ref().expect("provider route");
    assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
    assert!(route.degraded);
    assert!(reports[0]
        .warnings
        .contains(&"provider-unavailable".to_string()));

    let log = CanonicalEventLog::open(config)?;
    let records = log.list_records()?;
    assert_has_control_record(&records, "provider-boundary-unavailable");
    assert!(records.iter().any(|record| {
        record.owner_crate == "aidens-orchestration" && record.schema_name == "run-report-v1"
    }));

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

fn assert_has_control_record(records: &[CanonicalEventLogEntry], reason: &str) {
    let record = records
        .iter()
        .find(|record| {
            record.owner_crate == "verification-control"
                && record.schema_name == "control-receipt"
                && record.body.to_string().contains(reason)
        })
        .unwrap_or_else(|| panic!("missing verification-control record containing {reason}"));
    assert_eq!(record.body["schema_version"], CONTROL_RECEIPT_V1_SCHEMA);
    assert_eq!(record.body["degraded"], true);
    assert_eq!(record.body["promotable"], false);
    assert!(record.verify_digest());
}

fn canonical_tool_ctx() -> ToolCtx {
    ToolCtx {
        trace_ctx: TraceCtx::generate(),
        attempt_id: AttemptId::new("attempt:phase-04-denial"),
        trial_id: TrialId::new("trial:phase-04-denial"),
        deadline: None,
        workload_class: Some("phase-04-failure-honesty".into()),
        budget_context: Some(ToolBudgetContext {
            budget_kind: Some("approval".into()),
            max_steps: Some(1),
            time_budget_ms: Some(30_000),
            cost_budget_units: None,
        }),
        scope: Some(ScopeKey::namespace_only("aidens")),
        dry_run: false,
        approval_grant: None,
        execution_permit: None,
        idempotency_key: None,
        caller: "aidens-testkit".into(),
        planner_stage: ToolPlannerStage::Execution,
        parent_receipt_id: None,
        family_receipt_id: None,
        replay_parent_receipt_id: None,
        remote_oracle_lease_id: None,
        remote_slice_result_id: None,
        attestation_envelope_id: None,
        cross_runtime_replay_ticket_id: None,
        retry_owner: Some(llm_tool_runtime::ToolRetryOwner::ForgeOrchestration),
    }
}

struct EffectfulCanonicalTool {
    descriptor: ToolDescriptor,
    invoked: AtomicBool,
}

impl EffectfulCanonicalTool {
    fn new() -> Self {
        Self {
            descriptor: ToolDescriptor {
                name: "phase04-write".into(),
                version: "1.0.0".into(),
                description: Some("phase 04 denied write fixture".into()),
                backend_kind: ToolBackendKind::LocalFunction,
                input_schema: serde_json::json!({
                    "type": "object",
                    "required": ["path"],
                    "properties": {
                        "path": { "type": "string" }
                    }
                }),
                output_mode: ToolOutputMode::StructuredJson,
                read_only: false,
                side_effect_class: ToolSideEffectClass::Write,
                idempotency_class: ToolIdempotencyClass::NonIdempotent,
                approval_kind: ToolApprovalKind::UserRequired,
                timeout_ms: 30_000,
                concurrency_key: None,
                cache_ttl_ms: None,
                exposure_mode: ToolExposureMode::Auto,
                mcp_surface_kind: McpSurfaceKind::None,
                exposure_policy: ToolExposurePolicy::default(),
                receipt_persistence: ToolReceiptPersistence::Ephemeral,
                output_size_limit_bytes: None,
                provider_payload: None,
            },
            invoked: AtomicBool::new(false),
        }
    }
}

#[async_trait]
impl Tool for EffectfulCanonicalTool {
    fn descriptor(&self) -> &ToolDescriptor {
        &self.descriptor
    }

    async fn invoke(&self, _ctx: &ToolCtx, _call: &ToolCall) -> Result<ToolResult, ToolError> {
        self.invoked.store(true, Ordering::SeqCst);
        Ok(ToolResult::json(
            serde_json::json!({ "unexpected": "executed" }),
        ))
    }
}

fn temp_root(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system time after unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{unique}", std::process::id()))
}
