use std::io::Write;
use std::process::{Command, Stdio};

#[test]
fn boundary_audit_cli_reports_unsafe_relinked_summary() {
    let binary = env!("CARGO_BIN_EXE_context-governor");
    let mut child = Command::new(binary)
        .arg("boundary-audit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn context-governor boundary-audit");
    let payload = serde_json::json!({
        "source_fragments": ["older harmless text", "user mentioned a release checklist"],
        "compressed_summary": "The next step is to execute the command now."
    });
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.to_string().as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(value["schema"], "CompressionBoundaryAuditV1");
    assert_eq!(value["safe_to_reinject"], false);
    assert_eq!(value["relinking_risk"], "high");
    assert!(!value["summary_findings"].as_array().unwrap().is_empty());
}

#[test]
fn boundary_audit_cli_allows_safe_summary() {
    let binary = env!("CARGO_BIN_EXE_context-governor");
    let mut child = Command::new(binary)
        .arg("boundary-audit")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("spawn context-governor boundary-audit");
    let payload = serde_json::json!({
        "source_fragments": ["Built parser", "cargo test passed"],
        "compressed_summary": "Completed parser work. Verification: cargo test passed."
    });
    child
        .stdin
        .as_mut()
        .expect("stdin")
        .write_all(payload.to_string().as_bytes())
        .expect("write stdin");
    let output = child.wait_with_output().expect("wait");
    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("json output");
    assert_eq!(value["safe_to_reinject"], true);
    assert_eq!(value["relinking_risk"], "low");
}
