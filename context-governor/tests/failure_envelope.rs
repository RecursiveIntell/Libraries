use std::io::Write;
use std::process::{Command, Stdio};

fn binary() -> &'static str {
    env!("CARGO_BIN_EXE_context-governor")
}

#[test]
fn capabilities_advertise_failure_envelope_v1() {
    let output = Command::new(binary()).arg("capabilities").output().unwrap();
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).unwrap();
    assert_eq!(
        value["failure_envelope"]["schema"],
        "ContextGovernorFailureV1"
    );
    assert_eq!(value["failure_envelope"]["flag"], "--failure-envelope-v1");
    assert_eq!(value["failure_envelope"]["stream"], "stderr");
}

#[test]
fn failure_envelope_is_machine_readable_and_does_not_depend_on_display_text() {
    let mut child = Command::new(binary())
        .args(["compact", "--failure-envelope-v1"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(br#"{"session_id":"x","messages":[]}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    let failure: serde_json::Value = serde_json::from_slice(&output.stderr).unwrap();
    assert_eq!(failure["schema"], "ContextGovernorFailureV1");
    assert_eq!(failure["operation"], "compact");
    assert_eq!(failure["code"], "empty_messages");
}

#[test]
fn human_cli_error_remains_human_without_negotiation_flag() {
    let mut child = Command::new(binary())
        .arg("compact")
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(br#"{"session_id":"x","messages":[]}"#)
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(serde_json::from_slice::<serde_json::Value>(&output.stderr).is_err());
    assert!(
        String::from_utf8_lossy(&output.stderr).contains("cannot compact an empty message list")
    );
}
