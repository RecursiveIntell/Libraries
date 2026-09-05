use agent_evidence_workbench::{
    receipt,
    storage::append_v2_event,
    v2::{
        evaluate, redact_text, ClaimEvidenceLinkV2, CommandEvidenceV2, CommandOutcomeV2,
        EvidenceRelationV2, ExplicitClaimV2, ReleaseTruthInputV2, RunEventV2, SourceBindingV2,
        SourceSnapshotV2,
    },
};
use claim_ledger::SupportState;
use std::fs;
use tempfile::tempdir;

fn command(id: &str, outcome: CommandOutcomeV2, argv: &[&str], stdout: &str) -> CommandEvidenceV2 {
    CommandEvidenceV2 {
        id: id.into(),
        execution_mode: "argv".into(),
        argv: argv.iter().map(|value| (*value).to_string()).collect(),
        cwd: "/fixture".into(),
        outcome,
        stdout: stdout.into(),
        stderr: String::new(),
        observed_at: "2026-09-05T00:00:00Z".into(),
        recorded_at: "2026-09-05T00:00:01Z".into(),
    }
}

fn binding() -> SourceBindingV2 {
    let s = SourceSnapshotV2 {
        repository_path: "/fixture".into(),
        head: "h".into(),
        tree: "t".into(),
        status: String::new(),
        is_clean: true,
        diff_digest: "d".into(),
        workspace_content_digest: "w".into(),
        observed_at: "2026-09-05T00:00:00Z".into(),
    };
    SourceBindingV2 {
        pre: s.clone(),
        post: s,
    }
}

#[test]
fn printed_success_cannot_satisfy_a_declared_test_requirement() {
    let input = ReleaseTruthInputV2 {
        schema_version: "aew.release-truth-input.v2".into(),
        run_id: "fixture-false-green".into(),
        claims: vec![
            ExplicitClaimV2 {
                id: "actual-test".into(),
                text: "the fixture test passed".into(),
                required_evidence: vec!["actual-test-command".into()],
            },
            ExplicitClaimV2 {
                id: "printed-success".into(),
                text: "tests pass".into(),
                required_evidence: vec!["actual-test-command".into()],
            },
            ExplicitClaimV2 {
                id: "blocked-check".into(),
                text: "the unavailable check passed".into(),
                required_evidence: vec!["blocked-command".into()],
            },
        ],
        commands: vec![
            command(
                "actual-test-command",
                CommandOutcomeV2::Passed,
                &["python3", "-c", "print('fixture_test_passed')"],
                "fixture_test_passed\n",
            ),
            command(
                "printed-output-command",
                CommandOutcomeV2::Passed,
                &["sh", "-c", "printf 'tests pass\\n'"],
                "tests pass\n",
            ),
            command("blocked-command", CommandOutcomeV2::Blocked, &[], ""),
        ],
        links: vec![
            ClaimEvidenceLinkV2 {
                claim_id: "actual-test".into(),
                evidence_id: "actual-test-command".into(),
                relation: EvidenceRelationV2::Supports,
            },
            ClaimEvidenceLinkV2 {
                claim_id: "printed-success".into(),
                evidence_id: "printed-output-command".into(),
                relation: EvidenceRelationV2::Mentions,
            },
            ClaimEvidenceLinkV2 {
                claim_id: "blocked-check".into(),
                evidence_id: "blocked-command".into(),
                relation: EvidenceRelationV2::Supports,
            },
        ],
        source_binding: Some(binding()),
    };

    let report = evaluate(&input).expect("fixture evaluates");
    assert_eq!(report.claims[0].support_state, SupportState::Supported);
    assert_eq!(report.claims[1].support_state, SupportState::Unsupported);
    assert_eq!(report.claims[2].support_state, SupportState::Unknown);
    assert_eq!(
        report.claims[2].command_outcomes,
        vec![CommandOutcomeV2::Blocked]
    );
    assert_ne!(report.canonical_digest, "");
}

#[test]
fn redaction_removes_secret_sentinels_before_persistence() {
    let redacted = redact_text("Authorization: Bearer secret-token-123\napi_key=sk-very-secret");
    assert!(!redacted.text.contains("secret-token-123"));
    assert!(!redacted.text.contains("sk-very-secret"));
    assert!(redacted.redaction_count >= 2);
}

#[test]
fn v2_event_storage_is_idempotent_and_rejects_conflicts() {
    let directory = tempdir().expect("tempdir");
    let event = RunEventV2 {
        schema_version: "aew.run-event.v2".into(),
        event_id: "event-1".into(),
        kind: "command_observed".into(),
        payload: serde_json::json!({"command": "true"}),
        observed_at: "2026-09-05T00:00:00Z".into(),
        recorded_at: "2026-09-05T00:00:01Z".into(),
    };
    let first = append_v2_event(directory.path(), "run-1", &event).expect("first append");
    let second = append_v2_event(directory.path(), "run-1", &event).expect("exact replay");
    assert_eq!(first, second);
    let mut conflicting = event.clone();
    conflicting.payload = serde_json::json!({"command": "false"});
    assert!(append_v2_event(directory.path(), "run-1", &conflicting).is_err());

    let scalar = RunEventV2 {
        schema_version: "aew.run-event.v2".into(),
        event_id: "scalar-event".into(),
        kind: "command_observed".into(),
        payload: serde_json::json!("Bearer scalar-secret-123"),
        observed_at: "2026-09-05T00:00:00Z".into(),
        recorded_at: "2026-09-05T00:00:01Z".into(),
    };
    let scalar_path = append_v2_event(directory.path(), "run-1", &scalar).expect("scalar append");
    let scalar_text = fs::read_to_string(scalar_path).expect("scalar event text");
    assert!(!scalar_text.contains("scalar-secret-123"));
    assert!(scalar_text.contains("redaction_policy_version"));
}

#[test]
fn receipt_key_file_is_accepted_without_key_material_on_argv() {
    let directory = tempdir().expect("tempdir");
    let path = directory.path().join("key.hex");
    fs::write(&path, "07".repeat(32) + "\n").expect("key file");
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&path, fs::Permissions::from_mode(0o600)).expect("private mode");
    }
    let key = receipt::parse_key_file(&path).expect("key file parses");
    assert_eq!(key.len(), 32);
}
