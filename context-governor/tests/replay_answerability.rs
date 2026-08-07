use context_governor::{
    evaluate_replay_answerability, CompactRequest, CompactionPolicy, Message,
    ReplayAnswerabilityQuestion,
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
fn replay_answerability_scores_operational_questions_after_compaction() {
    let messages = vec![
        msg("system", "You are a coding agent."),
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass.",
        ),
        msg(
            "assistant",
            "Decision: use deterministic JSON parsing, not regex.",
        ),
        msg(
            "tool",
            &format!(
                "{}\nerror[E0425]: cannot find value `parser`\n/src/lib.rs",
                "bulk log\n".repeat(800)
            ),
        ),
        msg("assistant", "Fixed compile error in /src/lib.rs."),
        msg("user", "Latest task: summarize what remains."),
    ];
    let questions = vec![
        ReplayAnswerabilityQuestion {
            question: "What must pass?".into(),
            expected_terms: vec!["cargo test must pass".into()],
            forbidden_terms: vec![],
        },
        ReplayAnswerabilityQuestion {
            question: "What parser strategy was chosen?".into(),
            expected_terms: vec!["deterministic JSON parsing".into()],
            forbidden_terms: vec!["regex".into()],
        },
        ReplayAnswerabilityQuestion {
            question: "Which compile error and file mattered?".into(),
            expected_terms: vec!["E0425".into(), "/src/lib.rs".into()],
            forbidden_terms: vec![],
        },
    ];

    let report = evaluate_replay_answerability(
        "answerability-fixture",
        CompactRequest {
            hmac_key_path: None,
            session_id: "answerability".into(),
            messages,
            policy: CompactionPolicy {
                target_tokens: 260,
                protect_first_n: 0,
                protect_last_n: 1,
                ..Default::default()
            },
            focus: None,
        },
        questions,
    )
    .expect("answerability report");

    let governed = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "context_governor")
        .expect("context-governor baseline");
    assert_eq!(governed.total_questions, 3);
    assert_eq!(governed.answerable_questions, 3);
    assert_eq!(governed.incorrect_action_risk, 0);
    assert!(governed.active_task_visible);
}
