use context_governor::{
    compact_context, filter_recall_candidate, hash_messages, BudgetMode, CompactRequest,
    CompactionPolicy, Message, RecallCandidate, RecallDecision,
};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content: content.to_string(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn preserves_latest_user_acceptance_gates_and_errors_verbatim() {
    let messages = vec![
        msg("system", "You are a coding agent."),
        msg("user", "Build the thing."),
        msg("assistant", "I will work."),
        msg("tool", &"noise ".repeat(2_000)),
        msg(
            "user",
            "Acceptance gate: cargo test -p context-governor must pass.",
        ),
        msg("assistant", "Ran tests."),
        msg("tool", "error: compilation failed at /tmp/example.rs:13"),
        msg("user", "Now preserve this exact latest instruction."),
    ];

    let receipt = compact_context(CompactRequest {
        session_id: "s1".into(),
        messages: messages.clone(),
        policy: CompactionPolicy {
            target_tokens: 220,
            ..Default::default()
        },
        focus: None,
    })
    .expect("compaction succeeds");

    let joined = receipt
        .compacted_messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    assert!(joined.contains("Now preserve this exact latest instruction."));
    assert!(joined.contains("Acceptance gate: cargo test -p context-governor must pass."));
    assert!(joined.contains("error: compilation failed at /tmp/example.rs:13"));
    assert!(!receipt.receipt.exact_fallback_refs.is_empty());
    assert!(receipt.receipt.token_savings_estimate > 0);
}

#[test]
fn compacted_summary_is_reference_only_and_has_expand_handles() {
    let messages = vec![
        msg("system", "System"),
        msg("user", "Initial request"),
        msg("assistant", "Historical implementation details."),
        msg("tool", &"long old output ".repeat(1_000)),
        msg("user", "Latest active request"),
    ];

    let result = compact_context(CompactRequest {
        session_id: "s2".into(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 180,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary = result
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("CONTEXT COMPACTION"))
        .unwrap();
    assert!(summary.content.contains("REFERENCE ONLY"));
    assert!(summary.content.contains("context_expand"));
    assert!(summary.content.contains(&result.receipt.receipt_id));
    assert!(result
        .exact_store
        .iter()
        .any(|r| r.content.contains("long old output")));
}

#[test]
fn speculative_and_artifact_recall_are_not_authoritative() {
    let speculative = RecallCandidate {
        source: Some("projects".into()),
        content: "This likely relates to a gap and would potentially connect systems.".into(),
        score: Some(0.9),
    };
    let artifact = RecallCandidate {
        source: Some("MASTER_CODEX_IMPLEMENTATION_PROMPT_FINAL_V2".into()),
        content: "[aicc_spec_pack.zip] MVP_IMPLEMENTATION_PLAN ROLLBACK_AND_QUARANTINE_PLAN".into(),
        score: Some(0.9),
    };

    let s = filter_recall_candidate(&speculative, "Hermes context compaction");
    let a = filter_recall_candidate(&artifact, "Hermes context compaction");
    assert_eq!(s.decision, RecallDecision::QuarantineSpeculative);
    assert_eq!(a.decision, RecallDecision::RejectNoise);
}

#[test]
fn artifact_recall_can_be_background_when_query_explicitly_matches() {
    let artifact = RecallCandidate {
        source: Some("MASTER_CODEX_IMPLEMENTATION_PROMPT_FINAL_V2".into()),
        content: "[aicc_spec_pack.zip] MVP_IMPLEMENTATION_PLAN".into(),
        score: Some(0.9),
    };

    let result = filter_recall_candidate(&artifact, "inspect AICC pack manifest");
    assert_eq!(result.decision, RecallDecision::AdmitBackground);
}

#[test]
fn receipt_hash_matches_final_compacted_messages_after_receipt_id_injection() {
    let result = compact_context(CompactRequest {
        session_id: "hash-final".into(),
        messages: vec![
            msg("system", "system"),
            msg("tool", &format!("{} HASH_NEEDLE", "bulk ".repeat(1_000))),
            msg("user", "latest active task"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    assert_eq!(
        result.receipt.compacted_transcript_blake3,
        hash_messages(&result.compacted_messages).unwrap()
    );
    assert_eq!(
        result.receipt.compacted_approx_tokens,
        context_governor::approx_tokens_messages(&result.compacted_messages)
    );
}

#[test]
fn summary_is_inserted_before_latest_user_so_latest_task_stays_active() {
    let result = compact_context(CompactRequest {
        session_id: "latest-after-summary".into(),
        messages: vec![
            msg("system", "system"),
            msg("assistant", &"historical detail ".repeat(500)),
            msg("user", "latest active task must be final"),
        ],
        policy: CompactionPolicy {
            target_tokens: 120,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary_idx = result
        .compacted_messages
        .iter()
        .position(|m| m.content.contains("CONTEXT COMPACTION"))
        .unwrap();
    let latest_idx = result
        .compacted_messages
        .iter()
        .position(|m| m.content == "latest active task must be final")
        .unwrap();

    assert!(summary_idx < latest_idx);
    assert_eq!(
        result.compacted_messages.last().unwrap().content,
        "latest active task must be final"
    );
}

#[test]
fn archived_durable_items_have_exact_store_records_even_when_kept() {
    let result = compact_context(CompactRequest {
        session_id: "archive-kept".into(),
        messages: vec![
            msg("system", "system"),
            msg(
                "user",
                "Decision: keep this durable architecture choice exact.",
            ),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 2_000,
            protect_first_n: 0,
            protect_last_n: 1,
            archive_memory_enabled: true,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let archived = result
        .allocation_plan
        .archived_item_ids
        .first()
        .expect("decision should be archived");
    assert!(result
        .exact_store
        .iter()
        .any(|item| item.item_id == *archived));
}

#[test]
fn hard_cascade_keeps_minimal_recovery_pointer_when_it_fits() {
    let result = compact_context(CompactRequest {
        session_id: "hard-minimal-pointer".into(),
        messages: vec![
            msg(
                "tool",
                &format!("{} HARD_POINTER_NEEDLE", "bulk ".repeat(1_000)),
            ),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            target_tokens: 140,
            protect_first_n: 0,
            protect_last_n: 1,
            budget_mode: BudgetMode::HardCascade,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    let summary = result
        .compacted_messages
        .iter()
        .find(|m| m.content.contains("CONTEXT COMPACTION"))
        .expect("minimal summary should remain");
    assert!(summary.content.contains(&result.receipt.receipt_id));
    assert!(summary.content.contains("fallback_item_ids"));
    assert!(result.receipt.compacted_approx_tokens <= 140);
}
