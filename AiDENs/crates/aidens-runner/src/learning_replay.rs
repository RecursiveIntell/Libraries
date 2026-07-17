//! Thin, non-authoritative retained-input replay projection for coding outcomes.
//!
//! Retrieval truth remains in `semantic-memory`: callers should use the store's
//! `search_replay_inputs_available`, `replay_search_from_stored_inputs`, or
//! `replay_search_receipt` APIs and retain the returned `SearchReplayReportV1`.
//! This module only compares caller-supplied canonical digests/results; it does
//! not persist inputs, replay results, or reinterpret the owner report.

#[cfg(not(test))]
use semantic_memory::{types::SearchReplayReportV1, MemoryStore};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayMode {
    NoReplay,
    StoreInputs,
    ReplayEvaluation,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReplayOutcome {
    ExactMatch,
    Mismatch,
    Drift,
    Inconclusive,
    NotAvailable,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CanonicalReplay {
    pub available: bool,
    pub inputs_stored: bool,
    pub input_digest: String,
    pub result_digest: String,
    pub verifier_digest: String,
    pub store_digest: String,
    pub environment_digest: String,
    /// Retained for provenance only; deliberately excluded from comparison.
    pub recorded_at: String,
}

impl CanonicalReplay {
    pub fn unavailable() -> Self {
        Self::empty(false, false)
    }
    pub fn inconclusive() -> Self {
        Self::empty(true, true)
    }
    pub fn stored_without_inputs() -> Self {
        Self::empty(true, false)
    }
    fn empty(available: bool, inputs_stored: bool) -> Self {
        Self {
            available,
            inputs_stored,
            input_digest: String::new(),
            result_digest: String::new(),
            verifier_digest: String::new(),
            store_digest: String::new(),
            environment_digest: String::new(),
            recorded_at: String::new(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ReplayComparison {
    pub outcome: ReplayOutcome,
    pub reason: &'static str,
}

pub fn compare(original: &CanonicalReplay, replay: &CanonicalReplay) -> ReplayComparison {
    if !original.available || !replay.available {
        return ReplayComparison {
            outcome: ReplayOutcome::NotAvailable,
            reason: "replay-not-available",
        };
    }
    if !original.inputs_stored || !replay.inputs_stored {
        return ReplayComparison {
            outcome: ReplayOutcome::Inconclusive,
            reason: "replay-inputs-not-stored",
        };
    }
    if original.input_digest.is_empty()
        || replay.input_digest.is_empty()
        || original.result_digest.is_empty()
        || replay.result_digest.is_empty()
    {
        return ReplayComparison {
            outcome: ReplayOutcome::Inconclusive,
            reason: "replay-evaluation-incomplete",
        };
    }
    if original.verifier_digest != replay.verifier_digest
        || original.store_digest != replay.store_digest
        || original.environment_digest != replay.environment_digest
    {
        return ReplayComparison {
            outcome: ReplayOutcome::Drift,
            reason: "verifier-store-or-environment-drift",
        };
    }
    if original.input_digest != replay.input_digest
        || original.result_digest != replay.result_digest
    {
        return ReplayComparison {
            outcome: ReplayOutcome::Mismatch,
            reason: "canonical-input-or-result-mismatch",
        };
    }
    ReplayComparison {
        outcome: ReplayOutcome::ExactMatch,
        reason: "canonical-semantic-match",
    }
}

/// Owner-API adapter: no local replay logic or report copy is introduced.
#[cfg(not(test))]
pub async fn replay_from_stored_inputs(
    store: &MemoryStore,
    receipt_id: &str,
) -> Result<SearchReplayReportV1, semantic_memory::MemoryError> {
    store.replay_search_from_stored_inputs(receipt_id).await
}

/// Owner-API adapter for caller-supplied retained inputs.
#[cfg(not(test))]
pub async fn replay_evaluation(
    store: &MemoryStore,
    receipt_id: &str,
    query: &str,
    top_k: Option<usize>,
) -> Result<SearchReplayReportV1, semantic_memory::MemoryError> {
    store
        .replay_search_receipt(receipt_id, query, top_k, None, None)
        .await
}

#[cfg(test)]
mod tests {
    use super::*;
    fn fixture() -> CanonicalReplay {
        CanonicalReplay {
            available: true,
            inputs_stored: true,
            input_digest: "i".into(),
            result_digest: "r".into(),
            verifier_digest: "v".into(),
            store_digest: "s".into(),
            environment_digest: "e".into(),
            recorded_at: "2026-01-01T00:00:00Z".into(),
        }
    }
    #[test]
    fn all_outcomes_are_classified() {
        assert_eq!(
            compare(&fixture(), &fixture()).outcome,
            ReplayOutcome::ExactMatch
        );
        let mut x = fixture();
        x.result_digest = "x".into();
        assert_eq!(compare(&fixture(), &x).outcome, ReplayOutcome::Mismatch);
        let mut x = fixture();
        x.environment_digest = "x".into();
        assert_eq!(compare(&fixture(), &x).outcome, ReplayOutcome::Drift);
        assert_eq!(
            compare(&fixture(), &CanonicalReplay::inconclusive()).outcome,
            ReplayOutcome::Inconclusive
        );
        assert_eq!(
            compare(&CanonicalReplay::unavailable(), &fixture()).outcome,
            ReplayOutcome::NotAvailable
        );
    }
    #[test]
    fn missing_inputs_are_inconclusive() {
        assert_eq!(ReplayMode::NoReplay, ReplayMode::NoReplay);
        assert_eq!(
            compare(&CanonicalReplay::stored_without_inputs(), &fixture()).outcome,
            ReplayOutcome::Inconclusive
        );
    }
    #[test]
    fn verifier_store_environment_drift() {
        for n in 0..3 {
            let mut x = fixture();
            if n == 0 {
                x.verifier_digest = "x".into()
            } else if n == 1 {
                x.store_digest = "x".into()
            } else {
                x.environment_digest = "x".into()
            }
            assert_eq!(compare(&fixture(), &x).outcome, ReplayOutcome::Drift);
        }
    }
    #[test]
    fn volatile_timestamps_are_ignored() {
        let mut x = fixture();
        x.recorded_at = "2099-01-01T00:00:00Z".into();
        assert_eq!(compare(&fixture(), &x).outcome, ReplayOutcome::ExactMatch);
    }
}
