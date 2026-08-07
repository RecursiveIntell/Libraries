use context_governor::{
    build_replay_probes, evaluate_replay_fixture, CompactRequest, CompactionPolicy, Message,
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
fn replay_probe_extraction_finds_operational_anchors() {
    let messages = vec![
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass.",
        ),
        msg("assistant", "Decision: use deterministic JSON parsing."),
        msg(
            "tool",
            "error[E0425]: cannot find value `parser` in /src/lib.rs",
        ),
        msg("user", "Latest task: summarize what remains."),
    ];

    let probes = build_replay_probes(&messages, 12);
    let needles = probes.iter().map(|p| p.needle.as_str()).collect::<Vec<_>>();

    assert!(needles.iter().any(|n| n.contains("cargo test must pass")));
    assert!(needles
        .iter()
        .any(|n| n.contains("deterministic JSON parsing")));
    assert!(needles.iter().any(|n| n.contains("E0425")));
    assert!(needles.iter().any(|n| n.contains("/src/lib.rs")));
    assert!(probes.iter().any(|p| p.category == "active_task"));
}

#[test]
fn replay_probe_extraction_uses_literal_substrings_for_long_active_task() {
    let long_task = format!(
        "Latest task: {}",
        "preserve this literal anchor ".repeat(20)
    );
    let messages = vec![msg("user", &long_task)];
    let probes = build_replay_probes(&messages, 4);
    let active = probes
        .iter()
        .find(|probe| probe.category == "active_task")
        .unwrap();

    assert!(long_task.contains(&active.needle));
    assert!(!active.needle.contains("..."));
}

#[test]
fn replay_fixture_scores_full_head_tail_and_context_governor() {
    let noisy_tool = format!(
        "{}\nerror[E0425]: cannot find value `parser` in /src/lib.rs\n",
        "bulk log line\n".repeat(800)
    );
    let messages = vec![
        msg("system", "You are a coding agent."),
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass.",
        ),
        msg("assistant", "Decision: use deterministic JSON parsing."),
        msg("tool", &noisy_tool),
        msg("assistant", "Fixed /src/lib.rs after E0425."),
        msg("user", "Latest task: summarize what remains."),
    ];

    let report = evaluate_replay_fixture(
        "synthetic-noisy-tool",
        CompactRequest {
            hmac_key_path: None,
            session_id: "synthetic-noisy-tool".into(),
            messages,
            policy: CompactionPolicy {
                target_tokens: 300,
                protect_first_n: 0,
                protect_last_n: 1,
                ..Default::default()
            },
            focus: None,
        },
        12,
    )
    .unwrap();

    let full = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "full")
        .unwrap();
    let head_tail = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "head_tail")
        .unwrap();
    let governed = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "context_governor")
        .unwrap();

    assert_eq!(full.recoverable_probes, full.total_probes);
    assert!(head_tail.recoverable_probes < full.total_probes);
    assert_eq!(governed.recoverable_probes, full.total_probes);
    assert!(governed.tokens < full.tokens);
    assert!(governed.active_task_visible);
}

#[test]
fn latest_user_duplicate_still_preserves_active_task_verbatim() {
    let messages = vec![
        msg("user", "implement everything. use claude where you can"),
        msg("assistant", "Working."),
        msg("user", "implement everything. use claude where you can"),
    ];
    let report = evaluate_replay_fixture(
        "latest-duplicate-active-task",
        CompactRequest {
            hmac_key_path: None,
            session_id: "latest-duplicate-active-task".into(),
            messages,
            policy: CompactionPolicy {
                target_tokens: 120,
                protect_first_n: 0,
                protect_last_n: 1,
                allocator: "aggressive_v1".into(),
                budget_mode: context_governor::BudgetMode::HardCascade,
                ..Default::default()
            },
            focus: None,
        },
        4,
    )
    .unwrap();
    let governed = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "context_governor")
        .unwrap();

    assert!(governed.active_task_visible);
}

#[test]
fn aggressive_replay_compaction_hits_budget_without_losing_exact_recovery() {
    let long_path_tool = format!(
        "running cargo test in /home/sikmindz/Coding/Libraries/context-governor\n{}\nerror[E0425]: cannot find value `parser` in /home/sikmindz/Coding/Libraries/context-governor/src/lib.rs\n",
        "bulk log line with /home/sikmindz/Coding/Libraries/context-governor/src/lib.rs\n".repeat(1200)
    );
    let long_path_evidence = format!(
        "Evidence receipt: /home/sikmindz/Coding/Libraries/context-governor/docs/eval-harness.md\n{}",
        "path-heavy evidence line /home/sikmindz/Coding/Libraries/context-governor/README.md\n".repeat(800)
    );
    let messages = vec![
        msg("system", "You are a coding agent."),
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass.",
        ),
        msg("tool", &long_path_tool),
        msg("assistant", "Decision: use deterministic JSON parsing."),
        msg("tool", &long_path_evidence),
        msg("user", "Latest task: summarize what remains."),
    ];

    let report = evaluate_replay_fixture(
        "synthetic-aggressive-budget",
        CompactRequest {
            hmac_key_path: None,
            session_id: "synthetic-aggressive-budget".into(),
            messages,
            policy: CompactionPolicy {
                target_tokens: 900,
                protect_first_n: 1,
                protect_last_n: 1,
                summary_max_chars: 2400,
                allocator: "aggressive_v1".into(),
                budget_mode: context_governor::BudgetMode::HardCascade,
                ..Default::default()
            },
            focus: None,
        },
        16,
    )
    .unwrap();

    let full = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "full")
        .unwrap();
    let governed = report
        .baselines
        .iter()
        .find(|baseline| baseline.name == "context_governor")
        .unwrap();

    assert!(
        governed.tokens <= 900,
        "governed tokens = {}",
        governed.tokens
    );
    assert!(
        governed.tokens * 10 < full.tokens,
        "expected >90% reduction: full={} governed={}",
        full.tokens,
        governed.tokens
    );
    assert_eq!(governed.recoverable_probes, full.total_probes);
    assert!(governed.active_task_visible);
}
