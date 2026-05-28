use aidens_contracts::{
    CapabilityGateOutcomeV1, ToolCallSourceV1, ToolLifecycleStateV1, TurnFinalStateV1, TurnModeV1,
};
use aidens_receipts::{CanonicalEventLog, CanonicalEventLogConfig};
use aidens_runner::{AiDENsRunInput, AiDENsRunner};
use aidens_tool_kit::ToolRegistryV1;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

#[tokio::test]
async fn test_agent_vertical_slice() -> Result<(), Box<dyn std::error::Error>> {
    let workspace = workspace_root();
    let agent_config_path = workspace.join("fixtures/test-agent/basic-agent.toml");
    let agent_config = load_test_agent_config(&agent_config_path)?;
    let runner_fixture_path = workspace.join(required_toml_str(
        &agent_config["provider"],
        "fixture",
        "provider.fixture",
    )?);
    let runner_fixture: Value =
        serde_json::from_str(&std::fs::read_to_string(&runner_fixture_path)?)?;

    let user_prompt = fixture_turn(&runner_fixture, 0, "user")?;
    let tool_turn = fixture_turn_value(&runner_fixture, 1, "assistant_tool_call")?;
    let final_turn = fixture_turn(&runner_fixture, 2, "assistant_final")?;
    let fixture_tool_name = required_str(tool_turn, "tool", "turns[1].tool")?;
    let tool_id = canonical_tool_id(fixture_tool_name)?;
    let tool_input = tool_turn
        .get("arguments")
        .cloned()
        .ok_or("turns[1].arguments missing")?;

    let root = temp_root("p20-2-test-agent");
    let repo = root.join("repo");
    let receipts = root.join("receipts");
    std::fs::create_dir_all(&repo)?;
    std::fs::write(
        repo.join(required_str(
            &tool_input,
            "path",
            "turns[1].arguments.path",
        )?),
        "AiDENs canonical test agent fixture\nstatus: executable vertical slice\n",
    )?;

    let mock_script = format!(
        "{{\"tool_call\":{{\"tool_id\":\"{tool_id}\",\"input\":{tool_input}}}}}\n---aidens-next-response---\n{final_turn} Tool evidence: {{{{last_tool_content}}}}"
    );
    let runner = AiDENsRunner::builder()
        .app_id(required_toml_str(
            &agent_config["agent"],
            "name",
            "agent.name",
        )?)
        .mock_provider(mock_script)
        .tools(ToolRegistryV1::safe_coding_with_dispatchers(&repo)?)
        .canonical_receipt_log_config(CanonicalEventLogConfig::for_root(&receipts))
        .build()?;

    let output = runner.run(AiDENsRunInput::new(user_prompt)).await?;

    let provider_route = output
        .receipt
        .provider_route
        .as_ref()
        .expect("provider route recorded");
    assert_eq!(provider_route.provider_kind, "mock");
    assert_eq!(provider_route.route_label, "mock");
    assert!(!provider_route.native_tool_loop);

    assert_eq!(
        output.turn_receipt.final_state,
        TurnFinalStateV1::FinalOutput
    );
    assert_eq!(output.turn_receipt.mode, TurnModeV1::ParserFallback);
    assert!(output.text.contains(final_turn));
    assert!(output.text.contains("executable vertical slice"));

    assert!(output
        .tool_exposure
        .exposed_tool_ids
        .contains(&tool_id.into()));
    assert!(output
        .receipt
        .tool_exposure_ids
        .contains(&output.tool_exposure.exposure_id));
    let read_decision = output
        .tool_exposure
        .decisions
        .iter()
        .find(|decision| decision.capability_id == tool_id)
        .expect("repo read gate decision");
    assert_eq!(read_decision.outcome, CapabilityGateOutcomeV1::Exposed);
    assert!(read_decision.executable_this_turn);
    assert!(!read_decision.permit_required);
    assert!(read_decision
        .lifecycle
        .contains(&ToolLifecycleStateV1::ExposedThisTurn));

    assert_eq!(output.receipt.tool_call_requests.len(), 1);
    let request = &output.receipt.tool_call_requests[0];
    assert_eq!(request.tool_id, tool_id);
    assert_eq!(request.source, ToolCallSourceV1::ParserFallback);
    assert!(request
        .reason_codes
        .contains(&"parser-fallback-tool-call".into()));

    assert_eq!(output.receipt.tool_invocation_receipts.len(), 1);
    let invocation = &output.receipt.tool_invocation_receipts[0];
    assert_eq!(invocation.tool_id, tool_id);
    assert!(invocation.succeeded);
    assert!(invocation.run_id.is_some());
    assert!(invocation.attempt_id.is_some());
    assert!(invocation.output_digest.is_some());
    assert_eq!(output.receipt.tool_call_results.len(), 1);
    assert!(output.receipt.tool_call_results[0].succeeded);

    assert!(!output.agency_policy_reports.is_empty());
    assert!(!output.receipt.agency_receipt_ids.is_empty());
    assert!(output.agency_policy_reports.iter().any(|report| report
        .receipt_schema_names()
        .contains("AgencyPolicyDecisionV1")));

    assert!(output.durable_receipt_records.len() >= 3);
    assert!(output
        .durable_receipt_records
        .iter()
        .all(|record| record.verify_digest()));
    assert!(output
        .durable_receipt_records
        .iter()
        .any(|record| record.schema_name == "tool-exposure-plan-v1"));
    assert!(output
        .durable_receipt_records
        .iter()
        .any(|record| record.schema_name == "run-report-v1"));
    assert!(output
        .durable_receipt_records
        .iter()
        .any(|record| record.schema_name == "agency-policy-report-v1"));
    let reopened = CanonicalEventLog::open(CanonicalEventLogConfig::for_root(&receipts))?;
    assert!(reopened.verify_digest(output.receipt.receipt_id.as_str())?);

    let expected_events = expected_event_names(&workspace)?;
    let actual_events = vec![
        "provider_route_selected",
        "tool_exposure_plan_created",
        "permit_checked",
        "tool_invocation_recorded",
        "agency_policy_evaluated",
        "final_response_recorded",
    ];
    assert_eq!(actual_events, expected_events);
    assert_no_unsupported_provider_claims(&output);

    let _ = std::fs::remove_dir_all(root);
    Ok(())
}

fn load_test_agent_config(path: &Path) -> Result<toml::Value, Box<dyn std::error::Error>> {
    Ok(toml::from_str(&std::fs::read_to_string(path)?)?)
}

fn fixture_turn<'a>(
    fixture: &'a Value,
    index: usize,
    expected_role: &str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    let value = fixture_turn_value(fixture, index, expected_role)?;
    required_str(value, "content", "turn.content")
}

fn fixture_turn_value<'a>(
    fixture: &'a Value,
    index: usize,
    expected_role: &str,
) -> Result<&'a Value, Box<dyn std::error::Error>> {
    let turn = fixture
        .get("turns")
        .and_then(Value::as_array)
        .and_then(|turns| turns.get(index))
        .ok_or("fixture turn missing")?;
    let role = required_str(turn, "role", "turn.role")?;
    assert_eq!(role, expected_role);
    Ok(turn)
}

fn required_str<'a>(
    value: &'a Value,
    key: &str,
    label: &'static str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .get(key)
        .and_then(Value::as_str)
        .ok_or_else(|| label.into())
}

fn required_toml_str<'a>(
    value: &'a toml::Value,
    key: &str,
    label: &'static str,
) -> Result<&'a str, Box<dyn std::error::Error>> {
    value
        .get(key)
        .and_then(toml::Value::as_str)
        .ok_or_else(|| label.into())
}

fn canonical_tool_id(tool_name: &str) -> Result<&'static str, Box<dyn std::error::Error>> {
    match tool_name {
        "repo.read" => Ok("aidens:repo-read:1"),
        _ => Err(format!("unsupported test-agent tool: {tool_name}").into()),
    }
}

fn expected_event_names(workspace: &Path) -> Result<Vec<String>, Box<dyn std::error::Error>> {
    let path = workspace.join("fixtures/runner/expected_test_agent_event_log.ndjson");
    let mut events = Vec::new();
    for line in std::fs::read_to_string(path)?.lines() {
        if line.trim().is_empty() {
            continue;
        }
        let value: Value = serde_json::from_str(line)?;
        events.push(required_str(&value, "event", "event")?.to_string());
    }
    Ok(events)
}

fn assert_no_unsupported_provider_claims(output: &aidens_runner::AiDENsRunOutput) {
    let body = serde_json::to_string(&output.receipt).expect("receipt serializes");
    for forbidden in [
        "openai-native-tool-loop-supported",
        "anthropic-native-tool-loop-supported",
        "openrouter-native-tool-loop-supported",
        "cloud-provider-supported",
    ] {
        assert!(
            !body.contains(forbidden),
            "forbidden provider claim: {forbidden}"
        );
    }
}

fn workspace_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(2)
        .expect("workspace root")
        .to_path_buf()
}

fn temp_root(label: &str) -> PathBuf {
    std::env::temp_dir().join(format!(
        "{label}-{}-{}",
        std::process::id(),
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .expect("system time")
            .as_nanos()
    ))
}
