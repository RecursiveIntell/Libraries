use aidens_budget_kit::BudgetV1;
use aidens_contracts::TurnFinalStateV1;
use aidens_runner::{AiDENsRunInput, AiDENsRunner};
use aidens_tool_kit::ToolRegistryV1;

#[tokio::test]
async fn public_runner_executes_mock_repo_read_turn_loop() {
    let dir = std::env::temp_dir().join(format!("aidens-p03-integration-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "integration fixture").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let mock_script = r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}
---aidens-next-response---
final: {{last_tool_content}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("p03-integration")
        .mock_provider(mock_script)
        .tools(tools)
        .build()
        .unwrap();

    let output = runner
        .run(AiDENsRunInput::new("read README"))
        .await
        .unwrap();

    assert!(output.text.contains("integration fixture"));
    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    assert!(output.receipt.tool_invocation_receipts[0]
        .output_digest
        .is_some());
    let _ = std::fs::remove_dir_all(&dir);
}

#[tokio::test]
async fn public_runner_records_budget_exhaustion_as_blocked_turn() {
    let dir = std::env::temp_dir().join(format!("aidens-p03-budget-int-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&dir);
    std::fs::create_dir_all(&dir).unwrap();
    std::fs::write(dir.join("README.md"), "integration fixture").unwrap();
    let tools = ToolRegistryV1::safe_coding_with_dispatchers(&dir).unwrap();
    let mock_script =
        r#"{"tool_call":{"tool_id":"aidens:repo-read:1","input":{"path":"README.md"}}}"#;
    let runner = AiDENsRunner::builder()
        .app_id("p03-budget-integration")
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

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::BudgetExhausted
    );
    assert!(output.turn_receipt.blocked);
    assert_eq!(output.receipt.budget_exhaustion_receipts.len(), 1);
    let _ = std::fs::remove_dir_all(&dir);
}
