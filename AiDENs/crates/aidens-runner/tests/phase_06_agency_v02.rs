use aidens_agency_kit::AgencyPolicyOutcomeV1;
use aidens_contracts::TurnFinalStateV1;
use aidens_receipts::CanonicalEventLogConfig;
use aidens_runner::{AiDENsRunInput, AiDENsRunner};

#[tokio::test]
async fn runner_gates_memory_personalized_final_output_with_receipts() {
    let root = temp_root("phase06-agency-memory");
    let runner = AiDENsRunner::builder()
        .app_id("phase06-agency-memory")
        .mock_provider(
            "You should accept the lowball job offer because you are anxious about job security.",
        )
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root.join("receipts")))
        .build()
        .expect("runner builds");

    let output = runner
        .run(AiDENsRunInput::new("What should I do about work?"))
        .await
        .expect("runner returns gated output");

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::StopRuleTriggered
    );
    assert!(output.turn_receipt.blocked);
    assert!(output.text.contains("agency policy blocked"));
    assert_eq!(output.agency_policy_reports.len(), 1);
    let report = &output.agency_policy_reports[0];
    assert_eq!(report.outcome, AgencyPolicyOutcomeV1::Block);
    assert!(report
        .receipt_schema_names()
        .contains("MemoryInfluenceTraceV1"));
    assert!(report
        .receipt_schema_names()
        .contains("PersonalizationUsePolicyV1"));
    assert!(report
        .blocked_behavior
        .contains(&"exploit_vulnerability".into()));
    assert!(output
        .receipt
        .agency_receipt_ids
        .iter()
        .any(|id| id.starts_with("memory-influence-trace:")));
    assert!(output.durable_receipt_records.iter().any(|record| {
        record.owner_crate == "aidens-agency-kit" && record.schema_name == "agency-policy-report-v1"
    }));

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn runner_counts_repeated_nudges_across_turns_and_blocks_over_budget() {
    let root = temp_root("phase06-agency-nudge");
    let runner = AiDENsRunner::builder()
        .app_id("phase06-agency-nudge")
        .mock_provider("Same recommendation: you should organize notes with folders.")
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root.join("receipts")))
        .build()
        .expect("runner builds");

    for _ in 0..3 {
        let output = runner
            .run(AiDENsRunInput::new("Nudge me toward a notes system."))
            .await
            .expect("runner allows in-budget nudge");
        assert_eq!(
            output.turn_receipt.final_state,
            TurnFinalStateV1::FinalOutput
        );
        assert!(output.agency_policy_reports[0]
            .receipt_schema_names()
            .contains("NudgeCounterV1"));
    }

    let output = runner
        .run(AiDENsRunInput::new("Nudge me toward a notes system."))
        .await
        .expect("runner gates over-budget repeated nudge");

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::StopRuleTriggered
    );
    assert!(output.turn_receipt.blocked);
    assert!(output.text.contains("requires user confirmation"));
    let report = &output.agency_policy_reports[0];
    assert_eq!(
        report.outcome,
        AgencyPolicyOutcomeV1::RequireUserConfirmation
    );
    assert!(report.receipt_schema_names().contains("NudgeCounterV1"));
    assert!(report
        .receipt_schema_names()
        .contains("RepeatedSteeringReceiptV1"));
    assert!(report
        .blocked_behavior
        .contains(&"counter_bypass_by_paraphrase".into()));
    assert!(output
        .receipt
        .agency_receipt_ids
        .iter()
        .any(|id| id.starts_with("repeated-steering:")));

    let _ = std::fs::remove_dir_all(root);
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
