//! MCP-server query projection with trusted-head verification (FINISH_PLAN
//! Phase 3.3).
//!
//! Contract (typed, fail-closed):
//! - `claim-ledger-mcp` gains `--trust-root <path>` + `--expected-head <path>`
//!   flags. With neither, the server stays exactly as merged (#43/#45):
//!   permanently `unanchored`, support projected Unknown.
//! - Trust root without a head file → START FAILS (typed
//!   `TrustedHeadError::MissingExpectedHead`), never a silent fallback.
//! - Head JSON: `ClaimLedgerMcpTrustedHeadV1` (strict, deny unknown fields):
//!   expected_sequence, ledger_entry_digest, anchor_admission_id,
//!   anchor_envelope_digest.
//! - Verification at query time: `verify_ledger` against the independent
//!   head; the anchor admission event must exist in the chain, record stage
//!   `fully_verified`, match `anchor_envelope_digest`, and its signer must
//!   equal the provisioned root's signer id. Projection folds the canonical
//!   `SnapshotFoldV1`; legacy SupportAdmission events never project (#43/#45
//!   fence); forged self-consistent ledgers are rejected by the head check
//!   (not merely masked).
//! - Unanchored mode (no flags) keeps the #43/#45 contract: no support
//!   projection, raw events only.

#![allow(clippy::unwrap_used, clippy::expect_used)]

use chrono::{TimeZone, Utc};
use claim_ledger::{
    admission_event_from_envelope, ArtifactEnvelopeV1, EnvelopeVerificationContext, LedgerEntry,
    LedgerEntryBuilder, LedgerEvent, PolicyAdmission, SupportState,
};
use claim_ledger_mcp::trusted_head::{
    load_projected_entries, parse_expected_head, TrustedHeadError,
};

const FIXTURE_SEED_HEX: &str = "9d61b19deffd2a60532a5c24476606bfca4f3b4a4f4fdc4a2d1d0e4f3e5b6c7a";
const FIXTURE_PUBLIC_HEX: &str = "dc152a48434a6d1342134b252c427b9fe636100e7908d88a91af73a428fb43c7";
const POLICY: &str = "recursiveintell:operator-admitted-support:v1";
const ARTIFACT: &[u8] = b"{\"schema\":\"ClaimLedgerBundleV2\",\"claims\":[],\"fixture\":true}";

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
        ARTIFACT,
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

/// The canonical operator admission event for the fixture bundle.
fn admission_event_entry(sequence: u64, previous: Option<String>) -> LedgerEntry {
    let payload =
        admission_event_from_envelope(&fixture_envelope(), ARTIFACT, &trusted_context()).unwrap();
    LedgerEntryBuilder::new(sequence, previous)
        .add_native_admission(&payload)
        .unwrap()
}

fn admission_payload_of(entry: &LedgerEntry) -> claim_ledger::AdmissionEventPayloadV1 {
    match &entry.event {
        LedgerEvent::AdmissionEvent { payload } => payload.clone(),
        other => panic!("expected admission event, got {}", other.type_name()),
    }
}

/// Head JSON pinned to the fixture chain + anchor admission.
fn head_json(entries: &[LedgerEntry]) -> String {
    let last = entries.last().unwrap();
    let anchor = admission_payload_of(
        entries
            .iter()
            .find(|e| matches!(e.event, LedgerEvent::AdmissionEvent { .. }))
            .unwrap(),
    );
    format!(
        "{{\"schema\":\"ClaimLedgerMcpTrustedHeadV1\",\"expected_sequence\":{},\"ledger_entry_digest\":\"{}\",\"anchor_admission_id\":\"{}\",\"anchor_envelope_digest\":\"{}\"}}",
        last.sequence, last.entry_digest, anchor.admission_id, anchor.envelope_digest
    )
}

fn trust_root_text() -> String {
    format!(
        "{{\"schema\":\"ClaimLedgerTrustRootV1\",\"signer_id\":\"local-operator\",\"ed25519_public_key_hex\":\"{FIXTURE_PUBLIC_HEX}\",\"authorized_signers\":[\"local-operator\"],\"admitted_policies\":[\"{POLICY}\"],\"not_before\":\"2026-09-30T03:59:59Z\",\"not_after\":\"2026-09-30T04:00:01Z\"}}"
    )
}

fn chain_json(entries: &[LedgerEntry]) -> String {
    entries
        .iter()
        .map(|e| claim_ledger::serialize_entry(e).unwrap())
        .collect::<Vec<_>>()
        .join("\n")
}

/// Chain: claim added → support judgment (bundle) → admission event.
fn anchored_chain() -> Vec<LedgerEntry> {
    let claim = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_a", "semantic-memory:fact:f1", "sp_1", "claim one")
        .unwrap();
    let judgment = LedgerEntryBuilder::new(2, Some(claim.entry_digest.clone()))
        .add_support_judgment(
            "judgment-1",
            "clm_a",
            "bundle-1",
            SupportState::Supported,
            "fixture",
        )
        .unwrap();
    let admission = admission_event_entry(3, Some(judgment.entry_digest.clone()));
    vec![claim, judgment, admission]
}

// --- 1. head-file parsing ------------------------------------------------------

#[test]
fn parses_expected_head_file() {
    let chain = anchored_chain();
    let head = parse_expected_head(&head_json(&chain)).unwrap();
    assert_eq!(head.expected_sequence, 3);
    assert_eq!(
        head.anchor_admission_id,
        admission_payload_of(&chain[2]).admission_id
    );
    assert_eq!(
        head.anchor_envelope_digest,
        admission_payload_of(&chain[2]).envelope_digest
    );
}

#[test]
fn rejects_malformed_or_untyped_head_json() {
    let chain = anchored_chain();
    let good = head_json(&chain);
    let untyped = good.replace("ClaimLedgerMcpTrustedHeadV1", "SomeOtherSchemaV9");
    assert!(matches!(
        parse_expected_head(&untyped),
        Err(TrustedHeadError::MalformedHead(_))
    ));
    let truncated = "{\"schema\":\"ClaimLedgerMcpTrustedHeadV1\"}";
    assert!(matches!(
        parse_expected_head(truncated),
        Err(TrustedHeadError::MalformedHead(_))
    ));
    let extra = good.trim_end_matches('}');
    let extra = format!(r#"{extra},"surprise":1}}"#);
    assert!(matches!(
        parse_expected_head(&extra),
        Err(TrustedHeadError::MalformedHead(_))
    ));
}

// --- 2. start-time fail-closed ---------------------------------------------------

#[test]
fn trust_root_without_head_file_fails_typed() {
    let chain = anchored_chain();
    let json = chain_json(&chain);
    let projection = load_projected_entries(&json, Some(&trust_root_text()), None);
    assert!(matches!(
        projection,
        Err(TrustedHeadError::MissingExpectedHead(_))
    ));
}

// --- 3. anchored positive path -----------------------------------------------------

#[test]
fn anchored_projection_projects_verified_support() {
    let chain = anchored_chain();
    let projection = load_projected_entries(
        &chain_json(&chain),
        Some(&trust_root_text()),
        Some(&head_json(&chain)),
    )
    .unwrap();
    assert!(projection.anchored);
    assert_eq!(
        projection.anchor.as_ref().unwrap().admission_id,
        admission_payload_of(&chain[2]).admission_id
    );
    assert_eq!(
        projection.anchor.as_ref().unwrap().signer_id,
        "local-operator"
    );
    assert_eq!(
        projection
            .anchor
            .as_ref()
            .unwrap()
            .envelope_verification_name,
        "fully_verified"
    );
    let state = projection
        .claim_support
        .get("clm_a")
        .copied()
        .expect("judged claim projected");
    assert_eq!(state, SupportState::Supported);
}

// --- 4. fail-closed negatives --------------------------------------------------------

#[test]
fn forged_self_consistent_ledger_is_rejected_by_trusted_head() {
    let claim = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_a", "semantic-memory:fact:f1", "sp_1", "claim one")
        .unwrap();
    let forged = LedgerEntryBuilder::new(2, Some(claim.entry_digest.clone()))
        .add_support_judgment(
            "judgment-1",
            "clm_a",
            "bundle-1",
            SupportState::Supported,
            "fixture",
        )
        .unwrap();
    // Head pins a DIFFERENT (honest) chain the attacker cannot match.
    let honest = anchored_chain();
    let error = load_projected_entries(
        &chain_json(&[claim, forged]),
        Some(&trust_root_text()),
        Some(&head_json(&honest)),
    )
    .unwrap_err();
    assert!(matches!(error, TrustedHeadError::LedgerInvalid(_)));
}

#[test]
fn anchor_envelope_digest_mismatch_fails_closed() {
    let chain = anchored_chain();
    let tampered_head = head_json(&chain).replace(
        &admission_payload_of(&chain[2]).envelope_digest,
        "0000000000000000000000000000000000000000000000000000000000000000",
    );
    let error = load_projected_entries(
        &chain_json(&chain),
        Some(&trust_root_text()),
        Some(&tampered_head),
    )
    .unwrap_err();
    assert!(matches!(error, TrustedHeadError::AnchorUnmatched(_)));
}

#[test]
fn anchor_signer_mismatch_fails_closed() {
    let chain = anchored_chain();
    // Root registers a DIFFERENT signer id; the anchor must not verify.
    let other_root = trust_root_text().replace("local-operator", "someone-else");
    let error = load_projected_entries(
        &chain_json(&chain),
        Some(&other_root),
        Some(&head_json(&chain)),
    )
    .unwrap_err();
    assert!(matches!(error, TrustedHeadError::SignerMismatch(_)));
}

#[test]
fn unresolvable_anchor_fails_closed() {
    let chain = anchored_chain();
    let head = head_json(&chain).replace(
        &admission_payload_of(&chain[2]).admission_id,
        "nadm_absent_admission_000000000000000000",
    );
    let error = load_projected_entries(&chain_json(&chain), Some(&trust_root_text()), Some(&head))
        .unwrap_err();
    assert!(matches!(error, TrustedHeadError::AnchorUnmatched(_)));
}

// --- 5. legacy-admission fence persists in anchored mode -----------------------------

#[test]
fn anchored_projection_ignores_legacy_support_admission() {
    let claim = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_b", "semantic-memory:fact:f2", "sp_2", "claim two")
        .unwrap();
    let legacy = LedgerEntryBuilder::new(2, Some(claim.entry_digest.clone()))
        .add_support_admission("receipt-1", "clm_b", "old", "new", SupportState::Supported)
        .unwrap();
    let admission = admission_event_entry(3, Some(legacy.entry_digest.clone()));
    let chain = vec![claim, legacy, admission];
    let projection = load_projected_entries(
        &chain_json(&chain),
        Some(&trust_root_text()),
        Some(&head_json(&chain)),
    )
    .unwrap();
    // The legacy admission must NOT surface as support even though the chain
    // carries a genuine admission event for the bundle.
    assert!(
        !projection.claim_support.contains_key("clm_b"),
        "legacy admission must stay unprojected: {:?}",
        projection.claim_support
    );
}

// --- 6. unanchored contract unchanged --------------------------------------------------

#[test]
fn no_flags_keeps_permanently_unanchored_projection() {
    let chain = anchored_chain();
    let projection = load_projected_entries(&chain_json(&chain), None, None).unwrap();
    assert!(!projection.anchored);
    assert!(
        projection.claim_support.is_empty(),
        "unanchored mode projects nothing"
    );
}
