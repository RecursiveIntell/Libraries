use context_governor::{
    compact_context, context_search, CompactRequest, CompactionPolicy, Message, SearchScope,
};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn content_aware_summary_keeps_cargo_error_lines_not_bulk_noise() {
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
            session_id: "content-compression".into(),
        messages: vec![
            msg("system", "system"),
            msg("tool", &format!("{}\nerror[E0425]: cannot find value `x`\nwarning: unused import\ntest result: FAILED\n{}", "bulk line\n".repeat(500), "tail noise\n".repeat(500))),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 160,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary = response
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("CONTEXT COMPACTION"))
        .unwrap();
    assert!(summary.content.contains("error[E0425]"));
    assert!(summary.content.contains("test result: FAILED"));
    assert!(summary.content.matches("bulk line").count() < 3);
}

#[test]
fn json_summary_keeps_keys_and_search_still_finds_exact_payload() {
    let json_payload = r#"{"alpha": 1, "beta": true, "nested": {"needle": "JSON_NEEDLE"}}"#;
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "json-compression".into(),
        messages: vec![msg("assistant", json_payload), msg("user", "latest")],
        policy: CompactionPolicy {
            target_tokens: 20,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary = response
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("CONTEXT COMPACTION"))
        .unwrap();
    assert!(summary.content.contains("json keys"));
    assert!(summary.content.contains("alpha"));
    assert!(summary.content.contains("nested"));

    let hits = context_search(&response, "JSON_NEEDLE", 5, SearchScope::ExactStore);
    assert_eq!(hits.len(), 1);
}

#[test]
fn search_result_summary_keeps_paths_and_match_lines_not_bulk_noise() {
    let search_payload = format!(
        "{}\n/home/demo/src/lib.rs:42: receipt_index_status includes store_bytes\n/home/demo/tests/store.rs:9: context_expand recovers exact omitted text\n{}",
        "irrelevant search noise\n".repeat(400),
        "tail noise\n".repeat(400)
    );
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "search-result-compression".into(),
        messages: vec![msg("tool", &search_payload), msg("user", "latest")],
        policy: CompactionPolicy {
            target_tokens: 80,
            protect_first_n: 0,
            protect_last_n: 1,
            allocator: "aggressive_v1".into(),
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary = response
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("CONTEXT COMPACTION"))
        .unwrap();
    assert!(summary.content.contains("/home/demo/src/lib.rs"));
    assert!(summary.content.contains("store_bytes"));
    assert!(summary.content.matches("irrelevant search noise").count() < 3);
}
