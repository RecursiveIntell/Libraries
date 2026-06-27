use context_governor::{
    compact_context, context_diff, context_expand, context_search, CompactRequest,
    CompactionPolicy, Message, SearchScope,
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
fn expand_recovers_exact_omitted_text_by_item_id() {
    let response = compact_context(CompactRequest {
        session_id: "tools".into(),
        messages: vec![
            msg("system", "system"),
            msg("user", "start"),
            msg(
                "tool",
                &format!("{} UNIQUE_NEEDLE", "old verbose output ".repeat(500)),
            ),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    let item_id = response
        .exact_store
        .iter()
        .find(|i| i.content.contains("UNIQUE_NEEDLE"))
        .unwrap()
        .item_id
        .clone();
    let expanded = context_expand(&response, &item_id, 20_000).unwrap();
    assert!(expanded.content.contains("UNIQUE_NEEDLE"));
    assert!(!expanded.truncated);
}

#[test]
fn search_finds_receipt_and_exact_store_content_without_dumping_everything() {
    let response = compact_context(CompactRequest {
        session_id: "search".into(),
        messages: vec![
            msg("system", "system"),
            msg("user", "start"),
            msg("tool", &format!("{} rare_token_alpha", "bulk ".repeat(600))),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    let hits = context_search(&response, "rare_token_alpha", 5, SearchScope::All);
    assert_eq!(hits.len(), 1);
    assert!(hits[0].snippet.contains("rare_token_alpha"));
    assert!(hits[0].snippet.len() < 360);
}

#[test]
fn diff_reports_policy_counts() {
    let response = compact_context(CompactRequest {
        session_id: "diff".into(),
        messages: vec![
            msg("system", "system"),
            msg("user", "Acceptance gate: exact text must stay."),
            msg("tool", &"bulk ".repeat(600)),
            msg(
                "assistant",
                "This likely connects to something speculative.",
            ),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    let diff = context_diff(&response);
    assert!(diff.kept_count >= 2);
    assert!(diff.summarized_count >= 1);
    assert!(diff.quarantined_count >= 1);
    assert!(diff.token_savings_estimate > 0);
}
