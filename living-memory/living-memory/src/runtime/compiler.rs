use std::collections::BTreeMap;

use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::runtime::mindstate::{EvidenceItem, MindState, OrderedFloat, TraceRef};
use crate::store::ForgeStore;

/// Compile a MindState from inputs using a BasisVersion.
///
/// Pulls evidence from semantic-memory handle (read-only).
/// Injects answer traces for novelty context.
pub fn compile_mindstate(
    request: &str,
    repo_context: &str,
    basis_version_id: &str,
    evidence: Vec<EvidenceItem>,
    store: &ForgeStore,
    config: &ForgeConfig,
    config_overrides: BTreeMap<String, String>,
) -> ForgeResult<MindState> {
    // Compute question signature for trace lookup
    let question_sig = compute_question_sig(request, repo_context);

    // Retrieve recent answer traces for this question
    let trace_rows = store.get_recent_traces_for_question(
        &question_sig,
        config.novelty.min_traces_for_orthogonality,
    )?;

    let traces: Vec<TraceRef> = trace_rows
        .into_iter()
        .map(|row| {
            let tags: Vec<String> =
                serde_json::from_str(&row.strategy_tags_json).unwrap_or_default();
            let score: f64 = serde_json::from_str::<serde_json::Value>(&row.score_json)
                .ok()
                .and_then(|v| v.get("weighted_total").and_then(|w| w.as_f64()))
                .unwrap_or(0.0);
            TraceRef {
                question_sig: row.question_sig,
                strategy_tags: tags,
                score: OrderedFloat(score),
            }
        })
        .collect();

    // Budget evidence
    let budgeted_evidence =
        mindstate_core::budget_evidence(evidence, config.mindstate.evidence_budget);

    Ok(MindState {
        request: request.to_string(),
        repo_context: repo_context.to_string(),
        evidence: budgeted_evidence,
        traces,
        basis_version_id: basis_version_id.to_string(),
        config_overrides,
    })
}

/// Compute a stable question signature from request + repo context.
pub fn compute_question_sig(request: &str, repo_context: &str) -> String {
    mindstate_core::compute_question_sig(request, repo_context)
}
