//! Append-only queue substrate for P11 daemon execution.

use aidens_contracts::{
    ArtifactId, CanonicalToolSideEffectClass, DaemonNamespaceV1, DuplicateSuppressionReportV1,
    JobStateV1, JobV1, QueueHopKindV1, QueueHopReportV1, QueueLeaseV1, SafeModeOperationV1,
    SafeModeReportV1, ScheduleOccurrenceV1, WakeSignalV1,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{File, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration as StdDuration, Instant};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum QueueError {
    #[error("queue io error at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("queue json error at {path}: {source}")]
    Json {
        path: PathBuf,
        #[source]
        source: serde_json::Error,
    },
    #[error("queue writer owner mismatch: expected {expected}, got {actual}")]
    WriterOwnerMismatch { expected: String, actual: String },
    #[error("queue opened read-only")]
    ReadOnly,
    #[error("job not found: {0}")]
    JobNotFound(String),
    #[error("job is terminal and cannot transition: {0}")]
    TerminalJob(String),
    #[error("job completion requires an active lease: {0}")]
    LeaseRequired(String),
    #[error("job lease mismatch for {job_id}: expected {expected}, got {actual}")]
    LeaseMismatch {
        job_id: String,
        expected: String,
        actual: String,
    },
    #[error("job lease expired for {job_id}: {lease_id}")]
    LeaseExpired { job_id: String, lease_id: String },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueLogEntryV1 {
    pub sequence: u64,
    pub namespace_id: ArtifactId,
    pub job: Option<JobV1>,
    pub lease: Option<QueueLeaseV1>,
    pub safe_mode_enabled: Option<bool>,
    pub queue_hop_receipt: Option<QueueHopReportV1>,
    pub duplicate_suppression_receipt: Option<DuplicateSuppressionReportV1>,
    pub safe_mode_receipt: Option<SafeModeReportV1>,
    pub recorded_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueSnapshotV1 {
    pub namespace: DaemonNamespaceV1,
    pub jobs: Vec<JobV1>,
    pub leases: Vec<QueueLeaseV1>,
    pub safe_mode_enabled: bool,
    pub records_seen: usize,
}

impl QueueSnapshotV1 {
    pub fn job(&self, job_id: &ArtifactId) -> Option<&JobV1> {
        self.jobs.iter().find(|job| &job.job_id == job_id)
    }

    pub fn logical_job_count_for(&self, idempotency_key: &str) -> usize {
        self.jobs
            .iter()
            .filter(|job| job.idempotency_key == idempotency_key)
            .count()
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueEnqueueOutcomeV1 {
    pub enqueued: bool,
    pub job: Option<JobV1>,
    pub existing_job: Option<JobV1>,
    pub queue_hop_receipt: QueueHopReportV1,
    pub duplicate_suppression_receipt: Option<DuplicateSuppressionReportV1>,
    pub safe_mode_receipt: Option<SafeModeReportV1>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueLeaseOutcomeV1 {
    pub job: JobV1,
    pub lease: QueueLeaseV1,
    pub queue_hop_receipt: QueueHopReportV1,
}

#[derive(Debug, Clone)]
pub struct DurableQueueLogV1 {
    namespace: DaemonNamespaceV1,
    root_path: PathBuf,
    log_path: PathBuf,
    writer_owner: Option<String>,
}

impl DurableQueueLogV1 {
    pub fn open(
        root_path: impl AsRef<Path>,
        namespace: DaemonNamespaceV1,
        writer_owner: impl Into<String>,
    ) -> Result<Self, QueueError> {
        let writer_owner = writer_owner.into();
        if writer_owner != namespace.daemon_owner {
            return Err(QueueError::WriterOwnerMismatch {
                expected: namespace.daemon_owner.clone(),
                actual: writer_owner,
            });
        }
        Self::open_inner(root_path, namespace, Some(writer_owner))
    }

    pub fn open_read_only(
        root_path: impl AsRef<Path>,
        namespace: DaemonNamespaceV1,
    ) -> Result<Self, QueueError> {
        Self::open_inner(root_path, namespace, None)
    }

    fn open_inner(
        root_path: impl AsRef<Path>,
        namespace: DaemonNamespaceV1,
        writer_owner: Option<String>,
    ) -> Result<Self, QueueError> {
        let root_path = root_path.as_ref().to_path_buf();
        std::fs::create_dir_all(&root_path).map_err(|source| QueueError::Io {
            path: root_path.clone(),
            source,
        })?;
        let log_path = root_path.join("queue.ndjson");
        ensure_file(&log_path)?;
        Ok(Self {
            namespace,
            root_path,
            log_path,
            writer_owner,
        })
    }

    pub fn namespace(&self) -> &DaemonNamespaceV1 {
        &self.namespace
    }

    pub fn root_path(&self) -> &Path {
        &self.root_path
    }

    pub fn enqueue_from_schedule(
        &self,
        occurrence: ScheduleOccurrenceV1,
    ) -> Result<QueueEnqueueOutcomeV1, QueueError> {
        let job = JobV1::new(
            occurrence.namespace_id.clone(),
            occurrence.idempotency_key.clone(),
            "schedule",
            occurrence.payload,
            occurrence.risk,
            Some(occurrence.occurrence_id),
            None,
        );
        self.enqueue_job(job)
    }

    pub fn enqueue_from_wake(
        &self,
        signal: WakeSignalV1,
    ) -> Result<QueueEnqueueOutcomeV1, QueueError> {
        let job = JobV1::new(
            signal.namespace_id.clone(),
            signal.idempotency_key.clone(),
            "wake",
            signal.payload,
            signal.risk,
            None,
            Some(signal.signal_id),
        );
        self.enqueue_job(job)
    }

    pub fn enqueue_job(&self, job: JobV1) -> Result<QueueEnqueueOutcomeV1, QueueError> {
        self.ensure_writer()?;
        let _lock = acquire_exclusive_lock(&self.log_path)?;
        let records = self.read_records()?;
        let snapshot = self.snapshot_from_records(&records);
        if snapshot.safe_mode_enabled && job.risk != CanonicalToolSideEffectClass::ReadOnly {
            let safe = SafeModeReportV1::new(
                self.namespace.namespace_id.clone(),
                SafeModeOperationV1::BlockedRiskyJob,
                true,
                Some(job.job_id.clone()),
                "safe-mode-blocked-new-risky-job",
            );
            let hop = QueueHopReportV1::new(
                self.namespace.namespace_id.clone(),
                job.job_id.clone(),
                None,
                QueueHopKindV1::SafeModeBlocked,
                None,
                JobStateV1::Queued,
                "safe-mode-blocked-new-risky-job",
            );
            self.append_record_locked(QueueLogEntryV1 {
                sequence: 0,
                namespace_id: self.namespace.namespace_id.clone(),
                job: None,
                lease: None,
                safe_mode_enabled: Some(true),
                queue_hop_receipt: Some(hop.clone()),
                duplicate_suppression_receipt: None,
                safe_mode_receipt: Some(safe.clone()),
                recorded_at: Utc::now(),
            })?;
            return Ok(QueueEnqueueOutcomeV1 {
                enqueued: false,
                job: Some(job),
                existing_job: None,
                queue_hop_receipt: hop,
                duplicate_suppression_receipt: None,
                safe_mode_receipt: Some(safe),
            });
        }

        if let Some(existing) = snapshot
            .jobs
            .iter()
            .find(|existing| existing.idempotency_key == job.idempotency_key)
            .cloned()
        {
            let duplicate = DuplicateSuppressionReportV1::new(
                self.namespace.namespace_id.clone(),
                job.idempotency_key.clone(),
                existing.job_id.clone(),
                job.source_kind,
            );
            let hop = QueueHopReportV1::new(
                self.namespace.namespace_id.clone(),
                existing.job_id.clone(),
                existing.lease_id.clone(),
                QueueHopKindV1::DuplicateSuppressed,
                Some(existing.state.clone()),
                existing.state.clone(),
                "duplicate-logical-job-suppressed",
            );
            self.append_record_locked(QueueLogEntryV1 {
                sequence: 0,
                namespace_id: self.namespace.namespace_id.clone(),
                job: None,
                lease: None,
                safe_mode_enabled: None,
                queue_hop_receipt: Some(hop.clone()),
                duplicate_suppression_receipt: Some(duplicate.clone()),
                safe_mode_receipt: None,
                recorded_at: Utc::now(),
            })?;
            return Ok(QueueEnqueueOutcomeV1 {
                enqueued: false,
                job: None,
                existing_job: Some(existing),
                queue_hop_receipt: hop,
                duplicate_suppression_receipt: Some(duplicate),
                safe_mode_receipt: None,
            });
        }

        let hop = QueueHopReportV1::new(
            self.namespace.namespace_id.clone(),
            job.job_id.clone(),
            None,
            QueueHopKindV1::Enqueued,
            None,
            JobStateV1::Queued,
            "job-enqueued",
        );
        self.append_record_locked(QueueLogEntryV1 {
            sequence: 0,
            namespace_id: self.namespace.namespace_id.clone(),
            job: Some(job.clone()),
            lease: None,
            safe_mode_enabled: None,
            queue_hop_receipt: Some(hop.clone()),
            duplicate_suppression_receipt: None,
            safe_mode_receipt: None,
            recorded_at: Utc::now(),
        })?;
        Ok(QueueEnqueueOutcomeV1 {
            enqueued: true,
            job: Some(job),
            existing_job: None,
            queue_hop_receipt: hop,
            duplicate_suppression_receipt: None,
            safe_mode_receipt: None,
        })
    }

    pub fn acquire_next_lease(
        &self,
        owner: impl Into<String>,
        ttl_seconds: i64,
    ) -> Result<Option<QueueLeaseOutcomeV1>, QueueError> {
        self.acquire_next_lease_at(owner, ttl_seconds, Utc::now())
    }

    pub fn acquire_next_lease_at(
        &self,
        owner: impl Into<String>,
        ttl_seconds: i64,
        now: DateTime<Utc>,
    ) -> Result<Option<QueueLeaseOutcomeV1>, QueueError> {
        self.ensure_writer()?;
        let owner = owner.into();
        let _lock = acquire_exclusive_lock(&self.log_path)?;
        let records = self.read_records()?;
        let snapshot = self.snapshot_from_records(&records);
        let leases_by_id = snapshot
            .leases
            .iter()
            .map(|lease| (lease.lease_id.clone(), lease.clone()))
            .collect::<BTreeMap<_, _>>();

        let mut jobs = snapshot.jobs;
        jobs.sort_by_key(|job| (job.created_at, job.job_id.as_str().to_string()));
        for job in jobs {
            let stale_lease = job
                .lease_id
                .as_ref()
                .and_then(|lease_id| leases_by_id.get(lease_id))
                .filter(|lease| lease.is_expired_at(now))
                .cloned();
            let can_lease = matches!(job.state, JobStateV1::Queued | JobStateV1::Retrying)
                || stale_lease.is_some();
            if !can_lease || job.state.is_terminal() {
                continue;
            }
            let from_state = job.state.clone();
            let lease = QueueLeaseV1::new(
                &job,
                owner.clone(),
                ttl_seconds,
                stale_lease.as_ref().map(|lease| lease.lease_id.clone()),
            );
            let mut leased_job = job.with_state(
                JobStateV1::Leased,
                Some(lease.lease_id.clone()),
                if stale_lease.is_some() {
                    "lease-expired-stolen"
                } else {
                    "lease-acquired"
                },
            );
            leased_job.updated_at = now;
            let hop = QueueHopReportV1::new(
                self.namespace.namespace_id.clone(),
                leased_job.job_id.clone(),
                Some(lease.lease_id.clone()),
                if stale_lease.is_some() {
                    QueueHopKindV1::LeaseStolen
                } else {
                    QueueHopKindV1::LeaseAcquired
                },
                Some(from_state),
                JobStateV1::Leased,
                if stale_lease.is_some() {
                    "lease-expired-stolen"
                } else {
                    "lease-acquired"
                },
            );
            self.append_record_locked(QueueLogEntryV1 {
                sequence: 0,
                namespace_id: self.namespace.namespace_id.clone(),
                job: Some(leased_job.clone()),
                lease: Some(lease.clone()),
                safe_mode_enabled: None,
                queue_hop_receipt: Some(hop.clone()),
                duplicate_suppression_receipt: None,
                safe_mode_receipt: None,
                recorded_at: Utc::now(),
            })?;
            return Ok(Some(QueueLeaseOutcomeV1 {
                job: leased_job,
                lease,
                queue_hop_receipt: hop,
            }));
        }
        Ok(None)
    }

    pub fn cancel_job(
        &self,
        job_id: &ArtifactId,
        reason: impl Into<String>,
    ) -> Result<QueueHopReportV1, QueueError> {
        self.transition_job(
            job_id,
            JobStateV1::Cancelled,
            QueueHopKindV1::Cancelled,
            None,
            reason,
            Utc::now(),
        )
    }

    pub fn complete_job(
        &self,
        job_id: &ArtifactId,
        lease_id: Option<ArtifactId>,
    ) -> Result<QueueHopReportV1, QueueError> {
        self.complete_job_at(job_id, lease_id, Utc::now())
    }

    pub fn complete_job_at(
        &self,
        job_id: &ArtifactId,
        lease_id: Option<ArtifactId>,
        now: DateTime<Utc>,
    ) -> Result<QueueHopReportV1, QueueError> {
        self.transition_job(
            job_id,
            JobStateV1::Completed,
            QueueHopKindV1::Executed,
            lease_id,
            "job-executed",
            now,
        )
    }

    pub fn poison_job(
        &self,
        job_id: &ArtifactId,
        reason: impl Into<String>,
    ) -> Result<QueueHopReportV1, QueueError> {
        self.transition_job(
            job_id,
            JobStateV1::Poisoned,
            QueueHopKindV1::Poisoned,
            None,
            reason,
            Utc::now(),
        )
    }

    pub fn set_safe_mode(
        &self,
        enabled: bool,
        reason: impl Into<String>,
    ) -> Result<SafeModeReportV1, QueueError> {
        self.ensure_writer()?;
        let _lock = acquire_exclusive_lock(&self.log_path)?;
        let records = self.read_records()?;
        let snapshot = self.snapshot_from_records(&records);
        let receipt = SafeModeReportV1::new(
            self.namespace.namespace_id.clone(),
            if enabled {
                SafeModeOperationV1::Entered
            } else {
                SafeModeOperationV1::Exited
            },
            enabled,
            None,
            reason,
        );
        self.append_record_locked(QueueLogEntryV1 {
            sequence: 0,
            namespace_id: self.namespace.namespace_id.clone(),
            job: None,
            lease: None,
            safe_mode_enabled: Some(enabled),
            queue_hop_receipt: None,
            duplicate_suppression_receipt: None,
            safe_mode_receipt: Some(receipt.clone()),
            recorded_at: Utc::now(),
        })?;
        if enabled {
            for job in snapshot.jobs {
                if !job.state.is_terminal() && job.risk != CanonicalToolSideEffectClass::ReadOnly {
                    let from_state = job.state.clone();
                    let lease_id = job.lease_id.clone();
                    let next_job = job.with_state(
                        JobStateV1::Cancelled,
                        lease_id,
                        "safe-mode-quarantined-existing-risky-job",
                    );
                    let hop = QueueHopReportV1::new(
                        self.namespace.namespace_id.clone(),
                        next_job.job_id.clone(),
                        next_job.lease_id.clone(),
                        QueueHopKindV1::Drained,
                        Some(from_state),
                        JobStateV1::Cancelled,
                        "safe-mode-quarantined-existing-risky-job",
                    );
                    self.append_record_locked(QueueLogEntryV1 {
                        sequence: 0,
                        namespace_id: self.namespace.namespace_id.clone(),
                        job: Some(next_job),
                        lease: None,
                        safe_mode_enabled: None,
                        queue_hop_receipt: Some(hop),
                        duplicate_suppression_receipt: None,
                        safe_mode_receipt: None,
                        recorded_at: Utc::now(),
                    })?;
                }
            }
        }
        Ok(receipt)
    }

    pub fn drain_non_terminal(
        &self,
        reason: impl Into<String>,
    ) -> Result<Vec<QueueHopReportV1>, QueueError> {
        self.ensure_writer()?;
        let reason = reason.into();
        let jobs = self.snapshot()?.jobs;
        let mut receipts = Vec::new();
        for job in jobs {
            if !job.state.is_terminal() {
                receipts.push(self.transition_job(
                    &job.job_id,
                    JobStateV1::Cancelled,
                    QueueHopKindV1::Drained,
                    job.lease_id.clone(),
                    reason.clone(),
                    Utc::now(),
                )?);
            }
        }
        Ok(receipts)
    }

    pub fn snapshot(&self) -> Result<QueueSnapshotV1, QueueError> {
        let records = self.read_records()?;
        Ok(self.snapshot_from_records(&records))
    }

    fn snapshot_from_records(&self, records: &[QueueLogEntryV1]) -> QueueSnapshotV1 {
        let mut jobs_by_id = BTreeMap::<ArtifactId, JobV1>::new();
        let mut leases_by_id = BTreeMap::<ArtifactId, QueueLeaseV1>::new();
        let mut safe_mode_enabled = self.namespace.safe_mode_enabled;
        for record in records {
            if record.namespace_id != self.namespace.namespace_id {
                continue;
            }
            if let Some(job) = &record.job {
                jobs_by_id.insert(job.job_id.clone(), job.clone());
            }
            if let Some(lease) = &record.lease {
                leases_by_id.insert(lease.lease_id.clone(), lease.clone());
            }
            if let Some(enabled) = record.safe_mode_enabled {
                safe_mode_enabled = enabled;
            }
        }
        QueueSnapshotV1 {
            namespace: self.namespace.clone().with_safe_mode(safe_mode_enabled),
            jobs: jobs_by_id.into_values().collect(),
            leases: leases_by_id.into_values().collect(),
            safe_mode_enabled,
            records_seen: records.len(),
        }
    }

    fn transition_job(
        &self,
        job_id: &ArtifactId,
        to_state: JobStateV1,
        hop_kind: QueueHopKindV1,
        lease_id: Option<ArtifactId>,
        reason: impl Into<String>,
        now: DateTime<Utc>,
    ) -> Result<QueueHopReportV1, QueueError> {
        self.ensure_writer()?;
        let reason = reason.into();
        let _lock = acquire_exclusive_lock(&self.log_path)?;
        let records = self.read_records()?;
        let snapshot = self.snapshot_from_records(&records);
        let job = snapshot
            .job(job_id)
            .cloned()
            .ok_or_else(|| QueueError::JobNotFound(job_id.as_str().to_string()))?;
        if job.state.is_terminal() {
            return Err(QueueError::TerminalJob(job_id.as_str().to_string()));
        }
        if to_state == JobStateV1::Completed {
            let requested_lease_id = lease_id
                .clone()
                .ok_or_else(|| QueueError::LeaseRequired(job_id.as_str().to_string()))?;
            let current_lease_id = job
                .lease_id
                .clone()
                .ok_or_else(|| QueueError::LeaseRequired(job_id.as_str().to_string()))?;
            if requested_lease_id != current_lease_id {
                return Err(QueueError::LeaseMismatch {
                    job_id: job_id.as_str().to_string(),
                    expected: current_lease_id.as_str().to_string(),
                    actual: requested_lease_id.as_str().to_string(),
                });
            }
            let lease = snapshot
                .leases
                .iter()
                .find(|lease| lease.lease_id == requested_lease_id && lease.active)
                .ok_or_else(|| QueueError::LeaseRequired(job_id.as_str().to_string()))?;
            if lease.is_expired_at(now) {
                return Err(QueueError::LeaseExpired {
                    job_id: job_id.as_str().to_string(),
                    lease_id: lease.lease_id.as_str().to_string(),
                });
            }
        }
        let from_state = job.state.clone();
        let next_job = job.with_state(to_state.clone(), lease_id.clone(), reason.clone());
        let hop = QueueHopReportV1::new(
            self.namespace.namespace_id.clone(),
            next_job.job_id.clone(),
            lease_id,
            hop_kind,
            Some(from_state),
            to_state,
            reason,
        );
        self.append_record_locked(QueueLogEntryV1 {
            sequence: 0,
            namespace_id: self.namespace.namespace_id.clone(),
            job: Some(next_job),
            lease: None,
            safe_mode_enabled: None,
            queue_hop_receipt: Some(hop.clone()),
            duplicate_suppression_receipt: None,
            safe_mode_receipt: None,
            recorded_at: Utc::now(),
        })?;
        Ok(hop)
    }

    fn ensure_writer(&self) -> Result<(), QueueError> {
        match &self.writer_owner {
            Some(owner) if owner == &self.namespace.daemon_owner => Ok(()),
            Some(owner) => Err(QueueError::WriterOwnerMismatch {
                expected: self.namespace.daemon_owner.clone(),
                actual: owner.clone(),
            }),
            None => Err(QueueError::ReadOnly),
        }
    }

    fn append_record_locked(&self, mut record: QueueLogEntryV1) -> Result<(), QueueError> {
        record.sequence = self.read_records()?.len() as u64 + 1;
        let line = serde_json::to_string(&record).map_err(|source| QueueError::Json {
            path: self.log_path.clone(),
            source,
        })?;
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.log_path)
            .map_err(|source| QueueError::Io {
                path: self.log_path.clone(),
                source,
            })?;
        writeln!(file, "{line}").map_err(|source| QueueError::Io {
            path: self.log_path.clone(),
            source,
        })?;
        file.sync_data().map_err(|source| QueueError::Io {
            path: self.log_path.clone(),
            source,
        })?;
        Ok(())
    }

    fn read_records(&self) -> Result<Vec<QueueLogEntryV1>, QueueError> {
        let file = File::open(&self.log_path).map_err(|source| QueueError::Io {
            path: self.log_path.clone(),
            source,
        })?;
        let reader = BufReader::new(file);
        let mut records = Vec::new();
        for line in reader.lines() {
            let line = line.map_err(|source| QueueError::Io {
                path: self.log_path.clone(),
                source,
            })?;
            if line.trim().is_empty() {
                continue;
            }
            records.push(
                serde_json::from_str::<QueueLogEntryV1>(&line).map_err(|source| {
                    QueueError::Json {
                        path: self.log_path.clone(),
                        source,
                    }
                })?,
            );
        }
        Ok(records)
    }
}

fn ensure_file(path: &Path) -> Result<(), QueueError> {
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| QueueError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|source| QueueError::Io {
            path: path.to_path_buf(),
            source,
        })?;
    Ok(())
}

fn acquire_exclusive_lock(path: &Path) -> Result<ExclusiveQueueLock, QueueError> {
    let lock_path = path.with_file_name(format!(
        "{}.lock",
        path.file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("queue")
    ));
    if let Some(parent) = lock_path.parent() {
        std::fs::create_dir_all(parent).map_err(|source| QueueError::Io {
            path: parent.to_path_buf(),
            source,
        })?;
    }
    let started = Instant::now();
    loop {
        match OpenOptions::new()
            .create_new(true)
            .write(true)
            .open(&lock_path)
        {
            Ok(mut file) => {
                writeln!(
                    file,
                    "pid={} acquired_at={}",
                    std::process::id(),
                    Utc::now().to_rfc3339()
                )
                .map_err(|source| QueueError::Io {
                    path: lock_path.clone(),
                    source,
                })?;
                file.sync_all().map_err(|source| QueueError::Io {
                    path: lock_path.clone(),
                    source,
                })?;
                return Ok(ExclusiveQueueLock { lock_path });
            }
            Err(source) if source.kind() == std::io::ErrorKind::AlreadyExists => {
                if started.elapsed() > StdDuration::from_secs(10) {
                    return Err(QueueError::Io {
                        path: lock_path,
                        source: std::io::Error::new(
                            std::io::ErrorKind::TimedOut,
                            "queue lock still held",
                        ),
                    });
                }
                thread::sleep(StdDuration::from_millis(5));
            }
            Err(source) => {
                return Err(QueueError::Io {
                    path: lock_path,
                    source,
                });
            }
        }
    }
}

struct ExclusiveQueueLock {
    lock_path: PathBuf,
}

impl Drop for ExclusiveQueueLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.lock_path);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use chrono::Duration;
    use std::sync::{Arc, Barrier};

    fn temp_root(name: &str) -> PathBuf {
        let root = std::env::temp_dir().join(format!("aidens-p11-{name}-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&root);
        root
    }

    fn namespace(root: &Path) -> DaemonNamespaceV1 {
        DaemonNamespaceV1::new("test-namespace", root.display().to_string(), "daemon-a")
    }

    #[test]
    fn duplicate_schedule_occurrence_is_suppressed_by_idempotency_key() {
        let root = temp_root("duplicate");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let occurrence = ScheduleOccurrenceV1::new(
            ns.namespace_id.clone(),
            "daily",
            "2026-04-27T00:00:00Z",
            Utc::now(),
            serde_json::json!({"task":"refresh"}),
            CanonicalToolSideEffectClass::ReadOnly,
        );
        assert!(occurrence.identity_is_not_timestamp_only());

        let first = queue.enqueue_from_schedule(occurrence.clone()).unwrap();
        let second = queue.enqueue_from_schedule(occurrence.clone()).unwrap();

        assert!(first.enqueued);
        assert!(!second.enqueued);
        assert_eq!(
            second
                .duplicate_suppression_receipt
                .as_ref()
                .unwrap()
                .idempotency_key,
            occurrence.idempotency_key
        );
        assert_eq!(
            queue
                .snapshot()
                .unwrap()
                .logical_job_count_for(&occurrence.idempotency_key),
            1
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn restart_preserves_queued_and_cancelled_jobs_without_resurrection() {
        let root = temp_root("restart");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let job = JobV1::new(
            ns.namespace_id.clone(),
            "manual:job-1",
            "manual",
            serde_json::json!({"task":"one"}),
            CanonicalToolSideEffectClass::ReadOnly,
            None,
            None,
        );
        let enqueued = queue.enqueue_job(job).unwrap().job.unwrap();
        queue
            .cancel_job(&enqueued.job_id, "operator-cancelled")
            .unwrap();

        let reopened = DurableQueueLogV1::open(&root, ns, "daemon-a").unwrap();
        let snapshot = reopened.snapshot().unwrap();
        let restored = snapshot.job(&enqueued.job_id).unwrap();
        assert_eq!(restored.state, JobStateV1::Cancelled);
        assert!(reopened
            .acquire_next_lease("daemon-a", 30)
            .unwrap()
            .is_none());
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn safe_mode_blocks_risky_jobs_and_allows_inspection_and_drain() {
        let root = temp_root("safe-mode");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let safe = queue.set_safe_mode(true, "operator-safe-mode").unwrap();
        assert!(safe.blocks_new_risky_jobs_but_allows_drain());

        let blocked = queue
            .enqueue_job(JobV1::new(
                ns.namespace_id.clone(),
                "shell:1",
                "manual",
                serde_json::json!({"cmd":"cargo check"}),
                CanonicalToolSideEffectClass::Admin,
                None,
                None,
            ))
            .unwrap();
        assert!(!blocked.enqueued);
        assert!(blocked.safe_mode_receipt.is_some());

        let read_only = queue
            .enqueue_job(JobV1::new(
                ns.namespace_id,
                "inspect:1",
                "manual",
                serde_json::json!({"inspect":true}),
                CanonicalToolSideEffectClass::ReadOnly,
                None,
                None,
            ))
            .unwrap();
        assert!(read_only.enqueued);
        assert_eq!(queue.snapshot().unwrap().jobs.len(), 1);
        assert_eq!(
            queue.drain_non_terminal("safe-mode-drain").unwrap().len(),
            1
        );
        assert_eq!(
            queue.snapshot().unwrap().jobs[0].state,
            JobStateV1::Cancelled
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn expired_lease_can_be_stolen_once_without_duplicate_job() {
        let root = temp_root("lease");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let job = JobV1::new(
            ns.namespace_id,
            "lease:job",
            "manual",
            serde_json::json!({"work":true}),
            CanonicalToolSideEffectClass::ReadOnly,
            None,
            None,
        );
        let enqueued = queue.enqueue_job(job).unwrap().job.unwrap();
        let first = queue.acquire_next_lease("daemon-a", 1).unwrap().unwrap();
        let stolen = queue
            .acquire_next_lease_at("daemon-b", 30, Utc::now() + Duration::seconds(5))
            .unwrap()
            .unwrap();
        assert_eq!(stolen.job.job_id, enqueued.job_id);
        assert_eq!(stolen.lease.stolen_from, Some(first.lease.lease_id));
        assert_eq!(queue.snapshot().unwrap().jobs.len(), 1);
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn concurrent_enqueue_is_single_writer_and_idempotent() {
        let root = temp_root("concurrent-enqueue");
        let ns = namespace(&root);
        let queue = Arc::new(DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap());
        let barrier = Arc::new(Barrier::new(16));
        let mut handles = Vec::new();

        for _ in 0..16 {
            let queue = Arc::clone(&queue);
            let barrier = Arc::clone(&barrier);
            let namespace_id = ns.namespace_id.clone();
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                let job = JobV1::new(
                    namespace_id,
                    "concurrent:one-logical-job",
                    "manual",
                    serde_json::json!({"work":true}),
                    CanonicalToolSideEffectClass::ReadOnly,
                    None,
                    None,
                );
                queue.enqueue_job(job).unwrap().enqueued
            }));
        }

        let enqueued_count = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|enqueued| *enqueued)
            .count();
        let snapshot = queue.snapshot().unwrap();
        let records = queue.read_records().unwrap();

        assert_eq!(enqueued_count, 1);
        assert_eq!(
            snapshot.logical_job_count_for("concurrent:one-logical-job"),
            1
        );
        assert_eq!(snapshot.jobs.len(), 1);
        assert_eq!(records.len(), 16);
        assert_eq!(
            records
                .iter()
                .map(|record| record.sequence)
                .collect::<Vec<_>>(),
            (1..=records.len() as u64).collect::<Vec<_>>()
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn concurrent_lease_acquisition_grants_one_active_lease() {
        let root = temp_root("concurrent-lease");
        let ns = namespace(&root);
        let queue = Arc::new(DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap());
        queue
            .enqueue_job(JobV1::new(
                ns.namespace_id,
                "lease:single-active",
                "manual",
                serde_json::json!({"work":true}),
                CanonicalToolSideEffectClass::ReadOnly,
                None,
                None,
            ))
            .unwrap();

        let barrier = Arc::new(Barrier::new(12));
        let mut handles = Vec::new();
        for worker in 0..12 {
            let queue = Arc::clone(&queue);
            let barrier = Arc::clone(&barrier);
            handles.push(std::thread::spawn(move || {
                barrier.wait();
                queue
                    .acquire_next_lease(format!("daemon-worker-{worker}"), 30)
                    .unwrap()
                    .is_some()
            }));
        }

        let lease_count = handles
            .into_iter()
            .map(|handle| handle.join().unwrap())
            .filter(|leased| *leased)
            .count();
        let snapshot = queue.snapshot().unwrap();

        assert_eq!(lease_count, 1);
        assert_eq!(snapshot.jobs.len(), 1);
        assert_eq!(snapshot.jobs[0].state, JobStateV1::Leased);
        assert_eq!(
            snapshot.leases.iter().filter(|lease| lease.active).count(),
            1
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn late_completion_after_ttl_is_rejected_and_job_stays_leased() {
        let root = temp_root("late-completion");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        queue
            .enqueue_job(JobV1::new(
                ns.namespace_id,
                "lease:late-completion",
                "manual",
                serde_json::json!({"work":true}),
                CanonicalToolSideEffectClass::ReadOnly,
                None,
                None,
            ))
            .unwrap();

        let now = Utc::now();
        let leased = queue
            .acquire_next_lease_at("daemon-a", 1, now)
            .unwrap()
            .unwrap();
        let err = queue
            .complete_job_at(
                &leased.job.job_id,
                Some(leased.lease.lease_id.clone()),
                now + Duration::seconds(2),
            )
            .unwrap_err();
        assert!(matches!(err, QueueError::LeaseExpired { .. }));

        let snapshot = queue.snapshot().unwrap();
        let job = snapshot.job(&leased.job.job_id).unwrap();
        assert_eq!(job.state, JobStateV1::Leased);
        assert_eq!(job.lease_id, Some(leased.lease.lease_id));
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn stale_lease_cannot_complete_after_lease_is_stolen() {
        let root = temp_root("stale-completion");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        queue
            .enqueue_job(JobV1::new(
                ns.namespace_id,
                "lease:stale-completion",
                "manual",
                serde_json::json!({"work":true}),
                CanonicalToolSideEffectClass::ReadOnly,
                None,
                None,
            ))
            .unwrap();
        let first = queue.acquire_next_lease("daemon-a", 1).unwrap().unwrap();
        let second = queue
            .acquire_next_lease_at("daemon-b", 30, Utc::now() + Duration::seconds(5))
            .unwrap()
            .unwrap();

        let err = queue
            .complete_job_at(
                &first.job.job_id,
                Some(first.lease.lease_id.clone()),
                Utc::now() + Duration::seconds(6),
            )
            .unwrap_err();
        assert!(matches!(err, QueueError::LeaseMismatch { .. }));
        assert_eq!(
            queue
                .snapshot()
                .unwrap()
                .job(&first.job.job_id)
                .unwrap()
                .lease_id,
            Some(second.lease.lease_id)
        );
        let _ = std::fs::remove_dir_all(&root);
    }

    #[test]
    fn entering_safe_mode_quarantines_existing_risky_jobs() {
        let root = temp_root("safe-mode-existing");
        let ns = namespace(&root);
        let queue = DurableQueueLogV1::open(&root, ns.clone(), "daemon-a").unwrap();
        let risky = queue
            .enqueue_job(JobV1::new(
                ns.namespace_id.clone(),
                "admin:queued-before-safe-mode",
                "manual",
                serde_json::json!({"cmd":"write"}),
                CanonicalToolSideEffectClass::Admin,
                None,
                None,
            ))
            .unwrap()
            .job
            .unwrap();
        let read_only = queue
            .enqueue_job(JobV1::new(
                ns.namespace_id,
                "read:queued-before-safe-mode",
                "manual",
                serde_json::json!({"inspect":true}),
                CanonicalToolSideEffectClass::ReadOnly,
                None,
                None,
            ))
            .unwrap()
            .job
            .unwrap();

        queue.set_safe_mode(true, "operator-safe-mode").unwrap();

        let snapshot = queue.snapshot().unwrap();
        let risky_after = snapshot.job(&risky.job_id).unwrap();
        let read_after = snapshot.job(&read_only.job_id).unwrap();
        assert_eq!(risky_after.state, JobStateV1::Cancelled);
        assert!(risky_after
            .reason_codes
            .contains(&"safe-mode-quarantined-existing-risky-job".to_string()));
        assert_eq!(read_after.state, JobStateV1::Queued);
        let _ = std::fs::remove_dir_all(&root);
    }
}
