use context_governor::{
    compact_context, compact_context_v2, hash_messages, hash_messages_sha256, CompactRequest,
    CompactResponse, CompactResponseV2, CompactionPolicy, Message,
};
use std::io::Write;
use std::os::fd::AsRawFd;
#[cfg(unix)]
use std::os::unix::process::CommandExt;
use std::process::{Command, Stdio};

#[cfg(unix)]
unsafe extern "C" {
    fn fcntl(fd: std::ffi::c_int, cmd: std::ffi::c_int, ...) -> std::ffi::c_int;
}
#[cfg(unix)]
const F_SETFD: std::ffi::c_int = 2;

struct GovernedFixture {
    key: tempfile::NamedTempFile,
    snapshot: tempfile::NamedTempFile,
}

impl GovernedFixture {
    fn new() -> Self {
        let mut key = tempfile::NamedTempFile::new().unwrap();
        key.write_all(&[0x73; 32]).unwrap();
        key.flush().unwrap();
        let key_id = context_governor::receipt_index::key_id(&[0x73; 32]).unwrap();
        let mut snapshot = tempfile::NamedTempFile::new().unwrap();
        snapshot
            .write_all(
                serde_json::to_string(&serde_json::json!({
                    "schema": "AresContextGovernorKeySnapshotV2",
                    "sequence": 1,
                    "active_key_id": key_id,
                    "retired_key_ids": [],
                    "compromised_key_ids": [],
                    "keys": {context_governor::receipt_index::key_id(&[0x73; 32]).unwrap(): "active"},
                }))
                .unwrap()
                .as_bytes(),
            )
            .unwrap();
        snapshot.flush().unwrap();
        Self { key, snapshot }
    }

    fn args(&self) -> Vec<String> {
        vec![
            "--governed-key-fd".into(),
            self.key.as_raw_fd().to_string(),
            "--governed-snapshot-fd".into(),
            self.snapshot.as_raw_fd().to_string(),
        ]
    }
}

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

fn run_cli(args: &[&str], stdin: &str) -> String {
    let mut child = Command::new(env!("CARGO_BIN_EXE_context-governor"))
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

fn run_governed_cli(args: &[String], stdin: &str, fixture: &GovernedFixture) -> String {
    let mut command = Command::new(env!("CARGO_BIN_EXE_context-governor"));
    command
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    #[cfg(unix)]
    unsafe {
        let key_fd = fixture.key.as_raw_fd();
        let snapshot_fd = fixture.snapshot.as_raw_fd();
        command.pre_exec(move || {
            for fd in [key_fd, snapshot_fd] {
                if fcntl(fd, F_SETFD, 0) == -1 {
                    return Err(std::io::Error::last_os_error());
                }
            }
            Ok(())
        });
    }
    let mut child = command.spawn().unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin.as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(
        output.status.success(),
        "stderr={}",
        String::from_utf8_lossy(&output.stderr)
    );
    String::from_utf8(output.stdout).unwrap()
}

#[test]
fn cli_compact_diff_store_search_and_expand_round_trip() {
    let dir = tempfile::tempdir().unwrap();
    let request = CompactRequest {
        hmac_key_path: None,
        session_id: "cli".into(),
        messages: vec![
            msg("system", "system"),
            msg("tool", &format!("{} CLI_NEEDLE", "verbose ".repeat(600))),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    };
    let request_json = serde_json::to_string(&request).unwrap();

    let response_json = run_cli(&["compact"], &request_json);
    let response: serde_json::Value = serde_json::from_str(&response_json).unwrap();
    let receipt_id = response["receipt"]["receipt_id"]
        .as_str()
        .unwrap()
        .to_string();
    let item_id = response["exact_store"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["content"].as_str().unwrap().contains("CLI_NEEDLE"))
        .unwrap()["item_id"]
        .as_str()
        .unwrap()
        .to_string();

    let diff_json = run_cli(&["diff"], &response_json);
    assert!(diff_json.contains("token_savings_estimate"));

    let store_json = run_cli(
        &["store", "--dir", dir.path().to_str().unwrap()],
        &response_json,
    );
    let stored: serde_json::Value = serde_json::from_str(&store_json).unwrap();
    assert_eq!(stored["receipt_id"], receipt_id);
    assert_eq!(stored["exact_recovery_state"], "persisted");
    assert_eq!(stored["verified"], true);

    let status_json = run_cli(&["status", "--dir", dir.path().to_str().unwrap()], "");
    assert!(status_json.contains("\"receipt_count\": 1"));
    assert!(status_json.contains("\"index_built\": false"));
    assert!(status_json.contains("\"searchable\": true"));
    assert!(status_json.contains(&receipt_id));

    let search_json = run_cli(
        &[
            "search",
            "--dir",
            dir.path().to_str().unwrap(),
            "--query",
            "CLI_NEEDLE",
        ],
        "",
    );
    assert!(search_json.contains(&receipt_id));
    assert!(search_json.contains("CLI_NEEDLE"));
    let indexed_status = run_cli(&["status", "--dir", dir.path().to_str().unwrap()], "");
    assert!(indexed_status.contains("\"index_built\": true"));

    let expand_json = run_cli(
        &[
            "expand",
            "--dir",
            dir.path().to_str().unwrap(),
            "--receipt",
            &receipt_id,
            "--item",
            &item_id,
        ],
        "",
    );
    assert!(expand_json.contains("CLI_NEEDLE"));

    let prune_json = run_cli(
        &[
            "prune",
            "--dir",
            dir.path().to_str().unwrap(),
            "--keep-last",
            "0",
        ],
        "",
    );
    assert!(prune_json.contains("\"removed_receipts\": 1"));
    let post_prune_status = run_cli(&["status", "--dir", dir.path().to_str().unwrap()], "");
    assert!(post_prune_status.contains("\"receipt_count\": 0"));
}

#[test]
fn no_args_cli_remains_backwards_compatible_compact() {
    let request = CompactRequest {
        hmac_key_path: None,
        session_id: "cli-compat".into(),
        messages: vec![msg("user", "latest")],
        policy: CompactionPolicy::default(),
        focus: None,
    };
    let out = run_cli(&[], &serde_json::to_string(&request).unwrap());
    assert!(out.contains("receipt_id"));
}

#[test]
fn finalize_cli_binds_receipt_to_adapter_emitted_messages() {
    let mut response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "cli-finalize".into(),
        messages: vec![msg("assistant", "old"), msg("user", "latest")],
        policy: CompactionPolicy::default(),
        focus: None,
    })
    .unwrap();
    response.compacted_messages[0].content = "adapter-finalized".into();

    let finalized_json = run_cli(&["finalize"], &serde_json::to_string(&response).unwrap());
    let finalized: CompactResponse = serde_json::from_str(&finalized_json).unwrap();

    assert_eq!(finalized.compacted_messages[0].content, "adapter-finalized");
    assert_eq!(
        finalized.receipt.compacted_transcript_blake3,
        hash_messages(&finalized.compacted_messages).unwrap()
    );
    assert_eq!(
        finalized.receipt.compacted_transcript_sha256,
        hash_messages_sha256(&finalized.compacted_messages).unwrap()
    );
}

#[test]
fn v2_cli_restarts_from_store_tip_and_expands_ancestor_source() {
    let dir = tempfile::tempdir().unwrap();
    let authority = GovernedFixture::new();
    let marker = "CLI_V2_RESTART_MARKER_4f20c831";
    let request = context_governor::CertifiedCompactRequest {
        session_id: "cli-v2-restart".into(),
        messages: vec![
            msg("system", "preserve exact evidence"),
            msg(
                "tool",
                &format!(
                    "{} {marker} {}",
                    "old ".repeat(1_000),
                    "tail ".repeat(1_000)
                ),
            ),
            msg("assistant", "inspected"),
            msg("user", "continue"),
        ],
        policy: CompactionPolicy {
            target_tokens: 180,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    };
    let dir_arg = dir.path().to_str().unwrap();
    let mut first_args = vec!["compact-v2".into(), "--dir".into(), dir_arg.into()];
    first_args.extend(authority.args());
    let first_json = run_governed_cli(
        &first_args,
        &serde_json::to_string(&request).unwrap(),
        &authority,
    );
    let first: CompactResponseV2 = serde_json::from_str(&first_json).unwrap();
    let source_id = first
        .source_evidence
        .iter()
        .find(|source| source.message.content.contains(marker))
        .unwrap()
        .source_id
        .clone();
    let mut store_args = vec!["store-v2".into(), "--dir".into(), dir_arg.into()];
    store_args.extend(authority.args());
    run_governed_cli(&store_args, &first_json, &authority);

    let mut second_messages = first.compacted_messages;
    second_messages.push(msg("assistant", "restart checkpoint"));
    second_messages.push(msg("user", "continue after restart"));
    let second_request = context_governor::CertifiedCompactRequest {
        messages: second_messages,
        ..request
    };
    let second_json = run_governed_cli(
        &first_args,
        &serde_json::to_string(&second_request).unwrap(),
        &authority,
    );
    let second: CompactResponseV2 = serde_json::from_str(&second_json).unwrap();
    assert_eq!(second.receipt.generation, 2);
    run_governed_cli(&store_args, &second_json, &authority);

    let mut expand_args = vec![
        "expand".into(),
        "--dir".into(),
        dir_arg.into(),
        "--receipt".into(),
        second.receipt.receipt_id.clone(),
        "--item".into(),
        source_id,
    ];
    expand_args.extend(authority.args());
    let expanded = run_governed_cli(&expand_args, "", &authority);
    assert!(expanded.contains(marker));
}

#[test]
fn v2_cli_finalize_prepare_recover_and_activate_is_two_phase() {
    let dir = tempfile::tempdir().unwrap();
    let authority = GovernedFixture::new();
    let request = context_governor::CertifiedCompactRequest {
        session_id: "cli-v2-two-phase".into(),
        messages: vec![
            msg("system", "preserve authenticated evidence"),
            msg("tool", &format!("{} TWO_PHASE_NEEDLE", "bulk ".repeat(800))),
            msg("user", "continue after durable host commit"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    };
    let dir_arg = dir.path().to_str().unwrap();
    let mut compact_args = vec!["compact-v2".into(), "--dir".into(), dir_arg.into()];
    compact_args.extend(authority.args());
    let candidate_json = run_governed_cli(
        &compact_args,
        &serde_json::to_string(&request).unwrap(),
        &authority,
    );
    let candidate: CompactResponseV2 = serde_json::from_str(&candidate_json).unwrap();

    let mut finalize_args = vec!["finalize-v2".into()];
    finalize_args.extend(authority.args());
    let finalized_json = run_governed_cli(
        &finalize_args,
        &serde_json::json!({
            "candidate": candidate,
            "compacted_messages": candidate.compacted_messages,
        })
        .to_string(),
        &authority,
    );
    let finalized: CompactResponseV2 = serde_json::from_str(&finalized_json).unwrap();

    let mut prepare_args = vec!["prepare-v2".into(), "--dir".into(), dir_arg.into()];
    prepare_args.extend(authority.args());
    let prepared_json = run_governed_cli(&prepare_args, &finalized_json, &authority);
    let prepared: serde_json::Value = serde_json::from_str(&prepared_json).unwrap();
    assert_eq!(prepared["generation"], 1);
    assert_eq!(prepared["verified"], true);
    assert!(prepared["created_utc"].is_string());
    assert!(!dir
        .path()
        .join(format!("{}.json", finalized.receipt.receipt_id))
        .exists());

    let mut pending_args = vec![
        "pending-v2".into(),
        "--dir".into(),
        dir_arg.into(),
        "--receipt".into(),
        finalized.receipt.receipt_id.clone(),
    ];
    pending_args.extend(authority.args());
    let pending_json = run_governed_cli(&pending_args, "", &authority);
    let pending: serde_json::Value = serde_json::from_str(&pending_json).unwrap();
    assert_eq!(pending.as_array().unwrap().len(), 1);
    assert_eq!(
        pending[0]["expected_compacted_message_count"],
        finalized.compacted_messages.len()
    );

    let mut activate_args = vec!["activate-v2".into(), "--dir".into(), dir_arg.into()];
    activate_args.extend(authority.args());
    let activated_json = run_governed_cli(
        &activate_args,
        &serde_json::json!({
            "receipt_id": finalized.receipt.receipt_id,
            "committed_messages": prepared["expected_compacted_messages"],
        })
        .to_string(),
        &authority,
    );
    let activated: serde_json::Value = serde_json::from_str(&activated_json).unwrap();
    assert_eq!(activated["activated"], true);
    assert_eq!(activated["verified"], true);

    let pending_after = run_governed_cli(&pending_args, "", &authority);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&pending_after).unwrap(),
        serde_json::json!([])
    );

    let unauthenticated_search = Command::new(env!("CARGO_BIN_EXE_context-governor"))
        .args(["search", "--dir", dir_arg, "--query", "TWO_PHASE_NEEDLE"])
        .output()
        .unwrap();
    assert!(!unauthenticated_search.status.success());
    assert!(String::from_utf8_lossy(&unauthenticated_search.stderr)
        .contains("governed key descriptors"));
}

#[test]
fn v2_cli_rejects_caller_selected_hmac_path() {
    let dir = tempfile::tempdir().unwrap();
    let request = serde_json::json!({"session_id":"reject", "messages":[{"role":"user","content":"latest"}], "policy":{}, "hmac_key_path":"/tmp/hostile"});
    let mut child = Command::new(env!("CARGO_BIN_EXE_context-governor"))
        .args([
            "compact-v2",
            "--dir",
            dir.path().to_str().unwrap(),
            "--hmac-key",
            "/tmp/hostile",
        ])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(request.to_string().as_bytes())
        .unwrap();
    let output = child.wait_with_output().unwrap();
    assert!(!output.status.success());
    assert!(String::from_utf8_lossy(&output.stderr).contains("ForbiddenCallerKeyMaterial"));
}

#[test]
fn v2_prompt_provenance_projection_is_bounded_without_losing_receipt_manifest() {
    let response = compact_context_v2(CompactRequest {
        hmac_key_path: None,
        session_id: "prompt-provenance-bound".into(),
        messages: vec![
            msg("system", "system"),
            msg("tool", "one"),
            msg("tool", "two"),
            msg("tool", "three"),
            msg("tool", "four"),
            msg("tool", "five"),
            msg("tool", "six"),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 1,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    assert!(response.receipt.covered_original_sources.len() > 4);
    let rendered: serde_json::Value = serde_json::from_str(&run_cli(
        &["render-prompt-v2"],
        &serde_json::to_string(&response).unwrap(),
    ))
    .unwrap();
    let user = rendered["user"].as_str().unwrap();
    let section = user
        .split("=== TRANSITIVE EXACT SOURCE IDS ===\n")
        .nth(1)
        .expect("V2 prompt exposes bounded provenance projection");
    let listed = response
        .receipt
        .covered_original_sources
        .iter()
        .filter(|source| section.contains(&source.source_id))
        .count();
    assert_eq!(listed, 4);
    assert!(section.contains("additional source IDs retained in the verified receipt store"));
}
