use context_governor::{
    compact_context, BudgetMode, CompactRequest, CompactionPolicy, ContextGovernorError, Message,
    TokenCounterKind,
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
fn token_counter_kind_is_recorded_in_receipt() {
    let response = compact_context(CompactRequest {
        session_id: "policy-token-counter".into(),
        messages: vec![msg("user", "latest")],
        policy: CompactionPolicy {
            token_counter: TokenCounterKind::ApproxChars,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    assert_eq!(
        response.receipt.token_counter,
        TokenCounterKind::ApproxChars
    );
    let json = serde_json::to_string(&response.receipt).unwrap();
    assert!(json.contains("approx_chars"));
}

#[test]
fn hard_cascade_keeps_output_under_budget_when_possible() {
    let response = compact_context(CompactRequest {
        session_id: "policy-hard".into(),
        messages: vec![
            msg("system", "sys"),
            msg("assistant", &"old low risk narrative ".repeat(1_000)),
            msg("tool", &"bulk log line\n".repeat(1_000)),
            msg("user", "latest concise task"),
        ],
        policy: CompactionPolicy {
            target_tokens: 220,
            protect_first_n: 0,
            protect_last_n: 1,
            budget_mode: BudgetMode::HardCascade,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    assert!(
        response.receipt.compacted_approx_tokens <= 220,
        "{}",
        response.receipt.compacted_approx_tokens
    );
    assert!(response
        .receipt
        .warnings
        .iter()
        .any(|w| w.contains("hard cascade")));
}

#[test]
fn fail_closed_errors_when_exact_preserve_exceeds_budget() {
    let err = compact_context(CompactRequest {
        session_id: "policy-fail".into(),
        messages: vec![msg(
            "user",
            &format!("latest must stay exact {}", "x ".repeat(1_000)),
        )],
        policy: CompactionPolicy {
            target_tokens: 12,
            budget_mode: BudgetMode::FailClosed,
            protect_first_n: 0,
            protect_last_n: 0,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap_err();

    assert!(matches!(err, ContextGovernorError::BudgetExceeded { .. }));
}
