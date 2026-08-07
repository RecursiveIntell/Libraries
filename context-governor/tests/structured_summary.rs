use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message};

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
fn structured_summary_extracts_operational_anchors() {
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "summary".into(),
        messages: vec![
            msg("system", "system"),
            msg(
                "user",
                "Build the parser. Acceptance gate: cargo test must pass.",
            ),
            msg("assistant", "Decision: use a deterministic parser."),
            msg(
                "tool",
                "running: cargo test\nerror[E0425]: cannot find value\n/src/lib.rs",
            ),
            msg("assistant", "Unresolved question: should JSON be strict?"),
            msg("user", "Latest task: finish the parser."),
        ],
        policy: CompactionPolicy {
            target_tokens: 260,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let structured = &response.receipt.summary_loss_report.structured_summary;
    assert_eq!(
        structured.active_task.as_deref(),
        Some("Latest task: finish the parser.")
    );
    assert!(structured
        .acceptance_gates
        .iter()
        .any(|s| s.contains("cargo test")));
    assert!(structured
        .decisions
        .iter()
        .any(|s| s.contains("deterministic parser")));
    assert!(structured.errors.iter().any(|s| s.contains("E0425")));
    assert!(structured.files.iter().any(|s| s.contains("/src/lib.rs")));
    assert!(structured.commands.iter().any(|s| s.contains("cargo test")));
    assert!(structured
        .unresolved_questions
        .iter()
        .any(|s| s.contains("JSON")));
    assert!(!structured.fallback_item_ids.is_empty());

    let summary_text = response
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("Structured anchors"))
        .unwrap()
        .content
        .clone();
    assert!(summary_text.contains("Latest task: finish the parser"));
    assert!(summary_text.contains("cargo test"));
}
