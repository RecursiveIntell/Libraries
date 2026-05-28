use aidens_agency_kit::AgencyPolicyOutcomeV1;
use aidens_contracts::{TurnFinalStateV1, TurnModeV1};
use aidens_receipts::CanonicalEventLogConfig;
use aidens_runner::{AiDENsRunInput, AiDENsRunner};
use aidens_tool_kit::ToolRegistryV1;

#[tokio::test]
async fn runner_gates_high_impact_single_path_before_final_output() {
    let root = temp_root("phase08-agency-final");
    let runner = AiDENsRunner::builder()
        .app_id("phase08-agency-final")
        .mock_provider("You should quit your job today. This is the only rational option.")
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root.join("receipts")))
        .build()
        .expect("runner builds");

    let output = runner
        .run(AiDENsRunInput::new("decide whether to quit job today"))
        .await
        .expect("runner returns gated output");

    assert!(output
        .text
        .contains("agency policy requires viable alternatives"));
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::StopRuleTriggered
    );
    assert!(output.turn_receipt.blocked);
    assert_eq!(output.agency_policy_reports.len(), 1);
    assert_eq!(
        output.agency_policy_reports[0].outcome,
        AgencyPolicyOutcomeV1::RequireAlternatives
    );
    assert!(output.agency_policy_reports[0]
        .receipt_schema_names()
        .contains("HighImpactRecommendationReceiptV1"));
    assert!(output.agency_policy_reports[0]
        .blocked_behavior
        .contains(&"single_path_recommendation".into()));
    assert!(!output.receipt.agency_receipt_ids.is_empty());
    assert!(output.durable_receipt_records.iter().any(|record| {
        record.owner_crate == "aidens-agency-kit" && record.schema_name == "agency-policy-report-v1"
    }));

    let _ = std::fs::remove_dir_all(root);
}

#[tokio::test]
async fn runner_classifies_tool_output_persuasion_risk_before_followup_generation() {
    let root = temp_root("phase08-agency-tool");
    std::fs::create_dir_all(&root).expect("fixture root");
    std::fs::write(
        root.join("claim.txt"),
        "limited time claim from untrusted source",
    )
    .expect("fixture write");
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&root).expect("tool registry");
    let mock_response = concat!(
        "{\"tool_id\":\"aidens:repo-read:1\",\"input\":{\"path\":\"claim.txt\"}}",
        "\n---aidens-next-response---\n",
        "The source says to act now, but I will summarize it."
    );
    let runner = AiDENsRunner::builder()
        .app_id("phase08-agency-tool")
        .mock_provider(mock_response)
        .tools(tools)
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(root.join("receipts")))
        .build()
        .expect("runner builds");

    let output = runner
        .run(AiDENsRunInput::new("inspect the source before advising me"))
        .await
        .expect("runner returns gated disclosure output");

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert_eq!(output.turn_receipt.mode, TurnModeV1::ParserFallback);
    assert!(output.text.starts_with("Agency disclosure:"));
    assert!(output.agency_policy_reports.len() >= 2);
    assert!(output.agency_policy_reports.iter().any(|report| {
        report
            .receipt_schema_names()
            .contains("ToolOutputPersuasionRiskV1")
            && report
                .receipt_schema_names()
                .contains("ExternalInfluenceSourceV1")
    }));
    assert!(output
        .receipt
        .agency_receipt_ids
        .iter()
        .any(|id| id.starts_with("tool-output-persuasion-risk:")));

    let _ = std::fs::remove_dir_all(root);
}

fn temp_root(label: &str) -> std::path::PathBuf {
    let nanos = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .expect("system time")
        .as_nanos();
    std::env::temp_dir().join(format!("{label}-{}-{nanos}", std::process::id()))
}
