use agent_graph::lifecycle::*;
use std::cell::{Cell, RefCell};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Default)]
struct Owners {
    authority_generation: Cell<Option<u64>>,
    authority: Cell<OwnerDecision>,
    effect_calls: Cell<usize>,
    mutation_calls: Cell<usize>,
    reconcile_calls: Cell<usize>,
    artifacts: RefCell<BTreeMap<String, ArtifactAccess>>,
    pins: RefCell<BTreeSet<String>>,
    withdrawal: RefCell<BTreeMap<String, OwnerDecision>>,
    durability: Cell<DurabilityFault>,
}

impl AuthorityOwner for Owners {
    fn current_generation(&self, _subject: &str) -> Option<u64> {
        self.authority_generation.get()
    }

    fn authorize(&self, _intent: &LifecycleIntent) -> OwnerDecision {
        self.authority.get()
    }
}

impl EffectOwner for Owners {
    fn perform(&self, request: &EffectRequest) -> EffectResolution {
        self.effect_calls.set(self.effect_calls.get() + 1);
        self.mutation_calls.set(self.mutation_calls.get() + 1);
        EffectResolution::Completed {
            outcome_ref: format!("outcome:{}", request.idempotency_key),
        }
    }

    fn reconcile(&self, request: &EffectRequest) -> EffectResolution {
        self.reconcile_calls.set(self.reconcile_calls.get() + 1);
        EffectResolution::Completed {
            outcome_ref: format!("outcome:{}", request.idempotency_key),
        }
    }
}

impl ArtifactOwner for Owners {
    fn access(&self, artifact_ref: &str) -> ArtifactAccess {
        self.artifacts
            .borrow()
            .get(artifact_ref)
            .copied()
            .unwrap_or(ArtifactAccess::Unavailable)
    }

    fn is_pinned(&self, artifact_ref: &str) -> OwnerDecision {
        if self.pins.borrow().contains(artifact_ref) {
            OwnerDecision::Confirmed
        } else {
            OwnerDecision::Rejected
        }
    }
}

impl WithdrawalOwner for Owners {
    fn withdraw(&self, owner: &str, _intent: &LifecycleIntent) -> OwnerDecision {
        self.withdrawal
            .borrow()
            .get(owner)
            .copied()
            .unwrap_or(OwnerDecision::Unavailable)
    }
}

impl DurabilityOwner for Owners {
    fn persist(&self, _record: &DurabilityRecord) -> DurabilityFault {
        self.durability.get()
    }
}

fn owners() -> Owners {
    let owners = Owners::default();
    owners.authority_generation.set(Some(7));
    owners.authority.set(OwnerDecision::Confirmed);
    owners.durability.set(DurabilityFault::None);
    owners
}

fn effect(key: &str) -> EffectRequest {
    EffectRequest {
        effect_ref: "provider-call".into(),
        idempotency_key: key.into(),
        generation: 7,
        valid_at: 10,
        recorded_at: 11,
    }
}

#[test]
fn mig_09_old_reader_typed_rejects_new_cursor_semantics_or_reads_compatible_projection() {
    let cursor = CheckpointCursor::V2(CursorV2 {
        checkpoint_ref: "cp-1".into(),
        frontier: vec!["node-b".into()],
        authority_generation: 7,
        valid_at: 10,
        recorded_at: 12,
    });
    let encoded = serde_json::to_vec(&cursor).unwrap();

    assert_eq!(
        CheckpointCursor::read_for_version(&encoded, 1),
        Err(CursorError::MigrationRequired {
            found: 2,
            reader: 1
        })
    );
    assert_eq!(
        cursor.compatible_v1_projection().unwrap().checkpoint_ref,
        "cp-1"
    );
}

#[test]
fn ctx_13_reconstructs_requirements_and_frontier_from_canonical_artifacts_without_transcript() {
    let artifacts = vec![
        CanonicalArtifact::Requirement {
            requirement_ref: "REQ-9".into(),
        },
        CanonicalArtifact::Frontier {
            ready: vec!["verify".into(), "publish".into()],
        },
    ];

    let rebuilt = LifecycleCoordinator::reconstruct(&artifacts).unwrap();
    assert_eq!(rebuilt.requirement_refs, vec!["REQ-9"]);
    assert_eq!(rebuilt.frontier, vec!["publish", "verify"]);
    assert!(!rebuilt.transcript_required);
}

#[test]
fn dep_12_crash_after_change_log_before_projection_rebuilds_same_invalidation_closure() {
    let entries = vec![
        ChangeLogEntry::dependency("basis", "analysis", 1),
        ChangeLogEntry::dependency("analysis", "closure", 2),
        ChangeLogEntry::invalidate("basis", 3),
    ];
    let expected = LifecycleCoordinator::rebuild_invalidation(&entries);
    let serialized = serde_json::to_vec(&entries).unwrap();
    let reopened: Vec<ChangeLogEntry> = serde_json::from_slice(&serialized).unwrap();

    assert_eq!(expected, vec!["analysis", "basis", "closure"]);
    assert_eq!(
        LifecycleCoordinator::rebuild_invalidation(&reopened),
        expected
    );
}

#[test]
fn dep_15_cached_impure_output_is_historical_only_and_never_replays_effect() {
    let owners = owners();
    let cached = CachedOutput {
        artifact_ref: "cached-provider-output".into(),
        purity: Purity::Impure,
        recorded_at: 9,
    };

    let decision = LifecycleCoordinator::reuse_cached(&cached, &owners, &effect("cache-key"));
    assert_eq!(decision.state, LifecycleState::HistoricalOnly);
    assert_eq!(decision.replay, ReplayRestriction::RecordedOnly);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn auth_01_revoked_current_authority_blocks_old_checkpoint() {
    let owners = owners();
    owners.authority_generation.set(Some(8));
    let checkpoint = CheckpointLease::new("cp-old", "subject", 7, 100);

    let decision = LifecycleCoordinator::resume_checkpoint(&checkpoint, 20, &owners);
    assert_eq!(decision.state, LifecycleState::Blocked);
    assert_eq!(decision.reason, LifecycleReason::GenerationFenced);
}

#[test]
fn auth_06_forgetting_enumerates_retained_removed_deferred_surfaces_and_replay_restrictions() {
    let report = LifecycleCoordinator::forget(vec![
        ForgetSurface::removed("prompt"),
        ForgetSurface::retained("receipt", ReplayRestriction::RecordedOnly),
        ForgetSurface::deferred("remote-index"),
    ]);

    assert_eq!(report.removed, vec!["prompt"]);
    assert_eq!(report.retained, vec!["receipt"]);
    assert_eq!(report.deferred, vec!["remote-index"]);
    assert_eq!(report.replay, ReplayRestriction::Forbidden);
    assert_eq!(report.state, LifecycleState::Partial);
}

#[test]
fn life_01_output_before_frontier_crash_reconciles_without_duplicate_provider_call() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let request = effect("life-01");
    coordinator.record_effect_outcome(&request, "outcome:life-01");

    let decision = coordinator.reconcile_output_before_frontier(&request, &owners);
    assert_eq!(decision.state, LifecycleState::Reconciled);
    assert_eq!(decision.outcome_ref.as_deref(), Some("outcome:life-01"));
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn life_02_effect_before_receipt_becomes_ambiguous_reconcile_and_no_blind_retry() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let request = effect("life-02");
    coordinator.record_effect_started(&request);

    let decision = coordinator.recover_effect(&request, &owners);
    assert_eq!(decision.state, LifecycleState::Reconciled);
    assert_eq!(decision.reason, LifecycleReason::AmbiguousStartedEffect);
    assert_eq!(owners.effect_calls.get(), 0);
    assert_eq!(owners.reconcile_calls.get(), 1);
}

#[test]
fn life_04_duplicate_publication_key_yields_one_owner_mutation_and_original_outcome() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let request = effect("publish-once");

    let first = coordinator.publish(&request, &owners);
    let duplicate = coordinator.publish(&request, &owners);
    assert_eq!(first, duplicate);
    assert_eq!(first.outcome_ref.as_deref(), Some("outcome:publish-once"));
    assert_eq!(owners.mutation_calls.get(), 1);
}

#[test]
fn life_05_late_child_after_parent_cancellation_cannot_publish_current() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    coordinator.cancel("parent");

    let decision = coordinator.publish_child("parent", &effect("late-child"), &owners);
    assert_eq!(decision.state, LifecycleState::HistoricalOnly);
    assert_eq!(decision.reason, LifecycleReason::ParentCancelled);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn life_06_expired_or_transferred_lease_fences_future_actions_and_preserves_started_ambiguity() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let expired = CheckpointLease::new("cp-expired", "subject", 7, 15);
    let started = effect("lease-started");
    coordinator.record_effect_started(&started);

    let future = LifecycleCoordinator::resume_checkpoint(&expired, 20, &owners);
    let ambiguous = coordinator.recover_effect(&started, &owners);
    assert_eq!(future.reason, LifecycleReason::LeaseExpired);
    assert_eq!(future.state, LifecycleState::Blocked);
    assert_eq!(ambiguous.reason, LifecycleReason::AmbiguousStartedEffect);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn life_07_recorded_replay_uses_zero_backend_network_or_effect_calls() {
    let owners = owners();
    let replay = RecordedReplay {
        outcome_ref: "recorded:42".into(),
        restriction: ReplayRestriction::RecordedOnly,
    };

    let decision = LifecycleCoordinator::replay_recorded(&replay, &owners);
    assert_eq!(decision.outcome_ref.as_deref(), Some("recorded:42"));
    assert_eq!(decision.state, LifecycleState::Terminal);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn life_08_fresh_run_gets_distinct_run_attempt_and_artifact_identity() {
    let first = ExecutionIdentity::fresh(1);
    let second = ExecutionIdentity::fresh(2);

    assert_ne!(first.run_ref, second.run_ref);
    assert_ne!(first.attempt_ref, second.attempt_ref);
    assert_ne!(first.artifact_ref, second.artifact_ref);
}

#[test]
fn life_09_outbox_crash_reconciles_idempotently_and_disclaims_atomic_cross_store_transaction() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let request = effect("outbox-key");
    coordinator.enqueue_outbox(request.clone());

    let first = coordinator.drain_outbox(&owners);
    let second = coordinator.drain_outbox(&owners);
    assert_eq!(first, second);
    assert_eq!(owners.mutation_calls.get(), 1);
    assert_eq!(
        coordinator.consistency_model(),
        ConsistencyModel::OwnerAckedOutbox
    );
    assert!(!coordinator.claims_cross_store_atomicity());
}

#[test]
fn life_10_unsupported_checkpoint_cursor_version_typed_reject_or_migrate() {
    let bytes = br#"{"version":99,"checkpoint_ref":"cp","frontier":[]}"#;
    assert_eq!(
        CheckpointCursor::read_for_version(bytes, 2),
        Err(CursorError::UnsupportedVersion(99))
    );
}

#[test]
fn life_11_active_pin_protects_artifact_or_marks_continuation_unavailable() {
    let owners = owners();
    owners.pins.borrow_mut().insert("artifact-pinned".into());
    owners
        .artifacts
        .borrow_mut()
        .insert("artifact-pinned".into(), ArtifactAccess::Available);

    let protected = LifecycleCoordinator::continue_from_artifact("artifact-pinned", &owners);
    let unavailable = LifecycleCoordinator::continue_from_artifact("artifact-missing", &owners);
    assert_eq!(protected.state, LifecycleState::Admitted);
    assert_eq!(unavailable.state, LifecycleState::Unavailable);
    assert_eq!(unavailable.replay, ReplayRestriction::Forbidden);
}

#[test]
fn kern_09_durable_graph_branch_decision_survives_crash_with_zero_effect_calls() {
    let owners = owners();
    let decision = BranchDecision {
        decision_ref: "branch-decision-1".into(),
        selected: vec!["safe-branch".into()],
        recorded_at: 12,
    };
    let bytes = serde_json::to_vec(&decision).unwrap();
    let reopened: BranchDecision = serde_json::from_slice(&bytes).unwrap();

    let restored = LifecycleCoordinator::restore_branch(reopened, &owners);
    assert_eq!(restored.selected, vec!["safe-branch"]);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn lif_01_cross_owner_withdrawal_stays_partial_until_every_owner_acknowledges() {
    let owners = owners();
    owners.withdrawal.borrow_mut().extend([
        ("authority".into(), OwnerDecision::Confirmed),
        ("artifact".into(), OwnerDecision::Unavailable),
    ]);
    let intent = LifecycleIntent::Withdraw {
        subject_ref: "subject".into(),
        valid_at: 10,
        recorded_at: 11,
    };

    let partial =
        LifecycleCoordinator::coordinate_withdrawal(&intent, &["authority", "artifact"], &owners);
    assert_eq!(partial.state, LifecycleState::Partial);
    assert_eq!(partial.pending_owners, vec!["artifact"]);

    owners
        .withdrawal
        .borrow_mut()
        .insert("artifact".into(), OwnerDecision::Confirmed);
    let complete =
        LifecycleCoordinator::coordinate_withdrawal(&intent, &["authority", "artifact"], &owners);
    assert_eq!(complete.state, LifecycleState::Terminal);
}

#[test]
fn lif_02_logical_wal_view_without_retained_bytes_is_restricted_or_unavailable_not_same_name_current(
) {
    let owners = owners();
    owners
        .artifacts
        .borrow_mut()
        .insert("logical-entry".into(), ArtifactAccess::MetadataOnly);

    let view = LifecycleCoordinator::logical_wal_view("logical-entry", &owners);
    assert_eq!(view.state, LifecycleState::Restricted);
    assert!(!view.current_payload);
    assert_eq!(view.replay, ReplayRestriction::RecordedOnly);
}

#[test]
fn lif_03_privacy_deletion_cancels_or_restricts_checkpoint_and_leaves_no_hidden_plaintext() {
    let mut coordinator = LifecycleCoordinator::default();
    coordinator.store_checkpoint_payload("cp-private", "secret plaintext");

    let report = coordinator.privacy_delete("cp-private");
    assert_eq!(report.state, LifecycleState::Cancelled);
    assert_eq!(report.replay, ReplayRestriction::Forbidden);
    assert!(!coordinator.contains_plaintext("secret plaintext"));
    assert_eq!(coordinator.checkpoint_payload("cp-private"), None);
}

#[test]
fn lif_04_revocation_between_join_and_publication_fences_or_redacts_current_output() {
    let owners = owners();
    let mut coordinator = LifecycleCoordinator::default();
    let joined = coordinator.capture_join("join-1", 7);
    owners.authority_generation.set(Some(8));

    let publication = coordinator.publish_join(&joined, &effect("joined-output"), &owners);
    assert_eq!(publication.state, LifecycleState::HistoricalOnly);
    assert_eq!(publication.reason, LifecycleReason::GenerationFenced);
    assert!(publication.redacted);
    assert_eq!(owners.effect_calls.get(), 0);
}

#[test]
fn lif_05_injected_fsync_rename_and_power_loss_faults_reopen_as_admitted_terminal_or_quarantined() {
    let owners = owners();
    let record = DurabilityRecord {
        record_ref: "record-1".into(),
        terminal: true,
    };

    owners.durability.set(DurabilityFault::FsyncBeforeRename);
    assert_eq!(
        LifecycleCoordinator::persist_and_reopen(&record, &owners).state,
        LifecycleState::Admitted
    );
    owners.durability.set(DurabilityFault::None);
    assert_eq!(
        LifecycleCoordinator::persist_and_reopen(&record, &owners).state,
        LifecycleState::Terminal
    );
    for fault in [DurabilityFault::RenameOnly, DurabilityFault::PowerLoss] {
        owners.durability.set(fault);
        assert_eq!(
            LifecycleCoordinator::persist_and_reopen(&record, &owners).state,
            LifecycleState::Quarantined
        );
    }
}
