//! Specialized LLM summarization prompt renderer for iterated compaction.
//!
//! Standard LLM summarization degrades when applied repeatedly to the same
//! context: each pass loses signal and the summary converges to vague
//! generalities. This module renders a prompt that is explicitly aware of the
//! compactor's native data structures — structured summaries, exact fallback
//! refs, plan state, step graphs, and loss reports — so the LLM preserves
//! maximum signal per token across repeated compaction cycles.
//!
//! The prompt is designed around seven principles:
//!
//! 1. **Bitemporal fact preservation**: Keep (timestamp, actor, decision,
//!    outcome) tuples verbatim. Never rephrase or merge them.
//! 2. **Fallback ref integrity**: Never summarize or omit `exact_fallback_ref`
//!    IDs. Losing a ref ID destroys exact recovery.
//! 3. **Living question queue**: Carry forward every unresolved question. Mark
//!    resolved only when explicit evidence appears in new messages.
//! 4. **Category compression**: Compress by category (files, commands, errors,
//!    decisions) not as a monolithic block.
//! 5. **Explicit loss tracking**: After each summary, append what was dropped
//!    or generalized to the loss report.
//! 6. **Step graph lineage**: Use the compaction lineage to extend — not
//!    replace — the prior plan state.
//! 7. **Authority anchoring**: The latest user message is the authority.
//!    Re-rank older summaries against the new focus.

use crate::{
    CompactResponse, ContextCompactionReceiptV1, ContextStepV1, ExactStoredItemV1, Message,
    PlanStateV1, StructuralFloorV1, StructuredContextSummaryV1, SummaryLossReportV1,
};

/// A rendered LLM summarization prompt ready for an API call.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedPromptV1 {
    /// The system prompt containing compaction-aware instructions.
    pub system: String,
    /// The user prompt containing the structured context to summarize.
    pub user: String,
    /// Schema version for tracing.
    pub schema: String,
}

/// Configuration for the prompt renderer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PromptConfigV1 {
    /// Maximum tokens the rendered prompt should target (approximate).
    pub max_prompt_tokens: usize,
    /// Whether to include the full step graph in the prompt.
    pub include_step_graph: bool,
    /// Whether to include exact fallback content blobs in the prompt.
    pub include_exact_content: bool,
    /// Whether to include the prior loss report.
    pub include_loss_report: bool,
    /// Optional focus override; if None, uses the response's latest user message.
    pub focus_override: Option<String>,
}

impl Default for PromptConfigV1 {
    fn default() -> Self {
        Self {
            max_prompt_tokens: 8_000,
            include_step_graph: true,
            include_exact_content: false,
            include_loss_report: true,
            focus_override: None,
        }
    }
}

/// Render a specialized LLM summarization prompt from a compaction response
/// and new messages that arrived after compaction.
///
/// The prompt is structured to exploit the compactor's native types rather
/// than treating the context as opaque text. This preserves more signal per
/// token across iterated compaction cycles.
pub fn render_summary_prompt(
    prior_response: &CompactResponse,
    new_messages: &[Message],
    config: &PromptConfigV1,
) -> RenderedPromptV1 {
    let system = render_system_prompt();
    let user = render_user_prompt(prior_response, new_messages, config);
    RenderedPromptV1 {
        system,
        user,
        schema: "RenderedPromptV1".to_string(),
    }
}

/// Render the system prompt with compaction-aware instructions.
fn render_system_prompt() -> String {
    r#"You are an iterated context compaction summarizer. Your output will be re-fed into future compaction cycles. You must preserve maximum signal per token across multiple compaction cycles.

## OUTPUT CONTRACT (READ FIRST)

Your VERY FIRST output token MUST be `===`. No exceptions. No preamble. No "Let me analyze...". No "We need to produce...". No reasoning. No commentary. Start directly with:

=== ACTIVE TASK ===

If you write anything before `=== ACTIVE TASK ===`, the output is rejected and the entire context is lost. This is the most important rule.

End your output after the PRIOR CONTEXT SUMMARY section. Do not add anything after it.

## ANTI-DEGRADATION RULES

### Information Entropy Floor
Each compaction cycle loses information. To minimize loss:
- HARD FACTS (file paths, function names, line numbers, error messages, exit codes, crate names) must be preserved VERBATIM. These are irrecoverable if lost.
- SOFT FACTS (narrative context, reasoning chains) can be compressed, but only if the hard facts they reference are preserved.
- DECISIONS must include the original rationale. "We decided X" without "because Y" is useless in cycle 3+.

### 1. Bitemporal Fact Preservation
Keep (timestamp, actor, decision, outcome) tuples VERBATIM. Never rephrase, merge, or paraphrase them. If a decision was made at a specific time by a specific actor, that fact must survive every compaction cycle intact.

### 2. Fallback Ref Integrity
NEVER summarize, rephrase, or omit `exact_fallback_ref` IDs. These are content-addressed retrieval keys. Losing a ref ID destroys the ability to recover exact original content. Copy the full list of ref IDs into your output unchanged.

### 3. Living Question Queue
Carry forward EVERY unresolved question. Mark a question as resolved ONLY when explicit evidence appears in new messages. An unresolved question dropped in cycle 1 is gone forever. If you are unsure whether it is resolved, keep it as unresolved.

### 4. Category Compression
Compress each category independently, not as a monolithic block:
- FILES: (path, last known state, pending action)
- COMMANDS: (command, exit status, key output line)
- ERRORS: verbatim if they block a task; otherwise (error type, context)
- DECISIONS: (decision, rationale, timestamp)

### 5. Explicit Loss Tracking
After your summary, list what was dropped or generalized under "SUMMARY LOSSES". Be specific: "Dropped: exact API endpoint URL from message 3" is useful. "Dropped: some details" is not.

### 6. Step Graph Lineage
Use the compaction lineage to EXTEND the prior plan state, not replace it. If the prior state had an active plan, carry it forward. If new messages change the plan, note the transition explicitly.

### 7. Authority Anchoring
The latest user message is the authority. Re-rank older summaries against the new focus. Demote — but do not drop — older context that is no longer relevant, unless it contains unresolved questions or active acceptance gates.

## OUTPUT FORMAT

Your output MUST follow this structure exactly. The first token must be `===`. No text before the first section header. No text after the last section.

=== ACTIVE TASK ===
[One sentence describing the current task]

=== ACCEPTANCE GATES ===
[List of active acceptance gates, or "None"]

=== FILES ===
[path | state | pending action]
[One per line, or "None"]

=== COMMANDS ===
[command | exit status | key output]
[One per line, or "None"]

=== ERRORS ===
[error | blocks task? | context]
[One per line, or "None"]

=== DECISIONS ===
[decision | rationale | timestamp]
[One per line, or "None"]

=== UNRESOLVED QUESTIONS ===
[question]
[One per line, or "None"]

=== EXACT FALLBACK REFS ===
[Copy the full list of ref IDs from the input, one per line. NEVER omit any.]

=== SUMMARY LOSSES ===
[What was dropped or generalized in this pass, or "None"]

=== PRIOR CONTEXT SUMMARY ===
[Your compressed summary of prior context, organized by the categories above. This is the main body of your output. Be dense and factual. No filler. No "The user discussed..." — just the facts.]"#.to_string()
}

/// Render the user prompt with structured context from the compaction response.
fn render_user_prompt(
    prior_response: &CompactResponse,
    new_messages: &[Message],
    config: &PromptConfigV1,
) -> String {
    let mut sections = Vec::new();

    // Focus / authority anchor
    let focus = config.focus_override.clone().unwrap_or_else(|| {
        prior_response
            .compacted_messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| m.content.clone())
            .unwrap_or_default()
    });

    sections.push(format!(
        "=== CURRENT FOCUS ===\n{}",
        if focus.is_empty() {
            "(No explicit focus)"
        } else {
            &focus
        }
    ));

    // Active task
    let summary = extract_structured_summary(prior_response);
    sections.push(format!(
        "=== ACTIVE TASK ===\n{}",
        summary
            .active_task
            .as_deref()
            .unwrap_or("(No active task recorded)")
    ));

    // Acceptance gates
    sections.push(format_section(
        "ACCEPTANCE GATES",
        &summary.acceptance_gates,
    ));

    // Files
    sections.push(format_section("FILES", &summary.files));

    // Commands
    sections.push(format_section("COMMANDS", &summary.commands));

    // Errors
    sections.push(format_section("ERRORS", &summary.errors));

    // Decisions
    sections.push(format_section("DECISIONS", &summary.decisions));

    // Unresolved questions
    sections.push(format_section(
        "UNRESOLVED QUESTIONS",
        &summary.unresolved_questions,
    ));

    // Exact fallback refs — ALWAYS included, never compressed
    sections.push(format_fallback_refs(&prior_response.receipt));

    // Exact store content (optional)
    if config.include_exact_content && !prior_response.exact_store.is_empty() {
        sections.push(format_exact_store(&prior_response.exact_store));
    }

    // Plan state
    sections.push(format_plan_state(&prior_response.plan_state));

    // Structural floor
    sections.push(format_structural_floor(&prior_response.structural_floor));

    // Step graph (optional)
    if config.include_step_graph && !prior_response.context_steps.is_empty() {
        sections.push(format_step_graph(&prior_response.context_steps));
    }

    // Loss report (optional)
    if config.include_loss_report {
        sections.push(format_loss_report(
            &prior_response.receipt.summary_loss_report,
        ));
    }

    // Durability state
    sections.push(format!(
        "=== RECOVERY DURABILITY ===\n{:?}",
        prior_response.receipt.recovery_durability
    ));

    // Prior compacted messages (the actual content to compress)
    sections.push(format_compacted_messages(
        &prior_response.compacted_messages,
    ));

    // New messages that arrived after compaction
    if !new_messages.is_empty() {
        sections.push(format_new_messages(new_messages));
    }

    sections.join("\n\n")
}

/// Extract the structured summary from the receipt, falling back to the
/// one embedded in summary_loss_report.
fn extract_structured_summary(response: &CompactResponse) -> StructuredContextSummaryV1 {
    response
        .receipt
        .summary_loss_report
        .structured_summary
        .clone()
}

fn format_section(title: &str, items: &[String]) -> String {
    if items.is_empty() {
        format!("=== {title} ===\nNone")
    } else {
        format!("=== {title} ===\n{}", items.join("\n"))
    }
}

fn format_fallback_refs(receipt: &ContextCompactionReceiptV1) -> String {
    if receipt.exact_fallback_refs.is_empty() {
        "=== EXACT FALLBACK REFS ===\nNone".to_string()
    } else {
        let lines: Vec<String> = receipt
            .exact_fallback_refs
            .iter()
            .map(|r| {
                format!(
                    "{} | source[{}-{}] | blake3:{} | ~{}tok",
                    r.item_id, r.start_index, r.end_index, &r.content_blake3, r.approx_tokens
                )
            })
            .collect();
        format!("=== EXACT FALLBACK REFS ===\n{}", lines.join("\n"))
    }
}

fn format_exact_store(store: &[ExactStoredItemV1]) -> String {
    let lines: Vec<String> = store
        .iter()
        .map(|item| {
            format!(
                "--- {} (source {:?}) ---\n{}\n--- END ---",
                item.item_id, item.source_indices, item.content
            )
        })
        .collect();
    format!("=== EXACT STORE CONTENT ===\n{}", lines.join("\n"))
}

fn format_plan_state(plan: &PlanStateV1) -> String {
    let mut parts = Vec::new();
    if !plan.current_plan.is_empty() {
        parts.push(format!("Plan: {}", plan.current_plan));
    }
    if !plan.acceptance_gates.is_empty() {
        parts.push(format!("Gates: {}", plan.acceptance_gates.join("; ")));
    }
    if !plan.decisions.is_empty() {
        parts.push(format!("Decisions: {}", plan.decisions.join("; ")));
    }
    if !plan.unresolved_questions.is_empty() {
        parts.push(format!(
            "Unresolved: {}",
            plan.unresolved_questions.join("; ")
        ));
    }
    if !plan.active_instruction_steps.is_empty() {
        parts.push(format!(
            "Active instruction steps: {:?}",
            plan.active_instruction_steps
        ));
    }
    if parts.is_empty() {
        "=== PLAN STATE ===\n(no active plan)".to_string()
    } else {
        format!("=== PLAN STATE ===\n{}", parts.join("\n"))
    }
}

fn format_structural_floor(floor: &StructuralFloorV1) -> String {
    let mut parts = Vec::new();
    if !floor.mandatory_steps.is_empty() {
        parts.push(format!("Mandatory steps: {:?}", floor.mandatory_steps));
    }
    if !floor.mandatory_item_ids.is_empty() {
        parts.push(format!(
            "Mandatory items: {}",
            floor.mandatory_item_ids.join(", ")
        ));
    }
    if let Some(ref latest) = floor.latest_user_item_id {
        parts.push(format!("Latest user item: {}", latest));
    }
    if !floor.acceptance_gate_item_ids.is_empty() {
        parts.push(format!(
            "Gate items: {}",
            floor.acceptance_gate_item_ids.join(", ")
        ));
    }
    if parts.is_empty() {
        "=== STRUCTURAL FLOOR ===\n(no mandatory floor)".to_string()
    } else {
        format!("=== STRUCTURAL FLOOR ===\n{}", parts.join("\n"))
    }
}

fn format_step_graph(steps: &[ContextStepV1]) -> String {
    let lines: Vec<String> = steps
        .iter()
        .map(|step| {
            let flags = [
                step.has_active_instruction.then_some("instruction"),
                step.has_error.then_some("error"),
                step.is_latest_user_step.then_some("latest-user"),
            ]
            .into_iter()
            .flatten()
            .collect::<Vec<_>>()
            .join(",");
            format!(
                "  step {} [{}-{}] initiator={} flags=[{}]",
                step.step_id,
                step.start_message_index,
                step.end_message_index,
                step.initiator_role,
                flags
            )
        })
        .collect();
    format!("=== STEP GRAPH ===\n{}", lines.join("\n"))
}

fn format_loss_report(report: &SummaryLossReportV1) -> String {
    let mut parts = Vec::new();
    if !report.preserved_claims.is_empty() {
        parts.push(format!("Preserved: {}", report.preserved_claims.join("; ")));
    }
    if !report.omitted_claims.is_empty() {
        parts.push(format!("Omitted: {}", report.omitted_claims.join("; ")));
    }
    if !report.evidence_lost.is_empty() {
        parts.push(format!(
            "Evidence lost: {}",
            report.evidence_lost.join("; ")
        ));
    }
    if !report.uncertainty_introduced.is_empty() {
        parts.push(format!(
            "Uncertainty: {}",
            report.uncertainty_introduced.join("; ")
        ));
    }
    if !report.high_risk_omissions.is_empty() {
        parts.push(format!(
            "High-risk: {}",
            report.high_risk_omissions.join("; ")
        ));
    }
    if parts.is_empty() {
        "=== PRIOR LOSS REPORT ===\n(no losses recorded)".to_string()
    } else {
        format!("=== PRIOR LOSS REPORT ===\n{}", parts.join("\n"))
    }
}

fn format_compacted_messages(messages: &[Message]) -> String {
    let lines: Vec<String> = messages
        .iter()
        .map(|m| {
            let truncated = truncate_content(&m.content, 500);
            format!("[{}] {}", m.role, truncated)
        })
        .collect();
    format!("=== COMPACTED MESSAGES ===\n{}", lines.join("\n"))
}

fn format_new_messages(messages: &[Message]) -> String {
    let lines: Vec<String> = messages
        .iter()
        .map(|m| {
            let truncated = truncate_content(&m.content, 500);
            format!("[{}] {}", m.role, truncated)
        })
        .collect();
    format!(
        "=== NEW MESSAGES (after compaction) ===\n{}",
        lines.join("\n")
    )
}

fn truncate_content(content: &str, max_chars: usize) -> String {
    let mut chars = content.chars();
    let truncated: String = chars.by_ref().take(max_chars).collect();
    if chars.next().is_some() {
        format!("{truncated}...[truncated]")
    } else {
        truncated
    }
}

/// Parse the LLM's structured output back into a `StructuredContextSummaryV1`
/// and a loss report. This allows the compactor to ingest the LLM's response
/// and feed it into the next compaction cycle.
#[derive(Debug, Clone, PartialEq, Eq, Default, serde::Serialize, serde::Deserialize)]
pub struct ParsedSummaryV1 {
    pub active_task: Option<String>,
    pub acceptance_gates: Vec<String>,
    pub files: Vec<String>,
    pub commands: Vec<String>,
    pub errors: Vec<String>,
    pub decisions: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub exact_fallback_refs: Vec<String>,
    pub summary_losses: Vec<String>,
    pub prior_context_summary: String,
}

/// Parse the LLM's structured output into a `ParsedSummaryV1`.
///
/// This is a simple section parser that looks for `=== SECTION ===` headers
/// and extracts the content between them. It is intentionally lenient:
/// missing sections default to empty, and extra whitespace is trimmed.
pub fn parse_summary_output(output: &str) -> ParsedSummaryV1 {
    let mut result = ParsedSummaryV1::default();
    let mut current_section: Option<String> = None;
    let mut current_lines: Vec<String> = Vec::new();

    for line in output.lines() {
        if line.starts_with("=== ") && line.ends_with(" ===") {
            if let Some(section) = &current_section {
                assign_section(&mut result, section, &current_lines);
            }
            let header = &line[4..line.len() - 4];
            current_section = Some(header.to_string());
            current_lines.clear();
        } else if let Some(_section) = &current_section {
            current_lines.push(line.to_string());
        }
    }
    if let Some(section) = &current_section {
        assign_section(&mut result, section, &current_lines);
    }

    result
}

fn assign_section(result: &mut ParsedSummaryV1, section: &str, lines: &[String]) {
    let content: Vec<String> = lines
        .iter()
        .map(|l| l.trim().to_string())
        .filter(|l| !l.is_empty() && l != "None")
        .collect();

    match section {
        "ACTIVE TASK" => {
            if !content.is_empty() {
                result.active_task = Some(content.join("\n"));
            }
        }
        "ACCEPTANCE GATES" => result.acceptance_gates = content,
        "FILES" => result.files = content,
        "COMMANDS" => result.commands = content,
        "ERRORS" => result.errors = content,
        "DECISIONS" => result.decisions = content,
        "UNRESOLVED QUESTIONS" => result.unresolved_questions = content,
        "EXACT FALLBACK REFS" => {
            // Extract just the ref IDs (first token before |)
            result.exact_fallback_refs = content
                .iter()
                .filter_map(|line| line.split('|').next().map(|s| s.trim().to_string()))
                .filter(|s| !s.is_empty())
                .collect();
        }
        "SUMMARY LOSSES" => result.summary_losses = content,
        "PRIOR CONTEXT SUMMARY" => {
            result.prior_context_summary = lines
                .iter()
                .map(|l| l.as_str())
                .collect::<Vec<_>>()
                .join("\n")
                .trim()
                .to_string();
        }
        _ => {}
    }
}

/// Estimate the approximate token count of the rendered prompt.
/// Uses the same whitespace-split heuristic as the compactor.
pub fn estimate_prompt_tokens(prompt: &RenderedPromptV1) -> usize {
    let total = format!("{} {}", prompt.system, prompt.user);
    total.split_whitespace().count().max(1)
}

/// Check whether a rendered prompt fits within the configured token budget.
pub fn fits_budget(prompt: &RenderedPromptV1, config: &PromptConfigV1) -> bool {
    estimate_prompt_tokens(prompt) <= config.max_prompt_tokens
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{AllocatorMode, CompactRequest, CompactionPolicy};

    fn make_test_response() -> CompactResponse {
        let request = CompactRequest {
            session_id: "test-session".to_string(),
            messages: vec![
                Message {
                    id: None,
                    role: "system".to_string(),
                    content: "You are a helpful assistant.".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
                Message {
                    id: None,
                    role: "user".to_string(),
                    content: "Fix the login bug in auth.rs".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
                Message {
                    id: None,
                    role: "assistant".to_string(),
                    content: "I'll fix the login bug. The issue is in the token validation logic."
                        .to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
                Message {
                    id: None,
                    role: "user".to_string(),
                    content: "Also add a test for the new validation".to_string(),
                    name: None,
                    metadata: std::collections::BTreeMap::new(),
                },
            ],
            policy: CompactionPolicy {
                target_tokens: 100,
                allocator: AllocatorMode::UtilityV2.as_str().to_string(),
                budget_mode: crate::BudgetMode::SoftWarn,
                ..Default::default()
            },
            focus: Some("login bug fix".to_string()),
        };
        crate::compact_context(request).unwrap()
    }

    #[test]
    fn renders_system_prompt_with_all_rules() {
        let system = render_system_prompt();
        assert!(system.contains("Bitemporal Fact Preservation"));
        assert!(system.contains("Fallback Ref Integrity"));
        assert!(system.contains("Living Question Queue"));
        assert!(system.contains("Category Compression"));
        assert!(system.contains("Explicit Loss Tracking"));
        assert!(system.contains("Step Graph Lineage"));
        assert!(system.contains("Authority Anchoring"));
        assert!(system.contains("=== ACTIVE TASK ==="));
        assert!(system.contains("=== EXACT FALLBACK REFS ==="));
        assert!(system.contains("=== SUMMARY LOSSES ==="));
    }

    #[test]
    fn renders_user_prompt_with_sections() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let user = render_user_prompt(&response, &[], &config);
        assert!(user.contains("=== CURRENT FOCUS ==="));
        assert!(user.contains("=== ACTIVE TASK ==="));
        assert!(user.contains("=== ACCEPTANCE GATES ==="));
        assert!(user.contains("=== FILES ==="));
        assert!(user.contains("=== EXACT FALLBACK REFS ==="));
        assert!(user.contains("=== PLAN STATE ==="));
        assert!(user.contains("=== STRUCTURAL FLOOR ==="));
        assert!(user.contains("=== RECOVERY DURABILITY ==="));
        assert!(user.contains("=== COMPACTED MESSAGES ==="));
    }

    #[test]
    fn includes_new_messages_section() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let new_messages = vec![Message {
            id: None,
            role: "user".to_string(),
            content: "Did you fix the bug?".to_string(),
            name: None,
            metadata: std::collections::BTreeMap::new(),
        }];
        let user = render_user_prompt(&response, &new_messages, &config);
        assert!(user.contains("=== NEW MESSAGES (after compaction) ==="));
        assert!(user.contains("Did you fix the bug?"));
    }

    #[test]
    fn includes_exact_store_content_when_enabled() {
        let mut response = make_test_response();
        response.exact_store.push(crate::ExactStoredItemV1 {
            item_id: "exact-1".to_string(),
            source_indices: vec![1],
            content: "Important exact content here".to_string(),
            content_blake3: "abc123".to_string(),
        });
        let config = PromptConfigV1 {
            include_exact_content: true,
            ..Default::default()
        };
        let user = render_user_prompt(&response, &[], &config);
        assert!(user.contains("=== EXACT STORE CONTENT ==="));
        assert!(user.contains("Important exact content here"));
    }

    #[test]
    fn excludes_exact_store_content_when_disabled() {
        let mut response = make_test_response();
        response.exact_store.push(crate::ExactStoredItemV1 {
            item_id: "exact-1".to_string(),
            source_indices: vec![1],
            content: "Important exact content here".to_string(),
            content_blake3: "abc123".to_string(),
        });
        let config = PromptConfigV1::default();
        let user = render_user_prompt(&response, &[], &config);
        assert!(!user.contains("=== EXACT STORE CONTENT ==="));
    }

    #[test]
    fn includes_step_graph_when_enabled() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let user = render_user_prompt(&response, &[], &config);
        // Step graph may or may not have steps depending on compaction
        if !response.context_steps.is_empty() {
            assert!(user.contains("=== STEP GRAPH ==="));
        }
    }

    #[test]
    fn includes_loss_report_when_enabled() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let user = render_user_prompt(&response, &[], &config);
        assert!(user.contains("=== PRIOR LOSS REPORT ==="));
    }

    #[test]
    fn full_render_roundtrip() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let prompt = render_summary_prompt(&response, &[], &config);
        assert!(!prompt.system.is_empty());
        assert!(!prompt.user.is_empty());
        assert!(estimate_prompt_tokens(&prompt) > 0);
        assert!(fits_budget(&prompt, &config));
    }

    #[test]
    fn parse_summary_output_extracts_all_sections() {
        let output = r#"=== ACTIVE TASK ===
Fix the login bug in auth.rs

=== ACCEPTANCE GATES ===
Login validation passes
Token expiry handled

=== FILES ===
auth.rs | modified | pending test

=== COMMANDS ===
cargo test | exit 0 | 3 passed

=== ERRORS ===
None

=== DECISIONS ===
Use JWT for token validation | security | 2026-07-11

=== UNRESOLVED QUESTIONS ===
Should we add rate limiting?

=== EXACT FALLBACK REFS ===
exact-1 | source[0-2] | blake3:abc | ~10tok
exact-2 | source[3-5] | blake3:def | ~20tok

=== SUMMARY LOSSES ===
Dropped: exact API endpoint URL from message 3

=== PRIOR CONTEXT SUMMARY ===
The user asked to fix a login bug. The assistant identified the issue in token validation."#;

        let parsed = parse_summary_output(output);
        assert_eq!(
            parsed.active_task.as_deref(),
            Some("Fix the login bug in auth.rs")
        );
        assert_eq!(parsed.acceptance_gates.len(), 2);
        assert_eq!(parsed.files.len(), 1);
        assert!(parsed.files[0].contains("auth.rs"));
        assert_eq!(parsed.commands.len(), 1);
        assert_eq!(parsed.errors.len(), 0);
        assert_eq!(parsed.decisions.len(), 1);
        assert_eq!(parsed.unresolved_questions.len(), 1);
        assert_eq!(parsed.exact_fallback_refs.len(), 2);
        assert_eq!(parsed.exact_fallback_refs[0], "exact-1");
        assert_eq!(parsed.exact_fallback_refs[1], "exact-2");
        assert_eq!(parsed.summary_losses.len(), 1);
        assert!(parsed.prior_context_summary.contains("login bug"));
    }

    #[test]
    fn parse_summary_output_handles_missing_sections() {
        let output = "=== ACTIVE TASK ===\nDo something\n";
        let parsed = parse_summary_output(output);
        assert_eq!(parsed.active_task.as_deref(), Some("Do something"));
        assert!(parsed.acceptance_gates.is_empty());
        assert!(parsed.files.is_empty());
        assert!(parsed.exact_fallback_refs.is_empty());
    }

    #[test]
    fn parse_summary_output_handles_none_values() {
        let output = "=== ERRORS ===\nNone\n=== FILES ===\nNone\n";
        let parsed = parse_summary_output(output);
        assert!(parsed.errors.is_empty());
        assert!(parsed.files.is_empty());
    }

    #[test]
    fn truncate_content_preserves_short_content() {
        let short = "Hello world";
        assert_eq!(truncate_content(short, 500), short);
    }

    #[test]
    fn truncate_content_truncates_long_content() {
        let long = "x".repeat(600);
        let result = truncate_content(&long, 500);
        assert!(result.ends_with("...[truncated]"));
        assert!(result.len() < 600);
    }

    #[test]
    fn truncate_content_counts_unicode_scalar_characters() {
        let content = "A😀中B";
        assert_eq!(truncate_content(content, 3), "A😀中...[truncated]");
    }

    #[test]
    fn truncate_content_keeps_exact_unicode_boundary_without_marker() {
        let content = "😀中";
        assert_eq!(truncate_content(content, 2), content);
    }

    #[test]
    fn focus_override_takes_precedence() {
        let response = make_test_response();
        let config = PromptConfigV1 {
            focus_override: Some("custom focus".to_string()),
            ..Default::default()
        };
        let user = render_user_prompt(&response, &[], &config);
        assert!(user.contains("custom focus"));
    }

    #[test]
    fn estimate_prompt_tokens_positive() {
        let response = make_test_response();
        let config = PromptConfigV1::default();
        let prompt = render_summary_prompt(&response, &[], &config);
        assert!(estimate_prompt_tokens(&prompt) > 100); // system prompt alone is substantial
    }
}
