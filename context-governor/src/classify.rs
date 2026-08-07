//! Classification types and plan detection for context-governor.
//!
//! Extracted from `lib.rs` to reduce the core monolith. Contains the
//! provider-neutral data types used for classifying context items, building
//! conversation steps, and extracting plan state, plus the `detect_plan_content`
//! heuristic. These are pure data types and a pure function — they depend only
//! on serde and std collections, not on the compaction engine internals.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};

use crate::{
    compact_preview, contains_any, contains_path_signal, count_tokens_text, detect_content_kind,
    hash_text, is_aggressive_allocator, CompactionPolicy, Message,
};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ItemType {
    LatestUserMessage,
    ActiveInstruction,
    AcceptanceGate,
    ToolCall,
    ToolResult,
    ErrorOutput,
    FilePathContext,
    Decision,
    UnresolvedQuestion,
    SourceEvidence,
    DurableFactCandidate,
    ProjectStateCandidate,
    ArtifactBoilerplate,
    StalePlan,
    DuplicateContext,
    LowRiskNarrative,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthorityClass {
    MustPreserveExact,
    EvidenceCritical,
    ActiveTask,
    VerifiedToolReceipt,
    DurableMemoryCandidate,
    #[default]
    SummaryOk,
    ArchiveOk,
    Discardable,
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum PreservationPolicy {
    KeepVerbatim,
    #[default]
    ExtractiveSummary,
    AbstractiveSummary,
    SemanticMemoryArchive,
    ReceiptOnly,
    OmitDuplicate,
    Quarantine,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentKind {
    PlainText,
    Json,
    Diff,
    Rust,
    Markdown,
    CargoOutput,
    ShellLog,
    SearchResults,
    #[default]
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContextItemV1 {
    pub schema: String,
    pub item_id: String,
    pub session_id: String,
    pub start_index: usize,
    pub end_index: usize,
    pub role_set: Vec<String>,
    pub char_count: usize,
    pub approx_tokens: usize,
    pub content_blake3: String,
    #[serde(default)]
    pub content_kind: ContentKind,
    pub item_type: ItemType,
    pub authority_class: AuthorityClass,
    pub preservation_policy: PreservationPolicy,
    pub risk_reasons: Vec<String>,
    pub source_message_ids: Vec<String>,
    pub priority_score: i32,
}

/// A provider-neutral structured content part. Preserves provider-native
/// unknown fields through metadata rather than dropping them.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StructuredContentPartV1 {
    /// The text content of this part, if any.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub text: String,
    /// The kind of content part (text, tool_call, tool_result, image, etc.).
    #[serde(default, rename = "part_kind")]
    pub part_kind: ContentPartKind,
    /// Provider-native tool call ID, if this part is a tool call or result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// Provider-native tool name, if this part is a tool call.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_name: Option<String>,
    /// Provider-native arguments JSON, if this part is a tool call.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool_arguments_json: String,
    /// Provider-native result content, if this part is a tool result.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub tool_result_content: String,
    /// Provider-native exit code or status, if available.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_exit_code: Option<i32>,
    /// Unknown provider-native fields preserved verbatim.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub provider_extras: BTreeMap<String, Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ContentPartKind {
    Text,
    ToolCall,
    ToolResult,
    Image,
    Audio,
    #[default]
    Unknown,
}

/// Links a tool call to its result within a step.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ToolCallLinkV1 {
    pub tool_call_id: String,
    pub tool_name: String,
    /// Index into the step's content parts for the call.
    pub call_part_index: usize,
    /// Index into the step's content parts for the result, if resolved.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result_part_index: Option<usize>,
    /// Whether the result has been received.
    #[serde(default)]
    pub result_received: bool,
}

/// A provider-neutral step in a conversation transcript.
/// Groups a user intent, assistant action/tool calls, tool results, and state deltas.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ContextStepV1 {
    /// Unique step ID.
    pub step_id: String,
    /// Index of the first message in this step.
    pub start_message_index: usize,
    /// Index after the last message in this step.
    pub end_message_index: usize,
    /// The role that initiated this step (usually "user").
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub initiator_role: String,
    /// Structured content parts extracted from the messages in this step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub content_parts: Vec<StructuredContentPartV1>,
    /// Tool call links within this step.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_call_links: Vec<ToolCallLinkV1>,
    /// Whether this step contains an active instruction or acceptance gate.
    #[serde(default)]
    pub has_active_instruction: bool,
    /// Whether this step contains an error or failure.
    #[serde(default)]
    pub has_error: bool,
    /// Whether this step is the latest user turn.
    #[serde(default)]
    pub is_latest_user_step: bool,
}

/// Explicit plan state extracted from the transcript.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct PlanStateV1 {
    /// Current active plan text, if any.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub current_plan: String,
    /// Active acceptance gates.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_gates: Vec<String>,
    /// Active decisions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub decisions: Vec<String>,
    /// Unresolved questions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub unresolved_questions: Vec<String>,
    /// Step indices that contain active instructions.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub active_instruction_steps: Vec<usize>,
}

/// Monotonic authority floor — items that must never be downgraded.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StructuralFloorV1 {
    /// Step indices containing must-preserve-exact content.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mandatory_steps: Vec<usize>,
    /// Item IDs that form the authority floor.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub mandatory_item_ids: Vec<String>,
    /// The latest user message item ID.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub latest_user_item_id: Option<String>,
    /// Active acceptance gate item IDs.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub acceptance_gate_item_ids: Vec<String>,
}

/// Detect plan-like structures in message content.
/// Returns true if the content contains numbered lists, phase markers,
/// checklist items, or explicit TODO/goal patterns that should survive compaction.
pub fn detect_plan_content(content: &str) -> bool {
    // Numbered step density: 3+ numbered lines
    let numbered_lines = content
        .lines()
        .filter(|line| {
            let trimmed = line.trim();
            trimmed
                .chars()
                .next()
                .map(|c| c.is_ascii_digit())
                .unwrap_or(false)
                && trimmed.chars().find(|c| !c.is_ascii_digit()) == Some('.')
        })
        .count();
    if numbered_lines >= 3 {
        return true;
    }
    // Checklist markers
    if content.contains("[ ]") || content.contains("[x]") || content.contains("[X]") {
        return true;
    }
    // Explicit plan language
    let lower = content.to_lowercase();
    if lower.contains("todo")
        || lower.contains("action item")
        || lower.contains("acceptance criteria")
        || lower.contains("implementation plan")
        || lower.contains("sprint")
        || lower.contains("milestone")
        || lower.contains("step ")
        || lower.contains("phase ")
    {
        return true;
    }
    false
}

/// Build provider-neutral steps from messages.
/// Groups consecutive messages into intent-action-result steps.
pub fn build_context_steps(messages: &[Message]) -> Vec<ContextStepV1> {
    let mut steps = Vec::new();
    let mut current_start = 0usize;
    let mut initiator_role = String::new();

    for (index, message) in messages.iter().enumerate() {
        let role = message.role.as_str();
        if role == "user" && index > current_start {
            steps.push(build_step(messages, current_start, index, &initiator_role));
            current_start = index;
        }
        if index == 0 || role == "user" {
            initiator_role = role.to_string();
        }
    }
    if current_start < messages.len() {
        steps.push(build_step(
            messages,
            current_start,
            messages.len(),
            &initiator_role,
        ));
    }

    // Mark the latest user step
    if let Some(last_step) = steps.last_mut() {
        last_step.is_latest_user_step = true;
    }

    steps
}

fn build_step(
    messages: &[Message],
    start: usize,
    end: usize,
    initiator_role: &str,
) -> ContextStepV1 {
    let mut content_parts = Vec::new();
    let tool_call_links = Vec::new();
    let mut has_active_instruction = false;
    let mut has_error = false;

    for message in messages[start..end].iter() {
        let content_lower = message.content.to_lowercase();
        if content_lower.contains("acceptance gate")
            || content_lower.contains("must pass")
            || content_lower.contains("must remain")
        {
            has_active_instruction = true;
        }
        if content_lower.contains("error")
            || content_lower.contains("traceback")
            || content_lower.contains("failed")
        {
            has_error = true;
        }

        if message.role == "tool" || message.role == "function" {
            content_parts.push(StructuredContentPartV1 {
                text: message.content.clone(),
                part_kind: ContentPartKind::ToolResult,
                tool_call_id: message
                    .metadata
                    .get("tool_call_id")
                    .and_then(|v| v.as_str())
                    .map(String::from),
                tool_result_content: message.content.clone(),
                tool_exit_code: message
                    .metadata
                    .get("exit_code")
                    .and_then(|v| v.as_i64())
                    .map(|i| i as i32),
                ..Default::default()
            });
        } else if message.role == "assistant" {
            // Check if content looks like a tool call (JSON with function/arguments)
            let trimmed = message.content.trim();
            if trimmed.starts_with('{')
                && (trimmed.contains("\"function\"")
                    || trimmed.contains("\"arguments\"")
                    || trimmed.contains("\"tool\""))
            {
                content_parts.push(StructuredContentPartV1 {
                    text: message.content.clone(),
                    part_kind: ContentPartKind::ToolCall,
                    tool_name: message
                        .metadata
                        .get("tool_name")
                        .and_then(|v| v.as_str())
                        .map(String::from),
                    tool_arguments_json: message.content.clone(),
                    ..Default::default()
                });
            } else {
                content_parts.push(StructuredContentPartV1 {
                    text: message.content.clone(),
                    part_kind: ContentPartKind::Text,
                    ..Default::default()
                });
            }
        } else {
            content_parts.push(StructuredContentPartV1 {
                text: message.content.clone(),
                part_kind: ContentPartKind::Text,
                ..Default::default()
            });
        }
    }

    ContextStepV1 {
        step_id: format!("step_{start}_{end}"),
        start_message_index: start,
        end_message_index: end,
        initiator_role: initiator_role.to_string(),
        content_parts,
        tool_call_links,
        has_active_instruction,
        has_error,
        is_latest_user_step: false,
    }
}

/// Extract explicit plan state from context steps.
pub fn extract_plan_state(steps: &[ContextStepV1], messages: &[Message]) -> PlanStateV1 {
    let _ = messages;
    let mut plan_state = PlanStateV1::default();

    for (step_idx, step) in steps.iter().enumerate() {
        if step.has_active_instruction {
            plan_state.active_instruction_steps.push(step_idx);
        }
        for part in &step.content_parts {
            let lower = part.text.to_lowercase();
            if lower.contains("acceptance gate:")
                || lower.contains("must pass")
                || lower.contains("must remain")
            {
                if let Some(extracted) = extract_line_after(&part.text, "acceptance gate:") {
                    plan_state.acceptance_gates.push(extracted);
                }
            }
            if lower.contains("decision:") || lower.contains("decided") {
                if let Some(extracted) = extract_line_after(&part.text, "decision:") {
                    plan_state.decisions.push(extracted);
                }
            }
            if lower.contains("unresolved question") || lower.contains('?') {
                plan_state
                    .unresolved_questions
                    .push(compact_preview(&part.text, 240));
            }
            if lower.contains("plan:") || lower.contains("phase 1") || lower.contains("phase 2") {
                plan_state.current_plan = compact_preview(&part.text, 500);
            }
        }
    }

    plan_state
}

fn extract_line_after(text: &str, marker: &str) -> Option<String> {
    let lower = text.to_lowercase();
    let pos = lower.find(marker)?;
    let after = &text[pos + marker.len()..];
    let line_end = after.find('\n').unwrap_or(after.len());
    Some(after[..line_end].trim().to_string())
}

/// Build the structural floor from context items.
pub fn build_structural_floor(items: &[ContextItemV1]) -> StructuralFloorV1 {
    let mut floor = StructuralFloorV1::default();

    for item in items.iter() {
        match item.authority_class {
            AuthorityClass::MustPreserveExact | AuthorityClass::ActiveTask => {
                floor.mandatory_item_ids.push(item.item_id.clone());
            }
            _ => {}
        }
        if matches!(item.item_type, ItemType::LatestUserMessage) {
            floor.latest_user_item_id = Some(item.item_id.clone());
        }
        if matches!(item.item_type, ItemType::AcceptanceGate) {
            floor.acceptance_gate_item_ids.push(item.item_id.clone());
        }
    }

    floor
}

pub(crate) fn classify_messages(
    session_id: &str,
    messages: &[Message],
    policy: &CompactionPolicy,
) -> Vec<ContextItemV1> {
    let latest_user = messages
        .iter()
        .rposition(|m| m.role == "user")
        .unwrap_or(messages.len() - 1);
    let mut seen_hashes = BTreeSet::new();
    messages
        .iter()
        .enumerate()
        .map(|(idx, msg)| {
            let h = hash_text(&msg.content);
            let duplicate = !seen_hashes.insert(h.clone());
            classify_message(session_id, idx, latest_user, msg, h, duplicate, policy)
        })
        .collect()
}

fn classify_message(
    session_id: &str,
    idx: usize,
    latest_user: usize,
    msg: &Message,
    content_hash: String,
    duplicate: bool,
    policy: &CompactionPolicy,
) -> ContextItemV1 {
    let content_l = msg.content.to_lowercase();
    let aggressive = is_aggressive_allocator(policy);
    let counted_tokens = count_tokens_text(&msg.content, policy);
    let msg_chars = msg.content.chars().count();
    let long_message = msg_chars > 600;
    let mut item_type = ItemType::LowRiskNarrative;
    let mut authority = AuthorityClass::SummaryOk;
    let mut policy = PreservationPolicy::ExtractiveSummary;
    let mut reasons = Vec::new();

    // Plan detection: structured plans survive compaction cycles.
    let has_plan = detect_plan_content(&msg.content);

    if msg.role == "system" || msg.role == "developer" {
        item_type = ItemType::ActiveInstruction;
        authority = AuthorityClass::MustPreserveExact;
        policy = PreservationPolicy::KeepVerbatim;
        reasons.push("system-or-developer-constraint".to_string());
    } else if idx == latest_user && msg.role == "user" {
        item_type = ItemType::LatestUserMessage;
        authority = AuthorityClass::ActiveTask;
        policy = PreservationPolicy::KeepVerbatim;
        reasons.push("latest-user-message".to_string());
    } else if contains_any(
        &content_l,
        &[
            "acceptance gate",
            "must pass",
            "required",
            "requirement",
            "do not",
            "never ",
        ],
    ) {
        item_type = ItemType::AcceptanceGate;
        authority = AuthorityClass::MustPreserveExact;
        policy = if aggressive && long_message {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("acceptance-or-instruction".to_string());
    } else if has_plan {
        item_type = ItemType::AcceptanceGate;
        authority = AuthorityClass::MustPreserveExact;
        policy = if aggressive && long_message {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("plan-content-detected".to_string());
    } else if contains_any(
        &content_l,
        &[
            "error:",
            "error[",
            "traceback",
            "panic",
            "failed",
            "compilation failed",
            "exit_code\":1",
            "exit code 1",
        ],
    ) {
        item_type = ItemType::ErrorOutput;
        authority = AuthorityClass::EvidenceCritical;
        policy = if msg.role == "tool" && msg.content.chars().count() > 600 {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("error-output".to_string());
    } else if contains_path_signal(&msg.content) {
        item_type = ItemType::FilePathContext;
        authority = AuthorityClass::EvidenceCritical;
        policy = if aggressive && long_message {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("path-signal".to_string());
    } else if msg.role == "tool" {
        item_type = ItemType::ToolResult;
        authority = AuthorityClass::VerifiedToolReceipt;
        policy = if msg.content.chars().count() > 600 {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("tool-result".to_string());
    } else if contains_any(
        &content_l,
        &["decided", "decision", "architecture", "verdict"],
    ) {
        item_type = ItemType::Decision;
        authority = AuthorityClass::DurableMemoryCandidate;
        policy = PreservationPolicy::SemanticMemoryArchive;
        reasons.push("decision-signal".to_string());
    } else if contains_any(&content_l, &["source:", "evidence", "verified", "receipt"]) {
        item_type = ItemType::SourceEvidence;
        authority = AuthorityClass::EvidenceCritical;
        policy = if aggressive && long_message {
            PreservationPolicy::ReceiptOnly
        } else {
            PreservationPolicy::KeepVerbatim
        };
        reasons.push("evidence-signal".to_string());
    }

    let authority_is_monotonic = matches!(
        authority,
        AuthorityClass::MustPreserveExact
            | AuthorityClass::EvidenceCritical
            | AuthorityClass::ActiveTask
            | AuthorityClass::VerifiedToolReceipt
    ) || matches!(policy, PreservationPolicy::KeepVerbatim);
    if duplicate && !authority_is_monotonic {
        item_type = ItemType::DuplicateContext;
        authority = AuthorityClass::Discardable;
        policy = PreservationPolicy::OmitDuplicate;
        reasons.push("duplicate-content".to_string());
    }

    if contains_any(
        &content_l,
        &[
            "likely",
            "potentially",
            "would likely",
            "may indicate",
            "logically connect",
        ],
    ) {
        if matches!(
            authority,
            AuthorityClass::SummaryOk | AuthorityClass::ArchiveOk
        ) && matches!(
            policy,
            PreservationPolicy::ExtractiveSummary | PreservationPolicy::AbstractiveSummary
        ) {
            item_type = ItemType::ArtifactBoilerplate;
            authority = AuthorityClass::Quarantine;
            policy = PreservationPolicy::Quarantine;
            reasons.push("speculative-language-quarantined".to_string());
        } else {
            reasons.push("speculative-language-non-downgrading-flag".to_string());
        }
    }

    ContextItemV1 {
        schema: "ContextItemV1".to_string(),
        item_id: format!("ctxi_{idx:04}_{}", &content_hash[..12]),
        session_id: session_id.to_string(),
        start_index: idx,
        end_index: idx,
        role_set: vec![msg.role.clone()],
        char_count: msg.content.chars().count(),
        approx_tokens: counted_tokens,
        content_blake3: content_hash,
        content_kind: detect_content_kind(&msg.role, &msg.content),
        item_type,
        authority_class: authority,
        preservation_policy: policy,
        risk_reasons: reasons,
        source_message_ids: msg.id.clone().into_iter().collect(),
        priority_score: 0,
    }
}
