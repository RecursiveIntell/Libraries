//! Daemon facade for P11 queue, schedule, wake, leases, and safe mode.

use aidens_contracts::{
    ArtifactId, CanonicalToolSideEffectClass, DaemonNamespaceV1, JobV1, QueueHopReportV1,
    QueueLeaseV1, SafeModeReportV1,
};
use aidens_queue_kit::{
    DurableQueueLogV1, QueueEnqueueOutcomeV1, QueueError, QueueLeaseOutcomeV1, QueueSnapshotV1,
};
use aidens_schedule_kit::{one_shot_occurrence, ScheduleError};
use aidens_wake_kit::{wake_signal, WakeError};
use chrono::{DateTime, Utc};
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum DaemonError {
    #[error(transparent)]
    Queue(#[from] QueueError),
    #[error(transparent)]
    Schedule(#[from] ScheduleError),
    #[error(transparent)]
    Wake(#[from] WakeError),
}

#[derive(Debug, Clone)]
pub struct DaemonControllerV1 {
    queue: DurableQueueLogV1,
}

impl DaemonControllerV1 {
    pub fn namespace(
        root_path: impl AsRef<Path>,
        name: impl Into<String>,
        owner: impl Into<String>,
    ) -> DaemonNamespaceV1 {
        let root_path = root_path.as_ref().display().to_string();
        DaemonNamespaceV1::new(name, root_path, owner)
    }

    pub fn open(
        root_path: impl AsRef<Path>,
        namespace: DaemonNamespaceV1,
        owner: impl Into<String>,
    ) -> Result<Self, DaemonError> {
        Ok(Self {
            queue: DurableQueueLogV1::open(root_path, namespace, owner)?,
        })
    }

    pub fn open_read_only(
        root_path: impl AsRef<Path>,
        namespace: DaemonNamespaceV1,
    ) -> Result<Self, DaemonError> {
        Ok(Self {
            queue: DurableQueueLogV1::open_read_only(root_path, namespace)?,
        })
    }

    pub fn queue_root(&self) -> PathBuf {
        self.queue.root_path().to_path_buf()
    }

    pub fn namespace_id(&self) -> ArtifactId {
        self.queue.namespace().namespace_id.clone()
    }

    pub fn enqueue_schedule_occurrence(
        &self,
        schedule_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        due_at: DateTime<Utc>,
        payload: serde_json::Value,
        risk: CanonicalToolSideEffectClass,
    ) -> Result<QueueEnqueueOutcomeV1, DaemonError> {
        let occurrence = one_shot_occurrence(
            self.namespace_id(),
            schedule_id,
            occurrence_key,
            due_at,
            payload,
            risk,
        )?;
        Ok(self.queue.enqueue_from_schedule(occurrence)?)
    }

    pub fn enqueue_wake_signal(
        &self,
        source: impl Into<String>,
        signal_key: impl Into<String>,
        payload: serde_json::Value,
        risk: CanonicalToolSideEffectClass,
    ) -> Result<QueueEnqueueOutcomeV1, DaemonError> {
        let signal = wake_signal(self.namespace_id(), source, signal_key, payload, risk)?;
        Ok(self.queue.enqueue_from_wake(signal)?)
    }

    pub fn enqueue_job(&self, job: JobV1) -> Result<QueueEnqueueOutcomeV1, DaemonError> {
        Ok(self.queue.enqueue_job(job)?)
    }

    pub fn acquire_next(
        &self,
        owner: impl Into<String>,
        ttl_seconds: i64,
    ) -> Result<Option<QueueLeaseOutcomeV1>, DaemonError> {
        Ok(self.queue.acquire_next_lease(owner, ttl_seconds)?)
    }

    pub fn cancel(
        &self,
        job_id: &ArtifactId,
        reason: impl Into<String>,
    ) -> Result<QueueHopReportV1, DaemonError> {
        Ok(self.queue.cancel_job(job_id, reason)?)
    }

    pub fn complete(
        &self,
        job_id: &ArtifactId,
        lease: &QueueLeaseV1,
    ) -> Result<QueueHopReportV1, DaemonError> {
        Ok(self
            .queue
            .complete_job(job_id, Some(lease.lease_id.clone()))?)
    }

    pub fn set_safe_mode(
        &self,
        enabled: bool,
        reason: impl Into<String>,
    ) -> Result<SafeModeReportV1, DaemonError> {
        Ok(self.queue.set_safe_mode(enabled, reason)?)
    }

    pub fn drain(&self, reason: impl Into<String>) -> Result<Vec<QueueHopReportV1>, DaemonError> {
        Ok(self.queue.drain_non_terminal(reason)?)
    }

    pub fn snapshot(&self) -> Result<QueueSnapshotV1, DaemonError> {
        Ok(self.queue.snapshot()?)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use aidens_contracts::JobStateV1;

    fn temp_root(name: &str) -> PathBuf {
        let root =
            std::env::temp_dir().join(format!("aidens-daemon-p11-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    #[test]
    fn daemon_restart_preserves_jobs_and_cancelled_state() {
        let root = temp_root("restart");
        let ns = DaemonControllerV1::namespace(&root, "daemon-restart", "daemon-a");
        let daemon = DaemonControllerV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let outcome = daemon
            .enqueue_schedule_occurrence(
                "once",
                "occurrence-1",
                Utc::now(),
                serde_json::json!({"task":"work"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        let job = outcome.job.unwrap();
        daemon.cancel(&job.job_id, "operator-cancelled").unwrap();

        let restarted = DaemonControllerV1::open(&root, ns, "daemon-a").unwrap();
        let snapshot = restarted.snapshot().unwrap();
        assert_eq!(snapshot.jobs.len(), 1);
        assert_eq!(snapshot.jobs[0].state, JobStateV1::Cancelled);
        assert!(restarted.acquire_next("daemon-a", 30).unwrap().is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn repeated_schedule_occurrence_suppresses_duplicate_job() {
        let root = temp_root("duplicate");
        let ns = DaemonControllerV1::namespace(&root, "daemon-duplicate", "daemon-a");
        let daemon = DaemonControllerV1::open(&root, ns, "daemon-a").unwrap();
        let first = daemon
            .enqueue_schedule_occurrence(
                "once",
                "same-logical-occurrence",
                Utc::now(),
                serde_json::json!({"task":"work"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        let second = daemon
            .enqueue_schedule_occurrence(
                "once",
                "same-logical-occurrence",
                Utc::now(),
                serde_json::json!({"task":"work"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        assert!(first.enqueued);
        assert!(!second.enqueued);
        assert!(second.duplicate_suppression_receipt.is_some());
        assert_eq!(daemon.snapshot().unwrap().jobs.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn safe_mode_blocks_risky_wake_and_allows_drain() {
        let root = temp_root("safe");
        let ns = DaemonControllerV1::namespace(&root, "daemon-safe", "daemon-a");
        let daemon = DaemonControllerV1::open(&root, ns, "daemon-a").unwrap();
        daemon.set_safe_mode(true, "operator-safe-mode").unwrap();
        let blocked = daemon
            .enqueue_wake_signal(
                "filesystem",
                "dangerous-change",
                serde_json::json!({"cmd":"cargo test"}),
                CanonicalToolSideEffectClass::Admin,
            )
            .unwrap();
        assert!(!blocked.enqueued);
        assert!(blocked.safe_mode_receipt.is_some());

        daemon
            .enqueue_wake_signal(
                "filesystem",
                "inspect-change",
                serde_json::json!({"path":"README.md"}),
                CanonicalToolSideEffectClass::ReadOnly,
            )
            .unwrap();
        assert_eq!(daemon.snapshot().unwrap().jobs.len(), 1);
        assert_eq!(daemon.drain("safe-mode-drain").unwrap().len(), 1);
        assert_eq!(
            daemon.snapshot().unwrap().jobs[0].state,
            JobStateV1::Cancelled
        );
        let _ = std::fs::remove_dir_all(&root);
    }
}
