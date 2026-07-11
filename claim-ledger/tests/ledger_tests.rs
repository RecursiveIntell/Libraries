//! Integration tests for claim-ledger.

use claim_ledger::{
    parse_ledger_entries, serialize_entry, verify_ledger, Claim, EvidenceBundle,
    ExpectedLedgerHead, LedgerEntry, LedgerEntryBuilder, LedgerEvent, SupportState,
};

fn head(entry: &LedgerEntry) -> ExpectedLedgerHead {
    ExpectedLedgerHead::new(entry.sequence, entry.entry_digest.clone())
}

#[test]
fn claim_creation_roundtrip() {
    let claim = Claim::new("source1", "span1", "The sky is blue", "fact");
    let json = serde_json::to_string(&claim).unwrap();
    let parsed: Claim = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.claim_id, claim.claim_id);
}

#[test]
fn evidence_bundle_links_to_claim() {
    let claim = Claim::new("source1", "span1", "Temperature is 72F", "measurement");
    let bundle = EvidenceBundle::new(&claim.claim_id);
    assert_eq!(bundle.claim_id, claim.claim_id);
    assert!(bundle.evidence_bundle_id.starts_with("evb_"));
}

#[test]
fn ledger_entry_contains_claim_event() {
    let entry = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_test123", "src_doc", "sp_001", "the sky is blue")
        .unwrap();
    assert_eq!(entry.sequence, 1);
    assert!(entry.previous_entry_digest.is_none());
    assert!(!entry.entry_digest.is_empty());
    assert!(matches!(entry.event, LedgerEvent::ClaimAdded { .. }));
}

#[test]
fn supersession_receipt_chains_and_binds_head() {
    let entry1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_v1", "src1", "span1", "version one")
        .unwrap();
    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
        .add_claim("clm_v2", "src1", "span1", "version two")
        .unwrap();
    let entry3 = LedgerEntryBuilder::new(3, Some(entry2.entry_digest.clone()))
        .add_claim("clm_v3", "src1", "span1", "version three")
        .unwrap();
    let verification = verify_ledger(&[entry1, entry2, entry3.clone()], &head(&entry3)).unwrap();
    assert_eq!(verification.last_sequence, 3);
}

#[test]
fn ledger_verification_detects_tampering() {
    let entry1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_abc", "src1", "sp1", "original claim")
        .unwrap();
    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
        .add_claim("clm_def", "src1", "sp2", "second claim")
        .unwrap();
    let expected_head = head(&entry2);
    let tampered = LedgerEntry {
        sequence: 2,
        previous_entry_digest: Some("invalid".into()),
        event: entry2.event,
        entry_digest: "wrong".into(),
    };
    assert!(verify_ledger(&[entry1, tampered], &expected_head).is_err());
}

#[test]
fn ledger_jsonl_parsing_is_strict() {
    let first = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_1", "s1", "sp1", "test")
        .unwrap();
    let jsonl = serialize_entry(&first).unwrap();
    let entries = parse_ledger_entries(&jsonl).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(parse_ledger_entries("not json")
        .unwrap_err()
        .to_string()
        .contains("line 1"));
}

#[test]
fn support_judgment_event_in_ledger() {
    let entry = LedgerEntryBuilder::new(1, None)
        .add_support_judgment(
            "sj_001",
            "clm_abc",
            "evb_123",
            SupportState::Supported,
            "operator_judgment",
        )
        .unwrap();
    assert!(matches!(entry.event, LedgerEvent::SupportJudgment { .. }));
}
