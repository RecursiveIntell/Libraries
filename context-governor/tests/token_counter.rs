use context_governor::{
    compact_context, CompactRequest, CompactionPolicy, Message, TokenCounterKind,
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
fn approx_words_counter_is_used_for_item_accounting() {
    let content = "alpha ".repeat(20);
    let response = compact_context(CompactRequest {
        session_id: "token-counter".into(),
        messages: vec![msg("user", &content)],
        policy: CompactionPolicy {
            token_counter: TokenCounterKind::ApproxWords,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    assert_eq!(response.allocation_plan.items[0].approx_tokens, 20);
    assert_eq!(response.receipt.original_approx_tokens, 24);
}

#[test]
fn approximate_token_counter_records_provider_budget_warning() {
    let response = compact_context(CompactRequest {
        session_id: "token-warning".into(),
        messages: vec![msg("user", "latest")],
        policy: CompactionPolicy::default(),
        focus: None,
    })
    .unwrap();

    assert!(response
        .receipt
        .warnings
        .iter()
        .any(|warning| warning.contains("token_counter is approximate")));
}
