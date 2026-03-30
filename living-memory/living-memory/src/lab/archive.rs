use crate::config::ForgeConfig;
use crate::error::ForgeResult;
use crate::lab::evaluate::ScoreVector;
use crate::runtime::novelty;
use crate::runtime::patch::types::StructuredPatch;
use crate::store::ForgeStore;

/// Result of an archive insertion attempt.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArchiveUpdate {
    Inserted,
    Replaced,
    BelowGate,
    NoChange,
}

/// Compute the cell key for a candidate.
pub fn compute_cell_key(
    score_vector: &ScoreVector,
    patch: &StructuredPatch,
    config: &ForgeConfig,
) -> String {
    let novelty_bin = compute_novelty_bin(score_vector.novelty, config);
    let stability_bin = compute_stability_bin(score_vector.stability, config);
    let tags = novelty::extract_strategy_tags(patch);
    let approach_family = novelty::determine_approach_family(&tags);

    format!("{novelty_bin}:{stability_bin}:{approach_family}")
}

/// Determine novelty bin from score.
fn compute_novelty_bin(novelty: f64, config: &ForgeConfig) -> String {
    for bin in &config.lab.archive.novelty_bins {
        if novelty >= bin.lo && novelty < bin.hi {
            return bin.name.clone();
        }
    }
    // Edge case: exactly 1.0
    if let Some(last) = config.lab.archive.novelty_bins.last() {
        if (novelty - last.hi).abs() < f64::EPSILON {
            return last.name.clone();
        }
    }
    "low".to_string()
}

/// Determine stability bin.
fn compute_stability_bin(_stability: f64, _config: &ForgeConfig) -> &'static str {
    // Single-run tasks use "stable" by default
    "stable"
}

/// Compare two candidates for archive dominance.
/// Uses fixed comparison weights — NOT the AlgebraSpec's operator_weights.
/// Rationale: archive ranking is cross-spec; it needs a stable comparator.
pub fn compute_score_summary(scores: &ScoreVector) -> f64 {
    // These weights are intentionally fixed for archive comparison consistency.
    // AlgebraSpec.operator_weights govern generation; these govern selection.
    0.70 * scores.correctness + 0.20 * scores.novelty + 0.10 * scores.stability
}

/// Insert a candidate into the archive if it qualifies.
pub fn archive_insert(
    store: &ForgeStore,
    candidate_id: &str,
    scores: &ScoreVector,
    patch: &StructuredPatch,
    config: &ForgeConfig,
    cea_fingerprint: Option<&str>,
) -> ForgeResult<ArchiveUpdate> {
    // Correctness gate
    if scores.correctness < config.lab.archive.correctness_gate {
        return Ok(ArchiveUpdate::BelowGate);
    }

    let cell_key = compute_cell_key(scores, patch, config);
    let score_summary = compute_score_summary(scores);

    // Check existing cell
    let existing = store.get_archive_cell(&cell_key)?;

    match existing {
        Some(cell) => {
            let existing_summary: f64 =
                serde_json::from_str(&cell.score_summary_json).unwrap_or(0.0);

            if score_summary > existing_summary {
                let summary_json = serde_json::to_string(&score_summary)?;
                store.upsert_archive_cell(
                    &cell_key,
                    candidate_id,
                    &summary_json,
                    cea_fingerprint,
                )?;
                Ok(ArchiveUpdate::Replaced)
            } else {
                Ok(ArchiveUpdate::NoChange)
            }
        }
        None => {
            let summary_json = serde_json::to_string(&score_summary)?;
            store.upsert_archive_cell(&cell_key, candidate_id, &summary_json, cea_fingerprint)?;
            Ok(ArchiveUpdate::Inserted)
        }
    }
}
