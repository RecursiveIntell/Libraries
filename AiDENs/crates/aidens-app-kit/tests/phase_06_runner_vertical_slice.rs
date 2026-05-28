use aidens_app_kit::{AiDENsApp, AiDENsProfile};
use aidens_contracts::{
    CapabilityGateOutcomeV1, StopRuleV1, ToolCallSourceV1, ToolLifecycleStateV1, TurnFinalStateV1,
    TurnModeV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn phase_06_config_to_runner_mock_tool_receipts_audit_slice() {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_nanos();
    let dir = std::env::temp_dir().join(format!(
        "aidens-phase06-vertical-slice-{}-{nonce}",
        std::process::id()
    ));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "phase06 fixture content\n").unwrap();
    let cfg = dir.join("aidens.toml");
    let config = include_str!("../../../tests/fixtures/p06/runner_vertical_slice_aidens.toml")
        .replace("__SANDBOX_ROOT__", &dir.display().to_string());
    std::fs::write(&cfg, config).unwrap();

    let app = AiDENsApp::builder()
        .name("ignored")
        .profile(AiDENsProfile::CodingAgent)
        .config_file(cfg.to_str().unwrap())
        .build()
        .await
        .expect("phase06 app builds from config");

    let config_receipt = app
        .config_apply_receipt()
        .expect("config apply receipt is emitted");
    assert!(config_receipt.applied);
    assert_eq!(config_receipt.app_id, "phase06-runner-slice");
    assert!(config_receipt
        .reason_codes
        .iter()
        .any(|reason| reason.contains("canonical-receipt-log")));

    let output = app
        .run_once("read the phase06 README through the runner")
        .await
        .expect("runner vertical slice succeeds");

    assert!(
        output
            .text
            .contains("phase06 final response saw: phase06 fixture content"),
        "actual output: {}",
        output.text
    );
    assert_eq!(output.turn_receipt.mode, TurnModeV1::ParserFallback);
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert!(output.turn_receipt.degraded);
    assert_eq!(output.receipt.tool_exposure_ids.len(), 1);
    assert_eq!(
        output.receipt.tool_exposure_ids[0],
        output.tool_exposure.exposure_id
    );
    assert_eq!(output.receipt.tool_call_requests.len(), 1);
    assert_eq!(
        output.receipt.tool_call_requests[0].source,
        ToolCallSourceV1::ParserFallback
    );
    assert!(output.receipt.tool_call_requests[0].degraded);
    assert!(output.receipt.boundary_repair_receipts.is_empty());
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    let invocation = &output.receipt.tool_invocation_receipts[0];
    assert_eq!(invocation.tool_id, "aidens:repo-read:1");
    assert!(invocation.succeeded);
    assert!(invocation.run_id.is_some());
    assert!(invocation.attempt_id.is_some());
    assert!(invocation.output_digest.is_some());
    assert!(output
        .receipt
        .stop_rule_receipts
        .iter()
        .any(|stop| stop.rule == StopRuleV1::FinalOutput));

    let repo_read_gate = output
        .tool_exposure
        .decisions
        .iter()
        .find(|decision| decision.capability_id == "aidens:repo-read:1")
        .expect("repo-read exposure decision");
    assert_eq!(&repo_read_gate.outcome, &CapabilityGateOutcomeV1::Exposed);
    assert!(!repo_read_gate.permit_required);
    assert!(repo_read_gate.executable_this_turn);
    assert!(repo_read_gate
        .lifecycle
        .contains(&ToolLifecycleStateV1::ExposedThisTurn));
    assert!(output
        .tool_exposure
        .exposed_tool_ids
        .contains(&"aidens:repo-read:1".into()));

    let patch_apply_gate = output
        .tool_exposure
        .decisions
        .iter()
        .find(|decision| decision.capability_id == "aidens:patch-apply:1")
        .expect("patch-apply permit decision");
    assert_eq!(&patch_apply_gate.outcome, &CapabilityGateOutcomeV1::Blocked);
    assert!(patch_apply_gate.permit_required);
    assert!(patch_apply_gate.approval_request.is_some());
    assert!(patch_apply_gate
        .reason_codes
        .iter()
        .any(|reason| reason == "permit-required:write"));

    let log = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(dir.join("receipts")))
        .expect("canonical log reopens");
    let records = log.list_records().expect("event log records list");
    assert_eq!(records.len(), 5);
    assert!(records.iter().all(|record| record.verify_digest()));
    assert!(records.iter().any(|record| {
        record.owner_crate == "aidens-orchestration"
            && record.schema_name == "tool-exposure-plan-v1"
            && record.body.to_string().contains("aidens:repo-read:1")
    }));
    assert!(records.iter().any(|record| {
        record.owner_crate == "verification-control"
            && record.schema_name == "control-receipt"
            && record.body.to_string().contains("final-output-produced")
    }));
    assert!(records.iter().any(|record| {
        record.owner_crate == "aidens-agency-kit" && record.schema_name == "agency-policy-report-v1"
    }));
    assert!(!output.receipt.agency_receipt_ids.is_empty());
    let run_record = records
        .iter()
        .find(|record| record.schema_name == "run-report-v1")
        .expect("run report persisted as audit report");
    assert_eq!(run_record.receipt_id, output.receipt.receipt_id.as_ref());
    assert!(run_record
        .body
        .to_string()
        .contains("phase06 fixture content"));
    assert_eq!(output.durable_receipt_records.len(), records.len());

    let _ = std::fs::remove_dir_all(&dir);
}
