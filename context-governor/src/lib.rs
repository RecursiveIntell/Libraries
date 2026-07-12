//! Crate-agnostic governed context compaction for AI agents.
//!
//! `context-governor` does not call an LLM and does not pretend to extend a
//! provider's hard context window. It allocates a limited prompt budget across
//! agent conversation messages, preserves high-risk context verbatim, demotes or
//! summarizes low-risk context, emits receipts, and keeps exact fallback ranges
//! for later expansion.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::collections::{BTreeMap, BTreeSet};
use thiserror::Error;
use uuid::Uuid;

pub mod high_roi;
pub use high_roi::*;

const CHARS_PER_TOKEN: usize = 4;
const SUMMARY_PREFIX: &str = "[CONTEXT COMPACTION — RECEIPT-BACKED REFERENCE ONLY]";

#[derive(Debug, Error)]
pub enum ContextGovernorError {
    #[error("cannot compact an empty message list")]
    EmptyMessages,
    #[error("serialization failed: {0}")]
    Serialization(#[from] serde_json::Error),
    #[error("io failed: {0}")]
    Io(#[from] std::io::Error),
    #[error("receipt not found: {0}")]
    ReceiptNotFound(String),
    #[error("context budget exceeded: target {target} tokens, actual {actual} tokens")]
    BudgetExceeded { target: usize, actual: usize },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct Message {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub id: Option<String>,
    pub role: String,
    pub content: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub metadata: BTreeMap<String, Value>,
}

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
#[serde(rename_all = "snake_case")]
pub enum BudgetMode {
    #[default]
    SoftWarn,
    HardCascade,
    FailClosed,
    /// Strict budget contract: if the compacted transcript exceeds
    /// `target_tokens`, compaction fails with `BudgetExceeded` instead of
    /// returning over-budget content with a warning.
    HardLimit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum TokenCounterKind {
    #[default]
    ApproxChars,
    ApproxWords,
    ProviderChatApprox,
    /// OpenAI cl100k-compatible tokenizer surface. In the default build this
    /// deliberately falls back to the provider chat approximation and emits a
    /// warning instead of silently pretending native tokenization happened.
    TiktokenCl100k,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum AllocatorMode {
    #[default]
    DeterministicV1,
    AggressiveV1,
    UtilityV2,
}

impl AllocatorMode {
    fn as_str(&self) -> &'static str {
        match self {
            Self::DeterministicV1 => "deterministic_v1",
            Self::AggressiveV1 => "aggressive_v1",
            Self::UtilityV2 => "utility_v2",
        }
    }
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

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextAllocationPlanV1 {
    pub schema: String,
    pub plan_id: String,
    pub session_id: String,
    pub created_utc: DateTime<Utc>,
    pub context_budget_tokens: usize,
    pub target_output_tokens: usize,
    pub allocator: String,
    pub items: Vec<ContextItemV1>,
    pub kept_item_ids: Vec<String>,
    pub summarized_item_ids: Vec<String>,
    pub archived_item_ids: Vec<String>,
    pub omitted_item_ids: Vec<String>,
    pub quarantined_item_ids: Vec<String>,
    /// Optional per-item audit trail for allocators that use scored selection.
    /// Older plan payloads intentionally deserialize without it.
    #[serde(default, skip_serializing_if = "BTreeMap::is_empty")]
    pub selection_evidence: BTreeMap<String, UtilitySelectionEvidenceV1>,
    /// Structural work counters for the compaction hot path. They count set/map
    /// membership operations rather than elapsed time, so regression tests do
    /// not depend on machine-specific timing.
    #[serde(default, skip_serializing_if = "HotPathOperationCountsV1::is_empty")]
    pub hot_path_operation_counts: HotPathOperationCountsV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct UtilitySelectionEvidenceV1 {
    pub authority: i32,
    pub focus_overlap: i32,
    pub recency: i32,
    pub unresolved_signal: i32,
    pub novelty: i32,
    pub redundancy: i32,
    pub recoverability: i32,
    pub utility: i32,
    #[serde(default)]
    pub selected: bool,
    #[serde(default)]
    pub mandatory: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub selected_utility_per_token_milli: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct HotPathOperationCountsV1 {
    pub disposition_membership_checks: usize,
    pub fallback_ref_lookups: usize,
    pub summary_membership_checks: usize,
    pub emitted_message_membership_checks: usize,
    pub unique_value_membership_checks: usize,
}

impl HotPathOperationCountsV1 {
    pub fn total_membership_operations(&self) -> usize {
        self.disposition_membership_checks
            + self.fallback_ref_lookups
            + self.summary_membership_checks
            + self.emitted_message_membership_checks
            + self.unique_value_membership_checks
    }

    fn is_empty(&self) -> bool {
        self.total_membership_operations() == 0
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExactFallbackRefV1 {
    pub item_id: String,
    pub start_index: usize,
    pub end_index: usize,
    pub content_blake3: String,
    pub approx_tokens: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct ExactStoredItemV1 {
    pub item_id: String,
    pub source_indices: Vec<usize>,
    pub content: String,
    pub content_blake3: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct StructuredContextSummaryV1 {
    pub active_task: Option<String>,
    pub acceptance_gates: Vec<String>,
    pub files: Vec<String>,
    pub commands: Vec<String>,
    pub errors: Vec<String>,
    pub decisions: Vec<String>,
    pub unresolved_questions: Vec<String>,
    pub fallback_item_ids: Vec<String>,
}

/// Describes where exact fallback can currently be recovered from. `compact_context`
/// only creates in-process data; it does not claim that data has been persisted.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum ExactRecoveryStateV1 {
    #[default]
    Unavailable,
    InResponse,
    Persisted,
}

/// Durability of exact recovery for a compaction receipt. This replaces the
/// previous boolean `exact_recovery_available` flag with an explicit state
/// machine describing where the exact fallback content currently lives.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum RecoveryDurabilityV1 {
    /// Exact fallback content is carried in the compaction response but has
    /// not yet been persisted.
    InResponse,
    /// Compaction completed but persistence has not been attempted yet.
    #[default]
    PersistencePending,
    /// Exact fallback content has been durably persisted by a store.
    Persisted,
    /// Exact fallback content was handed off to an external archive.
    ExternallyArchived,
    /// Exact fallback content was pruned and is no longer recoverable.
    Pruned,
    /// No exact fallback content exists for this receipt.
    Unavailable,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct SummaryLossReportV1 {
    pub schema: String,
    pub preserved_claims: Vec<String>,
    pub omitted_claims: Vec<String>,
    pub evidence_lost: Vec<String>,
    pub uncertainty_introduced: Vec<String>,
    #[serde(default)]
    pub exact_recovery_state: ExactRecoveryStateV1,
    pub high_risk_omissions: Vec<String>,
    #[serde(default)]
    pub structured_summary: StructuredContextSummaryV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextCompactionReceiptV1 {
    pub schema: String,
    pub receipt_id: String,
    pub session_id: String,
    pub parent_session_id: Option<String>,
    pub created_utc: DateTime<Utc>,
    pub engine: String,
    pub engine_version: String,
    pub original_message_count: usize,
    pub compacted_message_count: usize,
    pub original_approx_tokens: usize,
    pub compacted_approx_tokens: usize,
    pub token_savings_estimate: isize,
    pub token_counter: TokenCounterKind,
    pub original_transcript_blake3: String,
    pub compacted_transcript_blake3: String,
    pub allocation_plan_id: String,
    pub semantic_memory_fact_ids: Vec<String>,
    pub semantic_memory_document_ids: Vec<String>,
    pub exact_fallback_refs: Vec<ExactFallbackRefV1>,
    pub summary_loss_report: SummaryLossReportV1,
    pub warnings: Vec<String>,
    /// Durability state of the exact fallback content. `compact_context` only
    /// produces in-process data, so it defaults to `InResponse`; a store that
    /// persists the receipt updates this to `Persisted`.
    #[serde(default)]
    pub recovery_durability: RecoveryDurabilityV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactRequest {
    pub session_id: String,
    pub messages: Vec<Message>,
    #[serde(default)]
    pub policy: CompactionPolicy,
    #[serde(default)]
    pub focus: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactionPolicy {
    pub target_tokens: usize,
    pub protect_first_n: usize,
    pub protect_last_n: usize,
    pub summary_max_chars: usize,
    pub allocator: String,
    #[serde(default)]
    pub semantic_memory_enabled: bool,
    #[serde(default)]
    pub archive_memory_enabled: bool,
    #[serde(default)]
    pub budget_mode: BudgetMode,
    #[serde(default)]
    pub token_counter: TokenCounterKind,
}

impl Default for CompactionPolicy {
    fn default() -> Self {
        Self {
            target_tokens: 8_000,
            protect_first_n: 3,
            protect_last_n: 8,
            summary_max_chars: 8_000,
            allocator: "deterministic_v1".to_string(),
            semantic_memory_enabled: false,
            archive_memory_enabled: false,
            budget_mode: BudgetMode::SoftWarn,
            token_counter: TokenCounterKind::ApproxChars,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompactResponse {
    pub receipt: ContextCompactionReceiptV1,
    pub allocation_plan: ContextAllocationPlanV1,
    pub compacted_messages: Vec<Message>,
    pub exact_store: Vec<ExactStoredItemV1>,
    /// Provider-neutral conversation steps built from the original messages.
    #[serde(default)]
    pub context_steps: Vec<ContextStepV1>,
    /// Explicit plan state extracted from the transcript.
    #[serde(default)]
    pub plan_state: PlanStateV1,
    /// Monotonic authority floor — items that must never be downgraded.
    #[serde(default)]
    pub structural_floor: StructuralFloorV1,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RecallCandidate {
    pub source: Option<String>,
    pub content: String,
    pub score: Option<f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RecallDecision {
    AdmitAuthoritative,
    AdmitBackground,
    DemoteArtifact,
    QuarantineSpeculative,
    RejectNoise,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RecallFilterResult {
    pub decision: RecallDecision,
    pub reasons: Vec<String>,
}

pub fn compact_context(request: CompactRequest) -> Result<CompactResponse, ContextGovernorError> {
    compact_context_with_memory_sink(request, None)
}

pub fn compact_context_with_memory_sink(
    request: CompactRequest,
    memory_sink: Option<&mut dyn MemorySink>,
) -> Result<CompactResponse, ContextGovernorError> {
    if request.messages.is_empty() {
        return Err(ContextGovernorError::EmptyMessages);
    }

    let original_tokens = count_tokens_messages(&request.messages, &request.policy);
    let original_hash = hash_messages(&request.messages)?;
    let mut items = classify_messages(&request.session_id, &request.messages, &request.policy);
    let allocator_mode = resolve_allocator(&request.policy);
    score_items(
        &mut items,
        request.focus.as_deref(),
        is_utility_allocator_mode(&allocator_mode).then_some(request.messages.as_slice()),
    );
    let allocator_unknown = {
        let raw = request.policy.allocator.trim().to_lowercase();
        !raw.is_empty()
            && !matches!(
                raw.as_str(),
                "deterministic"
                    | "deterministic_v1"
                    | "aggressive"
                    | "aggressive_v1"
                    | "utility_v2"
            )
    };
    let mut warning_from_allocator = Vec::new();
    if allocator_unknown {
        warning_from_allocator.push(format!(
            "allocator '{}' is unknown; normalized to '{}'",
            request.policy.allocator,
            allocator_mode.as_str(),
        ));
    }
    let context_steps = build_context_steps(&request.messages);
    let plan_state = extract_plan_state(&context_steps, &request.messages);
    let structural_floor = build_structural_floor(&items);
    let mut plan = allocate_items(
        &request.session_id,
        items,
        &request.policy,
        allocator_mode,
        request.focus.as_deref(),
        &request.messages,
    );
    let mut warnings = warning_from_allocator;
    if matches!(
        request.policy.token_counter,
        TokenCounterKind::ApproxChars
            | TokenCounterKind::ApproxWords
            | TokenCounterKind::ProviderChatApprox
    ) {
        warnings.push(
            "token_counter is approximate; provider-native token budgets may differ".to_string(),
        );
    }
    if matches!(
        request.policy.token_counter,
        TokenCounterKind::ProviderChatApprox | TokenCounterKind::TiktokenCl100k
    ) {
        warnings.push(
            "provider_chat_approx includes chat-role overhead but is not a native tokenizer"
                .to_string(),
        );
    }
    if matches!(
        request.policy.token_counter,
        TokenCounterKind::TiktokenCl100k
    ) {
        warnings.push(
            "tiktoken_cl100k requested but native tokenizer feature is not compiled; using provider_chat_approx fallback".to_string(),
        );
    }

    let (exact_store, exact_store_checks) = build_exact_store(&request.messages, &plan);
    plan.hot_path_operation_counts.disposition_membership_checks = exact_store_checks;
    let receipt_id = format!("ctxr_{}", Uuid::new_v4().simple());
    let (structured_summary, unique_value_checks) =
        build_structured_summary(&request.messages, &plan);
    plan.hot_path_operation_counts
        .unique_value_membership_checks = unique_value_checks;
    let (mut summary, summary_membership_checks) = build_summary(
        &request.messages,
        &plan,
        &request.policy,
        &structured_summary,
        &receipt_id,
    );
    plan.hot_path_operation_counts.summary_membership_checks = summary_membership_checks;
    let boundary_sources = request
        .messages
        .iter()
        .map(|message| message.content.clone())
        .collect::<Vec<_>>();
    let boundary_audit = audit_compression_boundary(&boundary_sources, &summary);
    if !boundary_audit.safe_to_reinject {
        summary = safe_recovery_summary(&receipt_id);
        warnings.push(
            "boundary audit quarantined an unsafe generated summary; only a recovery pointer was reinjected"
                .to_string(),
        );
    }
    let (mut compacted_messages, emitted_message_membership_checks) =
        assemble_compacted_messages(&request.messages, &plan, &summary, &request.policy);
    plan.hot_path_operation_counts
        .emitted_message_membership_checks = emitted_message_membership_checks;
    if matches!(request.policy.budget_mode, BudgetMode::HardCascade) {
        warnings.push("hard cascade budget mode active".to_string());
    }

    if matches!(
        request.policy.budget_mode,
        BudgetMode::HardCascade | BudgetMode::FailClosed
    ) {
        compacted_messages = enforce_budget(compacted_messages, &request.policy, &mut warnings)?;
    }

    let compacted_tokens = count_tokens_messages(&compacted_messages, &request.policy);
    if matches!(request.policy.budget_mode, BudgetMode::HardLimit)
        && compacted_tokens > request.policy.target_tokens
    {
        return Err(ContextGovernorError::BudgetExceeded {
            target: request.policy.target_tokens,
            actual: compacted_tokens,
        });
    }
    let compacted_hash = hash_messages(&compacted_messages)?;
    let item_by_id = plan
        .items
        .iter()
        .map(|item| (item.item_id.as_str(), item))
        .collect::<BTreeMap<_, _>>();
    let exact_fallback_refs = exact_store
        .iter()
        .filter_map(|stored| {
            item_by_id
                .get(stored.item_id.as_str())
                .map(|item| ExactFallbackRefV1 {
                    item_id: item.item_id.clone(),
                    start_index: item.start_index,
                    end_index: item.end_index,
                    content_blake3: item.content_blake3.clone(),
                    approx_tokens: item.approx_tokens,
                })
        })
        .collect::<Vec<_>>();
    plan.hot_path_operation_counts.fallback_ref_lookups = exact_store.len();

    if compacted_tokens > request.policy.target_tokens {
        warnings.push(format!(
            "compacted output still exceeds target budget: {} > {} tokens",
            compacted_tokens, request.policy.target_tokens
        ));
    }

    let mut receipt = ContextCompactionReceiptV1 {
        schema: "ContextCompactionReceiptV1".to_string(),
        receipt_id: receipt_id.clone(),
        session_id: request.session_id.clone(),
        parent_session_id: None,
        created_utc: Utc::now(),
        engine: "context-governor".to_string(),
        engine_version: env!("CARGO_PKG_VERSION").to_string(),
        original_message_count: request.messages.len(),
        compacted_message_count: compacted_messages.len(),
        original_approx_tokens: original_tokens,
        compacted_approx_tokens: compacted_tokens,
        token_savings_estimate: original_tokens as isize - compacted_tokens as isize,
        token_counter: request.policy.token_counter.clone(),
        original_transcript_blake3: original_hash,
        compacted_transcript_blake3: compacted_hash,
        allocation_plan_id: plan.plan_id.clone(),
        semantic_memory_fact_ids: Vec::new(),
        semantic_memory_document_ids: Vec::new(),
        exact_fallback_refs,
        summary_loss_report: build_loss_report(&plan, structured_summary, !exact_store.is_empty()),
        warnings,
        recovery_durability: RecoveryDurabilityV1::InResponse,
    };

    if request.policy.archive_memory_enabled || request.policy.semantic_memory_enabled {
        if let Some(sink) = memory_sink {
            let compact_response = CompactResponse {
                receipt: receipt.clone(),
                allocation_plan: plan.clone(),
                compacted_messages: compacted_messages.clone(),
                exact_store: exact_store.clone(),
                context_steps: context_steps.clone(),
                plan_state: plan_state.clone(),
                structural_floor: structural_floor.clone(),
            };
            let outcome = archive_response_to_memory(&compact_response, sink)?;
            receipt.semantic_memory_fact_ids = outcome.record_ids;
            if outcome.records_attempted == 0 {
                receipt.warnings.push(
                    "archive_memory_enabled true but no matching durable/archive candidates were found".to_string(),
                );
            }
            if request.policy.semantic_memory_enabled && !request.policy.archive_memory_enabled {
                receipt
                    .warnings
                    .push("semantic_memory_enabled=true but archive_memory_enabled=false; semantic IDs may be partial".to_string());
            }
        } else {
            receipt.warnings.push(
                "archive requested but no memory sink was supplied; semantic-memory IDs are intentionally empty".to_string(),
            );
        }
    }

    Ok(CompactResponse {
        receipt,
        allocation_plan: plan,
        compacted_messages,
        exact_store,
        context_steps,
        plan_state,
        structural_floor,
    })
}

pub fn filter_recall_candidate(candidate: &RecallCandidate, query: &str) -> RecallFilterResult {
    let content_l = candidate.content.to_lowercase();
    let source_l = candidate.source.clone().unwrap_or_default().to_lowercase();
    let query_l = query.to_lowercase();
    let mut reasons = Vec::new();

    let speculative_markers = [
        "likely",
        "would likely",
        "potentially",
        "may indicate",
        "could imply",
        "logically connect",
        "intended to",
        "seems to",
    ];
    if speculative_markers.iter().any(|m| content_l.contains(m)) {
        reasons.push("speculative-language-without-evidence".to_string());
        return RecallFilterResult {
            decision: RecallDecision::QuarantineSpeculative,
            reasons,
        };
    }

    let artifact_markers = [
        "_spec_pack.zip",
        "_bundle.zip",
        "pack_manifest",
        "mvp_implementation_plan",
        "rollback_and_quarantine_plan",
        "master_codex_implementation_prompt",
        "migration from prior drafts",
        "profile_i",
    ];
    let is_artifact = artifact_markers
        .iter()
        .any(|m| content_l.contains(m) || source_l.contains(m));
    if is_artifact {
        let explicit = [
            "aicc",
            "context pack",
            "pack manifest",
            "zpy",
            "gpt-pro-synthesis",
        ]
        .iter()
        .any(|m| query_l.contains(m));
        reasons.push("artifact-pack-marker".to_string());
        return RecallFilterResult {
            decision: if explicit {
                RecallDecision::AdmitBackground
            } else {
                RecallDecision::RejectNoise
            },
            reasons,
        };
    }

    if source_l.contains("tool-receipts")
        && !query_l.contains("tool")
        && !query_l.contains("receipt")
    {
        reasons.push("tool-receipt-not-query-matched".to_string());
        return RecallFilterResult {
            decision: RecallDecision::DemoteArtifact,
            reasons,
        };
    }

    if content_l.contains("verified")
        || content_l.contains("passed")
        || content_l.contains("source:")
    {
        reasons.push("verification-signal".to_string());
        RecallFilterResult {
            decision: RecallDecision::AdmitAuthoritative,
            reasons,
        }
    } else {
        RecallFilterResult {
            decision: RecallDecision::AdmitBackground,
            reasons,
        }
    }
}

pub fn approx_tokens_text(text: &str) -> usize {
    (text.chars().count() / CHARS_PER_TOKEN).max(1)
}

fn approx_word_tokens(text: &str) -> usize {
    text.split_whitespace()
        .map(|token| {
            if token.ends_with('.')
                || token.ends_with(',')
                || token.ends_with(';')
                || token.ends_with('!')
                || token.ends_with('?')
            {
                (token.chars().count() / CHARS_PER_TOKEN) + 1
            } else {
                token.chars().count() / CHARS_PER_TOKEN
            }
        })
        .sum::<usize>()
        .max(1)
}

pub fn approx_tokens_messages(messages: &[Message]) -> usize {
    messages
        .iter()
        .map(|m| approx_tokens_text(&m.content) + 4)
        .sum()
}

fn count_tokens_text(text: &str, policy: &CompactionPolicy) -> usize {
    match policy.token_counter {
        TokenCounterKind::ApproxChars => approx_tokens_text(text),
        TokenCounterKind::ApproxWords => approx_word_tokens(text),
        TokenCounterKind::ProviderChatApprox | TokenCounterKind::TiktokenCl100k => {
            provider_chat_approx_tokens(text)
        }
    }
}

fn count_tokens_messages(messages: &[Message], policy: &CompactionPolicy) -> usize {
    messages
        .iter()
        .map(|m| {
            count_tokens_text(&m.content, policy)
                + message_overhead_tokens(policy, &m.role, m.name.as_deref())
        })
        .sum()
}

fn message_overhead_tokens(policy: &CompactionPolicy, role: &str, name: Option<&str>) -> usize {
    match policy.token_counter {
        TokenCounterKind::ProviderChatApprox | TokenCounterKind::TiktokenCl100k => {
            let role_overhead = match role {
                "system" => 5,
                "user" => 4,
                "assistant" => 4,
                "tool" => 7,
                _ => 4,
            };
            role_overhead + usize::from(name.is_some())
        }
        TokenCounterKind::ApproxChars | TokenCounterKind::ApproxWords => 4,
    }
}

fn provider_chat_approx_tokens(text: &str) -> usize {
    let chars = text.chars().count();
    let whitespace_tokens = text.split_whitespace().count();
    let punctuation_tokens = text
        .chars()
        .filter(|ch| {
            matches!(
                ch,
                '{' | '}' | '[' | ']' | ':' | ',' | '"' | '`' | '/' | '\\'
            )
        })
        .count()
        / 4;
    let char_estimate = (chars / 4).max(1);
    char_estimate
        .max(whitespace_tokens + punctuation_tokens)
        .max(1)
}

pub fn hash_messages(messages: &[Message]) -> Result<String, ContextGovernorError> {
    let rendered = serde_json::to_string(messages)?;
    Ok(hash_text(&rendered))
}

pub fn hash_text(text: &str) -> String {
    blake3::hash(text.as_bytes()).to_hex().to_string()
}

fn classify_messages(
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

fn resolve_allocator(policy: &CompactionPolicy) -> AllocatorMode {
    if policy.allocator.trim().is_empty() {
        return AllocatorMode::DeterministicV1;
    }
    match policy.allocator.trim().to_lowercase().as_str() {
        "deterministic" | "deterministic_v1" => AllocatorMode::DeterministicV1,
        "aggressive" | "aggressive_v1" => AllocatorMode::AggressiveV1,
        "utility_v2" => AllocatorMode::UtilityV2,
        _ => AllocatorMode::DeterministicV1,
    }
}

fn is_aggressive_allocator(policy: &CompactionPolicy) -> bool {
    matches!(resolve_allocator(policy), AllocatorMode::AggressiveV1)
}
fn is_aggressive_allocator_mode(mode: &AllocatorMode) -> bool {
    matches!(mode, AllocatorMode::AggressiveV1)
}

fn is_utility_allocator_mode(mode: &AllocatorMode) -> bool {
    matches!(mode, AllocatorMode::UtilityV2)
}

fn score_items(
    items: &mut [ContextItemV1],
    focus: Option<&str>,
    utility_messages: Option<&[Message]>,
) {
    let focus_terms = lexical_terms(focus.unwrap_or_default());
    for item in items {
        let mut score = match item.item_type {
            ItemType::LatestUserMessage => 1000,
            ItemType::ActiveInstruction => 900,
            ItemType::AcceptanceGate => 850,
            ItemType::ErrorOutput => 800,
            ItemType::SourceEvidence => 750,
            ItemType::ToolResult => 650,
            ItemType::FilePathContext => 650,
            ItemType::Decision => 600,
            ItemType::UnresolvedQuestion => 575,
            ItemType::DurableFactCandidate | ItemType::ProjectStateCandidate => 500,
            ItemType::LowRiskNarrative => 100,
            ItemType::ArtifactBoilerplate => -300,
            ItemType::StalePlan => -350,
            ItemType::DuplicateContext => -500,
            ItemType::Unknown | ItemType::ToolCall => 50,
        };
        if matches!(
            item.authority_class,
            AuthorityClass::MustPreserveExact | AuthorityClass::ActiveTask
        ) {
            score += 300;
        }
        if focus.is_some() && !matches!(item.preservation_policy, PreservationPolicy::Quarantine) {
            let focus_matches = utility_messages.map_or(true, |messages| {
                messages
                    .get(item.start_index)
                    .map(|message| {
                        focus_terms
                            .intersection(&lexical_terms(&message.content))
                            .next()
                            .is_some()
                    })
                    .unwrap_or(false)
            });
            if focus_matches {
                score += 25;
            }
        }
        item.priority_score = score;
    }
}

fn allocate_items(
    session_id: &str,
    items: Vec<ContextItemV1>,
    policy: &CompactionPolicy,
    allocator: AllocatorMode,
    focus: Option<&str>,
    messages: &[Message],
) -> ContextAllocationPlanV1 {
    if is_utility_allocator_mode(&allocator) {
        return allocate_items_utility_v2(session_id, items, policy, focus, messages);
    }

    allocate_items_v1(session_id, items, policy, allocator)
}

fn allocate_items_v1(
    session_id: &str,
    items: Vec<ContextItemV1>,
    policy: &CompactionPolicy,
    allocator: AllocatorMode,
) -> ContextAllocationPlanV1 {
    let mut used_tokens = 0usize;
    let mut kept = Vec::new();
    let mut summarized = Vec::new();
    let mut archived = Vec::new();
    let mut omitted = Vec::new();
    let mut quarantined = Vec::new();

    let len = items.len();
    let aggressive = is_aggressive_allocator_mode(&allocator);
    for item in &items {
        let in_head = item.start_index < policy.protect_first_n;
        let in_tail = item.start_index + policy.protect_last_n >= len;
        let protected_head = in_head && (!aggressive || item.approx_tokens <= 1_000);
        let protected_tail = in_tail
            && (!aggressive
                || item.approx_tokens <= 300
                || matches!(item.item_type, ItemType::LatestUserMessage));
        match item.preservation_policy {
            PreservationPolicy::Quarantine => quarantined.push(item.item_id.clone()),
            PreservationPolicy::OmitDuplicate => omitted.push(item.item_id.clone()),
            PreservationPolicy::SemanticMemoryArchive => {
                archived.push(item.item_id.clone());
                if protected_tail || item.priority_score >= 650 {
                    kept.push(item.item_id.clone());
                    used_tokens = used_tokens.saturating_add(item.approx_tokens);
                } else {
                    summarized.push(item.item_id.clone());
                }
            }
            PreservationPolicy::KeepVerbatim => {
                let must_keep = matches!(item.item_type, ItemType::LatestUserMessage)
                    || protected_head
                    || (!aggressive && protected_tail);
                let fits_aggressive_budget = !aggressive
                    || used_tokens + item.approx_tokens <= policy.target_tokens.saturating_div(2);
                if must_keep || fits_aggressive_budget {
                    kept.push(item.item_id.clone());
                    used_tokens = used_tokens.saturating_add(item.approx_tokens);
                } else {
                    summarized.push(item.item_id.clone());
                }
            }
            PreservationPolicy::ReceiptOnly => {
                if protected_head
                    || protected_tail
                    || used_tokens + item.approx_tokens <= policy.target_tokens
                {
                    summarized.push(item.item_id.clone());
                } else {
                    omitted.push(item.item_id.clone());
                }
            }
            PreservationPolicy::ExtractiveSummary | PreservationPolicy::AbstractiveSummary => {
                let must_keep = matches!(item.item_type, ItemType::LatestUserMessage)
                    || protected_head
                    || protected_tail;
                if must_keep
                    || (!aggressive && used_tokens + item.approx_tokens <= policy.target_tokens / 2)
                {
                    kept.push(item.item_id.clone());
                    used_tokens = used_tokens.saturating_add(item.approx_tokens);
                } else {
                    summarized.push(item.item_id.clone());
                }
            }
        }
    }

    ContextAllocationPlanV1 {
        schema: "ContextAllocationPlanV1".to_string(),
        plan_id: format!("ctxp_{}", Uuid::new_v4().simple()),
        session_id: session_id.to_string(),
        created_utc: Utc::now(),
        context_budget_tokens: policy.target_tokens,
        target_output_tokens: policy.target_tokens,
        allocator: allocator.as_str().to_string(),
        items,
        kept_item_ids: kept,
        summarized_item_ids: summarized,
        archived_item_ids: archived,
        omitted_item_ids: omitted,
        quarantined_item_ids: quarantined,
        selection_evidence: BTreeMap::new(),
        hot_path_operation_counts: HotPathOperationCountsV1::default(),
    }
}

#[derive(Debug)]
struct UtilityCandidate {
    index: usize,
    cost: usize,
    utility: i32,
    authority: i32,
}

fn allocate_items_utility_v2(
    session_id: &str,
    items: Vec<ContextItemV1>,
    policy: &CompactionPolicy,
    focus: Option<&str>,
    messages: &[Message],
) -> ContextAllocationPlanV1 {
    let focus_terms = lexical_terms(focus.unwrap_or_default());
    let content_hash_counts = items.iter().fold(BTreeMap::new(), |mut counts, item| {
        *counts.entry(item.content_blake3.as_str()).or_insert(0usize) += 1;
        counts
    });
    let len = items.len();
    let mut kept = BTreeSet::new();
    let mut summarized = BTreeSet::new();
    let mut archived = BTreeSet::new();
    let mut omitted = BTreeSet::new();
    let mut quarantined = BTreeSet::new();
    let mut evidence = BTreeMap::new();
    let mut used_tokens = 0usize;

    for (index, item) in items.iter().enumerate() {
        let protected = item.start_index < policy.protect_first_n
            || item.start_index.saturating_add(policy.protect_last_n) >= len;
        let mandatory = !matches!(
            item.preservation_policy,
            PreservationPolicy::Quarantine | PreservationPolicy::OmitDuplicate
        ) && (protected
            || matches!(
                item.item_type,
                ItemType::LatestUserMessage
                    | ItemType::ActiveInstruction
                    | ItemType::AcceptanceGate
                    | ItemType::ErrorOutput
                    | ItemType::SourceEvidence
            )
            || matches!(item.authority_class, AuthorityClass::MustPreserveExact));
        let mut selection = utility_selection_evidence(
            item,
            messages
                .get(item.start_index)
                .map(|message| message.content.as_str())
                .unwrap_or_default(),
            index,
            len,
            &focus_terms,
            *content_hash_counts
                .get(item.content_blake3.as_str())
                .unwrap_or(&1),
        );
        selection.mandatory = mandatory;

        match item.preservation_policy {
            PreservationPolicy::Quarantine => {
                quarantined.insert(item.item_id.clone());
            }
            PreservationPolicy::OmitDuplicate => {
                omitted.insert(item.item_id.clone());
            }
            PreservationPolicy::SemanticMemoryArchive => {
                archived.insert(item.item_id.clone());
                if mandatory {
                    kept.insert(item.item_id.clone());
                    used_tokens = used_tokens.saturating_add(item_total_tokens(item, policy));
                    selection.selected = true;
                } else {
                    summarized.insert(item.item_id.clone());
                }
            }
            PreservationPolicy::ReceiptOnly => {
                summarized.insert(item.item_id.clone());
            }
            PreservationPolicy::KeepVerbatim
            | PreservationPolicy::ExtractiveSummary
            | PreservationPolicy::AbstractiveSummary => {
                if mandatory {
                    kept.insert(item.item_id.clone());
                    used_tokens = used_tokens.saturating_add(item_total_tokens(item, policy));
                    selection.selected = true;
                } else {
                    summarized.insert(item.item_id.clone());
                }
            }
        }
        if selection.selected {
            selection.selected_utility_per_token_milli = Some(utility_per_token_milli(
                selection.utility,
                item_total_tokens(item, policy),
            ));
        }
        evidence.insert(item.item_id.clone(), selection);
    }

    let reserve = estimated_synthetic_reserve_tokens(policy);
    let optional_budget = policy.target_tokens.saturating_sub(reserve);
    let mut candidates = items
        .iter()
        .enumerate()
        .filter_map(|(index, item)| {
            let selection = evidence.get(&item.item_id)?;
            if selection.mandatory
                || selection.utility <= 0
                || !matches!(
                    item.preservation_policy,
                    PreservationPolicy::KeepVerbatim
                        | PreservationPolicy::ExtractiveSummary
                        | PreservationPolicy::AbstractiveSummary
                        | PreservationPolicy::SemanticMemoryArchive
                )
            {
                return None;
            }
            Some(UtilityCandidate {
                index,
                cost: item_total_tokens(item, policy),
                utility: selection.utility,
                authority: selection.authority,
            })
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        let left_ratio = i64::from(left.utility) * right.cost as i64;
        let right_ratio = i64::from(right.utility) * left.cost as i64;
        right_ratio
            .cmp(&left_ratio)
            .then_with(|| right.authority.cmp(&left.authority))
            .then_with(|| left.index.cmp(&right.index))
            .then_with(|| items[left.index].item_id.cmp(&items[right.index].item_id))
    });
    for candidate in candidates {
        if used_tokens.saturating_add(candidate.cost) > optional_budget {
            continue;
        }
        let item = &items[candidate.index];
        used_tokens = used_tokens.saturating_add(candidate.cost);
        kept.insert(item.item_id.clone());
        summarized.remove(&item.item_id);
        if let Some(selection) = evidence.get_mut(&item.item_id) {
            selection.selected = true;
            selection.selected_utility_per_token_milli =
                Some(utility_per_token_milli(selection.utility, candidate.cost));
        }
    }

    let ordered = |ids: &BTreeSet<String>| {
        items
            .iter()
            .filter(|item| ids.contains(&item.item_id))
            .map(|item| item.item_id.clone())
            .collect::<Vec<_>>()
    };
    let kept_item_ids = ordered(&kept);
    let summarized_item_ids = ordered(&summarized);
    let archived_item_ids = ordered(&archived);
    let omitted_item_ids = ordered(&omitted);
    let quarantined_item_ids = ordered(&quarantined);
    ContextAllocationPlanV1 {
        schema: "ContextAllocationPlanV1".to_string(),
        plan_id: format!("ctxp_{}", Uuid::new_v4().simple()),
        session_id: session_id.to_string(),
        created_utc: Utc::now(),
        context_budget_tokens: policy.target_tokens,
        target_output_tokens: policy.target_tokens,
        allocator: AllocatorMode::UtilityV2.as_str().to_string(),
        items,
        kept_item_ids,
        summarized_item_ids,
        archived_item_ids,
        omitted_item_ids,
        quarantined_item_ids,
        selection_evidence: evidence,
        hot_path_operation_counts: HotPathOperationCountsV1::default(),
    }
}

fn item_total_tokens(item: &ContextItemV1, policy: &CompactionPolicy) -> usize {
    item.approx_tokens.saturating_add(message_overhead_tokens(
        policy,
        item.role_set
            .first()
            .map(String::as_str)
            .unwrap_or("assistant"),
        None,
    ))
}

fn estimated_synthetic_reserve_tokens(policy: &CompactionPolicy) -> usize {
    let pointer = format!(
        "{SUMMARY_PREFIX}\nEarlier context was compacted. Exact fallback remains available. \\
Use context_expand(receipt_id=..., item_id=...) to recover omitted text."
    );
    count_tokens_text(&pointer, policy)
        .saturating_add(message_overhead_tokens(
            policy,
            "assistant",
            Some("context_governor"),
        ))
        .saturating_add(12)
}

fn lexical_terms(text: &str) -> BTreeSet<String> {
    text.split(|ch: char| !ch.is_alphanumeric() && ch != '_')
        .filter(|term| term.chars().count() >= 2)
        .map(|term| term.to_lowercase())
        .collect()
}

fn utility_selection_evidence(
    item: &ContextItemV1,
    content: &str,
    index: usize,
    len: usize,
    focus_terms: &BTreeSet<String>,
    duplicate_count: usize,
) -> UtilitySelectionEvidenceV1 {
    let authority = match item.authority_class {
        AuthorityClass::MustPreserveExact => 1_000,
        AuthorityClass::EvidenceCritical => 850,
        AuthorityClass::ActiveTask => 900,
        AuthorityClass::VerifiedToolReceipt => 600,
        AuthorityClass::DurableMemoryCandidate => 400,
        AuthorityClass::SummaryOk => 120,
        AuthorityClass::ArchiveOk => 60,
        AuthorityClass::Discardable => -300,
        AuthorityClass::Quarantine => -500,
    };
    let item_terms = lexical_terms(content);
    let focus_overlap_count = focus_terms.intersection(&item_terms).count();
    let focus_overlap = i32::try_from(focus_overlap_count).unwrap_or(i32::MAX) * 100;
    let recency = if len <= 1 {
        100
    } else {
        i32::try_from(index.saturating_mul(100) / (len - 1)).unwrap_or(100)
    };
    let unresolved_signal = match item.item_type {
        ItemType::UnresolvedQuestion => 100,
        ItemType::ErrorOutput => 90,
        ItemType::Decision => 50,
        ItemType::LatestUserMessage => 100,
        _ => 0,
    };
    let novelty = if duplicate_count == 1 { 35 } else { 0 };
    let redundancy = if duplicate_count > 1 { -350 } else { 0 };
    let recoverability = match item.preservation_policy {
        PreservationPolicy::ReceiptOnly
        | PreservationPolicy::SemanticMemoryArchive
        | PreservationPolicy::OmitDuplicate
        | PreservationPolicy::Quarantine => -20,
        _ => 0,
    };
    UtilitySelectionEvidenceV1 {
        authority,
        focus_overlap,
        recency,
        unresolved_signal,
        novelty,
        redundancy,
        recoverability,
        utility: authority
            + focus_overlap
            + recency
            + unresolved_signal
            + novelty
            + redundancy
            + recoverability,
        selected: false,
        mandatory: false,
        selected_utility_per_token_milli: None,
    }
}

fn utility_per_token_milli(utility: i32, tokens: usize) -> u64 {
    u64::try_from(utility.max(0))
        .unwrap_or(0)
        .saturating_mul(1_000)
        .saturating_div(u64::try_from(tokens.max(1)).unwrap_or(1))
}

fn build_exact_store(
    messages: &[Message],
    plan: &ContextAllocationPlanV1,
) -> (Vec<ExactStoredItemV1>, usize) {
    let dispositions = PlanDispositionIndex::new(plan);
    let mut checks = 0usize;
    let store = plan
        .items
        .iter()
        .filter(|item| {
            checks += 1;
            dispositions.requires_exact_store(item)
        })
        .filter_map(|item| {
            messages.get(item.start_index).map(|m| ExactStoredItemV1 {
                item_id: item.item_id.clone(),
                source_indices: vec![item.start_index],
                content: m.content.clone(),
                content_blake3: item.content_blake3.clone(),
            })
        })
        .collect();
    (store, checks)
}

struct PlanDispositionIndex<'a> {
    kept: BTreeSet<&'a str>,
    summarized: BTreeSet<&'a str>,
    archived: BTreeSet<&'a str>,
    omitted: BTreeSet<&'a str>,
    quarantined: BTreeSet<&'a str>,
}

impl<'a> PlanDispositionIndex<'a> {
    fn new(plan: &'a ContextAllocationPlanV1) -> Self {
        let as_set = |ids: &'a [String]| ids.iter().map(String::as_str).collect();
        Self {
            kept: as_set(&plan.kept_item_ids),
            summarized: as_set(&plan.summarized_item_ids),
            archived: as_set(&plan.archived_item_ids),
            omitted: as_set(&plan.omitted_item_ids),
            quarantined: as_set(&plan.quarantined_item_ids),
        }
    }

    fn is_kept(&self, item: &ContextItemV1) -> bool {
        self.kept.contains(item.item_id.as_str())
    }

    fn is_summarized_or_archived(&self, item: &ContextItemV1) -> bool {
        self.summarized.contains(item.item_id.as_str())
            || self.archived.contains(item.item_id.as_str())
    }

    fn requires_exact_store(&self, item: &ContextItemV1) -> bool {
        self.summarized.contains(item.item_id.as_str())
            || self.archived.contains(item.item_id.as_str())
            || self.omitted.contains(item.item_id.as_str())
            || self.quarantined.contains(item.item_id.as_str())
            || matches!(item.preservation_policy, PreservationPolicy::ReceiptOnly)
    }
}

fn build_structured_summary(
    messages: &[Message],
    plan: &ContextAllocationPlanV1,
) -> (StructuredContextSummaryV1, usize) {
    let mut unique_values = BTreeSet::new();
    let mut checks = 0usize;
    let mut out = StructuredContextSummaryV1 {
        active_task: messages
            .iter()
            .rev()
            .find(|m| m.role == "user")
            .map(|m| compact_preview(&m.content, 240)),
        fallback_item_ids: plan
            .summarized_item_ids
            .iter()
            .chain(plan.omitted_item_ids.iter())
            .chain(plan.quarantined_item_ids.iter())
            .cloned()
            .collect(),
        ..Default::default()
    };

    for msg in messages {
        let content_l = msg.content.to_lowercase();
        if content_l.contains("acceptance gate") || content_l.contains("must pass") {
            push_unique_indexed(
                &mut out.acceptance_gates,
                &mut unique_values,
                "acceptance_gates",
                compact_preview(&msg.content, 240),
                &mut checks,
            );
        }
        if content_l.contains("decision:") || content_l.contains("decided") {
            push_unique_indexed(
                &mut out.decisions,
                &mut unique_values,
                "decisions",
                compact_preview(&msg.content, 240),
                &mut checks,
            );
        }
        if content_l.contains("unresolved question") || content_l.contains('?') {
            push_unique_indexed(
                &mut out.unresolved_questions,
                &mut unique_values,
                "unresolved_questions",
                compact_preview(&msg.content, 240),
                &mut checks,
            );
        }
        if content_l.contains("error")
            || content_l.contains("traceback")
            || content_l.contains("failed")
        {
            push_unique_indexed(
                &mut out.errors,
                &mut unique_values,
                "errors",
                important_lines(&msg.content, &["error", "failed", "traceback"], 240),
                &mut checks,
            );
        }
        for file in extract_file_like_tokens(&msg.content) {
            push_unique_indexed(
                &mut out.files,
                &mut unique_values,
                "files",
                file,
                &mut checks,
            );
        }
        for command in extract_command_like_lines(&msg.content) {
            push_unique_indexed(
                &mut out.commands,
                &mut unique_values,
                "commands",
                command,
                &mut checks,
            );
        }
    }
    (out, checks)
}

fn push_unique_indexed(
    values: &mut Vec<String>,
    seen: &mut BTreeSet<(String, String)>,
    category: &str,
    value: String,
    checks: &mut usize,
) {
    if !value.is_empty() {
        *checks += 1;
        if seen.insert((category.to_string(), value.clone())) {
            values.push(value);
        }
    }
}

fn push_summary_list(lines: &mut Vec<String>, label: &str, values: &[String]) {
    if values.is_empty() {
        return;
    }
    lines.push(format!("- {label}:"));
    for value in values.iter().take(5) {
        lines.push(format!("  - {}", compact_preview(value, 220)));
    }
}

fn extract_file_like_tokens(text: &str) -> Vec<String> {
    text.split_whitespace()
        .map(|token| {
            token.trim_matches(|c: char| {
                c == ',' || c == ':' || c == ';' || c == ')' || c == '(' || c == '`' || c == '"'
            })
        })
        .filter(|token| {
            token.contains("/home/")
                || token.contains("/usr/")
                || token.contains("/etc/")
                || token.contains("/var/")
                || token.contains("/tmp/")
                || token.starts_with("~/")
                || token.starts_with("/")
                || token.ends_with(".rs")
                || token.ends_with(".py")
                || token.ends_with(".ts")
                || token.ends_with(".tsx")
                || token.ends_with(".js")
                || token.ends_with(".md")
                || token.ends_with(".toml")
                || token.ends_with(".yaml")
                || token.ends_with(".yml")
                || token.ends_with(".json")
                || token.ends_with(".go")
                || token.ends_with(".c")
                || token.ends_with(".h")
        })
        .map(|token| token.to_string())
        .collect()
}

fn extract_command_like_lines(text: &str) -> Vec<String> {
    text.lines()
        .map(str::trim)
        .filter(|line| {
            line.starts_with("running:")
                || line.starts_with('$')
                || line.starts_with("cargo ")
                || line.starts_with("npm ")
                || line.starts_with("python ")
                || line.starts_with("git ")
        })
        .map(|line| compact_preview(line, 240))
        .collect()
}

fn important_lines(text: &str, needles: &[&str], max_chars: usize) -> String {
    let mut out = Vec::new();
    for line in text.lines() {
        let line_l = line.to_lowercase();
        if needles.iter().any(|needle| line_l.contains(needle)) {
            out.push(line.trim());
        }
        if out.len() >= 5 {
            break;
        }
    }
    if out.is_empty() {
        compact_preview(text, max_chars)
    } else {
        compact_preview(&out.join(" | "), max_chars)
    }
}

fn build_summary(
    messages: &[Message],
    plan: &ContextAllocationPlanV1,
    policy: &CompactionPolicy,
    structured: &StructuredContextSummaryV1,
    receipt_id: &str,
) -> (String, usize) {
    let dispositions = PlanDispositionIndex::new(plan);
    let mut checks = 0usize;
    let mut lines = vec![
        format!("{SUMMARY_PREFIX}"),
        "Earlier turns were compacted. This summary is background, not an active task.".to_string(),
        "Only the latest user message after this summary is active.".to_string(),
        format!("Use context_expand(receipt_id=\"{receipt_id}\", item_id=...) to recover exact omitted text when needed."),
        String::new(),
        "## Structured anchors".to_string(),
    ];
    if let Some(active_task) = &structured.active_task {
        lines.push(format!(
            "- active_task: {}",
            compact_preview(active_task, 220)
        ));
    }
    push_summary_list(&mut lines, "acceptance_gates", &structured.acceptance_gates);
    push_summary_list(&mut lines, "commands", &structured.commands);
    push_summary_list(&mut lines, "errors", &structured.errors);
    push_summary_list(&mut lines, "files", &structured.files);
    push_summary_list(&mut lines, "decisions", &structured.decisions);
    push_summary_list(
        &mut lines,
        "unresolved_questions",
        &structured.unresolved_questions,
    );
    if !structured.fallback_item_ids.is_empty() {
        let shown = structured
            .fallback_item_ids
            .iter()
            .take(30)
            .cloned()
            .collect::<Vec<_>>();
        let hidden = structured
            .fallback_item_ids
            .len()
            .saturating_sub(shown.len());
        let suffix = if hidden > 0 {
            format!(" (+{hidden} more in receipt)")
        } else {
            String::new()
        };
        lines.push(format!(
            "- fallback_item_ids: {}{}",
            shown.join(", "),
            suffix
        ));
    }

    lines.push(String::new());
    lines.push("## Preserved / summarized context".to_string());
    for item in &plan.items {
        checks += 1;
        if dispositions.is_summarized_or_archived(item)
            || matches!(item.preservation_policy, PreservationPolicy::ReceiptOnly)
        {
            if let Some(msg) = messages.get(item.start_index) {
                let preview = content_aware_preview(&item.content_kind, &msg.content, 220);
                lines.push(format!(
                    "- {} {:?}/{:?}/{:?}: {}",
                    item.item_id,
                    item.item_type,
                    item.content_kind,
                    item.preservation_policy,
                    preview
                ));
            }
        }
    }
    if !plan.quarantined_item_ids.is_empty() {
        lines.push(String::new());
        lines.push(format!(
            "## Quarantined context\n{} items were excluded from authoritative context due to speculative/artifact signals; exact fallback remains available.",
            plan.quarantined_item_ids.len()
        ));
    }
    let mut summary = lines.join("\n");
    if summary.chars().count() > policy.summary_max_chars {
        summary = summary
            .chars()
            .take(policy.summary_max_chars)
            .collect::<String>();
        summary.push_str("\n[summary truncated by context-governor budget]");
    }
    (summary, checks)
}

fn assemble_compacted_messages(
    messages: &[Message],
    plan: &ContextAllocationPlanV1,
    summary: &str,
    policy: &CompactionPolicy,
) -> (Vec<Message>, usize) {
    let mut out = Vec::new();
    let dispositions = PlanDispositionIndex::new(plan);
    let mut emitted = BTreeSet::new();
    let mut checks = 0usize;
    let aggressive = is_aggressive_allocator(policy);
    let item_by_index = plan
        .items
        .iter()
        .map(|i| (i.start_index, i))
        .collect::<BTreeMap<_, _>>();

    let tail_start = messages.len().saturating_sub(policy.protect_last_n);
    let latest_user_idx = messages.iter().rposition(|message| message.role == "user");

    for (idx, msg) in messages.iter().enumerate() {
        if Some(idx) == latest_user_idx {
            continue;
        }
        if idx >= tail_start {
            continue;
        }
        if idx < policy.protect_first_n {
            let Some(item) = item_by_index.get(&idx) else {
                out.push(msg.clone());
                emitted.insert((msg.role.clone(), msg.content.clone()));
                continue;
            };
            let keep_head = matches!(item.item_type, ItemType::LatestUserMessage)
                || (dispositions.is_kept(item) && (!aggressive || item.approx_tokens <= 1_000))
                || (matches!(
                    item.authority_class,
                    AuthorityClass::MustPreserveExact | AuthorityClass::ActiveTask
                ) && (!aggressive || item.approx_tokens <= 1_000));
            if keep_head {
                out.push(msg.clone());
                emitted.insert((msg.role.clone(), msg.content.clone()));
            }
        } else if let Some(item) = item_by_index.get(&idx) {
            checks += 1;
            if dispositions.is_kept(item) && emitted.insert((msg.role.clone(), msg.content.clone()))
            {
                out.push(msg.clone());
            }
        }
    }

    let summary_msg = Message {
        id: Some(format!("summary_{}", plan.plan_id)),
        role: choose_summary_role(out.last().map(|m| m.role.as_str())).to_string(),
        content: summary.to_string(),
        name: Some("context_governor".to_string()),
        metadata: BTreeMap::from([("compressed_summary".to_string(), Value::Bool(true))]),
    };
    emitted.insert((summary_msg.role.clone(), summary_msg.content.clone()));
    out.push(summary_msg);

    for (offset, msg) in messages[tail_start..].iter().enumerate() {
        let idx = tail_start + offset;
        if Some(idx) == latest_user_idx {
            continue;
        }
        let _ = item_by_index.get(&idx); // item exists but we always keep tail
                                         // Always include tail messages within protect_last_n, regardless of
                                         // kept_set. The built-in compressor always includes the last N messages;
                                         // excluding them because the allocator summarized them causes data loss
                                         // of recent context the model needs.
        checks += 1;
        if emitted.insert((msg.role.clone(), msg.content.clone())) {
            out.push(msg.clone());
        }
    }
    // The latest user message is an identity-bearing active instruction, not a
    // role/content equivalence class. Emit it exactly once and last.
    if let Some(idx) = latest_user_idx {
        out.push(messages[idx].clone());
    }
    (out, checks)
}

fn choose_summary_role(previous: Option<&str>) -> &'static str {
    match previous {
        Some("assistant") => "user",
        _ => "assistant",
    }
}

fn build_loss_report(
    plan: &ContextAllocationPlanV1,
    structured_summary: StructuredContextSummaryV1,
    exact_fallback_in_response: bool,
) -> SummaryLossReportV1 {
    SummaryLossReportV1 {
        schema: "SummaryLossReportV1".to_string(),
        preserved_claims: plan.kept_item_ids.clone(),
        omitted_claims: plan.omitted_item_ids.clone(),
        evidence_lost: plan
            .omitted_item_ids
            .iter()
            .chain(plan.summarized_item_ids.iter())
            .cloned()
            .collect(),
        uncertainty_introduced: plan.quarantined_item_ids.clone(),
        exact_recovery_state: if exact_fallback_in_response {
            ExactRecoveryStateV1::InResponse
        } else {
            ExactRecoveryStateV1::Unavailable
        },
        high_risk_omissions: plan
            .items
            .iter()
            .filter(|i| {
                plan.omitted_item_ids.contains(&i.item_id)
                    && matches!(
                        i.authority_class,
                        AuthorityClass::MustPreserveExact | AuthorityClass::EvidenceCritical
                    )
            })
            .map(|i| i.item_id.clone())
            .collect(),
        structured_summary,
    }
}

fn safe_recovery_summary(receipt_id: &str) -> String {
    format!(
        "{SUMMARY_PREFIX}\nEarlier context was quarantined by the post-compression boundary audit. \\
This is background, not an active task.\nUse context_expand(receipt_id=\"{receipt_id}\", item_id=...) to recover exact omitted text when needed."
    )
}

fn compact_preview(text: &str, max_chars: usize) -> String {
    let clean = text.split_whitespace().collect::<Vec<_>>().join(" ");
    if clean.chars().count() <= max_chars {
        clean
    } else {
        let mut s = clean.chars().take(max_chars).collect::<String>();
        s.push_str("...");
        s
    }
}

fn content_aware_preview(kind: &ContentKind, text: &str, max_chars: usize) -> String {
    match kind {
        ContentKind::Json => json_preview(text, max_chars),
        ContentKind::Diff => important_lines(text, &["diff --git", "@@", "+++", "---"], max_chars),
        ContentKind::CargoOutput => important_lines(
            text,
            &["error", "warning", "test result", "failed", "finished"],
            max_chars,
        ),
        ContentKind::ShellLog => {
            important_lines(text, &["error", "failed", "exit", "$"], max_chars)
        }
        ContentKind::SearchResults => important_lines(
            text,
            &["/home/", ":", "LINE", "match", ".rs", ".py", ".md"],
            max_chars,
        ),
        ContentKind::Rust => important_lines(
            text,
            &["pub ", "fn ", "struct ", "impl ", "use ", "error", "TODO"],
            max_chars,
        ),
        ContentKind::Markdown => important_lines(text, &["#", "##", "- ", "```"], max_chars),
        _ => compact_preview(text, max_chars),
    }
}

fn json_preview(text: &str, max_chars: usize) -> String {
    if let Ok(value) = serde_json::from_str::<Value>(text) {
        if let Some(obj) = value.as_object() {
            let keys = obj.keys().take(12).cloned().collect::<Vec<_>>().join(", ");
            return compact_preview(&format!("json keys: {keys}"), max_chars);
        }
    }
    compact_preview(text, max_chars)
}

fn detect_content_kind(role: &str, content: &str) -> ContentKind {
    let trimmed = content.trim();
    let lower = trimmed.to_lowercase();
    if serde_json::from_str::<Value>(trimmed).is_ok() {
        return ContentKind::Json;
    }
    if lower.starts_with("diff --git") || lower.contains("\n@@ ") {
        return ContentKind::Diff;
    }
    if lower.contains("error[")
        || lower.contains("test result:")
        || lower.contains("warning:")
        || lower.contains("compiling ")
    {
        return ContentKind::CargoOutput;
    }
    if trimmed.starts_with("# ") || trimmed.contains("\n## ") || trimmed.contains("\n- ") {
        return ContentKind::Markdown;
    }
    if lower.contains("fn main") || lower.contains("pub struct") || lower.contains("impl ") {
        return ContentKind::Rust;
    }
    if looks_like_search_results(trimmed) {
        return ContentKind::SearchResults;
    }
    if role == "tool" {
        return ContentKind::ShellLog;
    }
    ContentKind::PlainText
}

fn looks_like_search_results(text: &str) -> bool {
    let mut path_line_count = 0usize;
    for line in text.lines().take(1_000) {
        let lower = line.to_lowercase();
        let has_path = lower.contains("/home/")
            || lower.contains("/tmp/")
            || lower.contains(".rs:")
            || lower.contains(".py:")
            || lower.contains(".md:")
            || lower.contains(".toml:")
            || lower.contains(".yaml:")
            || lower.contains(".json:");
        let has_location_separator = line.matches(':').count() >= 2;
        if has_path && has_location_separator {
            path_line_count += 1;
        }
    }
    path_line_count >= 1
}

fn enforce_budget(
    mut messages: Vec<Message>,
    policy: &CompactionPolicy,
    warnings: &mut Vec<String>,
) -> Result<Vec<Message>, ContextGovernorError> {
    let target = policy.target_tokens;
    if count_tokens_messages(&messages, policy) <= target {
        return Ok(messages);
    }
    let Some(summary_idx) = messages
        .iter()
        .position(|m| m.content.starts_with(SUMMARY_PREFIX))
    else {
        let minimum = count_tokens_messages(&messages, policy);
        return Err(ContextGovernorError::BudgetExceeded {
            target,
            actual: minimum,
        });
    };

    let mut without_summary = messages.clone();
    without_summary.remove(summary_idx);
    let required = count_tokens_messages(&without_summary, policy);
    if required > target {
        if matches!(policy.budget_mode, BudgetMode::HardCascade) {
            warnings.push(format!(
                "hard budget not met: hard cascade removed summary but exact protected minimum still exceeds target budget: {required} > {target} tokens"
            ));
            return Ok(without_summary);
        }
        return Err(ContextGovernorError::BudgetExceeded {
            target,
            actual: required,
        });
    }

    let minimal_summary = minimal_recovery_summary(&messages[summary_idx].content);
    if let Some(summary) = truncate_summary_to_fit(&messages, summary_idx, target, policy) {
        let changed = messages[summary_idx].content != summary;
        messages[summary_idx].content = summary;
        if changed {
            warnings.push(
                "hard cascade truncated summary using the selected token counter to satisfy target budget"
                    .to_string(),
            );
        }
    } else if summary_fits(&messages, summary_idx, &minimal_summary, target, policy) {
        messages[summary_idx].content = minimal_summary;
        warnings.push("hard cascade reduced summary to minimal recovery pointer".to_string());
    } else {
        messages.remove(summary_idx);
        warnings.push("hard cascade removed summary to satisfy target budget".to_string());
    }
    Ok(messages)
}

fn summary_fits(
    messages: &[Message],
    summary_idx: usize,
    content: &str,
    target: usize,
    policy: &CompactionPolicy,
) -> bool {
    let mut candidate = messages.to_vec();
    candidate[summary_idx].content = content.to_string();
    count_tokens_messages(&candidate, policy) <= target
}

fn truncate_summary_to_fit(
    messages: &[Message],
    summary_idx: usize,
    target: usize,
    policy: &CompactionPolicy,
) -> Option<String> {
    let original = &messages[summary_idx].content;
    let char_count = original.chars().count();
    let mut low = 0usize;
    let mut high = char_count;
    let mut best = None;
    while low <= high {
        let middle = low + (high - low) / 2;
        let prefix = original.chars().take(middle).collect::<String>();
        let candidate = format!(
            "{prefix}\n[summary truncated by context-governor budget; exact fallback remains available]"
        );
        if summary_fits(messages, summary_idx, &candidate, target, policy) {
            best = Some(candidate);
            low = middle.saturating_add(1);
        } else if middle == 0 {
            break;
        } else {
            high = middle - 1;
        }
    }
    best
}

fn minimal_recovery_summary(summary: &str) -> String {
    let mut lines = vec![
        SUMMARY_PREFIX.to_string(),
        "Hard cascade kept only the recovery pointer; exact fallback remains available."
            .to_string(),
    ];
    for line in summary.lines() {
        if line.contains("context_expand(") || line.starts_with("- fallback_item_ids:") {
            lines.push(compact_preview(line, 600));
        }
    }
    lines.join("\n")
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn contains_path_signal(text: &str) -> bool {
    text.contains("/home/")
        || text.contains("~/")
        || text.contains("Cargo.toml")
        || text.contains("pyproject.toml")
        || text.contains(".rs")
        || text.contains(".py")
        || text.contains(".ts")
        || text.contains(".tsx")
        || text.contains(".md")
        || text.contains(".yaml")
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum SearchScope {
    All,
    Summary,
    ExactStore,
    Receipt,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextSearchHit {
    pub source: String,
    pub item_id: Option<String>,
    pub snippet: String,
    pub content_blake3: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextExpandResult {
    pub item_id: String,
    pub content: String,
    pub content_blake3: String,
    pub truncated: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ContextDiffSummary {
    pub kept_count: usize,
    pub summarized_count: usize,
    pub archived_count: usize,
    pub omitted_count: usize,
    pub quarantined_count: usize,
    pub original_approx_tokens: usize,
    pub compacted_approx_tokens: usize,
    pub token_savings_estimate: isize,
    pub warnings: Vec<String>,
    #[serde(default)]
    pub content_kind_reductions: BTreeMap<String, usize>,
}

pub fn context_expand(
    response: &CompactResponse,
    item_id: &str,
    max_chars: usize,
) -> Option<ContextExpandResult> {
    response
        .exact_store
        .iter()
        .find(|item| item.item_id == item_id)
        .map(|item| {
            let total_chars = item.content.chars().count();
            let truncated = total_chars > max_chars;
            let content = if truncated {
                item.content.chars().take(max_chars).collect::<String>()
            } else {
                item.content.clone()
            };
            ContextExpandResult {
                item_id: item.item_id.clone(),
                content,
                content_blake3: item.content_blake3.clone(),
                truncated,
            }
        })
}

pub fn context_search(
    response: &CompactResponse,
    query: &str,
    top_k: usize,
    scope: SearchScope,
) -> Vec<ContextSearchHit> {
    let query_l = query.to_lowercase();
    let mut scored_hits = Vec::new();

    let mut add_hit = |source: &'static str,
                       rank: usize,
                       item_id: Option<String>,
                       rendered: &str,
                       hash: Option<String>| {
        if rendered.to_lowercase().contains(&query_l) {
            scored_hits.push((
                rank,
                ContextSearchHit {
                    source: source.to_string(),
                    item_id,
                    snippet: snippet_around(rendered, query, 320),
                    content_blake3: hash,
                },
            ));
        }
    };

    if matches!(scope, SearchScope::All | SearchScope::ExactStore) {
        for item in &response.exact_store {
            add_hit(
                "exact_store",
                90,
                Some(item.item_id.clone()),
                &item.content,
                Some(item.content_blake3.clone()),
            );
        }
    }

    if matches!(scope, SearchScope::All | SearchScope::Summary) {
        for msg in &response.compacted_messages {
            add_hit(
                "compacted_messages",
                60,
                msg.id.clone(),
                &msg.content,
                Some(hash_text(&msg.content)),
            );
        }
    }

    if matches!(scope, SearchScope::All | SearchScope::Receipt) {
        if let Ok(rendered) = serde_json::to_string(&response.receipt) {
            add_hit(
                "receipt",
                40,
                Some(response.receipt.receipt_id.clone()),
                &rendered,
                Some(hash_text(&rendered)),
            );
        }
    }

    scored_hits.sort_by(|a, b| {
        b.0.cmp(&a.0)
            .then_with(|| a.1.source.cmp(&b.1.source))
            .then_with(|| a.1.snippet.cmp(&b.1.snippet))
    });

    let mut seen = std::collections::HashSet::new();
    let mut hits = Vec::new();
    for (_, hit) in scored_hits {
        let key = format!(
            "{}:{}",
            hit.source,
            hit.item_id.clone().unwrap_or_else(|| "receipt".to_string())
        );
        if seen.insert(key) {
            hits.push(hit);
            if hits.len() >= top_k {
                break;
            }
        }
    }

    hits
}

pub fn context_diff(response: &CompactResponse) -> ContextDiffSummary {
    let mut content_kind_reductions: BTreeMap<String, usize> = BTreeMap::new();
    let reduced_ids = response
        .allocation_plan
        .summarized_item_ids
        .iter()
        .chain(response.allocation_plan.archived_item_ids.iter())
        .chain(response.allocation_plan.omitted_item_ids.iter())
        .chain(response.allocation_plan.quarantined_item_ids.iter())
        .cloned()
        .collect::<BTreeSet<_>>();
    for item in &response.allocation_plan.items {
        if reduced_ids.contains(&item.item_id) {
            *content_kind_reductions
                .entry(format!("{:?}", item.content_kind))
                .or_insert(0) += 1;
        }
    }
    ContextDiffSummary {
        kept_count: response.allocation_plan.kept_item_ids.len(),
        summarized_count: response.allocation_plan.summarized_item_ids.len(),
        archived_count: response.allocation_plan.archived_item_ids.len(),
        omitted_count: response.allocation_plan.omitted_item_ids.len(),
        quarantined_count: response.allocation_plan.quarantined_item_ids.len(),
        original_approx_tokens: response.receipt.original_approx_tokens,
        compacted_approx_tokens: response.receipt.compacted_approx_tokens,
        token_savings_estimate: response.receipt.token_savings_estimate,
        warnings: response.receipt.warnings.clone(),
        content_kind_reductions,
    }
}

fn snippet_around(content: &str, query: &str, max_chars: usize) -> String {
    let content_l = content.to_lowercase();
    let query_l = query.to_lowercase();
    let start_byte = content_l
        .find(&query_l)
        .map(|idx| floor_char_boundary(content, idx.saturating_sub(max_chars / 3)))
        .unwrap_or(0);
    let mut snippet = content[start_byte..]
        .chars()
        .take(max_chars)
        .collect::<String>();
    if snippet.chars().count() == max_chars {
        snippet.push_str("...");
    }
    snippet
}

fn floor_char_boundary(text: &str, mut idx: usize) -> usize {
    idx = idx.min(text.len());
    while idx > 0 && !text.is_char_boundary(idx) {
        idx -= 1;
    }
    idx
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayProbe {
    pub category: String,
    pub needle: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayBaselineScore {
    pub name: String,
    pub visible_probes: usize,
    pub recoverable_probes: usize,
    pub total_probes: usize,
    pub visible_rate: f64,
    pub recoverable_rate: f64,
    pub tokens: usize,
    pub active_task_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayFixtureReport {
    pub fixture_id: String,
    pub original_message_count: usize,
    pub compacted_message_count: usize,
    pub probes: Vec<ReplayProbe>,
    pub baselines: Vec<ReplayBaselineScore>,
    pub receipt_id: String,
    pub warnings: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ReplayAnswerabilityQuestion {
    pub question: String,
    pub expected_terms: Vec<String>,
    pub forbidden_terms: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayAnswerabilityBaselineScore {
    pub name: String,
    pub answerable_questions: usize,
    pub total_questions: usize,
    pub answerability_rate: f64,
    pub incorrect_action_risk: usize,
    pub tokens: usize,
    pub active_task_visible: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ReplayAnswerabilityReport {
    pub fixture_id: String,
    pub baselines: Vec<ReplayAnswerabilityBaselineScore>,
    pub receipt_id: String,
    pub warnings: Vec<String>,
}

pub fn build_replay_probes(messages: &[Message], max_probes: usize) -> Vec<ReplayProbe> {
    let mut probes = Vec::new();
    if let Some(latest_user) = messages.iter().rev().find(|message| message.role == "user") {
        push_probe(
            &mut probes,
            "active_task",
            literal_probe_fragment(&latest_user.content, 120),
            max_probes,
        );
    }

    for message in messages {
        let content_l = message.content.to_lowercase();
        if content_l.contains("acceptance gate") || content_l.contains("must pass") {
            push_probe(
                &mut probes,
                "acceptance_gate",
                important_lines(&message.content, &["acceptance gate", "must pass"], 160),
                max_probes,
            );
        }
        if content_l.contains("decision:") || content_l.contains("decided") {
            push_probe(
                &mut probes,
                "decision",
                important_lines(&message.content, &["decision", "decided"], 160),
                max_probes,
            );
        }
        if content_l.contains("error")
            || content_l.contains("traceback")
            || content_l.contains("failed")
        {
            push_probe(
                &mut probes,
                "error",
                important_lines(&message.content, &["error", "traceback", "failed"], 160),
                max_probes,
            );
        }
        for file in extract_file_like_tokens(&message.content) {
            push_probe(&mut probes, "file", file, max_probes);
        }
        if probes.len() >= max_probes {
            break;
        }
    }
    probes
}

pub fn evaluate_replay_answerability(
    fixture_id: impl Into<String>,
    request: CompactRequest,
    questions: Vec<ReplayAnswerabilityQuestion>,
) -> Result<ReplayAnswerabilityReport, ContextGovernorError> {
    let fixture_id = fixture_id.into();
    let active_task = request
        .messages
        .iter()
        .rev()
        .find(|message| message.role == "user")
        .map(|message| literal_probe_fragment(&message.content, 120))
        .unwrap_or_default();

    let full_text = join_message_content(&request.messages);
    let full_tokens = approx_tokens_messages(&request.messages);
    let head_tail_messages = head_tail_messages(&request.messages, 1, 1);
    let head_tail_text = join_message_content(&head_tail_messages);
    let head_tail_tokens = approx_tokens_messages(&head_tail_messages);

    let response = compact_context(request)?;
    let compacted_text = join_message_content(&response.compacted_messages);
    let compacted_tokens = response.receipt.compacted_approx_tokens;

    let baselines = vec![
        score_answerability_text_baseline(
            "full",
            &full_text,
            full_tokens,
            &questions,
            &active_task,
        ),
        score_answerability_text_baseline(
            "head_tail",
            &head_tail_text,
            head_tail_tokens,
            &questions,
            &active_task,
        ),
        score_answerability_context_governor(
            &response,
            &compacted_text,
            compacted_tokens,
            &questions,
            &active_task,
        ),
    ];

    Ok(ReplayAnswerabilityReport {
        fixture_id,
        baselines,
        receipt_id: response.receipt.receipt_id,
        warnings: response.receipt.warnings,
    })
}

pub fn evaluate_replay_fixture(
    fixture_id: impl Into<String>,
    request: CompactRequest,
    max_probes: usize,
) -> Result<ReplayFixtureReport, ContextGovernorError> {
    let fixture_id = fixture_id.into();
    let original_message_count = request.messages.len();
    let probes = build_replay_probes(&request.messages, max_probes);
    let active_task = probes
        .iter()
        .find(|probe| probe.category == "active_task")
        .map(|probe| probe.needle.as_str())
        .unwrap_or("");

    let full_text = join_message_content(&request.messages);
    let full_tokens = approx_tokens_messages(&request.messages);
    let head_tail_messages = head_tail_messages(&request.messages, 1, 1);
    let head_tail_text = join_message_content(&head_tail_messages);
    let head_tail_tokens = approx_tokens_messages(&head_tail_messages);

    let response = compact_context(request)?;
    let compacted_text = join_message_content(&response.compacted_messages);
    let compacted_tokens = response.receipt.compacted_approx_tokens;

    let baselines = vec![
        score_text_baseline("full", &full_text, full_tokens, &probes, active_task),
        score_text_baseline(
            "head_tail",
            &head_tail_text,
            head_tail_tokens,
            &probes,
            active_task,
        ),
        score_context_governor(
            &response,
            &compacted_text,
            compacted_tokens,
            &probes,
            active_task,
        ),
    ];

    Ok(ReplayFixtureReport {
        fixture_id,
        original_message_count,
        compacted_message_count: response.compacted_messages.len(),
        probes,
        baselines,
        receipt_id: response.receipt.receipt_id,
        warnings: response.receipt.warnings,
    })
}

fn literal_probe_fragment(text: &str, max_chars: usize) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

fn push_probe(probes: &mut Vec<ReplayProbe>, category: &str, needle: String, max_probes: usize) {
    let mut needle = compact_preview(needle.trim(), 180);
    if let Some(stripped) = needle.strip_suffix("...") {
        needle = stripped.to_string();
    }
    if needle.len() < 4 || probes.len() >= max_probes {
        return;
    }
    if probes.iter().any(|probe| probe.needle == needle) {
        return;
    }
    probes.push(ReplayProbe {
        category: category.to_string(),
        needle,
    });
}

fn join_message_content(messages: &[Message]) -> String {
    messages
        .iter()
        .map(|message| message.content.as_str())
        .collect::<Vec<_>>()
        .join("\n")
}

fn head_tail_messages(messages: &[Message], head: usize, tail: usize) -> Vec<Message> {
    if messages.len() <= head + tail {
        return messages.to_vec();
    }
    messages
        .iter()
        .take(head)
        .chain(messages.iter().skip(messages.len().saturating_sub(tail)))
        .cloned()
        .collect()
}

fn text_contains_probe(text: &str, needle: &str) -> bool {
    text.contains(needle) || compact_preview(text, usize::MAX / 4).contains(needle)
}

fn score_text_baseline(
    name: &str,
    text: &str,
    tokens: usize,
    probes: &[ReplayProbe],
    active_task: &str,
) -> ReplayBaselineScore {
    let visible = probes
        .iter()
        .filter(|probe| text_contains_probe(text, &probe.needle))
        .count();
    build_score(
        name,
        visible,
        visible,
        probes.len(),
        tokens,
        text_contains_probe(text, active_task),
    )
}

fn score_answerability_text_baseline(
    name: &str,
    text: &str,
    tokens: usize,
    questions: &[ReplayAnswerabilityQuestion],
    active_task: &str,
) -> ReplayAnswerabilityBaselineScore {
    let answerable = questions
        .iter()
        .filter(|question| answerability_terms_present(text, question))
        .count();
    let incorrect_action_risk = questions
        .iter()
        .filter(|question| forbidden_terms_present(text, question))
        .count();
    build_answerability_score(
        name,
        answerable,
        questions.len(),
        incorrect_action_risk,
        tokens,
        text_contains_probe(text, active_task),
    )
}

fn score_answerability_context_governor(
    response: &CompactResponse,
    compacted_text: &str,
    tokens: usize,
    questions: &[ReplayAnswerabilityQuestion],
    active_task: &str,
) -> ReplayAnswerabilityBaselineScore {
    let answerable = questions
        .iter()
        .filter(|question| {
            question.expected_terms.iter().all(|term| {
                text_contains_probe(compacted_text, term)
                    || !context_search(response, term, 1, SearchScope::All).is_empty()
            })
        })
        .count();
    let incorrect_action_risk = questions
        .iter()
        .filter(|question| forbidden_terms_present(compacted_text, question))
        .count();
    build_answerability_score(
        "context_governor",
        answerable,
        questions.len(),
        incorrect_action_risk,
        tokens,
        text_contains_probe(compacted_text, active_task),
    )
}

fn answerability_terms_present(text: &str, question: &ReplayAnswerabilityQuestion) -> bool {
    question
        .expected_terms
        .iter()
        .all(|term| text_contains_probe(text, term))
}

fn forbidden_terms_present(text: &str, question: &ReplayAnswerabilityQuestion) -> bool {
    let text_l = text.to_lowercase();
    question.forbidden_terms.iter().any(|term| {
        let term_l = term.to_lowercase();
        let negated = text_l.contains(&format!("not {term_l}"))
            || text_l.contains(&format!("no {term_l}"))
            || text_l.contains(&format!("avoid {term_l}"));
        !negated && text_contains_probe(text, term)
    })
}

fn build_answerability_score(
    name: &str,
    answerable_questions: usize,
    total_questions: usize,
    incorrect_action_risk: usize,
    tokens: usize,
    active_task_visible: bool,
) -> ReplayAnswerabilityBaselineScore {
    let denominator = total_questions.max(1) as f64;
    ReplayAnswerabilityBaselineScore {
        name: name.to_string(),
        answerable_questions,
        total_questions,
        answerability_rate: answerable_questions as f64 / denominator,
        incorrect_action_risk,
        tokens,
        active_task_visible,
    }
}

fn score_context_governor(
    response: &CompactResponse,
    compacted_text: &str,
    tokens: usize,
    probes: &[ReplayProbe],
    active_task: &str,
) -> ReplayBaselineScore {
    let visible = probes
        .iter()
        .filter(|probe| text_contains_probe(compacted_text, &probe.needle))
        .count();
    let recoverable = probes
        .iter()
        .filter(|probe| !context_search(response, &probe.needle, 1, SearchScope::All).is_empty())
        .count();
    build_score(
        "context_governor",
        visible,
        recoverable,
        probes.len(),
        tokens,
        text_contains_probe(compacted_text, active_task),
    )
}

fn build_score(
    name: &str,
    visible_probes: usize,
    recoverable_probes: usize,
    total_probes: usize,
    tokens: usize,
    active_task_visible: bool,
) -> ReplayBaselineScore {
    let denominator = total_probes.max(1) as f64;
    ReplayBaselineScore {
        name: name.to_string(),
        visible_probes,
        recoverable_probes,
        total_probes,
        visible_rate: visible_probes as f64 / denominator,
        recoverable_rate: recoverable_probes as f64 / denominator,
        tokens,
        active_task_visible,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct MemoryArchiveRecordV1 {
    pub schema: String,
    pub receipt_id: String,
    pub item_id: String,
    pub content_blake3: String,
    pub content_kind: ContentKind,
    pub archive_reason: String,
    pub sensitivity: String,
    pub preview: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct MemoryArchiveOutcome {
    pub record_ids: Vec<String>,
    pub records_attempted: usize,
}

pub trait MemorySink {
    fn archive_context_record(
        &mut self,
        record: &MemoryArchiveRecordV1,
    ) -> Result<Option<String>, ContextGovernorError>;
}

pub fn archive_response_to_memory(
    response: &CompactResponse,
    sink: &mut dyn MemorySink,
) -> Result<MemoryArchiveOutcome, ContextGovernorError> {
    let mut outcome = MemoryArchiveOutcome::default();
    for item in &response.allocation_plan.items {
        let should_archive = response
            .allocation_plan
            .archived_item_ids
            .contains(&item.item_id)
            || matches!(
                item.authority_class,
                AuthorityClass::DurableMemoryCandidate | AuthorityClass::EvidenceCritical
            );
        if !should_archive {
            continue;
        }
        let Some(stored) = response
            .exact_store
            .iter()
            .find(|stored| stored.item_id == item.item_id)
        else {
            continue;
        };
        let record = MemoryArchiveRecordV1 {
            schema: "MemoryArchiveRecordV1".to_string(),
            receipt_id: response.receipt.receipt_id.clone(),
            item_id: item.item_id.clone(),
            content_blake3: stored.content_blake3.clone(),
            content_kind: item.content_kind.clone(),
            archive_reason: format!("semantic archive {:?}", item.item_type),
            sensitivity: "internal".to_string(),
            preview: content_aware_preview(&item.content_kind, &stored.content, 280),
        };
        outcome.records_attempted += 1;
        if let Some(id) = sink.archive_context_record(&record)? {
            outcome.record_ids.push(id);
        }
    }
    Ok(outcome)
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct StoredContextSearchHit {
    pub receipt_id: String,
    pub hit: ContextSearchHit,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileContextStoreStatusV1 {
    pub schema: String,
    pub root: String,
    pub receipt_count: usize,
    pub total_bytes: u64,
    pub stale_tmp_files_removed: usize,
    pub index_built: bool,
    pub searchable: bool,
    pub last_receipt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileContextStorePruneResultV1 {
    pub schema: String,
    pub kept_receipts: usize,
    pub removed_receipts: usize,
    pub total_bytes: u64,
    pub index_built: bool,
    pub last_receipt: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct FileContextStoreSaveResultV1 {
    pub schema: String,
    pub receipt_id: String,
    pub path: std::path::PathBuf,
    pub exact_recovery_state: ExactRecoveryStateV1,
    pub verified: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
struct PersistedReceiptIndexV1 {
    schema: String,
    receipt_ids: Vec<String>,
    token_to_receipts: BTreeMap<String, Vec<String>>,
}

#[derive(Debug, Clone)]
struct ReceiptFileInfo {
    receipt_id: String,
    path: std::path::PathBuf,
    created_utc: Option<DateTime<Utc>>,
}

#[derive(Debug, Clone)]
pub struct FileContextStore {
    root: std::path::PathBuf,
    // In-memory inverted index: token -> set of receipt_ids that contain it.
    // Built lazily on first search() call and invalidated on save().
    index: std::cell::RefCell<
        Option<std::collections::HashMap<String, std::collections::HashSet<String>>>,
    >,
}

impl FileContextStore {
    pub fn new(root: impl AsRef<std::path::Path>) -> Self {
        Self {
            root: root.as_ref().to_path_buf(),
            index: std::cell::RefCell::new(None),
        }
    }

    pub fn save(
        &self,
        response: &CompactResponse,
    ) -> Result<std::path::PathBuf, ContextGovernorError> {
        Ok(self.save_with_status(response)?.path)
    }

    pub fn save_with_status(
        &self,
        response: &CompactResponse,
    ) -> Result<FileContextStoreSaveResultV1, ContextGovernorError> {
        std::fs::create_dir_all(&self.root)?;
        let path = self.path_for_receipt(&response.receipt.receipt_id);
        let tmp_path = path.with_extension("json.tmp");
        let mut persisted = response.clone();
        let exact_recovered = !persisted.exact_store.is_empty();
        persisted.receipt.summary_loss_report.exact_recovery_state = if exact_recovered {
            ExactRecoveryStateV1::Persisted
        } else {
            ExactRecoveryStateV1::Unavailable
        };
        persisted.receipt.recovery_durability = if exact_recovered {
            RecoveryDurabilityV1::Persisted
        } else {
            RecoveryDurabilityV1::Unavailable
        };
        let json = serde_json::to_string_pretty(&persisted)?;
        std::fs::write(&tmp_path, json)?;
        std::fs::rename(&tmp_path, &path)?;
        // Invalidate the in-memory index so the next search rebuilds it
        *self.index.borrow_mut() = None;
        self.rebuild_and_persist_index()?;
        let verified = self.load(&response.receipt.receipt_id).map(|loaded| {
            loaded.receipt.summary_loss_report.exact_recovery_state
                == persisted.receipt.summary_loss_report.exact_recovery_state
                && loaded.exact_store == persisted.exact_store
        })?;
        Ok(FileContextStoreSaveResultV1 {
            schema: "FileContextStoreSaveResultV1".to_string(),
            receipt_id: response.receipt.receipt_id.clone(),
            path,
            exact_recovery_state: persisted.receipt.summary_loss_report.exact_recovery_state,
            verified,
        })
    }

    pub fn load(&self, receipt_id: &str) -> Result<CompactResponse, ContextGovernorError> {
        let path = self.path_for_receipt(receipt_id);
        if !path.exists() {
            return Err(ContextGovernorError::ReceiptNotFound(
                receipt_id.to_string(),
            ));
        }
        let json = std::fs::read_to_string(path)?;
        Ok(serde_json::from_str(&json)?)
    }

    pub fn list_receipts(&self) -> Result<Vec<String>, ContextGovernorError> {
        Ok(self
            .scan_receipts()?
            .0
            .into_iter()
            .map(|info| info.receipt_id)
            .collect())
    }

    pub fn status(&self) -> Result<FileContextStoreStatusV1, ContextGovernorError> {
        let (receipts, total_bytes, stale_tmp_files_removed) = self.scan_receipts()?;
        let index_built =
            self.index.borrow().is_some() || self.persisted_index_matches(&receipts)?;
        Ok(FileContextStoreStatusV1 {
            schema: "FileContextStoreStatusV1".to_string(),
            root: self.root.display().to_string(),
            receipt_count: receipts.len(),
            total_bytes,
            stale_tmp_files_removed,
            index_built,
            searchable: index_built || receipts.is_empty(),
            last_receipt: receipts.last().map(|info| info.receipt_id.clone()),
        })
    }

    pub fn prune_receipts_keep_last(
        &self,
        keep_last: usize,
    ) -> Result<FileContextStorePruneResultV1, ContextGovernorError> {
        let (receipts, _, _) = self.scan_receipts()?;
        let remove_count = receipts.len().saturating_sub(keep_last);
        for info in receipts.iter().take(remove_count) {
            std::fs::remove_file(&info.path)?;
        }
        *self.index.borrow_mut() = None;
        self.rebuild_and_persist_index()?;
        let status = self.status()?;
        Ok(FileContextStorePruneResultV1 {
            schema: "FileContextStorePruneResultV1".to_string(),
            kept_receipts: status.receipt_count,
            removed_receipts: remove_count,
            total_bytes: status.total_bytes,
            index_built: status.index_built,
            last_receipt: status.last_receipt,
        })
    }

    fn scan_receipts(&self) -> Result<(Vec<ReceiptFileInfo>, u64, usize), ContextGovernorError> {
        if !self.root.exists() {
            return Ok((Vec::new(), 0, 0));
        }
        let mut ids = Vec::new();
        let mut total_bytes = 0u64;
        let mut stale_tmp_files_removed = 0usize;
        for entry in std::fs::read_dir(&self.root)? {
            let entry = entry?;
            let path = entry.path();
            // Clean up stale .tmp files from interrupted save() calls
            if path.extension().and_then(|ext| ext.to_str()) == Some("tmp") {
                if std::fs::remove_file(&path).is_ok() {
                    stale_tmp_files_removed += 1;
                }
                continue;
            }
            if path.extension().and_then(|ext| ext.to_str()) != Some("json") {
                continue;
            }
            if path.file_name().and_then(|name| name.to_str()) == Some(".receipt-index.json") {
                continue;
            }
            let Some(stem) = path.file_stem().and_then(|stem| stem.to_str()) else {
                continue;
            };
            if let Ok(metadata) = entry.metadata() {
                total_bytes = total_bytes.saturating_add(metadata.len());
                let created_utc = std::fs::read_to_string(&path)
                    .ok()
                    .and_then(|json| serde_json::from_str::<CompactResponse>(&json).ok())
                    .map(|response| response.receipt.created_utc);
                ids.push(ReceiptFileInfo {
                    receipt_id: stem.to_string(),
                    path,
                    created_utc,
                });
            }
        }
        ids.sort_by(|a, b| {
            a.created_utc
                .cmp(&b.created_utc)
                .then_with(|| a.receipt_id.cmp(&b.receipt_id))
        });
        Ok((ids, total_bytes, stale_tmp_files_removed))
    }

    pub fn expand(
        &self,
        receipt_id: &str,
        item_id: &str,
        max_chars: usize,
    ) -> Result<ContextExpandResult, ContextGovernorError> {
        let response = self.load(receipt_id)?;
        context_expand(&response, item_id, max_chars)
            .ok_or_else(|| ContextGovernorError::ReceiptNotFound(item_id.to_string()))
    }

    /// Build an in-memory inverted index from all stored receipts.
    /// Tokenizes content on whitespace and stores receipt_id sets per token.
    fn build_index(
        &self,
    ) -> Result<
        std::collections::HashMap<String, std::collections::HashSet<String>>,
        ContextGovernorError,
    > {
        let mut idx: std::collections::HashMap<String, std::collections::HashSet<String>> =
            std::collections::HashMap::new();
        for receipt_id in self.list_receipts()? {
            let response = self.load(&receipt_id)?;
            // Index exact_store content
            for item in &response.exact_store {
                for token in item.content.split_whitespace() {
                    let token_l = token.to_lowercase();
                    idx.entry(token_l).or_default().insert(receipt_id.clone());
                }
            }
            // Index compacted_messages content
            for msg in &response.compacted_messages {
                for token in msg.content.split_whitespace() {
                    let token_l = token.to_lowercase();
                    idx.entry(token_l).or_default().insert(receipt_id.clone());
                }
            }
        }
        Ok(idx)
    }

    fn rebuild_and_persist_index(&self) -> Result<(), ContextGovernorError> {
        let index = self.build_index()?;
        self.persist_index(&index)?;
        *self.index.borrow_mut() = Some(index);
        Ok(())
    }

    fn persisted_index_matches(
        &self,
        receipts: &[ReceiptFileInfo],
    ) -> Result<bool, ContextGovernorError> {
        let Some(index) = self.load_persisted_index()? else {
            return Ok(false);
        };
        let receipt_ids = receipts
            .iter()
            .map(|info| info.receipt_id.clone())
            .collect::<Vec<_>>();
        Ok(index.receipt_ids == receipt_ids)
    }

    fn load_persisted_index(
        &self,
    ) -> Result<Option<PersistedReceiptIndexV1>, ContextGovernorError> {
        let path = self.index_path();
        if !path.exists() {
            return Ok(None);
        }
        let json = std::fs::read_to_string(path)?;
        Ok(Some(serde_json::from_str(&json)?))
    }

    fn persist_index(
        &self,
        index: &std::collections::HashMap<String, std::collections::HashSet<String>>,
    ) -> Result<(), ContextGovernorError> {
        std::fs::create_dir_all(&self.root)?;
        let receipt_ids = self.list_receipts()?;
        let token_to_receipts = index
            .iter()
            .map(|(token, receipt_set)| {
                let mut receipts = receipt_set.iter().cloned().collect::<Vec<_>>();
                receipts.sort();
                (token.clone(), receipts)
            })
            .collect::<BTreeMap<_, _>>();
        let persisted = PersistedReceiptIndexV1 {
            schema: "PersistedReceiptIndexV1".to_string(),
            receipt_ids,
            token_to_receipts,
        };
        let path = self.index_path();
        let tmp_path = path.with_extension("json.tmp");
        std::fs::write(&tmp_path, serde_json::to_string_pretty(&persisted)?)?;
        std::fs::rename(tmp_path, path)?;
        Ok(())
    }

    fn index_from_persisted(
        persisted: PersistedReceiptIndexV1,
    ) -> std::collections::HashMap<String, std::collections::HashSet<String>> {
        persisted
            .token_to_receipts
            .into_iter()
            .map(|(token, receipts)| (token, receipts.into_iter().collect()))
            .collect()
    }

    /// Ensure the index is built and return a reference to it.
    fn with_index<F, R>(&self, f: F) -> Result<R, ContextGovernorError>
    where
        F: FnOnce(&std::collections::HashMap<String, std::collections::HashSet<String>>) -> R,
    {
        let mut guard = self.index.borrow_mut();
        if guard.is_none() {
            let receipts = self.scan_receipts()?.0;
            if let Some(persisted) = self.load_persisted_index()? {
                let receipt_ids = receipts
                    .iter()
                    .map(|info| info.receipt_id.clone())
                    .collect::<Vec<_>>();
                if persisted.receipt_ids == receipt_ids {
                    *guard = Some(Self::index_from_persisted(persisted));
                }
            }
            if guard.is_none() {
                let index = self.build_index()?;
                self.persist_index(&index)?;
                *guard = Some(index);
            }
        }
        Ok(f(guard.as_ref().unwrap()))
    }

    pub fn search(
        &self,
        query: &str,
        top_k: usize,
        scope: SearchScope,
    ) -> Result<Vec<StoredContextSearchHit>, ContextGovernorError> {
        // Use the inverted index to find candidate receipts that contain
        // any query token, then do full search only on those candidates.
        // If the index returns no candidates (e.g. the query is a substring
        // of a token, not a full token), fall back to searching all receipts.
        let query_tokens: Vec<String> =
            query.split_whitespace().map(|t| t.to_lowercase()).collect();
        let candidate_ids: Vec<String> = if query_tokens.is_empty() {
            self.list_receipts()?
        } else {
            let candidates = self.with_index(|idx| {
                let mut candidates: std::collections::HashSet<String> =
                    std::collections::HashSet::new();
                for token in &query_tokens {
                    if let Some(receipt_ids) = idx.get(token) {
                        for rid in receipt_ids {
                            candidates.insert(rid.clone());
                        }
                    }
                }
                let mut sorted: Vec<String> = candidates.into_iter().collect();
                sorted.sort();
                sorted
            })?;
            if candidates.is_empty() {
                // No exact token matches — fall back to full scan so that
                // substring queries (e.g. "NEEDLE" in "NEEDLE_PARSER") still work.
                self.list_receipts()?
            } else {
                candidates
            }
        };

        let mut out = Vec::new();
        for receipt_id in candidate_ids {
            let response = self.load(&receipt_id)?;
            for hit in context_search(&response, query, top_k, scope) {
                out.push(StoredContextSearchHit {
                    receipt_id: receipt_id.clone(),
                    hit,
                });
                if out.len() >= top_k {
                    return Ok(out);
                }
            }
        }
        Ok(out)
    }

    fn path_for_receipt(&self, receipt_id: &str) -> std::path::PathBuf {
        let safe = receipt_id
            .chars()
            .map(|ch| {
                if ch.is_ascii_alphanumeric() || ch == '_' || ch == '-' {
                    ch
                } else {
                    '_'
                }
            })
            .collect::<String>();
        self.root.join(format!("{safe}.json"))
    }

    fn index_path(&self) -> std::path::PathBuf {
        self.root.join(".receipt-index.json")
    }
}
