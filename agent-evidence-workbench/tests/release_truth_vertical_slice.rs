use agent_evidence_workbench::v2::{ReleaseTruthReportV2, SourceSnapshotV2};
use claim_ledger::SupportState;
use serde::Deserialize;
use std::{fs, path::Path, process::Command};
use tempfile::tempdir;

#[derive(Debug, Deserialize)]
struct EvaluateOutput {
    report: ReleaseTruthReportV2,
    redaction_count: usize,
    recorded_event: Option<String>,
}

fn command(argv: &[&str], cwd: &Path) {
    let result = Command::new(argv[0])
        .args(&argv[1..])
        .current_dir(cwd)
        .output()
        .expect("command launches");
    assert!(
        result.status.success(),
        "{} failed: {}",
        argv.join(" "),
        String::from_utf8_lossy(&result.stderr)
    );
}

#[test]
fn release_truth_fixture_survives_process_restart_and_redacts_persistence() {
    let directory = tempdir().expect("tempdir");
    let repo = directory.path().join("repository");
    fs::create_dir_all(&repo).expect("repository directory");
    let repo = repo.as_path();
    command(&["git", "init", "-q"], repo);
    command(
        &["git", "config", "user.email", "fixture@example.invalid"],
        repo,
    );
    command(&["git", "config", "user.name", "AEW Fixture"], repo);
    fs::write(repo.join("README.md"), "fixture\n").expect("fixture source");
    command(&["git", "add", "README.md"], repo);
    command(&["git", "commit", "-qm", "fixture"], repo);

    let binary = env!("CARGO_BIN_EXE_aew");
    let snapshot = Command::new(binary)
        .arg("snapshot-v2")
        .current_dir(repo)
        .output()
        .expect("snapshot process launches");
    assert!(snapshot.status.success());
    let source: SourceSnapshotV2 = serde_json::from_slice(&snapshot.stdout).expect("snapshot JSON");
    assert!(source.is_clean);
    assert!(!source.head.is_empty());
    assert!(!source.tree.is_empty());

    let input = directory.path().join("input.json");
    let mut fixture: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/release-truth-v2.json"))
            .expect("fixture JSON");
    fixture["source_binding"] = serde_json::json!({"pre": source, "post": source});
    fs::write(
        &input,
        serde_json::to_vec(&fixture).expect("fixture serialization"),
    )
    .expect("fixture input");
    let first = Command::new(binary)
        .args(["evaluate-v2", "--input"])
        .arg(&input)
        .arg("--record")
        .current_dir(repo)
        .output()
        .expect("evaluation process launches");
    assert!(
        first.status.success(),
        "{}",
        String::from_utf8_lossy(&first.stderr)
    );
    let first_output: EvaluateOutput =
        serde_json::from_slice(&first.stdout).expect("evaluation JSON");
    assert_eq!(first_output.redaction_count, 1);
    assert!(first_output.recorded_event.is_some());
    assert_eq!(
        first_output.report.claims[0].support_state,
        SupportState::Supported
    );
    assert_eq!(
        first_output.report.claims[1].support_state,
        SupportState::Unsupported
    );
    assert_eq!(
        first_output.report.claims[2].support_state,
        SupportState::Unknown
    );

    let second = Command::new(binary)
        .args(["evaluate-v2", "--input"])
        .arg(&input)
        .arg("--record")
        .current_dir(repo)
        .output()
        .expect("second evaluation process launches");
    assert!(
        second.status.success(),
        "{}",
        String::from_utf8_lossy(&second.stderr)
    );
    let second_output: EvaluateOutput =
        serde_json::from_slice(&second.stdout).expect("second evaluation JSON");
    assert_eq!(
        first_output.report.canonical_digest,
        second_output.report.canonical_digest
    );
    let events = repo.join(".aew/v2/runs/fixture-false-green-v2/events");
    let entries = fs::read_dir(&events).expect("event directory").count();
    assert_eq!(entries, 1);
    let event = fs::read_to_string(
        fs::read_dir(&events)
            .expect("event directory")
            .next()
            .expect("event entry")
            .expect("event path")
            .path(),
    )
    .expect("event text");
    assert!(!event.contains("secret-token-123"));
}

#[test]
fn capture_v2_executes_the_bound_command_and_records_a_real_source_pair() {
    let directory = tempdir().expect("tempdir");
    let repo = directory.path().join("repository");
    fs::create_dir_all(&repo).expect("repository directory");
    let repo = repo.as_path();
    command(&["git", "init", "-q"], repo);
    command(
        &["git", "config", "user.email", "fixture@example.invalid"],
        repo,
    );
    command(&["git", "config", "user.name", "AEW Fixture"], repo);
    fs::write(repo.join("README.md"), "fixture\n").expect("fixture source");
    command(&["git", "add", "README.md"], repo);
    command(&["git", "commit", "-qm", "fixture"], repo);

    let mut request: serde_json::Value =
        serde_json::from_str(include_str!("../fixtures/release-truth-v2.json"))
            .expect("fixture JSON");
    request["commands"] = serde_json::json!([]);
    request["claims"] = serde_json::json!([request["claims"][0].clone()]);
    request["links"] = serde_json::json!([request["links"][0].clone()]);
    request
        .as_object_mut()
        .expect("request object")
        .remove("source_binding");
    let input = directory.path().join("capture-request.json");
    fs::write(
        &input,
        serde_json::to_vec(&request).expect("request serialization"),
    )
    .expect("capture request");

    let binary = env!("CARGO_BIN_EXE_aew");
    let output = Command::new(binary)
        .args(["capture-v2", "--input"])
        .arg(&input)
        .args(["--evidence-id", "actual-test-command", "--"])
        .args(["python3", "-c", "print('fixture_test_passed')"])
        .current_dir(repo)
        .output()
        .expect("capture process launches");
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    let captured: EvaluateOutput = serde_json::from_slice(&output.stdout).expect("capture JSON");
    assert_eq!(
        captured.report.claims[0].support_state,
        SupportState::Supported
    );
    assert!(captured.recorded_event.is_some());
    let event = fs::read_to_string(
        repo.join(".aew/v2/runs/fixture-false-green-v2/events")
            .read_dir()
            .expect("event directory")
            .next()
            .expect("event")
            .expect("event entry")
            .path(),
    )
    .expect("event text");
    assert!(event.contains("fixture_test_passed"));
}
