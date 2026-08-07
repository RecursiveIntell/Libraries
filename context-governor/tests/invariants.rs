use context_governor::{compact_context, BudgetMode, CompactRequest, CompactionPolicy, Message};

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
fn latest_user_survives_all_budget_modes() {
    // HardLimit intentionally fails when budget is insufficient — skip it here.
    let modes = [BudgetMode::SoftWarn, BudgetMode::HardCascade];
    for mode in &modes {
        let response = compact_context(CompactRequest {
            session_id: format!("invariant-mode-{:?}", mode),
            messages: vec![
                msg("system", "old system prompt"),
                msg("assistant", &"old narrative ".repeat(500)),
                msg("tool", &"bulk log ".repeat(500)),
                msg("user", "LATEST_USER_INVARIANT_MARKER"),
            ],
            policy: CompactionPolicy {
                target_tokens: 100,
                budget_mode: mode.clone(),
                protect_first_n: 0,
                protect_last_n: 1,
                ..Default::default()
            },
            focus: None,
        })
        .unwrap();
        let last = response.compacted_messages.last().unwrap();
        assert_eq!(last.role, "user");
        assert!(
            last.content.contains("LATEST_USER_INVARIANT_MARKER"),
            "Mode {:?} lost latest user message",
            mode
        );
    }
}

#[test]
fn latest_user_survives_after_many_cycles() {
    let mut messages = Vec::new();
    for i in 0..20 {
        messages.push(msg(
            "assistant",
            &format!("turn {} response with some content padding", i),
        ));
        messages.push(msg("user", &format!("turn {} follow-up question", i)));
    }
    messages.push(msg("user", "FINAL_CYCLE_USER_MARKER"));

    let response = compact_context(CompactRequest {
        session_id: "cycle-invariant".into(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 500,
            budget_mode: BudgetMode::HardCascade,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let last = response.compacted_messages.last().unwrap();
    assert_eq!(last.role, "user");
    assert!(last.content.contains("FINAL_CYCLE_USER_MARKER"));
}

#[test]
fn unsafe_summary_policy_roundtrips_through_json() {
    // Verify the policy serializes and deserializes correctly.
    let policy = CompactionPolicy {
        unsafe_summary_policy: context_governor::UnsafeSummaryPolicy::FailClosed,
        ..Default::default()
    };
    let json = serde_json::to_string(&policy).unwrap();
    assert!(json.contains("fail_closed"));
    let rt: CompactionPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(
        rt.unsafe_summary_policy,
        context_governor::UnsafeSummaryPolicy::FailClosed
    );
}

#[test]
fn default_unsafe_summary_policy_is_fallback_extract() {
    let policy = CompactionPolicy::default();
    assert_eq!(
        policy.unsafe_summary_policy,
        context_governor::UnsafeSummaryPolicy::FallbackExtract
    );
}

#[test]
fn checkpoint_policy_defaults_and_serializes() {
    use context_governor::{CheckpointPolicy, CheckpointStrategy};
    let policy = CompactionPolicy::default();
    assert_eq!(policy.checkpoint.strategy, CheckpointStrategy::AfterN(2));
    assert_eq!(policy.checkpoint.max_checkpoints_per_session, Some(10));

    // Explicit override round-trips through JSON
    let custom = CheckpointPolicy {
        strategy: CheckpointStrategy::IneffectiveOnly,
        max_checkpoints_per_session: Some(3),
    };
    let json = serde_json::to_string(&custom).unwrap();
    assert!(json.contains("ineffective_only"));
    let rt: CheckpointPolicy = serde_json::from_str(&json).unwrap();
    assert_eq!(rt.strategy, CheckpointStrategy::IneffectiveOnly);
    assert_eq!(rt.max_checkpoints_per_session, Some(3));

    // Threshold variant
    let threshold = CheckpointPolicy {
        strategy: CheckpointStrategy::ThresholdPct(50),
        max_checkpoints_per_session: None,
    };
    let json = serde_json::to_string(&threshold).unwrap();
    assert!(json.contains("threshold_pct"));
}
