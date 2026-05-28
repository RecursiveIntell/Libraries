//! One-shot schedule occurrence construction for P11.

use aidens_contracts::{ArtifactId, CanonicalToolSideEffectClass, ScheduleOccurrenceV1};
use chrono::{DateTime, Utc};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ScheduleError {
    #[error("schedule id must not be empty")]
    EmptyScheduleId,
    #[error("occurrence key must not be empty")]
    EmptyOccurrenceKey,
}

pub fn one_shot_occurrence(
    namespace_id: ArtifactId,
    schedule_id: impl Into<String>,
    occurrence_key: impl Into<String>,
    due_at: DateTime<Utc>,
    payload: serde_json::Value,
    risk: CanonicalToolSideEffectClass,
) -> Result<ScheduleOccurrenceV1, ScheduleError> {
    let schedule_id = schedule_id.into();
    let occurrence_key = occurrence_key.into();
    if schedule_id.trim().is_empty() {
        return Err(ScheduleError::EmptyScheduleId);
    }
    if occurrence_key.trim().is_empty() {
        return Err(ScheduleError::EmptyOccurrenceKey);
    }
    Ok(ScheduleOccurrenceV1::new(
        namespace_id,
        schedule_id,
        occurrence_key,
        due_at,
        payload,
        risk,
    ))
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::DaemonNamespaceV1;

    #[test]
    fn occurrence_identity_includes_schedule_key_and_payload_digest() {
        let ns = DaemonNamespaceV1::new("schedule-test", "target/p11", "daemon");
        let occurrence = one_shot_occurrence(
            ns.namespace_id,
            "daily-refresh",
            "2026-04-27T00:00:00Z",
            Utc::now(),
            serde_json::json!({"task":"refresh"}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap();
        assert!(occurrence.identity_is_not_timestamp_only());
        assert!(occurrence.idempotency_key.contains("daily-refresh"));
        assert!(occurrence.idempotency_key.contains("2026-04-27T00:00:00Z"));
    }

    #[test]
    fn empty_occurrence_key_is_rejected_before_recurring_logic_exists() {
        let ns = DaemonNamespaceV1::new("schedule-test-empty", "target/p11", "daemon");
        let err = one_shot_occurrence(
            ns.namespace_id,
            "daily-refresh",
            "",
            Utc::now(),
            serde_json::json!({}),
            CanonicalToolSideEffectClass::ReadOnly,
        )
        .unwrap_err();
        assert!(matches!(err, ScheduleError::EmptyOccurrenceKey));
    }
}
