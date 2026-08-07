use context_governor::{
    compact_context, context_search, CompactRequest, CompactionPolicy, Message, SearchScope,
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

fn main() {
    let transcript = vec![
        msg("system", "You are a coding agent."),
        msg(
            "user",
            "Build parser. Acceptance gate: cargo test must pass.",
        ),
        msg("assistant", "Decision: use deterministic JSON parsing."),
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
    let required = [
        "cargo test must pass",
        "deterministic JSON parsing",
        "E0425",
        "/src/lib.rs",
    ];
    let full_tokens = context_governor::approx_tokens_messages(&transcript);
    let head_tail = vec![
        transcript[0].clone(),
        transcript[transcript.len() - 1].clone(),
    ];
    let head_tail_text = head_tail
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    let response = compact_context(CompactRequest {
        hmac_key_path: None,
        session_id: "replay-eval".into(),
        messages: transcript,
        policy: CompactionPolicy {
            target_tokens: 260,
            protect_first_n: 0,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    })
    .expect("compaction succeeds");
    let compacted_text = response
        .compacted_messages
        .iter()
        .map(|m| m.content.as_str())
        .collect::<Vec<_>>()
        .join("\n");

    println!("mode,visible,recoverable,total_required,tokens");
    println!(
        "full,{},{},{},{}",
        required.len(),
        required.len(),
        required.len(),
        full_tokens
    );
    let head_tail_visible = required
        .iter()
        .filter(|needle| head_tail_text.contains(**needle))
        .count();
    println!(
        "head_tail,{},{},{},{}",
        head_tail_visible,
        head_tail_visible,
        required.len(),
        context_governor::approx_tokens_messages(&head_tail)
    );
    let compacted_visible = required
        .iter()
        .filter(|needle| compacted_text.contains(**needle))
        .count();
    let recoverable = required
        .iter()
        .filter(|needle| !context_search(&response, needle, 1, SearchScope::All).is_empty())
        .count();
    println!(
        "context_governor,{},{},{},{}",
        compacted_visible,
        recoverable,
        required.len(),
        response.receipt.compacted_approx_tokens
    );
}
