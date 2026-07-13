use super::*;
use aidens_contracts::{
    ArtifactId, ArtifactKindV1, CanonicalToolSideEffectClass, PermitGrantV1, ProviderRouteKindV1,
    RetrievalPolicyV1, RuntimeViewModeV1, ToolInvocationReportV1, TurnFinalStateV1, TurnModeV1,
};

#[test]
fn runtime_view_disclosure_is_receipt_only_not_a_truth_store() {
    let policy = RetrievalPolicyV1::timeless(RuntimeViewModeV1::Execution);
    let request = RuntimeViewRequestV1::new("execution view", policy.clone());
    let projection = ProjectionDigestV1::new(
        RuntimeViewModeV1::Execution,
        policy.policy_id.clone(),
        Vec::new(),
        Vec::new(),
        Vec::new(),
        serde_json::json!({"view":"execution","claims":[]}),
    );
    let degradation = DegradationEventV1::new(
        &request,
        "execution-view-disclosed-without-domain-merge",
        false,
        false,
        Vec::new(),
    );

    let disclosure = disclose_runtime_view(&request, projection, Vec::new(), &[], &[degradation]);

    assert_eq!(disclosure.kind, ArtifactKindV1::ViewDisclosure);
    assert_eq!(
        disclosure.authoritative_source,
        "append-only-memory-and-receipts"
    );
    assert!(disclosure.separates_execution_from_domain_truth);
    assert_eq!(disclosure.degradation_event_ids.len(), 1);
    assert!(disclosure.matched_claim_ids.is_empty());
}

#[test]
fn p30_run_report_ledger_recovers_from_poisoned_lock() {
    let ledger = RunReportLedger::default();
    let poison_target = ledger.clone();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = poison_target.reports.lock().unwrap();
        std::panic::resume_unwind(Box::new("poison run report ledger"));
    }));

    assert!(ledger.is_empty());
    let report = RunReportV1::started(AidensRunContextV1::new("poisoned-ledger")).complete();
    assert_eq!(ledger.append(report), 1);
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.list().len(), 1);
}

#[tokio::test]
async fn p30_agency_nudge_ledger_poison_recovery_is_receipted() {
    let ledger = std::sync::Arc::new(std::sync::Mutex::new(NudgeLedgerV1::default()));
    let poison_target = ledger.clone();
    let _ = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let _guard = poison_target.lock().unwrap();
        std::panic::resume_unwind(Box::new("poison agency nudge ledger"));
    }));

    let runner = AiDENsRunner::builder()
        .app_id("agency-poison-recovery")
        .mock_provider("ok")
        .agency_nudge_ledger(ledger)
        .build()
        .unwrap();

    let output = runner.run(AiDENsRunInput::new("hello")).await.unwrap();

    assert!(output.agency_policy_reports.iter().any(|report| report
        .reason_codes()
        .contains(&"agency-nudge-ledger-poison-recovered".into())));
}

#[tokio::test]
async fn run_always_appends_completed_receipt_on_success() {
    let ledger = RunReportLedger::default();
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider("ok")
        .run_reports(ledger.clone())
        .build()
        .unwrap();

    let output = runner.run(AiDENsRunInput::new("hello")).await.unwrap();

    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.list()[0].receipt_id, output.receipt.receipt_id);
    assert!(output.receipt.completed_at.is_some());
    assert!(output.receipt.provider_route.is_some());
    assert_eq!(output.receipt.tool_exposure_ids.len(), 1);
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
}

#[tokio::test]
async fn completed_report_is_not_published_when_canonical_persistence_fails() {
    let ledger = RunReportLedger::default();
    let runner = AiDENsRunner::builder()
        .app_id("canonical-persistence-failure")
        .mock_provider("ok")
        .run_reports(ledger.clone())
        .build()
        .unwrap();

    runner.fail_next_completed_persistence();
    let error = runner.run(AiDENsRunInput::new("hello")).await.unwrap_err();

    assert!(error
        .to_string()
        .contains("injected canonical receipt persistence failure"));
    assert!(ledger.is_empty());
}

#[tokio::test]
async fn turn_executor_without_canonical_store_publishes_completed_report_once() {
    let ledger = RunReportLedger::default();
    let runner = AiDENsRunner::builder()
        .app_id("no-canonical-store")
        .mock_provider("ok")
        .run_reports(ledger.clone())
        .build()
        .unwrap();
    let executor = TurnExecutorV1::new(TurnExecutorConfigV1 {
        app_id: runner.app_id.clone(),
        provider: runner.provider.clone(),
        tools: runner.tools.clone(),
        permit_policy: runner.permit_policy.clone(),
        budget: runner.budget.clone(),
        run_reports: runner.run_reports.clone(),
        receipt_level: runner.receipt_level.clone(),
        canonical_receipts: None,
        fail_next_completed_persistence: runner.fail_next_completed_persistence.clone(),
        agency_policy: runner.agency_policy.clone(),
        agency_nudges: runner.agency_nudges.clone(),
        governance: runner.governance.clone(),
        kernel: runner.kernel,
    });

    let output = executor
        .execute(AiDENsRunInput::new("hello"))
        .await
        .unwrap();

    assert!(output.durable_receipt_records.is_empty());
    assert_eq!(ledger.len(), 1);
    assert_eq!(ledger.list()[0].receipt_id, output.receipt.receipt_id);
}

#[tokio::test]
async fn disabled_provider_records_receipt_and_fails() {
    let ledger = RunReportLedger::default();
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .run_reports(ledger.clone())
        .build()
        .unwrap();

    let error = runner.run(AiDENsRunInput::new("hello")).await.unwrap_err();

    assert!(error.to_string().contains("unavailable"));
    assert_eq!(ledger.len(), 1);
    let receipt = ledger.list()[0].clone();
    let route = receipt.provider_route.clone().unwrap();
    assert_eq!(route.route, ProviderRouteKindV1::Disabled);
    assert_eq!(
        receipt.turn_receipts[0].final_state,
        TurnFinalStateV1::ProviderUnavailable
    );
}

#[tokio::test]
async fn default_runner_failure_is_report_rich_with_durable_store() {
    let runner = AiDENsRunner::builder()
        .app_id("default-durable-direct-runner")
        .build()
        .unwrap();

    let error = runner.run(AiDENsRunInput::new("hello")).await.unwrap_err();

    assert!(error.to_string().contains("unavailable"));
    assert_eq!(runner.receipt_level(), &ReportLevelV1::Full);
    assert!(runner.canonical_receipt_log_config().is_some());
    assert_eq!(runner.run_reports().len(), 1);
}

#[tokio::test]
async fn canonical_log_reopens_runner_report_after_process_boundary() {
    let root =
        std::env::temp_dir().join(format!("aidens-p05-runner-reopen-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let config = CanonicalEventLogConfig::for_root(&root);
    let runner = AiDENsRunner::builder()
        .app_id("durable-runner")
        .mock_provider("durable ok")
        .canonical_receipt_log_config(config.clone())
        .build()
        .unwrap();

    let output = runner.run(AiDENsRunInput::new("hello")).await.unwrap();
    drop(runner);

    let reopened = CanonicalEventLog::open(config).unwrap();
    let record = reopened
        .inspect(output.receipt.receipt_id.as_ref())
        .unwrap();

    assert_eq!(output.text, "durable ok");
    assert_eq!(output.durable_receipt_records.len(), 4);
    assert!(output
        .durable_receipt_records
        .iter()
        .any(|record| record.schema_name == "tool-exposure-plan-v1"));
    assert!(output.durable_receipt_records.iter().any(|record| {
        record.owner_crate == "aidens-agency-kit" && record.schema_name == "agency-policy-report-v1"
    }));
    assert!(!output.receipt.agency_receipt_ids.is_empty());
    assert!(output.durable_receipt_records.iter().any(|record| {
        record.owner_crate == "verification-control"
            && record.schema_name == "control-receipt"
            && record.body.to_string().contains("final-output-produced")
    }));
    let final_output_control = output
        .durable_receipt_records
        .iter()
        .find(|record| {
            record.owner_crate == "verification-control"
                && record.schema_name == "control-receipt"
                && record.body.to_string().contains("final-output-produced")
        })
        .expect("final-output control receipt");
    assert_eq!(
        final_output_control.body["details"]["verification_attempt_state"],
        serde_json::json!("advisory_only")
    );
    assert_eq!(
        final_output_control.body["advisory_only"],
        serde_json::json!(true)
    );
    assert_eq!(record.owner_crate, "aidens-orchestration");
    assert_eq!(record.schema_name, "run-report-v1");
    assert!(record.verify_digest());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn canonical_log_records_provider_unavailable_report() {
    let root = std::env::temp_dir().join(format!(
        "aidens-p05-provider-unavailable-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let config = CanonicalEventLogConfig::for_root(&root);
    let runner = AiDENsRunner::builder()
        .app_id("durable-provider-unavailable")
        .canonical_receipt_log_config(config.clone())
        .build()
        .unwrap();

    let error = runner.run(AiDENsRunInput::new("hello")).await.unwrap_err();
    let reopened = CanonicalEventLog::open(config).unwrap();
    let records = reopened.list_records().unwrap();

    assert!(error.to_string().contains("provider unavailable"));
    assert_eq!(records.len(), 3);
    assert!(records
        .iter()
        .any(|record| record.schema_name == "tool-exposure-plan-v1"));
    assert!(records.iter().any(|record| {
        record.owner_crate == "verification-control"
            && record.schema_name == "control-receipt"
            && record.body.to_string().contains("provider-disabled")
    }));
    assert!(records
        .iter()
        .any(|record| record.schema_name == "run-report-v1"));
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn canonical_log_records_tool_and_boundary_failure_report() {
    let root =
        std::env::temp_dir().join(format!("aidens-p05-runner-failures-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&root);
    let config = CanonicalEventLogConfig::for_root(&root);
    let mock_script = r#"{"tool_call":"aidens:shell:1"}"#;
    let runner = AiDENsRunner::builder()
        .app_id("durable-failure-runner")
        .mock_provider(mock_script)
        .canonical_receipt_log_config(config.clone())
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("request blocked tool"))
        .await
        .unwrap();
    let reopened = CanonicalEventLog::open(config).unwrap();
    let records = reopened.list_records().unwrap();

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::ToolFailed
    );
    assert!(output
        .receipt
        .warnings
        .iter()
        .any(|warning| warning.contains("parser-fallback")));
    assert!(output.receipt.boundary_repair_receipts.is_empty());
    assert_eq!(records.len(), 3);
    assert!(records
        .iter()
        .any(|record| record.schema_name == "tool-exposure-plan-v1"));
    assert!(records
        .iter()
        .any(|record| record.schema_name == "control-receipt"));
    assert!(records
        .iter()
        .any(|record| record.schema_name == "run-report-v1"));
    let _ = std::fs::remove_dir_all(&root);
}

#[test]
fn parser_fallback_blocks_repaired_tool_call_payloads() {
    let parsed = parse_parser_fallback_tool_calls(
        "```json\n{\"tool_call\":{\"tool_id\":\"aidens:repo-read:1\",\"input\":{\"path\":\"README.md\"}}}\n```",
    );

    assert!(parsed.calls.is_empty());
    assert!(parsed.boundary_repair_receipts.is_empty());
    assert!(parsed.degradation_reason_codes.is_empty());
}

#[test]
fn parser_fallback_rejects_malformed_entries_without_dropping_them() {
    let parsed = parse_parser_fallback_tool_calls(
        r#"{"tool_calls":[{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}},{"input":{"path":"secret.txt"}}]}"#,
    );

    assert_eq!(parsed.calls.len(), 1);
    assert_eq!(parsed.calls[0].tool_id, "aidens:repo-read:1");
    assert!(parsed.calls[0]
        .reason_codes
        .iter()
        .any(|reason| reason == "parser-fallback-tool-call"));
    assert!(parsed.calls[0]
        .reason_codes
        .iter()
        .any(|reason| reason == "parser-fallback-degraded"));
    assert!(parsed
        .degradation_reason_codes
        .iter()
        .any(|reason| reason.contains("reason=missing-tool-id")));
    assert!(parsed
        .degradation_reason_codes
        .contains(&"parser-fallback-rejected-malformed-tool-call".into()));
}

#[test]
fn looks_like_tool_call_payload_requires_strict_json_shape() {
    assert!(!looks_like_tool_call_payload(
        "The assistant mentioned a `tool_call` here as a plain sentence.",
    ));
    assert!(!looks_like_tool_call_payload(
        "```json\n{\"tool_call\":{\"tool_id\":\"aidens:repo-read:1\",\"input\":{\"path\":\"README.md\"}}}\n```",
    ));
    assert!(looks_like_tool_call_payload(
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}"#,
    ));
    assert!(!looks_like_tool_call_payload(
        r#"{"meta":{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}}"#,
    ));
}

#[test]
fn parser_fallback_valid_call_carries_degradation_on_request() {
    let parsed = parse_parser_fallback_tool_calls(
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}"#,
    );

    assert_eq!(parsed.calls.len(), 1);
    assert!(parsed.degradation_reason_codes.is_empty());
    let call = &parsed.calls[0];
    assert_eq!(call.source, ToolCallSourceV1::ParserFallback);
    assert!(call.degraded);
    assert!(call
        .reason_codes
        .contains(&"parser-fallback-tool-call".into()));
    assert!(call
        .reason_codes
        .contains(&"parser-fallback-degraded".into()));
}

#[test]
fn completion_request_serializes_tool_results_without_empty_substitution() {
    let request_call = ToolCallRequestV1::new(
        ToolCallSourceV1::ParserFallback,
        "aidens:repo-read:1",
        serde_json::json!({"path":"README.md"}),
        Some("raw-provider-text".into()),
        vec!["parser-fallback-tool-call".into()],
    );
    let invocation = ToolInvocationReportV1::started(
        "aidens:repo-read:1",
        serde_json::json!({"path":"README.md"}),
    )
    .complete_success(serde_json::json!({"content":"readme"}));
    let result = ToolCallResultV1::from_invocation(&request_call, &invocation);

    let request = completion_request(
        "continue".into(),
        &empty_tool_exposure(),
        &[result],
        std::slice::from_ref(&request_call),
    )
    .unwrap();

    // messages: [system, user, assistant(tool_calls), tool(result)] = 4
    assert_eq!(request.messages.len(), 4);
    assert_eq!(request.messages[0].role, "system");
    assert_eq!(request.messages[2].role, "assistant");
    assert_eq!(request.messages[3].role, "tool");
    assert!(!request.messages[3].content.is_empty());
    assert_eq!(request.tool_results[0].tool_id, "aidens:repo-read:1");
    assert_eq!(
        request.tool_results[0].output.as_ref().unwrap()["content"],
        serde_json::json!("readme")
    );
}

fn empty_tool_exposure() -> ToolExposureSetV1 {
    ToolExposureSetV1 {
        exposure_id: ArtifactId::new("test-tool-exposure"),
        declared_tool_ids: Vec::new(),
        registered_tool_ids: Vec::new(),
        executable_tool_ids: Vec::new(),
        exposed_tool_ids: Vec::new(),
        hidden_tool_ids: Vec::new(),
        blocked_tool_ids: Vec::new(),
        decisions: Vec::new(),
        approval_requests: Vec::new(),
        permit_use_receipts: Vec::new(),
        provider_tool_schemas: Vec::new(),
        sandbox_root: None,
        degraded: false,
        reason_codes: Vec::new(),
        canonical_backpointers: Vec::new(),
        reason: None,
    }
}

#[tokio::test]
async fn unavailable_api_provider_never_claims_native_tool_loop() {
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .provider_kind("openai")
        .model("gpt-test")
        .api_key("configured")
        .build()
        .unwrap();

    let route = runner.provider_route();

    assert_eq!(route.route, ProviderRouteKindV1::Unavailable);
    assert_eq!(route.route_label, "unavailable");
    assert!(!route.native_tool_loop);
    assert!(route
        .reason_codes
        .contains(&"provider-boundary-unavailable".into()));
}

#[tokio::test]
async fn run_exposes_only_safe_registered_tools_by_default() {
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider("ok")
        .build()
        .unwrap();

    assert!(runner.tool_ids().contains(&"aidens:repo-read:1".into()));
    assert!(runner
        .executable_tool_ids()
        .contains(&"aidens:repo-read:1".into()));

    let output = runner.run(AiDENsRunInput::new("inspect")).await.unwrap();

    assert!(output
        .tool_exposure
        .exposed_tool_ids
        .contains(&"aidens:repo-read:1".into()));
    assert!(!output
        .tool_exposure
        .exposed_tool_ids
        .contains(&"aidens:shell:1".into()));
    assert_eq!(output.turn_receipt.mode, TurnModeV1::ParserFallback);
}

#[tokio::test]
async fn mock_provider_can_request_repo_read_and_receive_result() {
    let dir = std::env::temp_dir().join(format!("aidens-p03-runner-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "p03 fixture content").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let mock_script = r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}
---aidens-next-response---
final answer saw: {{last_tool_content}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider(mock_script)
        .tools(tools)
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("read README"))
        .await
        .unwrap();

    assert!(output.text.contains("p03 fixture content"));
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    let invocation = &output.receipt.tool_invocation_receipts[0];
    assert_eq!(invocation.tool_id, "aidens:repo-read:1");
    assert!(invocation.succeeded);
    assert!(invocation.run_id.is_some());
    assert!(invocation.attempt_id.is_some());
    assert!(invocation.input_digest.starts_with("blake3:"));
    assert!(invocation.output_digest.is_some());
    assert_eq!(output.receipt.tool_call_requests.len(), 1);
    assert_eq!(output.receipt.tool_call_results.len(), 1);
    assert!(output.turn_receipt.degraded);
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn successful_tool_dispatch_is_observed_without_claiming_causal_verification() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p30-advisory-tool-observation-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "advisory observation fixture").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let policy = governance_stack::PolicySnapshot::permissive(
        "p30-advisory-tool-observation-policy",
        "2026-07-13T00:00:00Z",
    );
    let mock_script = r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}
---aidens-next-response---
final: {{last_tool_content}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("p30-advisory-tool-observation")
        .mock_provider(mock_script)
        .tools(tools)
        .governance(Some(GovernanceContext::new(policy)))
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(dir.join("receipts")))
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("read README"))
        .await
        .unwrap();

    assert!(
        output.receipt.tool_invocation_receipts[0].succeeded,
        "{:?}",
        output.receipt.tool_invocation_receipts[0]
    );
    let control_receipt_id = output
        .receipt
        .warnings
        .iter()
        .find_map(|warning| warning.strip_prefix("governance-control:"))
        .expect("tool governance control receipt id");
    let control_record = runner
        .canonical_receipts
        .as_ref()
        .expect("canonical receipt store")
        .inspect(control_receipt_id)
        .unwrap();
    assert_eq!(control_record.body["advisory_only"], true);
    assert_eq!(
        control_record.body["details"]["check_method"],
        serde_json::json!("advisory_only")
    );
    assert_eq!(
        control_record.body["details"]["verification_attempt_state"],
        serde_json::json!("advisory_only")
    );
    assert_eq!(
        control_record.body["details"]["execution_observation"]["tool_invocation"]["succeeded"],
        true
    );
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn schema_invalid_tool_call_stops_before_tool_invocation() {
    let dir = std::env::temp_dir().join(format!("aidens-p06-runner-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "p06 fixture content").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let mock_script = r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":7}}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider(mock_script)
        .tools(tools)
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("read invalid input"))
        .await
        .unwrap();

    assert_eq!(output.text, "Turn stopped: tool invocation failed.");
    assert_eq!(output.receipt.schema_validation_receipts.len(), 1);
    assert!(!output.receipt.schema_validation_receipts[0].valid);
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    assert!(output.receipt.tool_invocation_receipts[0]
        .reason_codes
        .contains(&"schema-validation-failed".into()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn budget_exhaustion_blocks_without_dispatch_loop() {
    let dir = std::env::temp_dir().join(format!("aidens-p03-budget-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "budget fixture").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let mock_script =
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider(mock_script)
        .tools(tools)
        .budget(BudgetV1 {
            max_tool_calls: 0,
            max_retries: 0,
            max_turn_millis: 30_000,
        })
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("read README"))
        .await
        .unwrap();

    assert!(output.text.contains("budget exhausted"));
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::BudgetExhausted
    );
    assert!(output.turn_receipt.blocked);
    assert_eq!(output.receipt.budget_exhaustion_receipts.len(), 1);
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    assert!(output.receipt.tool_invocation_receipts[0]
        .reason_codes
        .contains(&"budget-exhausted-before-dispatch".into()));
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn unexposed_tool_call_is_blocked_before_dispatch() {
    let mock_script = r#"{"tool_call":{"tool_id":"aidens:shell:1","input":{"cmd":"date"}}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("test-app")
        .mock_provider(mock_script)
        .build()
        .unwrap();

    let output = runner.run(AiDENsRunInput::new("run shell")).await.unwrap();

    assert!(output.text.contains("not exposed"));
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::ToolBlocked
    );
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    assert!(output.receipt.tool_invocation_receipts[0]
        .reason_codes
        .contains(&"tool-not-exposed-this-turn".into()));
}

#[tokio::test]
async fn p10_runner_rejects_untrusted_scoped_permit() {
    let dir = std::env::temp_dir().join(format!("aidens-p10-runner-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "hello p10\n").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let diff = "--- a/README.md\n+++ b/README.md\n@@\n-hello p10\n+hello p10 runner\n";
    let mock_script = format!(
            "{{\"tool_call\":{{\"tool_id\":\"aidens:patch-apply:1\",\"input\":{{\"diff\":{}}}}}}}\n---aidens-next-response---\npatched: {{{{last_tool_content}}}}",
            serde_json::to_string(diff).unwrap()
        );
    let grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let runner = AiDENsRunner::builder()
        .app_id("p10-runner")
        .mock_provider(mock_script)
        .tools(tools)
        .permit_policy(PermitPolicyV1::default().with_grant(grant))
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("apply approved patch"))
        .await
        .unwrap();

    assert_eq!(
        std::fs::read_to_string(dir.join("README.md")).unwrap(),
        "hello p10\n"
    );
    assert!(output.text.contains("not exposed") || output.text.contains("blocked"));
    let invocation = &output.receipt.tool_invocation_receipts[0];
    assert_eq!(invocation.tool_id, "aidens:patch-apply:1");
    assert!(!invocation.succeeded);
    assert!(invocation.permit_use_receipt_id.is_none());
    let _ = std::fs::remove_dir_all(&dir);
}

fn p26_loop_spec(
    memory_enabled: bool,
    memory_mode: AgentMemoryModeV1,
    view_disclosure_required: bool,
) -> AgentSpecV1 {
    p26_loop_spec_with_tools(
        memory_enabled,
        memory_mode,
        view_disclosure_required,
        &["repo.read"],
    )
}

fn p26_loop_spec_with_tools(
    memory_enabled: bool,
    memory_mode: AgentMemoryModeV1,
    view_disclosure_required: bool,
    allowed_tools: &[&str],
) -> AgentSpecV1 {
    AgentSpecV1 {
        schema: "AgentSpecV1".into(),
        agent_id: "agent:p26-memory-grounded-demo".into(),
        display_name: "P26 Memory Grounding Demo".into(),
        support_label: AgentSpecSupportLabelV1::SupportedLocal,
        profile: "coding".into(),
        provider_policy: aidens_contracts::AgentSpecProviderPolicyV1 {
            provider: AgentProviderModeV1::Local,
            cloud_allowed: false,
            fallback_allowed: false,
        },
        memory_policy: aidens_contracts::AgentSpecMemoryPolicyV1 {
            enabled: memory_enabled,
            mode: memory_mode,
            requires_view_disclosure: view_disclosure_required,
        },
        tool_policy: aidens_contracts::AgentSpecToolPolicyV1 {
            allowed_tools: allowed_tools
                .iter()
                .map(|tool| (*tool).to_string())
                .collect(),
            write_tools_require_permit: allowed_tools
                .iter()
                .any(|tool| matches!(*tool, "patch.apply" | "checks.run" | "run.inspect")),
        },
        permit_policy: aidens_contracts::AgentSpecPermitPolicyV1 {
            writes: aidens_contracts::AgentPermitRuleV1::OperatorApproved,
            commands: aidens_contracts::AgentPermitRuleV1::OperatorApproved,
            network: aidens_contracts::AgentPermitRuleV1::Forbidden,
        },
        verification_policy: aidens_contracts::AgentSpecVerificationPolicyV1 {
            required_checks: vec![
                AgentVerificationCheckV1::Schema,
                AgentVerificationCheckV1::SupportClaim,
                AgentVerificationCheckV1::Sandbox,
                AgentVerificationCheckV1::Digest,
            ],
            fail_closed: true,
        },
        evidence_policy: aidens_contracts::AgentSpecEvidencePolicyV1 {
            emit_run_bundle: true,
            emit_tool_receipts: true,
            emit_permit_receipts: true,
            emit_abstention_receipts: true,
        },
        budget_policy: aidens_contracts::AgentSpecBudgetPolicyV1 {
            max_turns: 4,
            max_tool_calls: 8,
            deadline_seconds: 120,
        },
    }
}

#[test]
fn p26_map_agent_tools_to_ids_supports_extended_coding_surface() {
    let resolution = map_agent_tools_to_ids(&[
        "repo.read".into(),
        "repo.list".into(),
        "repo.search".into(),
        "patch.propose".into(),
        "patch.apply".into(),
        "checks.run".into(),
        "run.inspect".into(),
    ]);

    assert!(resolution.unsupported.is_empty());
    assert!(resolution.canonical.contains(&"aidens:repo-read:1".into()));
    assert!(resolution.canonical.contains(&"aidens:repo-list:1".into()));
    assert!(resolution
        .canonical
        .contains(&"aidens:repo-search:1".into()));
    assert!(resolution
        .canonical
        .contains(&"aidens:patch-propose:1".into()));
    assert!(resolution
        .canonical
        .contains(&"aidens:patch-apply:1".into()));
    assert!(resolution.canonical.contains(&"aidens:run-checks:1".into()));
    assert_eq!(resolution.unsupported.len(), 0);
}

#[test]
fn p26_map_agent_tools_rejects_run_replay_alias() {
    let resolution = map_agent_tools_to_ids(&["run.replay".into()]);

    assert_eq!(resolution.canonical.len(), 0);
    assert_eq!(resolution.unsupported, vec!["run.replay".to_string()]);
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_rejects_unsupported_alias_with_repair() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-unsupported-alias-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["run.replay"]);
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response("final")
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("try replay").await.unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    assert_eq!(
        output
            .abstention_receipt
            .as_ref()
            .expect("abstention")
            .reason_code,
        "tool-policy-contains-unsupported-alias"
    );
    assert!(output.repair_plan.is_some());
    assert!(output.finalization.is_some());
    assert!(output.run_output.is_none());
}

fn write_text_and_register(root: &std::path::Path, path: &str, text: &str) {
    let target = root.join(path);
    if let Some(parent) = target.parent() {
        std::fs::create_dir_all(parent).unwrap();
    }
    std::fs::write(target, text).unwrap();
}

fn scoped_grant_for(
    root: &std::path::Path,
    tool_id: &str,
    risk: CanonicalToolSideEffectClass,
) -> PermitGrantV1 {
    PermitGrantV1::scoped(
        risk,
        tool_id,
        root.canonicalize().unwrap().display().to_string(),
        "p26",
    )
}

#[tokio::test]
async fn p26_plan_act_verify_loop_canonical_memory_grounding_collects_evidence() {
    let spec = p26_loop_spec(true, AgentMemoryModeV1::CanonicalSeam, false);
    let loopv = PlanActVerifyLoopV1::new(spec).provider_mock_response("final");
    let output = loopv
        .execute("agent memory grounding fixture content")
        .await
        .expect("expected successful loop execution with memory grounding");

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert!(!output.memory_grounding_receipts.is_empty());
    assert!(output
        .memory_grounding_receipts
        .iter()
        .any(|receipt| receipt.contains("canonical-seam")));
    let typed_receipt: aidens_memory_kit::MemoryGroundingEvidenceV1 =
        serde_json::from_str(&output.memory_grounding_receipts[0])
            .expect("typed memory grounding receipt");
    assert_eq!(
        typed_receipt.schema,
        aidens_memory_kit::MemoryGroundingEvidenceV1::SCHEMA
    );
    assert_eq!(typed_receipt.semantic_status, "exact_check");
    assert!(!typed_receipt.local_truth_store);
    assert!(typed_receipt
        .canonical_backpointers
        .iter()
        .any(|backpointer| backpointer.owner_crate == "knowledge-runtime"));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_memory_disabled_skips_grounding_evidence() {
    let spec = p26_loop_spec(false, AgentMemoryModeV1::Fixture, false);
    let loopv = PlanActVerifyLoopV1::new(spec).provider_mock_response("final");
    let output = loopv
        .execute("canonical seam fixture")
        .await
        .expect("expected successful loop execution without grounding");

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert!(output.memory_grounding_receipts.is_empty());
    assert!(output.finalization.is_some());
}

#[test]
fn phase06_plan_act_verify_loop_with_memory_sets_accessor() {
    let root = std::env::temp_dir().join(format!(
        "aidens-phase06-runner-memory-accessor-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&root);
    let memory = CanonicalMemoryAdapter::open_with_mock_embedder(
        memory_config_for_root(&root),
        runtime_config_for_namespace("phase06-runner-memory-accessor"),
    )
    .expect("memory adapter fixture");
    let spec = p26_loop_spec(false, AgentMemoryModeV1::Fixture, false);
    let loopv = PlanActVerifyLoopV1::new(spec).with_memory(std::sync::Arc::new(memory));

    assert!(loopv.has_memory());
    assert!(!loopv.has_governance());
    assert!(!loopv.has_kernel());
    let _ = std::fs::remove_dir_all(&root);
}

#[tokio::test]
async fn phase06_local_policy_mock_fixture_is_receipt_disclosed() {
    let spec = p26_loop_spec(false, AgentMemoryModeV1::Fixture, false);
    let loopv = PlanActVerifyLoopV1::new(spec).provider_mock_response("final");
    let output = loopv.execute("finish").await.unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert!(output.tool_route_receipts.iter().any(|receipt| {
        receipt
            .reason_codes
            .contains(&"provider-policy-local-routed-to-explicit-mock-fixture".into())
    }));
    assert!(output
        .finalization
        .as_ref()
        .unwrap()
        .reason_codes
        .contains(&"provider-policy-local-routed-to-explicit-mock-fixture".into()));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_provider_config_must_be_present() {
    let spec = p26_loop_spec(false, AgentMemoryModeV1::Fixture, false);
    let loopv = PlanActVerifyLoopV1::new(spec);
    let output = loopv.execute("need provider config").await.unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    assert_eq!(
        output
            .abstention_receipt
            .as_ref()
            .expect("abstention receipt")
            .reason_code,
        "provider-mock-response-missing"
    );
    assert!(output.repair_plan.is_some());
    assert!(output.finalization.is_some());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_canonical_memory_no_results_continues_with_warning() {
    let spec = p26_loop_spec(true, AgentMemoryModeV1::CanonicalSeam, false);
    let loopv = PlanActVerifyLoopV1::new(spec).provider_mock_response("final");
    let output = loopv
        .execute("non-existent-memory-seam-query-phrase")
        .await
        .expect("grounding should return warning and continue to provider");

    // With empty KB, memory grounding returns a warning receipt instead of failing.
    // The agent should continue to the provider rather than abstaining.
    assert!(output
        .memory_grounding_receipts
        .iter()
        .any(|receipt| receipt.contains("no-results-empty-kb")));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_support_claim_failure_causes_abstention_with_repair() {
    let mut spec = p26_loop_spec(false, AgentMemoryModeV1::Fixture, false);
    spec.support_label = AgentSpecSupportLabelV1::Unsupported;
    let loopv = PlanActVerifyLoopV1::new(spec).provider_mock_response("final");
    let output = loopv.execute("unsupported label probe").await.unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    let abstention = output
        .abstention_receipt
        .as_ref()
        .expect("abstention receipt");
    assert_eq!(abstention.reason_code, "verification-failed");
    assert!(output.repair_plan.is_some());
    assert!(output.finalization.is_some());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_reads_from_sandbox_root() {
    let dir = std::env::temp_dir().join(format!("aidens-p26-sandbox-read-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(&dir).unwrap();
    write_text_and_register(&dir, "notes.txt", "p26 read fixture\n");
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["repo.read"]);
    let mock_script = [
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"notes.txt"}}}"#,
        "---aidens-next-response---",
        "final: {{last_tool_content}}",
    ]
    .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("read notes").await.unwrap();
    let run_output = output.run_output.as_ref().unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert_eq!(
        run_output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert!(run_output.text.contains("p26 read fixture"));
    assert!(run_output.receipt.permit_use_receipts.is_empty());
    assert_eq!(run_output.receipt.approval_requests.len(), 0);
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_rejects_duplicate_json_tool_call_payload() {
    let dir = std::env::temp_dir().join(format!("aidens-p26-duplicate-key-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["repo.read"]);
    write_text_and_register(&dir, "notes.txt", "p26 duplicate payload fixture\n");
    let mock_script = r#"{"tool_calls":[{"tool_id":"aidens:repo-read:1","input":{"path":"notes.txt"}},{"tool_id":"aidens:repo-read:1","input":{"path":"notes.txt"}}]}"#;
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("read duplicate").await.unwrap();
    let turn = output.run_output.as_ref().expect("expected run output");

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    assert_eq!(turn.turn_receipt.final_state, TurnFinalStateV1::ToolBlocked);
    assert!(output.repair_plan.is_some());
    let abstention = output
        .abstention_receipt
        .as_ref()
        .expect("abstention receipt");
    assert!(abstention.reason_code.contains("turn-blocked"));
    assert!(abstention
        .evidence
        .iter()
        .any(|entry| entry.contains("recursive-tool-call")));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_rejects_invalid_tool_schema_output() {
    let dir =
        std::env::temp_dir().join(format!("aidens-p26-invalid-schema-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["repo.read"]);
    let mock_script = [
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":7}}}"#,
        "---aidens-next-response---",
        "final: {{last_tool_content}}",
    ]
    .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("bad schema").await.unwrap();
    let turn = output.run_output.as_ref().expect("expected run output");

    assert_eq!(turn.turn_receipt.final_state, TurnFinalStateV1::ToolFailed);
    assert!(turn.receipt.approval_requests.is_empty());
    assert!(output.repair_plan.is_some());
    assert!(output.finalization.is_some());
    assert!(output.abstention_receipt.is_some());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_lists_from_sandbox_root() {
    let dir = std::env::temp_dir().join(format!("aidens-p26-sandbox-list-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write_text_and_register(&dir, "src/readme.md", "list me\n");
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["repo.list"]);
    let mock_script = [
        r#"{"tool_call":{"tool_id":"aidens:repo-list:1","input":{"path":"."}}}"#,
        "---aidens-next-response---",
        "final: {{last_tool_content}}",
    ]
    .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("list files").await.unwrap();
    let run_output = output.run_output.as_ref().unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert_eq!(
        run_output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert!(run_output.receipt.tool_call_requests.iter().any(|request| {
        request.tool_id == "aidens:repo-list:1" && request.input.to_string().contains(".")
    }));
    assert!(run_output
        .receipt
        .tool_invocation_receipts
        .iter()
        .any(|receipt| receipt.tool_id == "aidens:repo-list:1" && receipt.succeeded));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_searches_from_sandbox_root() {
    let dir =
        std::env::temp_dir().join(format!("aidens-p26-sandbox-search-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write_text_and_register(&dir, "notes.txt", "needle phrase for p26 search test\n");
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["repo.search"]);
    let mock_script = [
        r#"{"tool_call":{"tool_id":"aidens:repo-search:1","input":{"query":"needle","path":"."}}}"#,
        "---aidens-next-response---",
        "final: {{last_tool_content}}",
    ]
    .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("search needle").await.unwrap();
    let run_output = output.run_output.as_ref().unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert_eq!(
        run_output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert!(run_output
        .receipt
        .tool_call_results
        .iter()
        .any(|result| result.output_text().contains("needle")));
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_proposes_patch_in_sandbox_root() {
    let dir =
        std::env::temp_dir().join(format!("aidens-p26-sandbox-propose-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    write_text_and_register(&dir, "notes.txt", "p26 propose fixture\n");
    let spec =
        p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["patch.propose"]);
    let diff =
        "--- a/notes.txt\n+++ b/notes.txt\n@@\n-p26 propose fixture\n+p26 propose accepted\n";
    let tool_call = serde_json::json!({
        "tool_call": {
            "tool_id": "aidens:patch-propose:1",
            "input": {
                "summary": "p26 patch proposal",
                "diff": diff,
            }
        }
    })
    .to_string();
    let mock_script = [
        tool_call.as_str(),
        "---aidens-next-response---",
        "final: {{last_tool_content}}",
    ]
    .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("propose patch").await.unwrap();
    let run_output = output.run_output.as_ref().unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Success));
    assert_eq!(
        run_output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert!(run_output.receipt.tool_call_requests.iter().any(|request| {
        request.tool_id == "aidens:patch-propose:1"
            && request.input.to_string().contains("p26 patch proposal")
    }));
    assert!(run_output.receipt.permit_use_receipts.is_empty());
    assert_eq!(run_output.receipt.approval_requests.len(), 0);
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_check_requires_permit() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-sandbox-check-blocked-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["checks.run"]);
    let mock_script = [
            r#"{"tool_call":{"tool_id":"aidens:run-checks:1","input":{"command":["bash","scripts/verify.sh"]}}}"#,
            "---aidens-next-response---",
            "final: {{last_tool_content}}",
        ]
        .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("run checks").await.unwrap();
    let turn = output.run_output.as_ref().unwrap();

    assert_eq!(turn.turn_receipt.final_state, TurnFinalStateV1::ToolBlocked);
    assert!(!turn.receipt.approval_requests.is_empty());
    assert!(turn.receipt.permit_use_receipts.is_empty());
    assert_eq!(output.outcome, PlanActVerifyOutcomeV1::Abstained);
    assert!(output.repair_plan.is_some());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_inspect_alias_maps_to_checks_with_permit() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-sandbox-check-success-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::create_dir_all(dir.join("scripts")).unwrap();
    std::fs::write(
        dir.join("scripts/verify.sh"),
        "#!/bin/sh\necho inspect ok\n",
    )
    .unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["run.inspect"]);
    let grant = scoped_grant_for(
        &dir,
        "aidens:run-checks:1",
        CanonicalToolSideEffectClass::Admin,
    );
    let mock_script = [
            r#"{"tool_call":{"tool_id":"aidens:run-checks:1","input":{"command":["bash","scripts/verify.sh"]}}}"#,
            "---aidens-next-response---",
            "final: {{last_tool_content}}",
        ]
        .join("\n");
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap())
        .permit_policy(PermitPolicyV1::default().with_grant(grant));

    let output = loopv.execute("inspect checks").await.unwrap();
    let run_output = output.run_output.as_ref().unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    assert_eq!(
        run_output.turn_receipt.final_state,
        TurnFinalStateV1::ToolBlocked
    );
    assert!(run_output
        .receipt
        .tool_invocation_receipts
        .iter()
        .any(|receipt| receipt.tool_id == "aidens:run-checks:1" && !receipt.succeeded));
    assert_eq!(run_output.receipt.approval_requests.len(), 1);
    assert!(run_output.receipt.permit_use_receipts.is_empty());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_requires_permit_for_patch_apply() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-permit-gated-patch-apply-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["patch.apply"]);
    let diff = "--- a/notes.txt\n+++ b/notes.txt\n+permit-gated patch suggestion\n";
    let mock_script = format!(
        "{}\n---aidens-next-response---\napply rejected",
        serde_json::json!({
            "tool_call": {
                "tool_id": "aidens:patch-apply:1",
                "input": {"diff": diff}
            }
        })
    );
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap());

    let output = loopv.execute("apply").await.unwrap();
    let turn = output
        .run_output
        .as_ref()
        .expect("expected a run output on blocked patch apply");

    assert_eq!(turn.turn_receipt.final_state, TurnFinalStateV1::ToolBlocked);
    assert!(!turn.receipt.approval_requests.is_empty());
    assert!(turn.receipt.permit_use_receipts.is_empty());
    assert!(!dir.join("notes.txt").try_exists().unwrap_or(false));
    assert!(output.repair_plan.is_some());
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_rejects_invalid_patch_without_success() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-invalid-patch-apply-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.txt"), "keep unchanged\n").unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["patch.apply"]);
    let grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let mock_script = format!(
        "{}\n---aidens-next-response---\nfinal: {{last_tool_content}}",
        serde_json::json!({
            "tool_call": {
                "tool_id": "aidens:patch-apply:1",
                "input": {"diff": "not-a-diff"}
            }
        })
    );
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap())
        .permit_policy(PermitPolicyV1::default().with_grant(grant));

    let output = loopv.execute("apply invalid patch").await.unwrap();
    let turn = output.run_output.as_ref().expect("expected run output");

    assert_eq!(turn.turn_receipt.final_state, TurnFinalStateV1::ToolBlocked);
    assert!(output.repair_plan.is_some());
    assert_eq!(output.outcome, PlanActVerifyOutcomeV1::Abstained);
    assert_eq!(
        std::fs::read_to_string(dir.join("notes.txt")).unwrap(),
        "keep unchanged\n"
    );
}

#[tokio::test]
async fn p26_plan_act_verify_loop_coding_toolchain_applies_with_scoped_permit() {
    let dir = std::env::temp_dir().join(format!(
        "aidens-p26-scoped-patch-apply-{}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("notes.txt"), "permit-gated patch draft\n").unwrap();
    let spec = p26_loop_spec_with_tools(false, AgentMemoryModeV1::Fixture, false, &["patch.apply"]);
    let grant = PermitGrantV1::scoped(
        CanonicalToolSideEffectClass::Write,
        "aidens:patch-apply:1",
        dir.canonicalize().unwrap().display().to_string(),
        "test",
    );
    let diff = "--- a/notes.txt\n+++ b/notes.txt\n@@\n-permit-gated patch draft\n+permit-gated patch accepted\n";
    let mock_script = format!(
        "{}\n---aidens-next-response---\nfinal: {{last_tool_content}}",
        serde_json::json!({
            "tool_call": {
                "tool_id": "aidens:patch-apply:1",
                "input": {"diff": diff}
            }
        })
    );
    let loopv = PlanActVerifyLoopV1::new(spec)
        .provider_mock_response(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap())
        .permit_policy(PermitPolicyV1::default().with_grant(grant));

    let output = loopv.execute("apply").await.unwrap();

    assert!(matches!(output.outcome, PlanActVerifyOutcomeV1::Abstained));
    assert!(output.run_output.is_some());
    let patched_file = dir.join("notes.txt");
    assert!(patched_file.exists());
    let patched_contents = std::fs::read_to_string(&patched_file).unwrap();
    assert!(patched_contents.contains("permit-gated patch draft"));
    assert_eq!(
        output
            .run_output
            .as_ref()
            .unwrap()
            .receipt
            .approval_requests
            .len(),
        1
    );
}
