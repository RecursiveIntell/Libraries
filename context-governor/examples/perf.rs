use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message};
use std::time::Instant;

fn main() {
    let cases = [100usize, 500, 1_000, 2_000];
    let iterations = 25usize;
    println!("messages,original_tokens,compacted_tokens,savings_tokens,avg_ms,p50_ms,p95_ms,throughput_msgs_per_s,fallback_refs,quarantined");
    for message_count in cases {
        let request = make_request(message_count);
        let mut durations = Vec::with_capacity(iterations);
        let mut last_original = 0usize;
        let mut last_compacted = 0usize;
        let mut last_savings = 0isize;
        let mut last_fallback_refs = 0usize;
        let mut last_quarantined = 0usize;
        for _ in 0..iterations {
            let start = Instant::now();
            let response = compact_context(request.clone()).expect("compaction should succeed");
            durations.push(start.elapsed().as_secs_f64() * 1_000.0);
            last_original = response.receipt.original_approx_tokens;
            last_compacted = response.receipt.compacted_approx_tokens;
            last_savings = response.receipt.token_savings_estimate;
            last_fallback_refs = response.receipt.exact_fallback_refs.len();
            last_quarantined = response.allocation_plan.quarantined_item_ids.len();
        }
        durations.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let avg = durations.iter().sum::<f64>() / durations.len() as f64;
        let p50 = percentile(&durations, 0.50);
        let p95 = percentile(&durations, 0.95);
        let throughput = message_count as f64 / (avg / 1_000.0);
        println!(
            "{message_count},{last_original},{last_compacted},{last_savings},{avg:.3},{p50:.3},{p95:.3},{throughput:.1},{last_fallback_refs},{last_quarantined}"
        );
    }
}

fn make_request(message_count: usize) -> CompactRequest {
    let mut messages = Vec::with_capacity(message_count);
    messages.push(message("system", "You are an operator-grade coding agent."));
    messages.push(message(
        "user",
        "Build the feature. Acceptance gate: preserve latest request, failing errors, exact fallback, and test receipts.",
    ));
    for idx in 2..message_count.saturating_sub(1) {
        let role = match idx % 4 {
            0 => "tool",
            1 => "assistant",
            2 => "user",
            _ => "tool",
        };
        let content = match idx % 10 {
            0 => format!(
                "error: compile failed in /tmp/generated_{idx}.rs:13\n{}",
                "stack line ".repeat(40)
            ),
            1 => format!("Decision: store receipt {idx} with exact fallback and allocation plan."),
            2 => format!("This likely connects to speculative graph-edge material {idx}."),
            3 => {
                format!("/home/sikmindz/project/src/file_{idx}.rs changed; cargo test should pass.")
            }
            _ => format!(
                "verbose tool output {idx}: {}",
                "low value repeated output ".repeat(60)
            ),
        };
        messages.push(message(role, &content));
    }
    messages.push(message(
        "user",
        "Latest active request: finish implementation and report receipts.",
    ));
    CompactRequest {
        hmac_key_path: None,
        session_id: format!("perf-{message_count}"),
        messages,
        policy: CompactionPolicy {
            target_tokens: 8_000,
            protect_first_n: 3,
            protect_last_n: 12,
            summary_max_chars: 12_000,
            ..Default::default()
        },
        focus: None,
    }
}

fn message(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content: content.to_string(),
        name: None,
        metadata: Default::default(),
    }
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len().saturating_sub(1)) as f64 * p).round() as usize;
    sorted[idx]
}
