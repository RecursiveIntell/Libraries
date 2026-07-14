//! Queue, schedule, wake, daemon, and safe-mode artifacts.
//!
//! These DTOs describe supported-local orchestration state only.

use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum JobStateV1 {
    Queued,
    Leased,
    Running,
    Completed,
    Retrying,
    Cancelled,
    Poisoned,
}

impl JobStateV1 {
    pub fn is_terminal(&self) -> bool {
        matches!(self, Self::Completed | Self::Cancelled | Self::Poisoned)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum QueueHopKindV1 {
    Enqueued,
    LeaseAcquired,
    LeaseStolen,
    Executed,
    Retried,
    Cancelled,
    DuplicateSuppressed,
    SafeModeBlocked,
    Drained,
    Poisoned,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
#[serde(rename_all = "kebab-case")]
pub enum SafeModeOperationV1 {
    Entered,
    Exited,
    BlockedRiskyJob,
    DrainAllowed,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct JobV1 {
    pub job_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub idempotency_key: String,
    pub attempt_family_id: ArtifactId,
    pub source_kind: String,
    pub occurrence_id: Option<ArtifactId>,
    pub wake_signal_id: Option<ArtifactId>,
    pub payload: serde_json::Value,
    pub payload_digest: String,
    pub risk: CanonicalToolSideEffectClass,
    pub state: JobStateV1,
    pub lease_id: Option<ArtifactId>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub cancelled_at: Option<DateTime<Utc>>,
}

impl JobV1 {
    pub fn new(
        namespace_id: ArtifactId,
        idempotency_key: impl Into<String>,
        source_kind: impl Into<String>,
        payload: serde_json::Value,
        risk: CanonicalToolSideEffectClass,
        occurrence_id: Option<ArtifactId>,
        wake_signal_id: Option<ArtifactId>,
    ) -> Self {
        let idempotency_key = idempotency_key.into();
        let payload_digest = non_authoritative_json_display_digest(&payload);
        let identity_material =
            format!("{}|{}|{}", namespace_id.as_str(), idempotency_key, payload_digest);
        let now = Utc::now();
        Self {
            job_id: local_artifact_id_from_stack_digest("job", &identity_material),
            kind: ArtifactKindV1::Job,
            namespace_id,
            attempt_family_id: local_artifact_id_from_stack_digest(
                "attempt-family",
                &format!("job-attempt-family|{identity_material}"),
            ),
            idempotency_key,
            source_kind: source_kind.into(),
            occurrence_id,
            wake_signal_id,
            payload,
            payload_digest,
            risk,
            state: JobStateV1::Queued,
            lease_id: None,
            reason_codes: Vec::new(),
            created_at: now,
            updated_at: now,
            cancelled_at: None,
        }
    }

    pub fn with_state(
        mut self,
        state: JobStateV1,
        lease_id: Option<ArtifactId>,
        reason: impl Into<String>,
    ) -> Self {
        let reason = reason.into();
        if !reason.trim().is_empty() {
            self.reason_codes.push(reason);
        }
        if state == JobStateV1::Cancelled {
            self.cancelled_at = Some(Utc::now());
        }
        self.state = state;
        self.lease_id = lease_id;
        self.updated_at = Utc::now();
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QueueLeaseV1 {
    pub lease_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub job_id: ArtifactId,
    pub attempt_family_id: ArtifactId,
    pub owner: String,
    pub acquired_at: DateTime<Utc>,
    pub expires_at: DateTime<Utc>,
    pub stolen_from: Option<ArtifactId>,
    pub active: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
}

impl QueueLeaseV1 {
    pub fn new(
        job: &JobV1,
        owner: impl Into<String>,
        ttl_seconds: i64,
        stolen_from: Option<ArtifactId>,
    ) -> Self {
        let acquired_at = Utc::now();
        let ttl_seconds = ttl_seconds.max(1);
        let owner = owner.into();
        let acquired_at_material = acquired_at.to_rfc3339_opts(SecondsFormat::Nanos, true);
        let material = format!(
            "{}|{}|{}|{}",
            job.namespace_id.as_str(), job.job_id.as_str(), owner, acquired_at_material
        );
        Self {
            lease_id: local_artifact_id_from_stack_digest("queue-lease", &material),
            kind: ArtifactKindV1::QueueLease,
            namespace_id: job.namespace_id.clone(),
            job_id: job.job_id.clone(),
            attempt_family_id: job.attempt_family_id.clone(),
            owner,
            acquired_at,
            expires_at: acquired_at + Duration::seconds(ttl_seconds),
            stolen_from,
            active: true,
            reason_codes: vec!["lease-acquired".into()],
        }
    }

    pub fn is_expired_at(&self, at: DateTime<Utc>) -> bool {
        self.expires_at <= at
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct ScheduleOccurrenceV1 {
    pub occurrence_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub schedule_id: String,
    pub occurrence_key: String,
    pub due_at: DateTime<Utc>,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub payload_digest: String,
    pub risk: CanonicalToolSideEffectClass,
    pub created_at: DateTime<Utc>,
}

impl ScheduleOccurrenceV1 {
    pub fn new(
        namespace_id: ArtifactId,
        schedule_id: impl Into<String>,
        occurrence_key: impl Into<String>,
        due_at: DateTime<Utc>,
        payload: serde_json::Value,
        risk: CanonicalToolSideEffectClass,
    ) -> Self {
        let schedule_id = schedule_id.into();
        let occurrence_key = occurrence_key.into();
        let payload_digest = non_authoritative_json_display_digest(&payload);
        let idempotency_key = format!(
            "schedule:{}:{}:{}:{}",
            namespace_id.as_str(), schedule_id, occurrence_key, payload_digest
        );
        Self {
            occurrence_id: local_artifact_id_from_stack_digest(
                "schedule-occurrence",
                &idempotency_key,
            ),
            kind: ArtifactKindV1::ScheduleOccurrence,
            namespace_id,
            schedule_id,
            occurrence_key,
            due_at,
            idempotency_key,
            payload,
            payload_digest,
            risk,
            created_at: Utc::now(),
        }
    }

    pub fn identity_is_not_timestamp_only(&self) -> bool {
        !self.schedule_id.trim().is_empty()
            && !self.occurrence_key.trim().is_empty()
            && self.idempotency_key.contains(&self.schedule_id)
            && self.idempotency_key.contains(&self.occurrence_key)
            && self.idempotency_key.contains(&self.payload_digest)
    }
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize, JsonSchema)]
pub struct WakeSignalV1 {
    pub signal_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub source: String,
    pub signal_key: String,
    pub idempotency_key: String,
    pub payload: serde_json::Value,
    pub payload_digest: String,
    pub risk: CanonicalToolSideEffectClass,
    pub received_at: DateTime<Utc>,
}

impl WakeSignalV1 {
    pub fn new(
        namespace_id: ArtifactId,
        source: impl Into<String>,
        signal_key: impl Into<String>,
        payload: serde_json::Value,
        risk: CanonicalToolSideEffectClass,
    ) -> Self {
        let source = source.into();
        let signal_key = signal_key.into();
        let payload_digest = non_authoritative_json_display_digest(&payload);
        let idempotency_key = format!(
            "wake:{}:{}:{}:{}",
            namespace_id.as_str(), source, signal_key, payload_digest
        );
        Self {
            signal_id: local_artifact_id_from_stack_digest("wake-signal", &idempotency_key),
            kind: ArtifactKindV1::WakeSignal,
            namespace_id,
            source,
            signal_key,
            idempotency_key,
            payload,
            payload_digest,
            risk,
            received_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DaemonNamespaceV1 {
    pub namespace_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub name: String,
    pub queue_root: String,
    pub daemon_owner: String,
    pub safe_mode_enabled: bool,
    pub max_lease_seconds: i64,
    pub idempotency_scope: String,
    pub created_at: DateTime<Utc>,
}

impl DaemonNamespaceV1 {
    pub fn new(
        name: impl Into<String>,
        queue_root: impl Into<String>,
        daemon_owner: impl Into<String>,
    ) -> Self {
        let name = name.into();
        let queue_root = queue_root.into();
        let daemon_owner = daemon_owner.into();
        let namespace_material = format!("{name}|{queue_root}|{daemon_owner}");
        Self {
            namespace_id: local_artifact_id_from_stack_digest(
                "daemon-namespace",
                &namespace_material,
            ),
            kind: ArtifactKindV1::DaemonNamespace,
            name,
            queue_root,
            daemon_owner,
            safe_mode_enabled: false,
            max_lease_seconds: 300,
            idempotency_scope: "namespace-id-plus-idempotency-key-plus-payload-digest".into(),
            created_at: Utc::now(),
        }
    }

    pub fn with_safe_mode(mut self, enabled: bool) -> Self {
        self.safe_mode_enabled = enabled;
        self
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct SafeModeReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub operation: SafeModeOperationV1,
    pub enabled: bool,
    pub affected_job_id: Option<ArtifactId>,
    pub new_risky_jobs_blocked: bool,
    pub inspection_allowed: bool,
    pub drain_allowed: bool,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl SafeModeReportV1 {
    pub fn new(
        namespace_id: ArtifactId,
        operation: SafeModeOperationV1,
        enabled: bool,
        affected_job_id: Option<ArtifactId>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("safe-mode"),
            kind: ArtifactKindV1::SafeMode,
            namespace_id,
            operation,
            enabled,
            affected_job_id,
            new_risky_jobs_blocked: enabled,
            inspection_allowed: true,
            drain_allowed: true,
            reason_codes: vec![reason.into()],
            recorded_at: Utc::now(),
        }
    }

    pub fn blocks_new_risky_jobs_but_allows_drain(&self) -> bool {
        self.new_risky_jobs_blocked && self.inspection_allowed && self.drain_allowed
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct DuplicateSuppressionReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub idempotency_key: String,
    pub existing_job_id: ArtifactId,
    pub suppressed_source_kind: String,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl DuplicateSuppressionReportV1 {
    pub fn new(
        namespace_id: ArtifactId,
        idempotency_key: impl Into<String>,
        existing_job_id: ArtifactId,
        suppressed_source_kind: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("duplicate-suppression"),
            kind: ArtifactKindV1::DuplicateSuppression,
            namespace_id,
            idempotency_key: idempotency_key.into(),
            existing_job_id,
            suppressed_source_kind: suppressed_source_kind.into(),
            reason_codes: vec!["duplicate-logical-job-suppressed".into()],
            recorded_at: Utc::now(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
pub struct QueueHopReportV1 {
    pub receipt_id: ArtifactId,
    pub kind: ArtifactKindV1,
    pub namespace_id: ArtifactId,
    pub job_id: ArtifactId,
    pub lease_id: Option<ArtifactId>,
    pub hop: QueueHopKindV1,
    pub from_state: Option<JobStateV1>,
    pub to_state: JobStateV1,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub reason_codes: Vec<String>,
    pub recorded_at: DateTime<Utc>,
}

impl QueueHopReportV1 {
    pub fn new(
        namespace_id: ArtifactId,
        job_id: ArtifactId,
        lease_id: Option<ArtifactId>,
        hop: QueueHopKindV1,
        from_state: Option<JobStateV1>,
        to_state: JobStateV1,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            receipt_id: display_only_unstable_id("queue-hop"),
            kind: ArtifactKindV1::QueueHop,
            namespace_id,
            job_id,
            lease_id,
            hop,
            from_state,
            to_state,
            reason_codes: vec![reason.into()],
            recorded_at: Utc::now(),
        }
    }
}
