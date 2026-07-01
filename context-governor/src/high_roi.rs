use crate::hash_text;
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::BTreeSet;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord, Default)]
#[serde(rename_all = "snake_case")]
pub enum RiskLevel {
    #[default]
    Low,
    Medium,
    High,
    Critical,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum GovernanceFailureMode {
    UnauthorizedLeakage,
    StalePropagation,
    ContradictionPersistence,
    ProvenanceCollapse,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernanceCase {
    pub case_id: String,
    pub mode: GovernanceFailureMode,
    pub passed: bool,
}

impl GovernanceCase {
    pub fn new(case_id: impl Into<String>, mode: GovernanceFailureMode, passed: bool) -> Self {
        Self {
            case_id: case_id.into(),
            mode,
            passed,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct GovernedMemoryHarnessReceiptV1 {
    pub schema: String,
    pub harness_id: String,
    pub total_cases: usize,
    pub passed_cases: usize,
    pub failed_cases: usize,
    pub failure_modes: Vec<GovernanceFailureMode>,
    pub certified: bool,
    pub reasons: Vec<String>,
}

pub fn evaluate_governed_memory(
    harness_id: impl Into<String>,
    cases: &[GovernanceCase],
) -> GovernedMemoryHarnessReceiptV1 {
    let mut failure_modes = cases
        .iter()
        .filter(|case| !case.passed)
        .map(|case| case.mode)
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect::<Vec<_>>();
    failure_modes.sort();
    let passed_cases = cases.iter().filter(|case| case.passed).count();
    let failed_cases = cases.len().saturating_sub(passed_cases);
    let mut reasons = Vec::new();
    for mode in &failure_modes {
        reasons.push(format!("failed-{mode:?}").to_lowercase());
    }
    GovernedMemoryHarnessReceiptV1 {
        schema: "GovernedMemoryHarnessReceiptV1".to_string(),
        harness_id: harness_id.into(),
        total_cases: cases.len(),
        passed_cases,
        failed_cases,
        failure_modes,
        certified: failed_cases == 0 && !cases.is_empty(),
        reasons,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolManifestEntry {
    pub name: String,
    pub description: String,
}

impl ToolManifestEntry {
    pub fn new(name: impl Into<String>, description: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ToolSurfaceFinding {
    pub reason: String,
    pub risk: RiskLevel,
    pub affected_tools: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct McpToolSurfaceAuditV1 {
    pub schema: String,
    pub tool_count: usize,
    pub combined_surface_risk: RiskLevel,
    pub findings: Vec<ToolSurfaceFinding>,
}

pub fn audit_mcp_tool_surface(tools: &[ToolManifestEntry]) -> McpToolSurfaceAuditV1 {
    let mut findings = Vec::new();
    for (left_idx, left) in tools.iter().enumerate() {
        for right in tools.iter().skip(left_idx + 1) {
            let combined = format!("{} {}", left.description, right.description).to_lowercase();
            let left_l = left.description.to_lowercase();
            let right_l = right.description.to_lowercase();
            let split_payload = contains_any(&combined, &["ignore previous", "instructions"])
                && contains_any(&combined, &["share a", "share b", "threshold", "combined"])
                && left_l != right_l
                && (contains_any(&left_l, &["ignore previous", "share a", "threshold"])
                    || contains_any(&right_l, &["ignore previous", "share a", "threshold"]))
                && (contains_any(&left_l, &["instructions", "share b", "combined"])
                    || contains_any(&right_l, &["instructions", "share b", "combined"]));
            if split_payload {
                findings.push(ToolSurfaceFinding {
                    reason: "split-instruction-fragments-across-tools".to_string(),
                    risk: RiskLevel::High,
                    affected_tools: vec![left.name.clone(), right.name.clone()],
                });
            }
        }
    }
    let combined_surface_risk = findings
        .iter()
        .map(|finding| finding.risk)
        .max()
        .unwrap_or(RiskLevel::Low);
    McpToolSurfaceAuditV1 {
        schema: "McpToolSurfaceAuditV1".to_string(),
        tool_count: tools.len(),
        combined_surface_risk,
        findings,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct BoundaryFinding {
    pub reason: String,
    pub risk: RiskLevel,
    pub location: String,
    pub snippet: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CompressionBoundaryAuditV1 {
    pub schema: String,
    pub source_findings: Vec<BoundaryFinding>,
    pub summary_findings: Vec<BoundaryFinding>,
    pub relinking_risk: RiskLevel,
    pub safe_to_reinject: bool,
}

pub fn audit_compression_boundary(
    source_fragments: &[String],
    compressed_summary: &str,
) -> CompressionBoundaryAuditV1 {
    let source_findings = source_fragments
        .iter()
        .enumerate()
        .flat_map(|(idx, fragment)| {
            scan_instruction_boundary(fragment, false, format!("source[{idx}]"))
        })
        .collect::<Vec<_>>();
    let summary_findings =
        scan_instruction_boundary(compressed_summary, true, "compressed_summary".to_string());
    let relinking_risk = if summary_findings
        .iter()
        .any(|finding| finding.risk >= RiskLevel::High)
    {
        RiskLevel::High
    } else if !source_findings.is_empty() {
        RiskLevel::Medium
    } else {
        RiskLevel::Low
    };
    CompressionBoundaryAuditV1 {
        schema: "CompressionBoundaryAuditV1".to_string(),
        source_findings,
        summary_findings,
        relinking_risk,
        safe_to_reinject: relinking_risk < RiskLevel::High,
    }
}

fn scan_instruction_boundary(
    text: &str,
    post_compression: bool,
    location: String,
) -> Vec<BoundaryFinding> {
    let lower = text.to_lowercase();
    let mut findings = Vec::new();
    if contains_any(
        &lower,
        &[
            "ignore previous",
            "disregard previous",
            "run the release command",
            "execute the command",
        ],
    ) {
        findings.push(BoundaryFinding {
            reason: if post_compression {
                "post-compression-instruction".to_string()
            } else {
                "source-instruction-fragment".to_string()
            },
            risk: if post_compression {
                RiskLevel::High
            } else {
                RiskLevel::Medium
            },
            location,
            snippet: preview(text, 160),
        });
    }
    findings
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RagEvalInput {
    pub task_id: String,
    pub closed_book_correct: bool,
    pub retrieved_answer_correct: bool,
    pub retrieval_used: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RagLeakageReceiptV1 {
    pub schema: String,
    pub task_id: String,
    pub certified_retrieval_gain: bool,
    pub reasons: Vec<String>,
}

pub fn evaluate_leakage_free_rag(input: RagEvalInput) -> RagLeakageReceiptV1 {
    let mut reasons = Vec::new();
    if input.closed_book_correct {
        reasons.push("closed-book-solved-task".to_string());
    }
    if !input.retrieval_used {
        reasons.push("retrieval-not-used".to_string());
    }
    if !input.retrieved_answer_correct {
        reasons.push("retrieved-answer-incorrect".to_string());
    }
    let certified_retrieval_gain =
        !input.closed_book_correct && input.retrieval_used && input.retrieved_answer_correct;
    if certified_retrieval_gain {
        reasons.push("retrieval-improved-unsolved-task".to_string());
    }
    RagLeakageReceiptV1 {
        schema: "RagLeakageReceiptV1".to_string(),
        task_id: input.task_id,
        certified_retrieval_gain,
        reasons,
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct EvidenceClaim {
    pub id: String,
    pub text: String,
}

impl EvidenceClaim {
    pub fn new(id: impl Into<String>, text: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            text: text.into(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConflictFinding {
    pub claim_a_id: String,
    pub claim_b_id: String,
    pub reason: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ConflictScreenReportV1 {
    pub schema: String,
    pub needs_expensive_review: bool,
    pub conflicts: Vec<ConflictFinding>,
}

pub fn screen_knowledge_conflicts(claims: &[EvidenceClaim]) -> ConflictScreenReportV1 {
    let mut conflicts = Vec::new();
    for (left_idx, left) in claims.iter().enumerate() {
        for right in claims.iter().skip(left_idx + 1) {
            let left_nums = numbers(&left.text);
            let right_nums = numbers(&right.text);
            if !left_nums.is_empty() && !right_nums.is_empty() && left_nums != right_nums {
                conflicts.push(ConflictFinding {
                    claim_a_id: left.id.clone(),
                    claim_b_id: right.id.clone(),
                    reason: "numeric-disagreement".to_string(),
                });
            }
            if negation_conflict(&left.text, &right.text) {
                conflicts.push(ConflictFinding {
                    claim_a_id: left.id.clone(),
                    claim_b_id: right.id.clone(),
                    reason: "negation-disagreement".to_string(),
                });
            }
        }
    }
    ConflictScreenReportV1 {
        schema: "ConflictScreenReportV1".to_string(),
        needs_expensive_review: !conflicts.is_empty(),
        conflicts,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum RetrievalRoute {
    FlatSearchOnly,
    GraphAssisted,
    ConflictAware,
    Synthesis,
    Temporal,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RetrievalRouteDecisionV1 {
    pub schema: String,
    pub route: RetrievalRoute,
    pub reasons: Vec<String>,
}

pub fn select_retrieval_route(query: &str) -> RetrievalRouteDecisionV1 {
    let lower = query.to_lowercase();
    let (route, reason) = if contains_any(
        &lower,
        &[
            "contradict",
            "conflict",
            "vs",
            "is it true",
            "but ",
            "compared",
        ],
    ) {
        (RetrievalRoute::ConflictAware, "conflict-sensitive")
    } else if contains_any(
        &lower,
        &[
            "summarize",
            "overview",
            "all themes",
            "landscape",
            "everything",
        ],
    ) {
        (RetrievalRoute::Synthesis, "synthesis")
    } else if contains_any(
        &lower,
        &["changed", "after", "before", "when", "current vs", "latest"],
    ) {
        (RetrievalRoute::Temporal, "temporal")
    } else if contains_any(
        &lower,
        &["connect", "relate", "depends", "between", "lead to"],
    ) {
        (RetrievalRoute::GraphAssisted, "multi-hop")
    } else {
        (RetrievalRoute::FlatSearchOnly, "simple-lookup")
    };
    RetrievalRouteDecisionV1 {
        schema: "RetrievalRouteDecisionV1".to_string(),
        route,
        reasons: vec![reason.to_string()],
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq, PartialOrd, Ord)]
#[serde(rename_all = "snake_case")]
pub enum MemoryModule {
    Representation,
    Organization,
    RetrievalUpdate,
    LifecycleGovernance,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MemoryModuleMetric {
    pub module: MemoryModule,
    pub score: f64,
    pub observations: usize,
}

impl MemoryModuleMetric {
    pub fn new(module: MemoryModule, score: f64, observations: usize) -> Self {
        Self {
            module,
            score,
            observations,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct AgentMemoryModuleReportV1 {
    pub schema: String,
    pub ready_for_public_claims: bool,
    pub missing_modules: Vec<MemoryModule>,
    pub min_score: f64,
    pub total_observations: usize,
}

pub fn evaluate_agent_memory_modules(metrics: &[MemoryModuleMetric]) -> AgentMemoryModuleReportV1 {
    let present = metrics
        .iter()
        .map(|metric| metric.module)
        .collect::<BTreeSet<_>>();
    let required = [
        MemoryModule::Representation,
        MemoryModule::Organization,
        MemoryModule::RetrievalUpdate,
        MemoryModule::LifecycleGovernance,
    ];
    let missing_modules = required
        .into_iter()
        .filter(|module| !present.contains(module))
        .collect::<Vec<_>>();
    let min_score = metrics
        .iter()
        .map(|metric| metric.score)
        .fold(1.0_f64, f64::min);
    let total_observations = metrics.iter().map(|metric| metric.observations).sum();
    AgentMemoryModuleReportV1 {
        schema: "AgentMemoryModuleReportV1".to_string(),
        ready_for_public_claims: missing_modules.is_empty()
            && min_score >= 0.5
            && total_observations >= required.len(),
        missing_modules,
        min_score,
        total_observations,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum InferenceSurface {
    HostedApi,
    LocalInference,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TokenImportance {
    pub token_id: String,
    pub importance: f64,
}

impl TokenImportance {
    pub fn new(token_id: impl Into<String>, importance: f64) -> Self {
        Self {
            token_id: token_id.into(),
            importance,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct SemanticKvRetentionPlanV1 {
    pub schema: String,
    pub surface: InferenceSurface,
    pub retained_token_ids: Vec<String>,
    pub boundary_notes: Vec<String>,
}

pub fn plan_semantic_kv_retention(
    tokens: &[TokenImportance],
    retain_count: usize,
    surface: InferenceSurface,
) -> SemanticKvRetentionPlanV1 {
    let mut ranked = tokens.to_vec();
    ranked.sort_by(|a, b| {
        b.importance
            .partial_cmp(&a.importance)
            .unwrap_or(Ordering::Equal)
            .then_with(|| a.token_id.cmp(&b.token_id))
    });
    let retained_token_ids = ranked
        .into_iter()
        .take(retain_count)
        .map(|token| token.token_id)
        .collect::<Vec<_>>();
    let mut boundary_notes = Vec::new();
    if surface == InferenceSurface::HostedApi {
        boundary_notes.push(
            "hosted APIs do not expose KV cache; use prompt/retrieval compaction instead"
                .to_string(),
        );
    }
    SemanticKvRetentionPlanV1 {
        schema: "SemanticKvRetentionPlanV1".to_string(),
        surface,
        retained_token_ids,
        boundary_notes,
    }
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionKind {
    TemporalTimeline,
    EntityView,
    CommunitySummary,
}

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum ProjectionFreshness {
    Fresh,
    Stale,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ProjectionReceiptV1 {
    pub schema: String,
    pub projection_id: String,
    pub kind: ProjectionKind,
    pub source_ids: Vec<String>,
    pub derivation_blake3: String,
    pub freshness: ProjectionFreshness,
}

pub fn build_projection_receipt(
    kind: ProjectionKind,
    source_ids: &[String],
    projection_content: &str,
    freshness: ProjectionFreshness,
) -> ProjectionReceiptV1 {
    let mut hash_material = source_ids.join("\n");
    hash_material.push('\n');
    hash_material.push_str(projection_content);
    let derivation_blake3 = hash_text(&hash_material);
    ProjectionReceiptV1 {
        schema: "ProjectionReceiptV1".to_string(),
        projection_id: format!("projection:{}", &derivation_blake3[..16]),
        kind,
        source_ids: source_ids.to_vec(),
        derivation_blake3,
        freshness,
    }
}

fn contains_any(text: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| text.contains(needle))
}

fn preview(text: &str, max_chars: usize) -> String {
    text.split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
        .chars()
        .take(max_chars)
        .collect()
}

fn numbers(text: &str) -> Vec<i64> {
    let mut out = Vec::new();
    let mut cur = String::new();
    for ch in text.chars() {
        if ch.is_ascii_digit() {
            cur.push(ch);
        } else if !cur.is_empty() {
            if let Ok(n) = cur.parse::<i64>() {
                out.push(n);
            }
            cur.clear();
        }
    }
    if !cur.is_empty() {
        if let Ok(n) = cur.parse::<i64>() {
            out.push(n);
        }
    }
    out
}

fn negation_conflict(left: &str, right: &str) -> bool {
    let normalize = |text: &str| {
        text.to_lowercase()
            .replace(" not ", " ")
            .replace(" no ", " ")
            .replace(" unsupported", " supported")
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let left_l = left.to_lowercase();
    let right_l = right.to_lowercase();
    let left_neg = contains_any(&left_l, &[" not ", " no ", "unsupported"]);
    let right_neg = contains_any(&right_l, &[" not ", " no ", "unsupported"]);
    left_neg != right_neg && normalize(left) == normalize(right)
}
