use context_governor::{
    compact_context, hash_messages, hash_messages_sha256, CompactRequest, CompactResponse,
    CompactionPolicy, Message,
};
use std::io::Write;
use std::process::{Command, Stdio};

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
