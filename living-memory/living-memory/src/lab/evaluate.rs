use std::collections::BTreeSet;

use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::exec::backend::CheckResult;
use crate::lab::suite::{EvalTask, TaskExpected};
use crate::runtime::novelty;
use crate::runtime::patch::types::StructuredPatch;
use crate::store::ForgeStore;

/// Score vector for an evaluation run.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct ScoreVector {
    pub correctness: f64,
    pub novelty: f64,
    pub stability: f64,
    pub weighted_total: f64,
    pub cea_confidence: Option<f64>,
    pub cea_predicted_correctness: Option<f64>,
}

/// Result of a single evaluation run.
#[derive(Debug)]
pub struct EvalRunResult {
    pub eval_id: String,
    pub candidate_id: String,
    pub task_id: String,
    pub scores: ScoreVector,
    pub check_result: Option<CheckResult>,
    pub patch_hash: String,
    pub structural_sig: String,
    pub mindstate_hash: String,
}

/// Compute raw correctness score from check results.
///
/// Returns a raw score in [0.0, 1.0]. Sub-weights are fixed:
/// fmt=0.10, clippy=0.30, test=0.60.
/// If a check is not required by the task (`TaskExpected`), full credit
/// is given for that sub-weight regardless of the actual result.
/// Task weights are NOT applied here — they are applied in `compute_scores`.
pub fn compute_correctness(check: &CheckResult, expected: &TaskExpected) -> f64 {
    let mut score = 0.0_f64;
    if !expected.require_fmt || check.fmt_pass {
        score += 0.10;
    }
    if !expected.require_clippy || check.clippy_pass {
        score += 0.30;
    }
    if !expected.require_tests || check.test_pass {
        score += 0.60;
    }
    score // in [0.0, 1.0]
}

/// Compute raw novelty score from strategy tags and recent traces.
///
/// Returns a raw score in [0.0, 1.0]. Task weights are NOT applied here.
pub fn compute_novelty_score(
    patch: &StructuredPatch,
    store: &ForgeStore,
    question_sig: &str,
    config: &ForgeConfig,
) -> ForgeResult<f64> {
    let tags = novelty::extract_strategy_tags(patch);

    // Get recent traces
    let traces = store.get_recent_traces_for_question(
        question_sig,
        config.novelty.min_traces_for_orthogonality,
    )?;

    let recent_tags_union: BTreeSet<String> = traces
        .iter()
        .flat_map(|t| {
            serde_json::from_str::<Vec<String>>(&t.strategy_tags_json).unwrap_or_default()
        })
        .collect();

    let tag_novelty = novelty::compute_tag_novelty(&tags, &recent_tags_union);

    // Anti-fluff guard: if tags are identical to last trace AND patch structure
    // matches, novelty is 0.
    if !traces.is_empty() {
        let last = &traces[0];
        let last_tags: Vec<String> =
            serde_json::from_str(&last.strategy_tags_json).unwrap_or_default();

        if tags == last_tags {
            // Compute a lightweight structural hash of this patch
            let current_patch_sig = {
                let mut h = blake3::Hasher::new();
                for edit in &patch.edits {
                    h.update(edit.path.to_string_lossy().as_bytes());
                    h.update(&(edit.ops.len() as u64).to_le_bytes());
                }
                h.finalize().to_hex().to_string()
            };
            // Compare structural sig to the stored one
            if current_patch_sig == last.structural_sig {
                return Ok(0.0);
            }
        }
    }

    Ok(tag_novelty)
}

/// Compute full score vector.
///
/// `correctness`, `novelty`, and `stability` are raw metrics in [0.0, 1.0].
/// `weighted_total` applies the task weights: w_c * correctness + w_n * novelty + w_s * stability.
/// Gates (`correctness_gate`, `promotion_min_suite_pass_rate`) apply to the raw metrics.
pub fn compute_scores(
    check: &CheckResult,
    patch: &StructuredPatch,
    task: &EvalTask,
    store: &ForgeStore,
    question_sig: &str,
    config: &ForgeConfig,
) -> ForgeResult<ScoreVector> {
    let correctness = compute_correctness(check, &task.expected);
    let novelty_raw = compute_novelty_score(patch, store, question_sig, config).unwrap_or(0.0);
    let stability = 0.0_f64; // Single-run default

    let w = &task.weights;
    let weighted_total =
        w.correctness * correctness + w.novelty * novelty_raw + w.stability * stability;

    Ok(ScoreVector {
        correctness,
        novelty: novelty_raw,
        stability,
        weighted_total,
        cea_confidence: None,
        cea_predicted_correctness: None,
    })
}

/// Persist an evaluation run to the store.
pub fn persist_eval_run(
    store: &ForgeStore,
    result: &EvalRunResult,
    backend: &str,
    violations_json: &str,
    logs_ref: &str,
    cea_run_hash: Option<&str>,
) -> ForgeResult<()> {
    let scores_json = serde_json::to_string(&result.scores)?;

    store.insert_eval_run(
        &result.eval_id,
        &result.candidate_id,
        &result.task_id,
        backend,
        0, // seed
        &result.mindstate_hash,
        &result.patch_hash,
        &result.structural_sig,
        &scores_json,
        violations_json,
        logs_ref,
        cea_run_hash,
    )
}
