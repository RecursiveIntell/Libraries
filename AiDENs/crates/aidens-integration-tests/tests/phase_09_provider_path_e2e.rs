use aidens_cli::{agent_new_command, agent_run_command, inspect_run_bundle_command};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase_09_mock_plan_act_verify_e2e_stores_exact_supported_local_receipt(
) -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-09-provider-path-e2e");
    let agent_dir = root.join("local-agent");
    let out = root.join("run");

    agent_new_command("local-coding", &agent_dir.display().to_string())?;
    std::fs::write(
        agent_dir.join("task.md"),
        "Read README.md and report evidence.\n",
    )?;

    let summary = agent_run_command(
        &agent_dir.join("agent.json").display().to_string(),
        &agent_dir.join("task.md").display().to_string(),
        &out.display().to_string(),
        Some(agent_dir.join("sandbox").display().to_string()),
        None,
        None,
    )?;

    assert!(summary.contains("outcome: Success"));
    assert!(summary.contains("run_bundle_store:"));

    let loop_output: Value = serde_json::from_str(&std::fs::read_to_string(
        out.join("plan-act-verify-output.json"),
    )?)?;
    assert_eq!(loop_output["outcome"], "Success");
    assert_eq!(
        loop_output["tool_call_receipts"][0]["succeeded_tool_calls"],
        1
    );

    let run_bundle: Value =
        serde_json::from_str(&std::fs::read_to_string(out.join("run-bundle.json"))?)?;
    assert_eq!(run_bundle["schema"], "AiDENsRunBundleV3");
    assert_eq!(run_bundle["support"]["support_tier"], "supported-local");
    assert_eq!(run_bundle["failure"]["degraded"], false);

    let inspected = inspect_run_bundle_command(&out.join("receipts").display().to_string())?;
    let inspected: Value = serde_json::from_str(&inspected)?;
    assert_eq!(inspected["bundle_schema"], "AiDENsRunBundleV3");
    assert_eq!(inspected["provider_route"], "mock");
    assert_eq!(inspected["event_log_digest_verified"], true);
    assert_eq!(
        inspected["run_bundle_store_record"]["semantic_status"],
        "exact_check"
    );
    assert!(inspected["canonical_record_count"].as_u64().unwrap() > 0);

    let _ = std::fs::remove_dir_all(&root);
    Ok(())
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
