//! V2 admission-chain tests (FINISH_PLAN Phase 3.2): native admission events
//! with distinct IDs, envelope-gated admission, snapshot fold determinism.
//!
//! Fixture keys are disposable RFC 8032 test-vector seeds used for tests only;
//! no operator seed material appears in this file. Wire vectors are pinned to
//! the merged P3.1 Python implementation (ClaimLedger PR #19, head ddd3a08):
//! 1. artifact digest: `ARTIFACT_DIGEST_DOMAIN` + u64BE len + bytes,
//!    `b"release artifact bytes"` -> `8405d41f17b860dd8b396a4b28eab15e8e632
//!    986fe727b66c068acc2b88c70ce`.
//! 2. signature preimage: `SIGNATURE_DOMAIN` + u64BE-len-prefixed fields
//!    (envelope digest, signer id, i64 nanos as EXACTLY 8 BE bytes, policy id,
//!    admitted byte). Pinned over (ab*32, local-operator, 1759000000000000000,
//!    recursiveintell:operator-admitted-support:v1, true) -> 218 bytes,
//!    sha256 `e8ced39e1c18ce8e6b9660f9d079a66faef4e2cb416a7bbceb97aee4bd6c940b`.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{TimeZone, Utc};
use claim_ledger::{
    admission_event_from_envelope, admission_signature_preimage, compact_ledger, fold_snapshot,
    native_admission_id, public_artifact_digest, verify_admission, AdmissionDecision,
    ClaimLedgerError, CompactionPolicy, EnvelopeVerificationContext, EnvelopeVerificationStatus,
    LedgerEntry, LedgerEntryBuilder, LedgerEvent, SnapshotFoldV1, UnprojectableEventPolicy,
};
use claim_ledger::{ArtifactEnvelopeV1, PolicyAdmission};

const SIGNATURE_DOMAIN: &[u8] = b"recursiveintell:artifact-envelope:signature:v1\0";

/// Disposable Ed25519 seed (RFC 8032 test-vector family, tests only).
const FIXTURE_SEED_HEX: &str = "9d61b19deffd2a60532a5c24476606bfca4f3b4a4f4fdc4a2d1d0e4f3e5b6c7a";
/// Derived public key, pinned by the P3.1 Python suite.
const FIXTURE_PUBLIC_HEX: &str = "dc152a48434a6d1342134b252c427b9fe636100e7908d88a91af73a428fb43c7";

/// `2026-09-30T04:00:00Z` in nanoseconds (== 1790740800000000000).
const FIXTURE_NS: i64 = 1_790_740_800_000_000_000;
/// P3.1-pinned Ed25519 signature bytes over the fixture preimage.
const FIXTURE_SIG_HEX: &str = concat!(
    "94070b49e3bbdcaf795a7b169ab379ef51bb633ad053c3e50a890c149f550baf5",
    "28a1b39b7e49ce62fe8a001a6cc4ac32987efea33b0820c8f13859390dcce00"
);
const POLICY: &str = "recursiveintell:operator-admitted-support:v1";

/// Canonical 161-byte fixture bundle bytes (python json.dumps(indent=2, sort_keys=True)).
fn fixture_bytes() -> Vec<u8> {
    b"{\n  \"claims\": [\n    {\n      \"claim_id\": \"c1\",\n      \"spans\": [\n        \"s1\"\n      ],\n      \"text\": \"fixture claim\"\n    }\n  ],\n  \"schema\": \"ClaimLedgerBundleV2\"\n}"
        .to_vec()
}

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

fn fixture_envelope() -> ArtifactEnvelopeV1 {
    let mut envelope = ArtifactEnvelopeV1::unsigned(
        &fixture_bytes(),
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

/// Mints a verified admission event (fixture envelope + trusted context).
fn fixture_admission_payload() -> claim_ledger::AdmissionEventPayloadV1 {
    admission_event_from_envelope(&fixture_envelope(), &fixture_bytes(), &trusted_context())
        .unwrap()
}

// --- 1. pinned wire vectors ----------------------------------------------------

#[test]
fn artifact_digest_matches_pinned_vector() {
    let digest = public_artifact_digest(b"release artifact bytes");
    assert_eq!(
        digest,
        "8405d41f17b860dd8b396a4b28eab15e8e632986fe727b66c068acc2b88c70ce"
    );
}

#[test]
fn admission_signature_preimage_matches_pinned_vector() {
    let preimage = admission_signature_preimage(
        &("ab".repeat(32)),
        "local-operator",
        1_759_000_000_000_000_000,
        POLICY,
        true,
    )
    .unwrap();
    assert_eq!(preimage.len(), 218);
    // Exact layout: domain prefix, then u64BE-len-prefixed fields.
    assert_eq!(&preimage[..SIGNATURE_DOMAIN.len()], SIGNATURE_DOMAIN);
    let ns = (1_759_000_000_000_000_000_i64).to_be_bytes();
    assert!(
        preimage.windows(8).any(|window| window == ns),
        "8-byte BE nanos embedded"
    );
    assert_eq!(
        claim_ledger::sha256_bytes(&preimage),
        "e8ced39e1c18ce8e6b9660f9d079a66faef4e2cb416a7bbceb97aee4bd6c940b"
    );
}

// --- 2. native admission construction + Ed25519 parity -------------------------

#[test]
fn rust_fixture_signature_matches_pinned_python_vector() {
    // Disposable seed signs the canonical preimage; the result must equal the
    // Python-pinned FIXTURE_SIG_HEX bit-for-bit (cross-language parity).
    let envelope = fixture_envelope();
    let expected: Vec<u8> = FIXTURE_SIG_HEX
        .as_bytes()
        .chunks(2)
        .map(|chunk| u8::from_str_radix(std::str::from_utf8(chunk).unwrap(), 16).unwrap())
        .collect();
    assert_eq!(
        envelope.signature.clone().unwrap(),
        expected,
        "Rust signature must equal the P3.1 pinned vector"
    );
    assert_eq!(
        envelope.signer_public_key().unwrap(),
        hex_to_bytes32(FIXTURE_PUBLIC_HEX)
    );
    assert_eq!(
        envelope.artifact_digest,
        public_artifact_digest(&fixture_bytes())
    );
}

#[test]
fn native_admission_id_is_distinct_and_deterministic() {
    let a = native_admission_id("local-operator", "digest-1", FIXTURE_NS, POLICY);
    let b = native_admission_id("local-operator", "digest-1", FIXTURE_NS + 1, POLICY);
    let c = native_admission_id("local-operator", "digest-1", FIXTURE_NS, POLICY);
    assert_ne!(a, b, "distinct timestamp -> distinct admission id");
    assert_eq!(a, c, "same identity parts -> same admission id");
    assert!(a.starts_with("nadm_"));
}

// --- 3. admission-event ledger entries -----------------------------------------

#[test]
fn verified_envelope_mints_admission_event_and_roundtrips() {
    let payload = fixture_admission_payload();
    assert!(payload.admission_id.starts_with("nadm_"));
    assert_eq!(
        payload.envelope_verification,
        EnvelopeVerificationStatus::FullyVerified
    );
    assert_eq!(
        payload.admitted_support_state,
        claim_ledger::SupportState::Supported
    );
    let entry = LedgerEntryBuilder::new(1, None)
        .add_native_admission(&payload)
        .unwrap();
    assert!(matches!(entry.event, LedgerEvent::AdmissionEvent { .. }));
    let json = claim_ledger::serialize_entry(&entry).unwrap();
    let parsed = claim_ledger::parse_ledger_entries(&json).unwrap();
    assert_eq!(parsed.len(), 1);
    match &parsed[0].event {
        LedgerEvent::AdmissionEvent {
            payload: parsed_payload,
        } => {
            assert_eq!(parsed_payload.admission_id, payload.admission_id);
            assert_eq!(parsed_payload.signer_id, "local-operator");
        }
        other => panic!("unexpected event: {}", other.type_name()),
    }
}

#[test]
fn admission_event_type_name_is_stable() {
    let entry = LedgerEntryBuilder::new(1, None)
        .add_native_admission(&fixture_admission_payload())
        .unwrap();
    assert_eq!(entry.event.type_name(), "admission_event");
}

// --- 4. envelope-gated admission (staged ladder) -------------------------------

#[test]
fn verified_admission_projects_support() {
    let envelope = fixture_envelope();
    match verify_admission(&envelope, Some(&fixture_bytes()), &trusted_context()).unwrap() {
        AdmissionDecision::ProjectSupport(state) => {
            assert_eq!(state, claim_ledger::SupportState::Supported);
        }
        other => panic!("expected ProjectSupport, got {other:?}"),
    }
}

#[test]
fn admission_with_tampered_artifact_bytes_is_quarantined() {
    let envelope = fixture_envelope();
    let decision =
        verify_admission(&envelope, Some(b"tampered bytes"), &trusted_context()).unwrap();
    match decision {
        AdmissionDecision::Quarantined(reason) => assert!(reason.contains("digest")),
        other => panic!("expected Quarantined, got {other:?}"),
    }
}

#[test]
fn admission_with_untrusted_key_is_signature_invalid() {
    let envelope = fixture_envelope();
    let context = EnvelopeVerificationContext::new(
        fixture_timestamp() - chrono::Duration::seconds(1),
        fixture_timestamp() + chrono::Duration::seconds(1),
    )
    // Different disposable key registered under the same signer id.
    .with_signer_key("local-operator", [9u8; 32])
    .authorize_signer("local-operator")
    .admit_policy(POLICY);
    let decision = verify_admission(&envelope, Some(&fixture_bytes()), &context).unwrap();
    match decision {
        AdmissionDecision::Quarantined(reason) => assert!(reason.contains("signature")),
        other => panic!("expected Quarantined, got {other:?}"),
    }
}

#[test]
fn admission_with_unauthorized_signer_is_quarantined() {
    let envelope = fixture_envelope();
    let context = EnvelopeVerificationContext::new(
        fixture_timestamp() - chrono::Duration::seconds(1),
        fixture_timestamp() + chrono::Duration::seconds(1),
    )
    .with_signer_key("local-operator", hex_to_bytes32(FIXTURE_PUBLIC_HEX))
    // Key registered but signer NOT authorized.
    .admit_policy(POLICY);
    let decision = verify_admission(&envelope, Some(&fixture_bytes()), &context).unwrap();
    match decision {
        AdmissionDecision::Quarantined(reason) => assert!(reason.contains("unauthorized")),
        other => panic!("expected Quarantined, got {other:?}"),
    }
}

#[test]
fn admission_outside_time_window_is_quarantined() {
    let envelope = fixture_envelope();
    let context = EnvelopeVerificationContext::new(
        fixture_timestamp() - chrono::Duration::hours(24),
        fixture_timestamp() - chrono::Duration::hours(23),
    )
    .with_signer_key("local-operator", hex_to_bytes32(FIXTURE_PUBLIC_HEX))
    .authorize_signer("local-operator")
    .admit_policy(POLICY);
    let decision = verify_admission(&envelope, Some(&fixture_bytes()), &context).unwrap();
    match decision {
        AdmissionDecision::Quarantined(reason) => assert!(reason.contains("time")),
        other => panic!("expected Quarantined, got {other:?}"),
    }
}

#[test]
fn admission_with_unadmitted_policy_is_quarantined() {
    let envelope = fixture_envelope();
    let context = EnvelopeVerificationContext::new(
        fixture_timestamp() - chrono::Duration::seconds(1),
        fixture_timestamp() + chrono::Duration::seconds(1),
    )
    .with_signer_key("local-operator", hex_to_bytes32(FIXTURE_PUBLIC_HEX))
    .authorize_signer("local-operator");
    // Policy never admitted by the verifier context.
    let decision = verify_admission(&envelope, Some(&fixture_bytes()), &context).unwrap();
    match decision {
        AdmissionDecision::Quarantined(reason) => assert!(reason.contains("policy")),
        other => panic!("expected Quarantined, got {other:?}"),
    }
}

// --- 5. duplicate native admission IDs are rejected -----------------------------

#[test]
fn fold_rejects_duplicate_native_admission_ids() {
    let payload = fixture_admission_payload();
    let first = LedgerEntryBuilder::new(1, None)
        .add_native_admission(&payload)
        .unwrap();
    // Distinct-payload replay reusing the SAME admission id.
    let mut replayed = payload.clone();
    replayed.signer_id = "other-signer".to_string();
    replayed.envelope_digest = "other-digest".to_string();
    let second = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
        .add_native_admission(&replayed)
        .unwrap();
    let mut fold = SnapshotFoldV1::default();
    fold.apply_admission_entry(&first).unwrap();
    let error = fold.apply_admission_entry(&second).unwrap_err();
    assert!(matches!(
        error,
        ClaimLedgerError::DuplicateNativeAdmissionId(_)
    ));
    assert_eq!(error.kind(), "duplicate_native_admission_id");
}

// --- 6. fold: admission events do not project; duplicates detected ---------------

fn chain_with_judgment() -> Vec<LedgerEntry> {
    let entry1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_a", "semantic-memory:fact:f1", "sp_1", "claim one")
        .unwrap();
    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
        .add_claim("clm_b", "src", "sp_2", "claim two")
        .unwrap();
    let entry3 = LedgerEntryBuilder::new(3, Some(entry2.entry_digest.clone()))
        .add_support_judgment(
            "sup_1",
            "clm_b",
            "evb_1",
            claim_ledger::SupportState::Supported,
            "manual",
        )
        .unwrap();
    vec![entry1, entry2, entry3]
}

#[test]
fn fold_snapshot_projects_claims_and_judgments() {
    let fixed = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    let snapshot = fold_snapshot(&chain_with_judgment(), fixed).unwrap();
    claim_ledger::verify_snapshot(&snapshot).unwrap();
    assert_eq!(snapshot.claims.len(), 2);
    assert_eq!(snapshot.support_judgments.len(), 1);
    assert_eq!(snapshot.claim_support.len(), 1);
    assert_eq!(
        snapshot.claim_support[0].support_state,
        claim_ledger::SupportState::Supported
    );
    assert_eq!(snapshot.fact_to_claim_links.len(), 1);
}

#[test]
fn fold_snapshot_digest_is_unchanged_by_admission_events() {
    // The same claim stream with and WITHOUT an interleaved AdmissionEvent:
    // projected content is bit-identical (admission events do not project),
    // while the checkpoint anchor advances through the admission entry.
    let fixed = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    let mut with_admission = chain_with_judgment();
    let previous_digest = with_admission[2].entry_digest.clone();
    with_admission.push(
        LedgerEntryBuilder::new(4, Some(previous_digest))
            .add_native_admission(&fixture_admission_payload())
            .unwrap(),
    );
    let without = fold_snapshot(&chain_with_judgment(), fixed).unwrap();
    let with = fold_snapshot(&with_admission, fixed).unwrap();
    assert_eq!(without.claims, with.claims);
    assert_eq!(without.fact_to_claim_links, with.fact_to_claim_links);
    assert_eq!(without.support_judgments, with.support_judgments);
    assert_eq!(without.claim_support, with.claim_support);
    assert_eq!(without.contradiction_states, with.contradiction_states);
    assert_eq!(without.last_compacted_sequence, 3);
    assert_eq!(with.last_compacted_sequence, 4);
    assert_ne!(
        with.last_compacted_entry_digest, without.last_compacted_entry_digest,
        "anchor must advance through the admission event"
    );
    claim_ledger::verify_snapshot(&with).unwrap();
    claim_ledger::verify_snapshot(&without).unwrap();
}

#[test]
fn compacting_past_an_admission_event_respects_policy() {
    let mut entries = chain_with_judgment();
    let previous_digest = entries[2].entry_digest.clone();
    entries.push(
        LedgerEntryBuilder::new(4, Some(previous_digest))
            .add_native_admission(&fixture_admission_payload())
            .unwrap(),
    );
    // FailClosed refuses to compact across the unprojectable admission event.
    let deny = CompactionPolicy {
        retain_tail_entries: 0,
        unprojectable_events: UnprojectableEventPolicy::FailClosed,
    };
    assert!(compact_ledger(&entries, &deny).is_err());
    // Retain compacts only up to the last projectable event.
    let retain = CompactionPolicy {
        retain_tail_entries: 0,
        unprojectable_events: UnprojectableEventPolicy::Retain,
    };
    let compacted = compact_ledger(&chain_with_judgment(), &retain).unwrap();
    claim_ledger::verify_compaction(
        &compacted.snapshot,
        &compacted.retained_tail,
        &compacted.receipt,
    )
    .unwrap();
    assert_eq!(compacted.snapshot.last_compacted_sequence, 3);
}

// --- 7. canonical fold across permutations ---------------------------------------

#[test]
fn fold_is_canonical_across_event_orders() {
    let fixed = Utc.timestamp_opt(1_800_000_000, 0).unwrap();
    // Chain A: claim one, then claim two, then judgment.
    let chain_a = chain_with_judgment();
    let mut fold_a = SnapshotFoldV1::default();
    fold_a.apply_chain(&chain_a).unwrap();
    let snap_a = fold_a.into_snapshot(fixed).unwrap();

    // Chain B: same projected event SET in a different stream order:
    // claim two first, then claim one, then the same judgment.
    let b1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_b", "src", "sp_2", "claim two")
        .unwrap();
    let b2 = LedgerEntryBuilder::new(2, Some(b1.entry_digest.clone()))
        .add_claim("clm_a", "semantic-memory:fact:f1", "sp_1", "claim one")
        .unwrap();
    let b3 = LedgerEntryBuilder::new(3, Some(b2.entry_digest.clone()))
        .add_support_judgment(
            "sup_1",
            "clm_b",
            "evb_1",
            claim_ledger::SupportState::Supported,
            "manual",
        )
        .unwrap();
    let mut fold_b = SnapshotFoldV1::default();
    fold_b.apply_chain(&[b1, b2, b3]).unwrap();
    let snap_b = fold_b.into_snapshot(fixed).unwrap();

    // The canonical invariant: identical projected vectors (sorted by stable
    // identity), independent of stream position. Anchors legitimately differ.
    assert_eq!(snap_a.claims, snap_b.claims);
    assert_eq!(snap_a.fact_to_claim_links, snap_b.fact_to_claim_links);
    assert_eq!(snap_a.support_judgments, snap_b.support_judgments);
    assert_eq!(snap_a.claim_support, snap_b.claim_support);
    assert_eq!(snap_a.contradiction_states, snap_b.contradiction_states);

    // Fold-vs-compaction cross-check on the identical stream: field-for-field
    // equality including anchors, then the whole-snapshot digest.
    let compacted = compact_ledger(
        &chain_a,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::Retain,
        },
    )
    .unwrap();
    let fold_c = SnapshotFoldV1::default();
    let mut fold_c = fold_c;
    fold_c.apply_chain(&chain_a).unwrap();
    let snap_c = fold_c.into_snapshot(fixed).unwrap();
    assert_eq!(snap_c.claims, compacted.snapshot.claims);
    assert_eq!(
        snap_c.fact_to_claim_links,
        compacted.snapshot.fact_to_claim_links
    );
    assert_eq!(
        snap_c.support_judgments,
        compacted.snapshot.support_judgments
    );
    assert_eq!(snap_c.claim_support, compacted.snapshot.claim_support);
    assert_eq!(
        snap_c.last_compacted_sequence,
        compacted.snapshot.last_compacted_sequence
    );
    assert_eq!(
        snap_c.last_compacted_entry_digest,
        compacted.snapshot.last_compacted_entry_digest
    );
    // Digest equality across procedures is NOT asserted: the digest covers
    // created_at (provenance), which compact_ledger sets to Utc::now() while
    // the fold uses the fixed timestamp. Content + anchor equality above is
    // the cross-procedure invariant.

    // Determinism: identical folds and identical created_at give equal digests.
    let mut fold_a2 = SnapshotFoldV1::default();
    fold_a2.apply_chain(&chain_a).unwrap();
    assert_eq!(
        snap_a.snapshot_digest,
        fold_a2.into_snapshot(fixed).unwrap().snapshot_digest
    );
}

// --- 8. duplicate seen-id detection via apply_chain ------------------------------

#[test]
fn fold_chain_stops_at_first_duplicate_admission() {
    let payload = fixture_admission_payload();
    let first = LedgerEntryBuilder::new(1, None)
        .add_native_admission(&payload)
        .unwrap();
    let second = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
        .add_native_admission(&payload)
        .unwrap();
    let mut fold = SnapshotFoldV1::default();
    let error = fold.apply_chain(&[first, second]).unwrap_err();
    assert!(matches!(
        error,
        ClaimLedgerError::DuplicateNativeAdmissionId(_)
    ));
    // Exactly the first admission was recorded before the failure.
    assert_eq!(fold.seen_admission_ids().len(), 1);
}
