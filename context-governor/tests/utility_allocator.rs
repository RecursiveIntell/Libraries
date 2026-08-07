use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message};

fn msg(role: &str, content: String) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content,
        name: None,
        metadata: Default::default(),
    }
}

fn utility_policy(target_tokens: usize) -> CompactionPolicy {
    CompactionPolicy {
        target_tokens,
        protect_first_n: 0,
        protect_last_n: 1,
        allocator: "utility_v2".to_string(),
        ..Default::default()
    }
}

fn item_id_at(response: &context_governor::CompactResponse, index: usize) -> String {
    response.allocation_plan.items[index].item_id.clone()
}

#[test]
fn utility_v2_prefers_focus_relevant_middle_item_at_tight_budget() {
    let irrelevant = format!("{} unrelated record", "filler ".repeat(70));
    let relevant = format!("{} parser focus implementation", "filler ".repeat(70));
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "utility-focus".into(),
        messages: vec![
            msg("assistant", irrelevant),
            msg("assistant", relevant),
            msg("user", "finish parser focus implementation".into()),
        ],
        policy: utility_policy(210),
        focus: Some("parser focus implementation".into()),
    })
    .unwrap();

    let irrelevant_id = item_id_at(&response, 0);
    let relevant_id = item_id_at(&response, 1);
    assert!(!response
        .allocation_plan
        .kept_item_ids
        .contains(&irrelevant_id));
    assert!(response
        .allocation_plan
        .kept_item_ids
        .contains(&relevant_id));
}

#[test]
fn utility_v2_reserves_mandatory_system_gate_error_and_latest_user() {
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "utility-mandatory".into(),
        messages: vec![
            msg("system", "system instruction must remain".into()),
            msg("assistant", "acceptance gate: cargo test must pass".into()),
            msg("tool", "error: verified compiler failure".into()),
            msg("assistant", "optional ".repeat(500)),
            msg("user", "latest user instruction".into()),
        ],
        policy: utility_policy(20),
        focus: None,
    })
    .unwrap();

    for index in [0, 1, 2, 4] {
        assert!(
            response
                .allocation_plan
                .kept_item_ids
                .contains(&item_id_at(&response, index)),
            "mandatory item at {index} was not kept"
        );
    }
    assert_eq!(
        response.compacted_messages.last().unwrap().content,
        "latest user instruction"
    );
}

#[test]
fn utility_v2_is_stable_and_exposes_selection_evidence() {
    let request = CompactRequest {
        hmac_key_path: None,
        session_id: "utility-stable".into(),
        messages: vec![
            msg("assistant", "old history ".repeat(80)),
            msg("assistant", "focus parser novel detail ".repeat(40)),
            msg("user", "complete parser detail".into()),
        ],
        policy: utility_policy(160),
        focus: Some("parser detail".into()),
    };
    let first = compact_context(request.clone()).unwrap();
    let second = compact_context(request).unwrap();

    assert_eq!(
        first.allocation_plan.kept_item_ids,
        second.allocation_plan.kept_item_ids
    );
    assert_eq!(
        normalized_compacted_content(&first),
        normalized_compacted_content(&second)
    );
    assert!(!first.allocation_plan.selection_evidence.is_empty());
}

#[test]
fn utility_v2_penalizes_duplicate_optional_content_against_novel_focus_content() {
    let repeated = format!("{} shared history", "duplicate ".repeat(60));
    let novel = format!("{} parser optimization unique", "parser ".repeat(60));
    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "utility-duplicate".into(),
        messages: vec![
            msg("assistant", repeated.clone()),
            msg("assistant", repeated),
            msg("assistant", novel),
            msg("user", "parser optimization".into()),
        ],
        policy: utility_policy(300),
        focus: Some("parser optimization".into()),
    })
    .unwrap();

    assert!(response
        .allocation_plan
        .kept_item_ids
        .contains(&item_id_at(&response, 2)));
    assert!(!response
        .allocation_plan
        .kept_item_ids
        .contains(&item_id_at(&response, 1)));
}

#[test]
fn utility_v2_hot_path_membership_work_is_linear_for_large_inputs() {
    let mut messages = Vec::with_capacity(4_002);
    for index in 0..4_000 {
        messages.push(msg(
            "assistant",
            format!("history item {index} /tmp/project/src/{index}.rs"),
        ));
    }
    messages.push(msg("assistant", "parser focus unique result".into()));
    messages.push(msg("user", "complete parser focus".into()));

    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "utility-linear".into(),
        messages,
        policy: utility_policy(700),
        focus: Some("parser focus".into()),
    })
    .unwrap();

    let work = &response.allocation_plan.hot_path_operation_counts;
    assert!(work.total_membership_operations() <= 4_002 * 24, "{work:?}");
}

#[test]
fn deterministic_v1_explicit_mode_matches_default_semantics() {
    let messages = vec![
        msg("system", "system constraint".into()),
        msg("assistant", "history ".repeat(200)),
        msg("tool", "error: retained evidence".into()),
        msg("user", "latest request".into()),
    ];
    let default = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "deterministic-fixture".into(),
        messages: messages.clone(),
        policy: CompactionPolicy {
            target_tokens: 150,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();
    let explicit = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "deterministic-fixture".into(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 150,
            protect_first_n: 0,
            protect_last_n: 1,
            allocator: "deterministic_v1".into(),
            ..Default::default()
        },
        focus: None,
    })
    .unwrap();

    assert_eq!(
        default.allocation_plan.kept_item_ids,
        explicit.allocation_plan.kept_item_ids
    );
    assert_eq!(
        default.allocation_plan.summarized_item_ids,
        explicit.allocation_plan.summarized_item_ids
    );
    assert_eq!(
        normalized_compacted_content(&default),
        normalized_compacted_content(&explicit)
    );
}

fn normalized_compacted_content(
    response: &context_governor::CompactResponse,
) -> Vec<(String, String)> {
    response
        .compacted_messages
        .iter()
        .map(|message| {
            (
                message.role.clone(),
                message
                    .content
                    .replace(&response.receipt.receipt_id, "RECEIPT")
                    .replace(&response.allocation_plan.plan_id, "PLAN"),
            )
        })
        .collect()
}
