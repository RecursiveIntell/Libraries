//! Classification types and plan detection for context-governor.
//!
//! Extracted from `lib.rs` to reduce the core monolith. Contains the
//! provider-neutral data types used for classifying context items, building
//! conversation steps, and extracting plan state, plus the `detect_plan_content`
//! heuristic. These are pure data types and a pure function — they depend only
//! on serde and std collections, not on the compaction engine internals.

use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::BTreeMap;

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
