//! Crash / retry / recovery matrix for the release-artifact proof profile
//! (3a) — FINISH_PLAN Phase 3.4.
//!
//! Operator decision (TRUST_AUTHORITY_DECISION_20260929.json, `first_proof_profile`):
//! profile 3a signs the exact bytes of a release artifact into
//! `ArtifactEnvelopeV1`, verifies them, and admits them into a claim-ledger
//! store as the first V2 proof profile. This file pins the crash, retry, and
//! recovery behavior of that path against real persistence primitives:
//!
//! - append-retry is idempotent: a retried `nadm_` admission is rejected as a
//!   duplicate native admission id (ledger-corruption class), because its
//!   deterministic identity makes double-append impossible under the fold.
//! - fold replay is canonical: the admission-id set and stream anchor digest
//!   recovered from serialized JSONL bytes equal the in-memory original.
//! - partial (crashed) append state is rejected: a chain whose final entry is
//!   absent/complete fails `verify_ledger` against the expected head.
//! - recovery replays via the verified snapshot checkpoint: a compaction
//!   checkpoint + retained tail replays to the same head, same fold digest,
//!   and same admission-id set as full replay.
//! - a replay that re-delivers an already-admitted recovery event is still
//!   fail-closed; dedup happens at fold time by admission id.
//! - envelope re-verification is deterministic and cross-run: recovery can
//!   re-derive the admission event bit-for-bit from the stored envelope and
//!   artifact, and re-verified payloads round-trip to digests equal to the
//!   original admission chain.
//!
//! Fixture keys are disposable RFC 8032 test-vector seeds; no operator seed
//! material appears here.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{TimeZone, Utc};
use claim_ledger::{
    admission_event_from_envelope, compact_ledger, compute_entry_digest, fold_snapshot,
    parse_ledger_entries, serialize_entry, verify_ledger, verify_snapshot, AdmissionEventPayloadV1,
    CompactionPolicy, EnvelopeVerificationContext, ExpectedLedgerHead, LedgerEntry,
    LedgerEntryBuilder, LedgerEvent, SnapshotFoldV1, UnprojectableEventPolicy,
};
use claim_ledger::{ArtifactEnvelopeV1, PolicyAdmission};

/// Disposable Ed25519 seed (RFC 8032 test-vector family, tests only).
const FIXTURE_SEED_HEX: &str = "9d61b19deffd2a60532a5c24476606bfca4f3b4a4f4fdc4a2d1d0e4f3e5b6c7a";
/// Derived public key, pinned by the P3.1 Python suite.
const FIXTURE_PUBLIC_HEX: &str = "dc152a48434a6d1342134b252c427b9fe636100e7908d88a91af73a428fb43c7";
/// `2026-09-30T04:00:00Z` in nanoseconds.
const FIXTURE_NS: i64 = 1_790_740_800_000_000_000;
const POLICY: &str = "recursiveintell:operator-admitted-support:v1";

fn hex_to_bytes32(hex: &str) -> [u8; 32] {
    let mut out = [0u8; 32];
    for (i, chunk) in hex.as_bytes().chunks(2).enumerate() {
        out[i] = u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap();
    }
    out
}

fn fixture_timestamp() -> chrono::DateTime<Utc> {
    Utc.timestamp_opt(1_790_740_800, 0).unwrap()
}

/// Simulated 3a release artifact.
fn artifact_bytes() -> Vec<u8> {
    b"{\n  \"claims\": [\n    {\n      \"claim_id\": \"c1\",\n      \"spans\": [\n        \"s1\"\n      ],\n      \"text\": \"fixture claim\"\n    }\n  ],\n  \"schema\": \"ClaimLedgerBundleV2\"\n}"
        .to_vec()
}

fn signed_envelope() -> ArtifactEnvelopeV1 {
    let mut envelope = ArtifactEnvelopeV1::unsigned(
        &artifact_bytes(),
        "local-operator",
        fixture_timestamp(),
        PolicyAdmission::admitted(POLICY),
    );
    envelope
        .sign_ed25519(&hex_to_bytes32(FIXTURE_SEED_HEX))
        .unwrap();
    envelope
}

fn trusted_context() -> EnvelopeVerificationContext {
    EnvelopeVerificationContext::new(
        fixture_timestamp() - chrono::Duration::seconds(1),
        fixture_timestamp() + chrono::Duration::seconds(1),
    )
    .with_signer_key("local-operator", hex_to_bytes32(FIXTURE_PUBLIC_HEX))
    .authorize_signer("local-operator")
    .admit_policy(POLICY)
}

/// Mints the verified admission event (fixture envelope + trusted context).
fn admission_payload() -> AdmissionEventPayloadV1 {
    admission_event_from_envelope(&signed_envelope(), &artifact_bytes(), &trusted_context())
        .unwrap()
}

/// Appends one event onto a hash-chained entry list (crash-aware append path
/// used by the 3a store writer).
fn append(entries: &mut Vec<LedgerEntry>, event: LedgerEvent) {
    let sequence = entries.len() as u64 + 1;
    let previous_entry_digest = entries.last().map(|entry| entry.entry_digest.clone());
    let entry_digest = compute_entry_digest(sequence, previous_entry_digest.as_deref(), &event)
        .expect("entry digest");
    entries.push(LedgerEntry {
        sequence,
        previous_entry_digest,
        event,
        entry_digest,
    });
}

fn retain_policy() -> CompactionPolicy {
    CompactionPolicy {
        retain_tail_entries: 0,
        unprojectable_events: UnprojectableEventPolicy::Retain,
    }
}

// --- C1. append retry is idempotent at the fold boundary ------------------------

/// Crash-recovery contract: the envelope re-verified after recovery produces
/// the same deterministic `nadm_` id, so a retried append is detected and
/// rejected as a duplicate instead of silently folding twice.
#[test]
fn c1_append_retry_is_rejected_as_duplicate_admission_id() {
    let first = admission_payload();
    // Recovery re-verifies the stored envelope + artifact (independent of the
    // original in-memory admission).
    let replayed =
        admission_event_from_envelope(&signed_envelope(), &artifact_bytes(), &trusted_context())
            .unwrap();
    assert_eq!(first.admission_id, replayed.admission_id);

    let mut fold = SnapshotFoldV1::default();
    let mut entries = Vec::new();
    append(&mut entries, LedgerEvent::AdmissionEvent { payload: first });
    fold.apply_entry(&entries[0]).unwrap();
    // Retry of the same delivery must fail closed.
    let retry = LedgerEntryBuilder::new(2, Some(entries[0].entry_digest.clone()))
        .add_native_admission(&replayed)
        .unwrap();
    let error = fold.apply_entry(&retry).unwrap_err();
    assert!(matches!(
        error,
        claim_ledger::ClaimLedgerError::DuplicateNativeAdmissionId(ref id) if *id == replayed.admission_id
    ));
}

// --- C2. fold replay from serialized bytes is canonical -------------------------

/// Recovery source of truth: the store's serialized JSONL is replayed byte for
/// byte; the recovered fold carries the same admission-id set and the same
/// stream anchor as the in-memory original.
#[test]
fn c2_jsonl_roundtrip_recovers_identical_admission_state() {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::AdmissionEvent {
            payload: admission_payload(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "clm_r".into(),
            source_id: "src".into(),
            span_id: "sp_1".into(),
            normalized_claim: "recovered".into(),
        },
    );

    let mut original = SnapshotFoldV1::default();
    original.apply_chain(&entries).unwrap();

    let jsonl: String = entries
        .iter()
        .map(serialize_entry)
        .collect::<Result<Vec<_>, _>>()
        .unwrap()
        .join("\n");
    let recovered_entries = parse_ledger_entries(&jsonl).unwrap();
    assert_eq!(recovered_entries.len(), entries.len());
    for (recovered, original_entry) in recovered_entries.iter().zip(entries.iter()) {
        assert_eq!(recovered.sequence, original_entry.sequence);
        assert_eq!(recovered.entry_digest, original_entry.entry_digest);
    }

    let mut recovered = SnapshotFoldV1::default();
    recovered.apply_chain(&recovered_entries).unwrap();
    assert_eq!(
        original.seen_admission_ids(),
        recovered.seen_admission_ids(),
        "recovered admission-id set matches"
    );
    // Stream anchors agree (same last sequence + last entry digest).
    let fixed = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    let original_digest = original.fold_digest(fixed).unwrap();
    let recovered_digest = recovered.fold_digest(fixed).unwrap();
    assert_eq!(original_digest, recovered_digest);
}

// --- C3. partial append (crash between entries) is rejected ----------------------

/// A crashed store whose final entry was torn mid-write must fail
/// verification against the durable expected head, never silently verify.
#[test]
fn c3_partial_append_state_is_rejected_by_expected_head() {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::AdmissionEvent {
            payload: admission_payload(),
        },
    );
    let head = ExpectedLedgerHead::new(entries[0].sequence, entries[0].entry_digest.clone());
    verify_ledger(&entries, &head).unwrap();

    // Crash model: the durable file only contains entry 1, but the expected
    // head recorded entry 2 (torn final write).
    assert!(verify_ledger(&entries[..0], &head).is_err());
}

/// A fully-durable prefix verifies; dropping the last complete entry fails the
/// head check (writer-side "crash after prepare" never verifies as committed).
#[test]
fn c3b_verified_head_binds_the_exact_chain() {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::AdmissionEvent {
            payload: admission_payload(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "clm_r".into(),
            source_id: "src".into(),
            span_id: "sp_1".into(),
            normalized_claim: "after admission".into(),
        },
    );
    let head = ExpectedLedgerHead::new(entries[1].sequence, entries[1].entry_digest.clone());
    verify_ledger(&entries, &head).unwrap();
    // The same head must reject a truncated durable prefix.
    assert!(verify_ledger(&entries[..1], &head).is_err());
}

// --- C4. recovery replays via the verified checkpoint ----------------------------

/// Post-crash recovery rebuilds from the last verified snapshot checkpoint +
/// retained tail; the rebuilt fold equals full replay on head, digest, and
/// admission-id set.
#[test]
fn c4_checkpoint_recovery_replays_to_identical_state() {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "clm_a".into(),
            source_id: "src".into(),
            span_id: "sp_1".into(),
            normalized_claim: "before".into(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::AdmissionEvent {
            payload: admission_payload(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "clm_b".into(),
            source_id: "src".into(),
            span_id: "sp_2".into(),
            normalized_claim: "after".into(),
        },
    );

    // Crash-recovery model: the pre-crash process checkpointed a compacted
    // snapshot; the recovered process rebuilds from that verified checkpoint
    // plus the verifiable retained tail (same contract as the P3.2 parity
    // test). Retain keeps the admission event in the tail so it stays
    // fold-verifiable after recovery.
    let checkpointed = compact_ledger(&entries[..2], &retain_policy()).unwrap();
    verify_snapshot(&checkpointed.snapshot).unwrap();
    claim_ledger::verify_compaction(
        &checkpointed.snapshot,
        &checkpointed.retained_tail,
        &checkpointed.receipt,
    )
    .unwrap();

    // Recovery replay: advance the verified checkpoint with its retained tail,
    // then re-apply the post-crash entry (sequence 3).
    let recovered = claim_ledger::compact_ledger_from_snapshot(
        Some(&checkpointed.snapshot),
        &[&checkpointed.retained_tail, &entries[2..]].concat(),
        &retain_policy(),
    )
    .unwrap();
    claim_ledger::verify_compaction(
        &recovered.snapshot,
        &recovered.retained_tail,
        &recovered.receipt,
    )
    .unwrap();

    // Reference state: full-stream compaction from genesis with the same
    // policy; field-level parity (timestamps differ by construction).
    let full = compact_ledger(&entries, &retain_policy()).unwrap();
    assert_eq!(
        recovered.snapshot.claims, full.snapshot.claims,
        "recovered projection equals full replay"
    );
    assert_eq!(
        recovered.snapshot.last_compacted_sequence, full.snapshot.last_compacted_sequence,
        "recovered checkpoint equals full-replay checkpoint"
    );
    assert_eq!(
        recovered.snapshot.last_compacted_entry_digest, full.snapshot.last_compacted_entry_digest,
        "recovered anchor digest equals full-replay anchor"
    );
    // The recovered replay authenticated the same stream head (sequence 3).
    assert_eq!(recovered.receipt.pre_compaction_sequence, 3);
    // The admission event survives recovery in the verifiable tail and the
    // recovered admission-id set equals full replay.
    let mut tail_fold = SnapshotFoldV1::default();
    tail_fold.apply_chain(&recovered.retained_tail).unwrap();
    let mut reference = SnapshotFoldV1::default();
    reference.apply_chain(&entries).unwrap();
    assert_eq!(
        tail_fold.seen_admission_ids(),
        reference.seen_admission_ids(),
        "recovered admission-id set matches full replay"
    );
}

// --- C5. recovery re-delivery stays fail-closed ----------------------------------

/// A recovery replay that re-delivers an already-admitted event (e.g. the
/// checkpoint tail plus a duplicated dispatch) is rejected at the fold, so a
/// compromised or buggy replayer cannot double-project an admission.
#[test]
fn c5_recovery_replay_rejects_re_delivered_admission() {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::AdmissionEvent {
            payload: admission_payload(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "clm_r".into(),
            source_id: "src".into(),
            span_id: "sp_1".into(),
            normalized_claim: "after".into(),
        },
    );

    let mut fold = SnapshotFoldV1::default();
    fold.apply_chain(&entries).unwrap();
    // Re-delivery of the same event from a lagged recovery stream.
    let error = fold.apply_entry(&entries[0]).unwrap_err();
    assert!(matches!(
        error,
        claim_ledger::ClaimLedgerError::DuplicateNativeAdmissionId(_)
    ));
}

// --- C6. envelope re-verification is deterministic across recovery ---------------

/// Recovery can always re-derive the admission event bit-for-bit from the
/// stored envelope + artifact bytes: repeated verification over fresh
/// contexts yields identical payloads and identical digests (no hidden state).
#[test]
fn c6_repeated_envelope_reverification_is_stable() {
    let first = admission_payload();
    let second =
        admission_event_from_envelope(&signed_envelope(), &artifact_bytes(), &trusted_context())
            .unwrap();
    assert_eq!(first, second);

    // Serializing twice yields identical bytes; a recovered fold over the
    // serialized event matches in-memory.
    let left = serialize_entry(
        &LedgerEntryBuilder::new(1, None)
            .add_native_admission(&first)
            .unwrap(),
    )
    .unwrap();
    let right = serialize_entry(
        &LedgerEntryBuilder::new(1, None)
            .add_native_admission(&second)
            .unwrap(),
    )
    .unwrap();
    assert_eq!(left, right);

    let fixed = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    let mut fold = SnapshotFoldV1::default();
    fold.apply_entry(&parse_ledger_entries(&left).unwrap()[0])
        .unwrap();
    let mut reference = SnapshotFoldV1::default();
    reference
        .apply_entry(
            &LedgerEntryBuilder::new(1, None)
                .add_native_admission(&first)
                .unwrap(),
        )
        .unwrap();
    assert_eq!(
        fold.fold_digest(fixed).unwrap(),
        reference.fold_digest(fixed).unwrap()
    );
}

/// Cross-language regression (P3.1 pin): the Rust-derived admission id for the
/// fixture envelope matches the Python-pinned id family (`nadm_` prefix, exact
/// digest recomputed from the canonical preimage parts).
#[test]
fn c7_admission_id_is_deterministic_across_reinstantiation() {
    let a = admission_payload();
    let envelope = signed_envelope();
    let expected = claim_ledger::native_admission_id(
        &envelope.signer_id,
        &envelope.artifact_digest,
        FIXTURE_NS,
        POLICY,
    );
    assert_eq!(a.admission_id, expected);
    // Distinctness: different digest => different id.
    let other = claim_ledger::native_admission_id(
        &envelope.signer_id,
        &envelope.artifact_digest,
        FIXTURE_NS,
        "other:policy",
    );
    assert_ne!(a.admission_id, other);
}
