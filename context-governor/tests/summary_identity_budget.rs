use context_governor::{
    compact_context, BudgetMode, CompactRequest, CompactionPolicy, ContextGovernorError, Message,
};
use serde_json::Value;

const PREFIX: &str = "[CONTEXT COMPACTION — RECEIPT-BACKED REFERENCE ONLY]";

fn message(role: &str, content: String) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content,
        name: None,
        metadata: Default::default(),
    }
}

fn inherited_summary(role: &str) -> Message {
    let mut inherited = message(
        role,
        format!("{PREFIX}\n{}", "retained prior detail. ".repeat(24)),
    );
    inherited.id = Some("summary_ctxp_previous".to_string());
    inherited.name = Some("context_governor".to_string());
    inherited
        .metadata
        .insert("compressed_summary".to_string(), Value::Bool(true));
    inherited
}

#[test]
fn hard_cascade_preserves_inherited_summary_and_edits_only_current_summary() {
    for role in ["assistant", "user"] {
        let inherited = inherited_summary(role);
        let latest = message("user", "Latest exact request.".to_string());
        let response = compact_context(CompactRequest {
            hmac_key_path: None,
            session_id: format!("budget-identity-{role}"),
            messages: vec![
                message("system", "System".to_string()),
                inherited.clone(),
                message("tool", "ordinary archived output. ".repeat(600)),
                latest.clone(),
            ],
            policy: CompactionPolicy {
                target_tokens: 300,
                protect_first_n: 2,
                protect_last_n: 1,
                summary_max_chars: 8000,
                budget_mode: BudgetMode::HardCascade,
                ..Default::default()
            },
            focus: None,
        })
        .expect("the generated summary can fit without rewriting inherited context");

        assert!(response.compacted_messages.iter().any(|msg| {
            msg.id == inherited.id
                && msg.role == inherited.role
                && msg.name == inherited.name
                && msg.metadata == inherited.metadata
                && msg.content == inherited.content
        }));
        let generated_id = format!("summary_{}", response.allocation_plan.plan_id);
        assert!(response
            .compacted_messages
            .iter()
            .any(|msg| msg.id.as_deref() == Some(generated_id.as_str())));
        assert!(response.receipt.compacted_approx_tokens <= 300);
        assert_eq!(
            response.compacted_messages.last().unwrap().content,
            latest.content
        );
        assert!(response
            .exact_store
            .iter()
            .any(|stored| stored.content.contains("ordinary archived output.")));
    }
}

#[test]
fn hard_cascade_refuses_when_inherited_protected_content_really_exceeds_budget() {
    for role in ["assistant", "user"] {
        let result = compact_context(CompactRequest {
            hmac_key_path: None,
            session_id: format!("budget-floor-{role}"),
            messages: vec![
                message("system", "System".to_string()),
                inherited_summary(role),
                message("tool", "ordinary archived output. ".repeat(600)),
                message("user", "Latest exact request.".to_string()),
            ],
            policy: CompactionPolicy {
                target_tokens: 50,
                protect_first_n: 2,
                protect_last_n: 1,
                budget_mode: BudgetMode::HardCascade,
                ..Default::default()
            },
            focus: None,
        });
        assert!(matches!(
            result,
            Err(ContextGovernorError::CannotMeetTarget { .. })
        ));
    }
}
