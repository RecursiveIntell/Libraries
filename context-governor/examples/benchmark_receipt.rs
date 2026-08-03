use context_governor::{
    approx_tokens_messages, compact_context, context_search, BudgetMode, CompactRequest,
    CompactionPolicy, Message, SearchScope,
};
use serde::Serialize;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Serialize)]
struct Receipt {
    created_utc: String,
    iterations: usize,
    cases: Vec<CaseReceipt>,
}

#[derive(Debug, Serialize)]
struct CaseReceipt {
    name: String,
    message_count: usize,
    original_tokens: usize,
    target_tokens: usize,
    context_governor_tokens: usize,
    token_reduction_rate: f64,
    avg_ms: f64,
    p50_ms: f64,
    p95_ms: f64,
    fallback_refs: usize,
    exact_store_items: usize,
    quarantined_items: usize,
    warning_count: usize,
    baselines: Vec<BaselineScore>,
}

#[derive(Debug, Serialize)]
struct BaselineScore {
    name: String,
    tokens: usize,
    visible: usize,
    recoverable: usize,
    total: usize,
    visible_rate: f64,
    recoverable_rate: f64,
    active_task_visible: bool,
}

fn main() {
    let iterations = std::env::var("CG_BENCH_ITERATIONS")
        .ok()
        .and_then(|value| value.parse::<usize>().ok())
        .unwrap_or(50);
    let out_dir = std::env::var("CG_BENCH_OUT")
        .map(PathBuf::from)
        .unwrap_or_else(|_| PathBuf::from("target/context-governor-bench"));
    fs::create_dir_all(&out_dir).expect("create benchmark output dir");

    let mut cases = Vec::new();
    for message_count in [100usize, 500, 1_000, 2_000] {
        cases.push(run_case(message_count, iterations));
    }

    let stamp = chrono::Utc::now().format("%Y%m%d-%H%M%S").to_string();
    let receipt = Receipt {
        created_utc: chrono::Utc::now().to_rfc3339(),
        iterations,
        cases,
    };
    let json_path = out_dir.join(format!("context-governor-benchmark-{stamp}.json"));
    let md_path = out_dir.join(format!("context-governor-benchmark-{stamp}.md"));
    fs::write(
        &json_path,
        serde_json::to_string_pretty(&receipt).expect("serialize json receipt"),
    )
    .expect("write json receipt");
    fs::write(&md_path, render_markdown(&receipt, &json_path)).expect("write markdown receipt");
    println!("{}", md_path.display());
    println!("{}", json_path.display());
    println!("{}", render_markdown(&receipt, &json_path));
}

fn run_case(message_count: usize, iterations: usize) -> CaseReceipt {
    let target_tokens = match message_count {
        100 => 4_000,
        500 => 8_000,
        1_000 => 12_000,
        _ => 16_000,
    };
    let request = make_request(message_count, target_tokens);
    let probes = make_probes(message_count);
    let original_tokens = approx_tokens_messages(&request.messages);
    let full_text = join_messages(&request.messages);
    let head_tail = budgeted_head_tail(&request.messages, target_tokens);
    let tail = budgeted_tail(&request.messages, target_tokens);

    let mut durations = Vec::with_capacity(iterations);
    let mut last_response = None;
    for _ in 0..iterations {
        let start = Instant::now();
        let response = compact_context(request.clone()).expect("context-governor compaction");
        durations.push(start.elapsed().as_secs_f64() * 1_000.0);
        last_response = Some(response);
    }
    durations.sort_by(|a, b| a.partial_cmp(b).expect("finite duration"));
    let avg_ms = durations.iter().sum::<f64>() / durations.len() as f64;
    let response = last_response.expect("at least one iteration");
    let governed_text = join_messages(&response.compacted_messages);
    let governed_score = score_context_governor(
        "context_governor_soft",
        &governed_text,
        &response,
        response.receipt.compacted_approx_tokens,
        &probes,
    );
    let mut hard_request = request.clone();
    hard_request.policy.budget_mode = BudgetMode::HardCascade;
    let hard_response = compact_context(hard_request).expect("hard cascade compaction");
    let hard_text = join_messages(&hard_response.compacted_messages);
    let hard_score = score_context_governor(
        "context_governor_hard",
        &hard_text,
        &hard_response,
        hard_response.receipt.compacted_approx_tokens,
        &probes,
    );

    CaseReceipt {
        name: format!("synthetic-agent-{message_count}"),
        message_count,
        original_tokens,
        target_tokens,
        context_governor_tokens: response.receipt.compacted_approx_tokens,
        token_reduction_rate: 1.0
            - (response.receipt.compacted_approx_tokens as f64 / original_tokens.max(1) as f64),
        avg_ms,
        p50_ms: percentile(&durations, 0.50),
        p95_ms: percentile(&durations, 0.95),
        fallback_refs: response.receipt.exact_fallback_refs.len(),
        exact_store_items: response.exact_store.len(),
        quarantined_items: response.allocation_plan.quarantined_item_ids.len(),
        warning_count: response.receipt.warnings.len(),
        baselines: vec![
            score_text("full", &full_text, original_tokens, &probes),
            score_text(
                "budgeted_head_tail",
                &join_messages(&head_tail),
                approx_tokens_messages(&head_tail),
                &probes,
            ),
            score_text(
                "budgeted_tail",
                &join_messages(&tail),
                approx_tokens_messages(&tail),
                &probes,
            ),
            governed_score,
            hard_score,
        ],
    }
}

fn make_request(message_count: usize, target_tokens: usize) -> CompactRequest {
    let mut messages = Vec::with_capacity(message_count);
    messages.push(msg(
        "system",
        "You are a coding agent. Preserve active task, acceptance gates, failing errors, file paths, decisions, and receipts.",
    ));
    messages.push(msg(
        "user",
        &format!(
            "Build context-governor benchmark case {message_count}. Acceptance gate: CG_GATE_{message_count} cargo test must pass."
        ),
    ));
    messages.push(msg(
        "assistant",
        &format!("Decision: CG_DECISION_{message_count} use deterministic governed compaction with exact fallback."),
    ));

    for idx in 3..message_count.saturating_sub(1) {
        let role = match idx % 5 {
            0 => "tool",
            1 => "assistant",
            2 => "user",
            3 => "tool",
            _ => "assistant",
        };
        let content = match idx {
            17 => format!(
                "error[E{message_count}17]: CG_ERROR_{message_count}_17 failed in /home/sikmindz/project/src/cg_case_{message_count}.rs\n{}",
                "stack frame ".repeat(120)
            ),
            41 => format!(
                "Source: verified receipt CG_RECEIPT_{message_count}_41 at /home/sikmindz/project/receipts/cg_{message_count}.json"
            ),
            73 => format!(
                "Unresolved question: should CG_QUESTION_{message_count}_73 use hard cascade for strict budget adapters?"
            ),
            _ if idx % 19 == 0 => format!(
                "This likely speculative note CG_SPEC_{message_count}_{idx} would likely connect unrelated systems. {}",
                "artifact boilerplate ".repeat(40)
            ),
            _ if idx % 13 == 0 => format!(
                "Decision: CG_DECISION_{message_count}_{idx} keep fallback refs for omitted tool output."
            ),
            _ if idx % 11 == 0 => format!(
                "/home/sikmindz/project/src/cg_file_{message_count}_{idx}.rs changed; cargo test should pass."
            ),
            _ if idx % 7 == 0 => format!(
                "tool output {idx}: {}",
                "low value generated log line ".repeat(80)
            ),
            _ => format!(
                "narrative turn {idx}: {}",
                "routine implementation discussion ".repeat(25)
            ),
        };
        messages.push(msg(role, &content));
    }
    messages.push(msg(
        "user",
        &format!(
            "Latest active request: CG_ACTIVE_{message_count} finish the benchmark, preserve receipts, and report exact results."
        ),
    ));
    CompactRequest {
        session_id: format!("benchmark-{message_count}"),
        messages,
        policy: CompactionPolicy {
            target_tokens,
            protect_first_n: 2,
            protect_last_n: 8,
            summary_max_chars: 16_000,
            ..Default::default()
        },
        focus: None,
    }
}

fn make_probes(message_count: usize) -> Vec<String> {
    vec![
        format!("CG_ACTIVE_{message_count}"),
        format!("CG_GATE_{message_count}"),
        format!("CG_DECISION_{message_count}"),
        format!("CG_ERROR_{message_count}_17"),
        format!("/home/sikmindz/project/src/cg_case_{message_count}.rs"),
        format!("CG_RECEIPT_{message_count}_41"),
        format!("CG_QUESTION_{message_count}_73"),
    ]
}

fn score_text(name: &str, text: &str, tokens: usize, probes: &[String]) -> BaselineScore {
    let visible = probes.iter().filter(|probe| text.contains(*probe)).count();
    build_score(
        name,
        tokens,
        visible,
        visible,
        probes.len(),
        text.contains(&probes[0]),
    )
}

fn score_context_governor(
    name: &str,
    text: &str,
    response: &context_governor::CompactResponse,
    tokens: usize,
    probes: &[String],
) -> BaselineScore {
    let visible = probes.iter().filter(|probe| text.contains(*probe)).count();
    let recoverable = probes
        .iter()
        .filter(|probe| !context_search(response, probe, 1, SearchScope::All).is_empty())
        .count();
    build_score(
        name,
        tokens,
        visible,
        recoverable,
        probes.len(),
        text.contains(&probes[0]),
    )
}

fn build_score(
    name: &str,
    tokens: usize,
    visible: usize,
    recoverable: usize,
    total: usize,
    active_task_visible: bool,
) -> BaselineScore {
    let denom = total.max(1) as f64;
    BaselineScore {
        name: name.to_string(),
        tokens,
        visible,
        recoverable,
        total,
        visible_rate: visible as f64 / denom,
        recoverable_rate: recoverable as f64 / denom,
        active_task_visible,
    }
}

fn budgeted_head_tail(messages: &[Message], target_tokens: usize) -> Vec<Message> {
    if messages.is_empty() {
        return Vec::new();
    }
    let mut out = vec![messages[0].clone()];
    let mut used = approx_tokens_messages(&out);
    let mut left = 1usize;
    let mut right = messages.len().saturating_sub(1);
    let mut take_tail = true;
    while left <= right {
        let idx = if take_tail { right } else { left };
        let candidate_tokens = context_governor::approx_tokens_text(&messages[idx].content) + 4;
        if used + candidate_tokens > target_tokens {
            if take_tail {
                if right == 0 {
                    break;
                }
                right = right.saturating_sub(1);
            } else {
                left += 1;
            }
            take_tail = !take_tail;
            continue;
        }
        out.push(messages[idx].clone());
        used += candidate_tokens;
        if take_tail {
            if right == 0 {
                break;
            }
            right = right.saturating_sub(1);
        } else {
            left += 1;
        }
        take_tail = !take_tail;
    }
    out.sort_by_key(|message| {
        messages
            .iter()
            .position(|candidate| {
                candidate.content == message.content && candidate.role == message.role
            })
            .unwrap_or(usize::MAX)
    });
    out.dedup_by(|a, b| a.role == b.role && a.content == b.content);
    out
}

fn budgeted_tail(messages: &[Message], target_tokens: usize) -> Vec<Message> {
    let mut out = Vec::new();
    let mut used = 0usize;
    for message in messages.iter().rev() {
        let candidate_tokens = context_governor::approx_tokens_text(&message.content) + 4;
        if used + candidate_tokens > target_tokens {
            continue;
        }
        out.push(message.clone());
        used += candidate_tokens;
    }
    out.reverse();
    out
}

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.to_string(),
        content: content.to_string(),
        name: None,
        metadata: Default::default(),
    }
}

fn join_messages(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn percentile(sorted: &[f64], p: f64) -> f64 {
    let idx = ((sorted.len().saturating_sub(1)) as f64 * p).round() as usize;
    sorted[idx]
}

fn render_markdown(receipt: &Receipt, json_path: &std::path::Path) -> String {
    let mut out = Vec::new();
    out.push("# context-governor benchmark receipt".to_string());
    out.push(String::new());
    out.push(format!("- Created: `{}`", receipt.created_utc));
    out.push(format!("- Iterations per case: `{}`", receipt.iterations));
    out.push(format!("- JSON receipt: `{}`", json_path.display()));
    out.push(String::new());
    out.push("| Case | Original tokens | Target | Governed tokens | Reduction | Avg ms | P95 ms | Fallback refs | Quarantined | Warnings |".to_string());
    out.push("|---|---:|---:|---:|---:|---:|---:|---:|---:|---:|".to_string());
    for case in &receipt.cases {
        out.push(format!(
            "| `{}` | {} | {} | {} | {:.1}% | {:.3} | {:.3} | {} | {} | {} |",
            case.name,
            case.original_tokens,
            case.target_tokens,
            case.context_governor_tokens,
            case.token_reduction_rate * 100.0,
            case.avg_ms,
            case.p95_ms,
            case.fallback_refs,
            case.quarantined_items,
            case.warning_count
        ));
    }
    out.push(String::new());
    out.push("## Probe Scores".to_string());
    for case in &receipt.cases {
        out.push(String::new());
        out.push(format!("### {}", case.name));
        out.push(
            "| Baseline | Tokens | Visible | Recoverable | Visible rate | Recoverable rate | Active task |"
                .to_string(),
        );
        out.push("|---|---:|---:|---:|---:|---:|---:|".to_string());
        for baseline in &case.baselines {
            out.push(format!(
                "| `{}` | {} | {}/{} | {}/{} | {:.1}% | {:.1}% | {} |",
                baseline.name,
                baseline.tokens,
                baseline.visible,
                baseline.total,
                baseline.recoverable,
                baseline.total,
                baseline.visible_rate * 100.0,
                baseline.recoverable_rate * 100.0,
                baseline.active_task_visible
            ));
        }
    }
    out.push(String::new());
    out.push("Claim boundary: this benchmark measures deterministic compaction throughput, token reduction, anchor visibility, and exact fallback recoverability on synthetic agent transcripts. It does not measure LLM task success or semantic summary quality.".to_string());
    out.push(String::new());
    out.join("\n")
}
