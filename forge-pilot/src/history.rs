use chrono::Utc;
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetAttemptRecord {
    pub bundle_id: Option<String>,
    pub selected_at: String,
    pub outcome_signature: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetHistoryEntry {
    pub stable_target_key: String,
    pub prior_attempt_bundle_ids: Vec<String>,
    pub prior_outcomes: Vec<String>,
    pub last_selected_at: Option<String>,
    pub retry_count: u32,
    pub exhausted: bool,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PilotHistory {
    entries: BTreeMap<String, TargetHistoryEntry>,
}

impl PilotHistory {
    /// Returns the history entry recorded for a stable target key, if any.
    pub fn entry(&self, stable_target_key: &str) -> Option<&TargetHistoryEntry> {
        self.entries.get(stable_target_key)
    }

    /// Returns how many attempts have been recorded for a stable target key.
    pub fn retry_count(&self, stable_target_key: &str) -> u32 {
        self.entry(stable_target_key)
            .map(|entry| entry.retry_count)
            .unwrap_or(0)
    }

    /// Returns true when a stable target key has been marked exhausted.
    pub fn is_exhausted(&self, stable_target_key: &str) -> bool {
        self.entry(stable_target_key)
            .map(|entry| entry.exhausted)
            .unwrap_or(false)
    }

    /// Marks a stable target key as selected for another attempt.
    pub fn mark_selected(&mut self, stable_target_key: &str) {
        let now = Utc::now().to_rfc3339();
        let entry = self
            .entries
            .entry(stable_target_key.to_string())
            .or_insert_with(|| TargetHistoryEntry {
                stable_target_key: stable_target_key.to_string(),
                prior_attempt_bundle_ids: Vec::new(),
                prior_outcomes: Vec::new(),
                last_selected_at: None,
                retry_count: 0,
                exhausted: false,
            });
        entry.retry_count += 1;
        entry.last_selected_at = Some(now);
    }

    /// Records the outcome of an attempted action for a stable target key.
    pub fn record_outcome(
        &mut self,
        stable_target_key: &str,
        bundle_id: Option<String>,
        outcome_signature: String,
        max_retries_per_target: u32,
    ) {
        let entry = self
            .entries
            .entry(stable_target_key.to_string())
            .or_insert_with(|| TargetHistoryEntry {
                stable_target_key: stable_target_key.to_string(),
                prior_attempt_bundle_ids: Vec::new(),
                prior_outcomes: Vec::new(),
                last_selected_at: None,
                retry_count: 0,
                exhausted: false,
            });

        // LIB-MED-002: cap history vectors to prevent unbounded growth
        const MAX_HISTORY_ENTRIES: usize = 20;

        if let Some(bundle_id) = bundle_id {
            entry.prior_attempt_bundle_ids.push(bundle_id);
        }
        if entry.prior_attempt_bundle_ids.len() > MAX_HISTORY_ENTRIES {
            entry
                .prior_attempt_bundle_ids
                .drain(..entry.prior_attempt_bundle_ids.len() - MAX_HISTORY_ENTRIES);
        }

        let repeated = entry
            .prior_outcomes
            .last()
            .map(|last| last == &outcome_signature)
            .unwrap_or(false);
        entry.prior_outcomes.push(outcome_signature);
        if entry.prior_outcomes.len() > MAX_HISTORY_ENTRIES {
            entry
                .prior_outcomes
                .drain(..entry.prior_outcomes.len() - MAX_HISTORY_ENTRIES);
        }
        if entry.retry_count >= max_retries_per_target || repeated {
            entry.exhausted = true;
        }
    }

    /// Iterates over all recorded history entries keyed by stable target key.
    pub fn entries(&self) -> impl Iterator<Item = (&String, &TargetHistoryEntry)> {
        self.entries.iter()
    }
}
