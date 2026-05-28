use aidens_cli::{agent_new_command, agent_run_command, inspect_run_bundle_command};
use serde_json::Value;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[test]
fn phase_08_run_bundle_store_survives_cli_reopen() -> Result<(), Box<dyn std::error::Error>> {
    let root = temp_root("phase-08-run-bundle-store");
    let agent_dir = root.join("local-agent");
    let out = root.join("run");

    agent_new_command("local-coding", &agent_dir.display().to_string())?;
    let summary = agent_run_command(
        &agent_dir.join("agent.json").display().to_string(),
        &agent_dir.join("task.md").display().to_string(),
        &out.display().to_string(),
        Some(agent_dir.join("sandbox").display().to_string()),
        None,
        None,
    )?;

    assert!(summary.contains("run_bundle_store:"));
    assert!(out.join("run-bundle.json").exists());
    assert!(out.join("run-bundle-store-record.json").exists());
    assert!(out
        .join("receipts")
        .join("run-bundles")
        .join("index.ndjson")
        .exists());

    let inspected = inspect_run_bundle_command(&out.join("receipts").display().to_string())?;
    let inspected: Value = serde_json::from_str(&inspected)?;
    assert_eq!(inspected["bundle_schema"], "AiDENsRunBundleV3");
    assert_eq!(inspected["event_log_digest_verified"], true);
    assert_eq!(
        inspected["run_bundle_store_record"]["artifact_kind"],
        "local_operator_run_bundle_store_record"
    );
    assert_eq!(
        inspected["run_bundle_store_record"]["semantic_status"],
        "degraded_exact_check"
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
