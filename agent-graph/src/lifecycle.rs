//! Cross-owner lifecycle coordination primitives.
//!
//! This module owns only logical ordering, fences, and projections.  Authority,
//! effects, artifact retention/access, withdrawal, and durable persistence stay
//! with the injected owner ports.  The outbox is intentionally an owner-acked
//! consistency protocol; it does not claim an atomic transaction across stores.

use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet, VecDeque};

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum OwnerDecision {
    Confirmed,
    Rejected,
    #[default]
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleIntent {
    Resume {
        checkpoint_ref: String,
        generation: u64,
        valid_at: u64,
        recorded_at: u64,
    },
    Publish {
        effect_ref: String,
        idempotency_key: String,
        generation: u64,
        valid_at: u64,
        recorded_at: u64,
    },
    Withdraw {
        subject_ref: String,
        valid_at: u64,
        recorded_at: u64,
    },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OwnerAcknowledgement {
    pub owner: String,
    pub decision: OwnerDecision,
    pub recorded_at: u64,
}

/// Current authority remains external to the coordinator.
pub trait AuthorityOwner {
    fn current_generation(&self, subject: &str) -> Option<u64>;
    fn authorize(&self, intent: &LifecycleIntent) -> OwnerDecision;
}

/// Impure mutation and reconciliation remain external to the coordinator.
pub trait EffectOwner {
    /// Atomically persist an attempt bound to the complete request before any effect.
    /// Return Started only after durable acknowledgement. An existing key (including
    /// a completed attempt) must return AlreadyStarted; a conflicting binding must
    /// return Rejected. This state must survive coordinator and owner restarts.
    fn begin_effect(&self, request: &EffectRequest) -> EffectStart;
    fn perform(&self, request: &EffectRequest) -> EffectResolution;
    fn reconcile(&self, request: &EffectRequest) -> EffectResolution;
}

/// Owner acknowledgement of a durable, exclusive effect attempt.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EffectStart {
    /// New attempt is durably recorded; the caller may cross the effect boundary once.
    Started,
    /// Prior attempt exists; only reconciliation is allowed.
    AlreadyStarted,
    /// Request binding was rejected.
    Rejected,
    /// Durable acknowledgement could not be obtained.
    Unavailable,
}

/// Artifact bytes, pins, and access decisions remain external.
pub trait ArtifactOwner {
    fn access(&self, artifact_ref: &str) -> ArtifactAccess;
    fn is_pinned(&self, artifact_ref: &str) -> OwnerDecision;
}

/// Each owner independently acknowledges a withdrawal.
pub trait WithdrawalOwner {
    fn withdraw(&self, owner: &str, intent: &LifecycleIntent) -> OwnerDecision;
}

/// Persistence and fsync outcomes remain owned by the durable store.
pub trait DurabilityOwner {
    fn persist(&self, record: &DurabilityRecord) -> DurabilityFault;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleState {
    Admitted,
    Partial,
    Blocked,
    Ambiguous,
    Reconciled,
    Terminal,
    HistoricalOnly,
    Restricted,
    Unavailable,
    Cancelled,
    Quarantined,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LifecycleReason {
    None,
    GenerationFenced,
    AuthorityRevoked,
    AuthorityUnavailable,
    LeaseExpired,
    AmbiguousStartedEffect,
    EffectBindingConflict,
    ParentCancelled,
    ArtifactUnavailable,
    ArtifactUnpinned,
    RetainedBytesMissing,
    PrivacyDeleted,
    DurabilityUnproven,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ReplayRestriction {
    FreshEffectsAllowed,
    RecordedOnly,
    ReconcileRequired,
    Forbidden,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct LifecycleDecision {
    pub state: LifecycleState,
    pub reason: LifecycleReason,
    pub replay: ReplayRestriction,
    pub outcome_ref: Option<String>,
    pub redacted: bool,
}

impl LifecycleDecision {
    fn new(state: LifecycleState, reason: LifecycleReason) -> Self {
        Self {
            state,
            reason,
            replay: ReplayRestriction::Forbidden,
            outcome_ref: None,
            redacted: false,
        }
    }

    fn completed(outcome_ref: String) -> Self {
        Self {
            state: LifecycleState::Terminal,
            reason: LifecycleReason::None,
            replay: ReplayRestriction::RecordedOnly,
            outcome_ref: Some(outcome_ref),
            redacted: false,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct EffectRequest {
    pub effect_ref: String,
    pub idempotency_key: String,
    pub generation: u64,
    pub valid_at: u64,
    pub recorded_at: u64,
}

impl EffectRequest {
    fn intent(&self) -> LifecycleIntent {
        LifecycleIntent::Publish {
            effect_ref: self.effect_ref.clone(),
            idempotency_key: self.idempotency_key.clone(),
            generation: self.generation,
            valid_at: self.valid_at,
            recorded_at: self.recorded_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EffectResolution {
    Completed { outcome_ref: String },
    Rejected,
    Unavailable,
    Ambiguous,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct OutboxEntry {
    pub request: EffectRequest,
    pub owner_outcome: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CanonicalArtifact {
    Requirement { requirement_ref: String },
    Frontier { ready: Vec<String> },
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Reconstruction {
    pub requirement_refs: Vec<String>,
    pub frontier: Vec<String>,
    pub transcript_required: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum ChangeLogEntry {
    Dependency {
        source: String,
        dependent: String,
        recorded_at: u64,
    },
    Invalidate {
        artifact_ref: String,
        recorded_at: u64,
    },
}

impl ChangeLogEntry {
    pub fn dependency(source: &str, dependent: &str, recorded_at: u64) -> Self {
        Self::Dependency {
            source: source.into(),
            dependent: dependent.into(),
            recorded_at,
        }
    }

    pub fn invalidate(artifact_ref: &str, recorded_at: u64) -> Self {
        Self::Invalidate {
            artifact_ref: artifact_ref.into(),
            recorded_at,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Purity {
    Pure,
    Impure,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CachedOutput {
    pub artifact_ref: String,
    pub purity: Purity,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CheckpointLease {
    pub checkpoint_ref: String,
    pub subject_ref: String,
    pub authority_generation: u64,
    pub expires_at: u64,
}

impl CheckpointLease {
    pub fn new(
        checkpoint_ref: &str,
        subject_ref: &str,
        authority_generation: u64,
        expires_at: u64,
    ) -> Self {
        Self {
            checkpoint_ref: checkpoint_ref.into(),
            subject_ref: subject_ref.into(),
            authority_generation,
            expires_at,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorV1 {
    pub checkpoint_ref: String,
    pub frontier: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CursorV2 {
    pub checkpoint_ref: String,
    pub frontier: Vec<String>,
    pub authority_generation: u64,
    pub valid_at: u64,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "version")]
pub enum CheckpointCursor {
    #[serde(rename = "1")]
    V1(CursorV1),
    #[serde(rename = "2")]
    V2(CursorV2),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum CursorError {
    InvalidEncoding,
    UnsupportedVersion(u64),
    MigrationRequired { found: u64, reader: u64 },
}

impl CheckpointCursor {
    pub fn read_for_version(bytes: &[u8], reader: u64) -> Result<Self, CursorError> {
        let value: serde_json::Value =
            serde_json::from_slice(bytes).map_err(|_| CursorError::InvalidEncoding)?;
        let raw_version = value.get("version").ok_or(CursorError::InvalidEncoding)?;
        let version = raw_version
            .as_u64()
            .or_else(|| raw_version.as_str().and_then(|v| v.parse().ok()))
            .ok_or(CursorError::InvalidEncoding)?;
        if version > 2 || version == 0 {
            return Err(CursorError::UnsupportedVersion(version));
        }
        if version > reader {
            return Err(CursorError::MigrationRequired {
                found: version,
                reader,
            });
        }
        serde_json::from_value(value).map_err(|_| CursorError::InvalidEncoding)
    }

    pub fn compatible_v1_projection(&self) -> Option<CursorV1> {
        match self {
            Self::V1(cursor) => Some(cursor.clone()),
            Self::V2(cursor) => Some(CursorV1 {
                checkpoint_ref: cursor.checkpoint_ref.clone(),
                frontier: cursor.frontier.clone(),
            }),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgetSurface {
    pub surface_ref: String,
    pub disposition: ForgetDisposition,
    pub replay: ReplayRestriction,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ForgetDisposition {
    Retained,
    Removed,
    Deferred,
}

impl ForgetSurface {
    pub fn removed(surface_ref: &str) -> Self {
        Self {
            surface_ref: surface_ref.into(),
            disposition: ForgetDisposition::Removed,
            replay: ReplayRestriction::Forbidden,
        }
    }

    pub fn retained(surface_ref: &str, replay: ReplayRestriction) -> Self {
        Self {
            surface_ref: surface_ref.into(),
            disposition: ForgetDisposition::Retained,
            replay,
        }
    }

    pub fn deferred(surface_ref: &str) -> Self {
        Self {
            surface_ref: surface_ref.into(),
            disposition: ForgetDisposition::Deferred,
            replay: ReplayRestriction::Forbidden,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ForgetReport {
    pub retained: Vec<String>,
    pub removed: Vec<String>,
    pub deferred: Vec<String>,
    pub replay: ReplayRestriction,
    pub state: LifecycleState,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct RecordedReplay {
    pub outcome_ref: String,
    pub restriction: ReplayRestriction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ExecutionIdentity {
    pub run_ref: String,
    pub attempt_ref: String,
    pub artifact_ref: String,
}

impl ExecutionIdentity {
    pub fn fresh(nonce: u64) -> Self {
        Self {
            run_ref: format!("run-{nonce}"),
            attempt_ref: format!("attempt-{nonce}"),
            artifact_ref: format!("artifact-{nonce}"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ConsistencyModel {
    OwnerAckedOutbox,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ArtifactAccess {
    Available,
    MetadataOnly,
    Restricted,
    Unavailable,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ArtifactView {
    pub state: LifecycleState,
    pub replay: ReplayRestriction,
    pub current_payload: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct BranchDecision {
    pub decision_ref: String,
    pub selected: Vec<String>,
    pub recorded_at: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct JoinFence {
    pub join_ref: String,
    pub authority_generation: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct WithdrawalReport {
    pub state: LifecycleState,
    pub acknowledgements: Vec<OwnerAcknowledgement>,
    pub pending_owners: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DurabilityRecord {
    pub record_ref: String,
    pub terminal: bool,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum DurabilityFault {
    #[default]
    None,
    FsyncBeforeRename,
    RenameOnly,
    PowerLoss,
}

#[derive(Debug, Default, Serialize, Deserialize)]
pub struct LifecycleCoordinator {
    #[serde(default)]
    effect_bindings: BTreeMap<String, EffectRequest>,
    effect_outcomes: BTreeMap<String, String>,
    started_effects: BTreeSet<String>,
    cancelled_parents: BTreeSet<String>,
    outbox: Vec<OutboxEntry>,
    checkpoint_payloads: BTreeMap<String, String>,
}

impl LifecycleCoordinator {
    pub fn reconstruct(artifacts: &[CanonicalArtifact]) -> Result<Reconstruction, LifecycleReason> {
        let mut requirements = BTreeSet::new();
        let mut frontier = BTreeSet::new();
        for artifact in artifacts {
            match artifact {
                CanonicalArtifact::Requirement { requirement_ref } => {
                    requirements.insert(requirement_ref.clone());
                }
                CanonicalArtifact::Frontier { ready } => frontier.extend(ready.iter().cloned()),
            }
        }
        if artifacts.is_empty() {
            return Err(LifecycleReason::ArtifactUnavailable);
        }
        Ok(Reconstruction {
            requirement_refs: requirements.into_iter().collect(),
            frontier: frontier.into_iter().collect(),
            transcript_required: false,
        })
    }

    pub fn rebuild_invalidation(entries: &[ChangeLogEntry]) -> Vec<String> {
        let mut edges: BTreeMap<String, BTreeSet<String>> = BTreeMap::new();
        let mut invalid = BTreeSet::new();
        for entry in entries {
            match entry {
                ChangeLogEntry::Dependency {
                    source, dependent, ..
                } => {
                    edges
                        .entry(source.clone())
                        .or_default()
                        .insert(dependent.clone());
                }
                ChangeLogEntry::Invalidate { artifact_ref, .. } => {
                    invalid.insert(artifact_ref.clone());
                }
            }
        }
        let mut queue: VecDeque<String> = invalid.iter().cloned().collect();
        while let Some(item) = queue.pop_front() {
            if let Some(dependents) = edges.get(&item) {
                for dependent in dependents {
                    if invalid.insert(dependent.clone()) {
                        queue.push_back(dependent.clone());
                    }
                }
            }
        }
        invalid.into_iter().collect()
    }

    pub fn reuse_cached(
        cached: &CachedOutput,
        _owner: &dyn EffectOwner,
        _request: &EffectRequest,
    ) -> LifecycleDecision {
        match cached.purity {
            Purity::Pure => LifecycleDecision {
                state: LifecycleState::Terminal,
                reason: LifecycleReason::None,
                replay: ReplayRestriction::RecordedOnly,
                outcome_ref: Some(cached.artifact_ref.clone()),
                redacted: false,
            },
            Purity::Impure => LifecycleDecision {
                state: LifecycleState::HistoricalOnly,
                reason: LifecycleReason::None,
                replay: ReplayRestriction::RecordedOnly,
                outcome_ref: Some(cached.artifact_ref.clone()),
                redacted: false,
            },
        }
    }

    pub fn resume_checkpoint(
        checkpoint: &CheckpointLease,
        trusted_now: u64,
        owner: &dyn AuthorityOwner,
    ) -> LifecycleDecision {
        if trusted_now > checkpoint.expires_at {
            return LifecycleDecision::new(LifecycleState::Blocked, LifecycleReason::LeaseExpired);
        }
        match owner.current_generation(&checkpoint.subject_ref) {
            Some(generation) if generation != checkpoint.authority_generation => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::GenerationFenced,
                );
            }
            None => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::AuthorityUnavailable,
                );
            }
            Some(_) => {}
        }
        let intent = LifecycleIntent::Resume {
            checkpoint_ref: checkpoint.checkpoint_ref.clone(),
            generation: checkpoint.authority_generation,
            valid_at: trusted_now,
            recorded_at: trusted_now,
        };
        match owner.authorize(&intent) {
            OwnerDecision::Confirmed => LifecycleDecision {
                state: LifecycleState::Admitted,
                reason: LifecycleReason::None,
                replay: ReplayRestriction::FreshEffectsAllowed,
                outcome_ref: None,
                redacted: false,
            },
            OwnerDecision::Rejected => {
                LifecycleDecision::new(LifecycleState::Blocked, LifecycleReason::AuthorityRevoked)
            }
            OwnerDecision::Unavailable => LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::AuthorityUnavailable,
            ),
        }
    }

    pub fn forget(surfaces: Vec<ForgetSurface>) -> ForgetReport {
        let mut report = ForgetReport {
            retained: Vec::new(),
            removed: Vec::new(),
            deferred: Vec::new(),
            replay: ReplayRestriction::RecordedOnly,
            state: LifecycleState::Terminal,
        };
        for surface in surfaces {
            report.replay = match (report.replay, surface.replay) {
                (ReplayRestriction::Forbidden, _) | (_, ReplayRestriction::Forbidden) => {
                    ReplayRestriction::Forbidden
                }
                (ReplayRestriction::ReconcileRequired, _)
                | (_, ReplayRestriction::ReconcileRequired) => ReplayRestriction::ReconcileRequired,
                _ => ReplayRestriction::RecordedOnly,
            };
            match surface.disposition {
                ForgetDisposition::Retained => report.retained.push(surface.surface_ref),
                ForgetDisposition::Removed => report.removed.push(surface.surface_ref),
                ForgetDisposition::Deferred => report.deferred.push(surface.surface_ref),
            }
        }
        if !report.removed.is_empty() || !report.deferred.is_empty() {
            report.replay = ReplayRestriction::Forbidden;
        }
        if !report.deferred.is_empty() {
            report.state = LifecycleState::Partial;
        }
        report
    }

    fn effect_binding_conflicts(&self, request: &EffectRequest) -> bool {
        match self.effect_bindings.get(&request.idempotency_key) {
            Some(bound) => bound != request,
            // Legacy snapshots carry no proof of the full request identity.
            None => {
                self.effect_outcomes.contains_key(&request.idempotency_key)
                    || self.started_effects.contains(&request.idempotency_key)
            }
        }
    }

    pub fn record_effect_outcome(
        &mut self,
        request: &EffectRequest,
        outcome_ref: &str,
    ) -> Result<(), LifecycleReason> {
        if self.effect_binding_conflicts(request) {
            return Err(LifecycleReason::EffectBindingConflict);
        }
        self.effect_bindings
            .insert(request.idempotency_key.clone(), request.clone());
        self.effect_outcomes
            .insert(request.idempotency_key.clone(), outcome_ref.into());
        self.started_effects.remove(&request.idempotency_key);
        Ok(())
    }

    pub fn record_effect_started(
        &mut self,
        request: &EffectRequest,
    ) -> Result<(), LifecycleReason> {
        if self.effect_binding_conflicts(request) {
            return Err(LifecycleReason::EffectBindingConflict);
        }
        self.effect_bindings
            .insert(request.idempotency_key.clone(), request.clone());
        self.started_effects.insert(request.idempotency_key.clone());
        Ok(())
    }

    pub fn reconcile_output_before_frontier(
        &self,
        request: &EffectRequest,
        _owner: &dyn EffectOwner,
    ) -> LifecycleDecision {
        if self.effect_binding_conflicts(request) {
            return LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::EffectBindingConflict,
            );
        }
        if let Some(outcome) = self.effect_outcomes.get(&request.idempotency_key) {
            let mut decision = LifecycleDecision::completed(outcome.clone());
            decision.state = LifecycleState::Reconciled;
            return decision;
        }
        LifecycleDecision::new(
            LifecycleState::Ambiguous,
            LifecycleReason::AmbiguousStartedEffect,
        )
    }

    pub fn recover_effect(
        &mut self,
        request: &EffectRequest,
        owner: &dyn EffectOwner,
    ) -> LifecycleDecision {
        if self.effect_binding_conflicts(request) {
            return LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::EffectBindingConflict,
            );
        }
        if let Some(outcome) = self.effect_outcomes.get(&request.idempotency_key) {
            let mut decision = LifecycleDecision::completed(outcome.clone());
            decision.state = LifecycleState::Reconciled;
            return decision;
        }
        if !self.started_effects.contains(&request.idempotency_key) {
            return LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::AmbiguousStartedEffect,
            );
        }
        match owner.reconcile(request) {
            EffectResolution::Completed { outcome_ref } => {
                if let Err(reason) = self.record_effect_outcome(request, &outcome_ref) {
                    return LifecycleDecision::new(LifecycleState::Blocked, reason);
                }
                LifecycleDecision {
                    state: LifecycleState::Reconciled,
                    reason: LifecycleReason::AmbiguousStartedEffect,
                    replay: ReplayRestriction::RecordedOnly,
                    outcome_ref: Some(outcome_ref),
                    redacted: false,
                }
            }
            EffectResolution::Rejected => LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::AmbiguousStartedEffect,
            ),
            EffectResolution::Unavailable | EffectResolution::Ambiguous => LifecycleDecision {
                state: LifecycleState::Ambiguous,
                reason: LifecycleReason::AmbiguousStartedEffect,
                replay: ReplayRestriction::ReconcileRequired,
                outcome_ref: None,
                redacted: false,
            },
        }
    }

    pub fn publish(
        &mut self,
        request: &EffectRequest,
        owner: &(impl AuthorityOwner + EffectOwner),
    ) -> LifecycleDecision {
        if self.effect_binding_conflicts(request) {
            return LifecycleDecision::new(
                LifecycleState::Blocked,
                LifecycleReason::EffectBindingConflict,
            );
        }
        if let Some(outcome) = self.effect_outcomes.get(&request.idempotency_key) {
            return LifecycleDecision::completed(outcome.clone());
        }
        match owner.current_generation(&request.effect_ref) {
            Some(generation) if generation != request.generation => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::GenerationFenced,
                );
            }
            None => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::AuthorityUnavailable,
                );
            }
            Some(_) => {}
        }
        match owner.authorize(&request.intent()) {
            OwnerDecision::Confirmed => {}
            OwnerDecision::Rejected => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::AuthorityRevoked,
                );
            }
            OwnerDecision::Unavailable => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::AuthorityUnavailable,
                );
            }
        }
        if self.started_effects.contains(&request.idempotency_key) {
            return self.recover_effect(request, owner);
        }
        let reconcile_only = match owner.begin_effect(request) {
            EffectStart::Started => false,
            EffectStart::AlreadyStarted => true,
            EffectStart::Rejected => {
                return LifecycleDecision::new(
                    LifecycleState::Blocked,
                    LifecycleReason::AuthorityRevoked,
                )
            }
            EffectStart::Unavailable => {
                return LifecycleDecision::new(
                    LifecycleState::Unavailable,
                    LifecycleReason::DurabilityUnproven,
                )
            }
        };
        if let Err(reason) = self.record_effect_started(request) {
            return LifecycleDecision::new(LifecycleState::Blocked, reason);
        }
        if reconcile_only {
            return self.recover_effect(request, owner);
        }
        match owner.perform(request) {
            EffectResolution::Completed { outcome_ref } => {
                if let Err(reason) = self.record_effect_outcome(request, &outcome_ref) {
                    return LifecycleDecision::new(LifecycleState::Blocked, reason);
                }
                LifecycleDecision::completed(outcome_ref)
            }
            EffectResolution::Rejected => {
                LifecycleDecision::new(LifecycleState::Blocked, LifecycleReason::AuthorityRevoked)
            }
            EffectResolution::Unavailable => LifecycleDecision::new(
                LifecycleState::Unavailable,
                LifecycleReason::AuthorityUnavailable,
            ),
            EffectResolution::Ambiguous => LifecycleDecision {
                state: LifecycleState::Ambiguous,
                reason: LifecycleReason::AmbiguousStartedEffect,
                replay: ReplayRestriction::ReconcileRequired,
                outcome_ref: None,
                redacted: false,
            },
        }
    }

    pub fn cancel(&mut self, parent_ref: &str) {
        self.cancelled_parents.insert(parent_ref.into());
    }

    pub fn publish_child(
        &mut self,
        parent_ref: &str,
        request: &EffectRequest,
        owner: &(impl AuthorityOwner + EffectOwner),
    ) -> LifecycleDecision {
        if self.cancelled_parents.contains(parent_ref) {
            return LifecycleDecision {
                state: LifecycleState::HistoricalOnly,
                reason: LifecycleReason::ParentCancelled,
                replay: ReplayRestriction::RecordedOnly,
                outcome_ref: None,
                redacted: true,
            };
        }
        self.publish(request, owner)
    }

    pub fn replay_recorded(replay: &RecordedReplay, _owner: &dyn EffectOwner) -> LifecycleDecision {
        LifecycleDecision {
            state: LifecycleState::Terminal,
            reason: LifecycleReason::None,
            replay: replay.restriction,
            outcome_ref: Some(replay.outcome_ref.clone()),
            redacted: false,
        }
    }

    pub fn enqueue_outbox(&mut self, request: EffectRequest) {
        if self
            .outbox
            .iter()
            .all(|entry| entry.request.idempotency_key != request.idempotency_key)
        {
            self.outbox.push(OutboxEntry {
                request,
                owner_outcome: None,
            });
        }
    }

    pub fn drain_outbox(
        &mut self,
        owner: &(impl AuthorityOwner + EffectOwner),
    ) -> Vec<LifecycleDecision> {
        let requests: Vec<_> = self
            .outbox
            .iter()
            .map(|entry| entry.request.clone())
            .collect();
        let decisions: Vec<_> = requests
            .iter()
            .map(|request| self.publish(request, owner))
            .collect();
        for (entry, decision) in self.outbox.iter_mut().zip(&decisions) {
            entry.owner_outcome = decision.outcome_ref.clone();
        }
        decisions
    }

    pub fn consistency_model(&self) -> ConsistencyModel {
        ConsistencyModel::OwnerAckedOutbox
    }

    pub fn claims_cross_store_atomicity(&self) -> bool {
        false
    }

    pub fn continue_from_artifact(
        artifact_ref: &str,
        owner: &dyn ArtifactOwner,
    ) -> LifecycleDecision {
        match (owner.access(artifact_ref), owner.is_pinned(artifact_ref)) {
            (ArtifactAccess::Available, OwnerDecision::Confirmed) => LifecycleDecision {
                state: LifecycleState::Admitted,
                reason: LifecycleReason::None,
                replay: ReplayRestriction::FreshEffectsAllowed,
                outcome_ref: Some(artifact_ref.into()),
                redacted: false,
            },
            (ArtifactAccess::Unavailable, _) => LifecycleDecision::new(
                LifecycleState::Unavailable,
                LifecycleReason::ArtifactUnavailable,
            ),
            (_, OwnerDecision::Unavailable) => LifecycleDecision::new(
                LifecycleState::Unavailable,
                LifecycleReason::AuthorityUnavailable,
            ),
            _ => LifecycleDecision::new(
                LifecycleState::Unavailable,
                LifecycleReason::ArtifactUnpinned,
            ),
        }
    }

    pub fn restore_branch(decision: BranchDecision, _owner: &dyn EffectOwner) -> BranchDecision {
        decision
    }

    pub fn coordinate_withdrawal(
        intent: &LifecycleIntent,
        owners: &[&str],
        port: &dyn WithdrawalOwner,
    ) -> WithdrawalReport {
        let recorded_at = match intent {
            LifecycleIntent::Resume { recorded_at, .. }
            | LifecycleIntent::Publish { recorded_at, .. }
            | LifecycleIntent::Withdraw { recorded_at, .. } => *recorded_at,
        };
        let acknowledgements: Vec<_> = owners
            .iter()
            .map(|owner| OwnerAcknowledgement {
                owner: (*owner).into(),
                decision: port.withdraw(owner, intent),
                recorded_at,
            })
            .collect();
        let pending_owners = acknowledgements
            .iter()
            .filter(|ack| ack.decision != OwnerDecision::Confirmed)
            .map(|ack| ack.owner.clone())
            .collect::<Vec<_>>();
        WithdrawalReport {
            state: if pending_owners.is_empty() {
                LifecycleState::Terminal
            } else {
                LifecycleState::Partial
            },
            acknowledgements,
            pending_owners,
        }
    }

    pub fn logical_wal_view(artifact_ref: &str, owner: &dyn ArtifactOwner) -> ArtifactView {
        match owner.access(artifact_ref) {
            ArtifactAccess::Available => ArtifactView {
                state: LifecycleState::Admitted,
                replay: ReplayRestriction::FreshEffectsAllowed,
                current_payload: true,
            },
            ArtifactAccess::MetadataOnly | ArtifactAccess::Restricted => ArtifactView {
                state: LifecycleState::Restricted,
                replay: ReplayRestriction::RecordedOnly,
                current_payload: false,
            },
            ArtifactAccess::Unavailable => ArtifactView {
                state: LifecycleState::Unavailable,
                replay: ReplayRestriction::Forbidden,
                current_payload: false,
            },
        }
    }

    pub fn store_checkpoint_payload(&mut self, checkpoint_ref: &str, payload: &str) {
        self.checkpoint_payloads
            .insert(checkpoint_ref.into(), payload.into());
    }

    pub fn privacy_delete(&mut self, checkpoint_ref: &str) -> LifecycleDecision {
        self.checkpoint_payloads.remove(checkpoint_ref);
        self.cancelled_parents.insert(checkpoint_ref.into());
        LifecycleDecision {
            state: LifecycleState::Cancelled,
            reason: LifecycleReason::PrivacyDeleted,
            replay: ReplayRestriction::Forbidden,
            outcome_ref: None,
            redacted: true,
        }
    }

    pub fn contains_plaintext(&self, plaintext: &str) -> bool {
        self.checkpoint_payloads
            .values()
            .any(|payload| payload.contains(plaintext))
    }

    pub fn checkpoint_payload(&self, checkpoint_ref: &str) -> Option<&str> {
        self.checkpoint_payloads
            .get(checkpoint_ref)
            .map(String::as_str)
    }

    pub fn capture_join(&mut self, join_ref: &str, authority_generation: u64) -> JoinFence {
        JoinFence {
            join_ref: join_ref.into(),
            authority_generation,
        }
    }

    pub fn publish_join(
        &mut self,
        joined: &JoinFence,
        request: &EffectRequest,
        owner: &(impl AuthorityOwner + EffectOwner),
    ) -> LifecycleDecision {
        if owner.current_generation(&joined.join_ref) != Some(joined.authority_generation) {
            return LifecycleDecision {
                state: LifecycleState::HistoricalOnly,
                reason: LifecycleReason::GenerationFenced,
                replay: ReplayRestriction::RecordedOnly,
                outcome_ref: None,
                redacted: true,
            };
        }
        self.publish(request, owner)
    }

    pub fn persist_and_reopen(
        record: &DurabilityRecord,
        owner: &dyn DurabilityOwner,
    ) -> LifecycleDecision {
        match owner.persist(record) {
            DurabilityFault::None if record.terminal => {
                LifecycleDecision::new(LifecycleState::Terminal, LifecycleReason::None)
            }
            DurabilityFault::None | DurabilityFault::FsyncBeforeRename => {
                LifecycleDecision::new(LifecycleState::Admitted, LifecycleReason::None)
            }
            DurabilityFault::RenameOnly | DurabilityFault::PowerLoss => LifecycleDecision::new(
                LifecycleState::Quarantined,
                LifecycleReason::DurabilityUnproven,
            ),
        }
    }
}
