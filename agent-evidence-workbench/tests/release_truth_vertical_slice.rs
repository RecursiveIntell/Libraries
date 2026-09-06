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

#[test]
fn prove_cli_builds_a_single_claim_source_bound_evidence_packet() {
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
    let output = Command::new(binary)
        .args([
            "prove",
            "--run-id",
            "friendly-proof",
            "--claim",
            "the fixture command completed",
            "--",
            "python3",
            "-c",
            "print('proof-ok')",
        ])
        .current_dir(repo)
        .output()
        .expect("prove process launches");
    let proved: EvaluateOutput = serde_json::from_slice(&output.stdout).expect("proof JSON");
    assert_eq!(proved.report.run_id, "friendly-proof");
    assert_eq!(proved.report.claims.len(), 1);
    assert_ne!(
        proved.report.claims[0].support_state,
        SupportState::Supported
    );
    assert!(proved.recorded_event.is_some());
    assert!(
        !output.status.success(),
        "unsupported caller claim must produce a nonzero prove exit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn prove_records_a_passing_command_without_adjudicating_a_broad_caller_claim() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_aew"))
        .args([
            "prove",
            "--run-id",
            "b1-broad-claim",
            "--claim",
            "The entire test suite passed and the software is production ready",
            "--",
            "true",
        ])
        .current_dir(repo)
        .output()
        .expect("prove process launches");
    let proved: EvaluateOutput = serde_json::from_slice(&output.stdout).expect("proof JSON");
    assert_eq!(proved.report.claims[0].command_outcomes.len(), 1);
    assert_eq!(
        proved.report.claims[0].command_outcomes[0],
        agent_evidence_workbench::v2::CommandOutcomeV2::Passed
    );
    assert_eq!(
        proved.report.source_binding.pre.workspace_content_digest,
        proved.report.source_binding.post.workspace_content_digest
    );
    let event_path = proved
        .recorded_event
        .expect("retained event path in stdout");
    let event: serde_json::Value =
        serde_json::from_slice(&fs::read(event_path).expect("retained event is readable"))
            .expect("event JSON");
    assert_eq!(
        event["payload"]["content"]["input"]["commands"][0]["argv"],
        serde_json::json!(["true"])
    );
    assert_eq!(
        event["payload"]["content"]["input"]["commands"][0]["outcome"],
        serde_json::json!("passed")
    );
    assert_ne!(
        proved.report.claims[0].support_state,
        SupportState::Supported
    );
    assert!(
        !output.status.success(),
        "unsupported caller claim must produce a nonzero prove exit: {}",
        String::from_utf8_lossy(&output.stderr)
    );
}

#[test]
fn prove_persists_failed_command_evidence_before_returning_nonzero() {
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

    let output = Command::new(env!("CARGO_BIN_EXE_aew"))
        .args([
            "prove",
            "--run-id",
            "b2-failed-command",
            "--claim",
            "the fixture command completed",
            "--",
            "false",
        ])
        .current_dir(repo)
        .output()
        .expect("prove process launches");
    let proved: EvaluateOutput = serde_json::from_slice(&output.stdout).expect("proof JSON");
    let event_path = proved
        .recorded_event
        .expect("retained event path in stdout");
    let event = fs::read_to_string(&event_path).expect("retained event is readable");
    assert!(event.contains("\"outcome\": \"failed\""));
    assert_ne!(
        proved.report.claims[0].support_state,
        SupportState::Supported
    );
    assert!(
        !output.status.success(),
        "failed command must produce a nonzero prove exit"
    );
}

#[test]
fn prove_detects_same_path_ordinary_untracked_source_content_drift() {
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
    fs::write(repo.join("ordinary-untracked.txt"), "before\n").expect("untracked source");

    let binary = env!("CARGO_BIN_EXE_aew");
    let before = Command::new(binary)
        .arg("snapshot-v2")
        .current_dir(repo)
        .output()
        .expect("pre-command snapshot launches");
    assert!(before.status.success());
    let before: SourceSnapshotV2 = serde_json::from_slice(&before.stdout).expect("snapshot JSON");

    let proof = Command::new(binary)
        .args([
            "prove",
            "--run-id",
            "b3-untracked-drift",
            "--claim",
            "the fixture command completed",
            "--",
            "sh",
            "-c",
            "printf 'after\\n' > ordinary-untracked.txt",
        ])
        .current_dir(repo)
        .output()
        .expect("prove process launches");

    let after = Command::new(binary)
        .arg("snapshot-v2")
        .current_dir(repo)
        .output()
        .expect("post-command snapshot launches");
    assert!(after.status.success());
    let after: SourceSnapshotV2 = serde_json::from_slice(&after.stdout).expect("snapshot JSON");
    assert_ne!(
        before.workspace_content_digest, after.workspace_content_digest,
        "same-path ordinary untracked source content must affect the snapshot digest"
    );
    assert!(
        !proof.status.success(),
        "a proof command that mutates ordinary untracked source must fail closed"
    );
}

#[test]
fn report_v2_projects_a_recorded_event_deterministically() {
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
    let recorded = Command::new(binary)
        .args(["evaluate-v2", "--input"])
        .arg(&input)
        .arg("--record")
        .current_dir(repo)
        .output()
        .expect("evaluation process launches");
    assert!(
        recorded.status.success(),
        "{}",
        String::from_utf8_lossy(&recorded.stderr)
    );
    let recorded: EvaluateOutput =
        serde_json::from_slice(&recorded.stdout).expect("evaluation JSON");
    let event_path = recorded.recorded_event.expect("recorded event path");
    let event_id = Path::new(&event_path)
        .file_stem()
        .and_then(|id| id.to_str())
        .expect("event ID")
        .to_string();

    let json = Command::new(binary)
        .args([
            "report-v2",
            "fixture-false-green-v2",
            &event_id,
            "--format",
            "json",
        ])
        .current_dir(repo)
        .output()
        .expect("report process launches");
    assert!(
        json.status.success(),
        "{}",
        String::from_utf8_lossy(&json.stderr)
    );
    let packet: serde_json::Value =
        serde_json::from_slice(&json.stdout).expect("review packet JSON");
    assert_eq!(packet["event"]["event_id"], event_id);
    assert_eq!(packet["canonical_digest"], recorded.report.canonical_digest);
    assert_eq!(packet["claims"][0]["assertion"], "the fixture test passed");
    assert_eq!(packet["commands"][0]["argv"][0], "python3");
    assert!(packet["human_reviewer_decision"].is_null());
    assert!(packet["terminal_release_decision"].is_null());
    assert!(packet["non_authority_boundary"]
        .as_str()
        .expect("boundary")
        .contains("does not make a terminal release decision"));
    assert!(!String::from_utf8_lossy(&json.stdout).contains("secret-token-123"));

    let repeated = Command::new(binary)
        .args([
            "report-v2",
            "fixture-false-green-v2",
            &event_id,
            "--format",
            "json",
        ])
        .current_dir(repo)
        .output()
        .expect("repeat report process launches");
    assert!(repeated.status.success());
    assert_eq!(json.stdout, repeated.stdout);

    let markdown = Command::new(binary)
        .args([
            "report-v2",
            "fixture-false-green-v2",
            &event_id,
            "--format",
            "markdown",
        ])
        .current_dir(repo)
        .output()
        .expect("markdown report process launches");
    assert!(markdown.status.success());
    assert!(String::from_utf8_lossy(&markdown.stdout).contains("# AEW V2 Review Packet"));
}

#[test]
fn report_v2_fails_closed_for_missing_or_tampered_events() {
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
    let missing = Command::new(binary)
        .args([
            "report-v2",
            "fixture-false-green-v2",
            "evaluation-0000000000000000000000000000000000000000000000000000000000000000",
        ])
        .current_dir(repo)
        .output()
        .expect("missing report process launches");
    assert!(!missing.status.success());
    assert!(missing.stdout.is_empty());

    let snapshot = Command::new(binary)
        .arg("snapshot-v2")
        .current_dir(repo)
        .output()
        .expect("snapshot process launches");
    assert!(snapshot.status.success());
    let source: SourceSnapshotV2 = serde_json::from_slice(&snapshot.stdout).expect("snapshot JSON");
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
    let recorded = Command::new(binary)
        .args(["evaluate-v2", "--input"])
        .arg(&input)
        .arg("--record")
        .current_dir(repo)
        .output()
        .expect("evaluation process launches");
    assert!(recorded.status.success());
    let recorded: EvaluateOutput =
        serde_json::from_slice(&recorded.stdout).expect("evaluation JSON");
    let event_path = recorded.recorded_event.expect("recorded event path");
    let event_id = Path::new(&event_path)
        .file_stem()
        .and_then(|id| id.to_str())
        .expect("event ID")
        .to_string();
    let mut event: serde_json::Value =
        serde_json::from_slice(&fs::read(&event_path).expect("event bytes")).expect("event JSON");
    event["payload"]["content"]["report"]["canonical_digest"] =
        serde_json::json!("tampered-digest");
    fs::write(
        &event_path,
        serde_json::to_vec_pretty(&event).expect("tampered event"),
    )
    .expect("overwrite event");

    let tampered = Command::new(binary)
        .args(["report-v2", "fixture-false-green-v2", &event_id])
        .current_dir(repo)
        .output()
        .expect("tampered report process launches");
    assert!(!tampered.status.success());
    assert!(tampered.stdout.is_empty());
}
