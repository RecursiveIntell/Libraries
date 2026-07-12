use context_governor::{build_context_steps, extract_plan_state, Message};

fn msg(role: &str, content: String) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content,
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn build_context_steps_groups_by_user_turns() {
    let messages = vec![
        msg("user", "first request".into()),
        msg("assistant", "response".into()),
        msg("tool", "tool output".into()),
        msg("user", "second request".into()),
        msg("assistant", "second response".into()),
    ];
    let steps = build_context_steps(&messages);
    assert_eq!(steps.len(), 2);
    assert_eq!(steps[0].start_message_index, 0);
    assert_eq!(steps[0].end_message_index, 3);
    assert_eq!(steps[1].start_message_index, 3);
    assert_eq!(steps[1].end_message_index, 5);
    assert!(steps[1].is_latest_user_step);
    assert!(!steps[0].is_latest_user_step);
}

#[test]
fn build_context_steps_detects_active_instructions() {
    let messages = vec![
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass".into(),
        ),
        msg("assistant", "Decision: use JSON parsing".into()),
        msg("user", "latest task".into()),
    ];
    let steps = build_context_steps(&messages);
    assert!(steps[0].has_active_instruction);
    assert!(!steps[1].has_active_instruction);
}

#[test]
fn build_context_steps_detects_errors() {
    let messages = vec![
        msg("user", "run task".into()),
        msg("tool", "error: compilation failed".into()),
        msg("user", "fix it".into()),
    ];
    let steps = build_context_steps(&messages);
    assert!(steps[0].has_error);
}

#[test]
fn extract_plan_state_finds_acceptance_gates_and_decisions() {
    let messages = vec![
        msg(
            "user",
            "Acceptance gate: cargo test --all-targets must pass".into(),
        ),
        msg("assistant", "Decision: use deterministic parsing".into()),
        msg("user", "latest task".into()),
    ];
    let steps = build_context_steps(&messages);
    let plan = extract_plan_state(&steps, &messages);
    assert!(plan
        .acceptance_gates
        .iter()
        .any(|g| g.contains("cargo test")));
    assert!(plan
        .decisions
        .iter()
        .any(|d| d.contains("deterministic parsing")));
}

#[test]
fn build_context_steps_preserves_tool_result_parts() {
    let messages = vec![
        msg("user", "run it".into()),
        msg("tool", "{\"exit_code\": 0}".into()),
        msg("user", "next".into()),
    ];
    let steps = build_context_steps(&messages);
    assert!(steps[0]
        .content_parts
        .iter()
        .any(|p| matches!(p.part_kind, context_governor::ContentPartKind::ToolResult)));
}

#[test]
fn build_context_steps_handles_single_message() {
    let messages = vec![msg("user", "single message".into())];
    let steps = build_context_steps(&messages);
    assert_eq!(steps.len(), 1);
    assert!(steps[0].is_latest_user_step);
}
