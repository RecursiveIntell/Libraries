//! Integration tests for claim-ledger.

use claim_ledger::{
    verify_ledger, Claim, EvidenceBundle, LedgerEntryBuilder, LedgerEvent, SupportState,
};

/// Test 1: claim_creation_roundtrip — create Claim, serialize, deserialize.
#[test]
fn claim_creation_roundtrip() {
    let claim = Claim::new("source1", "span1", "The sky is blue", "fact");

    // Verify claim was created with expected fields
    assert!(!claim.claim_id.is_empty());
    assert_eq!(claim.source_id, "source1");
    assert_eq!(claim.span_id, "span1");
    assert_eq!(claim.claim_text, "The sky is blue");
    assert_eq!(claim.claim_type, "fact");
    assert_eq!(claim.status, "pending");

    // Serialize to JSON
    let json = serde_json::to_string(&claim).unwrap();
    assert!(json.contains("claim_id"));
    assert!(json.contains("The sky is blue"));

    // Deserialize back
    let parsed: Claim = serde_json::from_str(&json).unwrap();
    assert_eq!(parsed.claim_id, claim.claim_id);
    assert_eq!(parsed.claim_text, claim.claim_text);
}

/// Test 2: evidence_bundle_links_to_claim — bind EvidenceBundle to Claim.
#[test]
fn evidence_bundle_links_to_claim() {
    let claim = Claim::new("source1", "span1", "Temperature is 72F", "measurement");
    let bundle = EvidenceBundle::new(&claim.claim_id);

    // Evidence bundle should reference the claim
    assert_eq!(bundle.claim_id, claim.claim_id);
    assert!(bundle.evidence_bundle_id.starts_with("evb_"));
    assert!(bundle.evidence_links.is_empty());
}

/// Test 3: ledger_entry_contains_claim_event — verify ledger entry for claim.
#[test]
fn ledger_entry_contains_claim_event() {
    let entry = LedgerEntryBuilder::new(1, None).add_claim(
        "clm_test123",
        "src_doc",
        "sp_001",
        "the sky is blue",
    );

    assert_eq!(entry.sequence, 1);
    assert!(entry.previous_entry_digest.is_none());
    assert!(!entry.entry_digest.is_empty());

    if let LedgerEvent::ClaimAdded {
        claim_id,
        source_id,
        span_id,
        normalized_claim,
    } = entry.event
    {
        assert_eq!(claim_id, "clm_test123");
        assert_eq!(source_id, "src_doc");
        assert_eq!(span_id, "sp_001");
        assert_eq!(normalized_claim, "the sky is blue");
    } else {
        panic!("Expected ClaimAdded event");
    }
}

/// Test 4: supersession_receipt_chains — append 3 versions, verify hash chain.
#[test]
fn supersession_receipt_chains() {
    // Build a chain of 3 ledger entries
    let entry1 =
        LedgerEntryBuilder::new(1, None).add_claim("clm_v1", "src1", "span1", "version one");

    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone())).add_claim(
        "clm_v2",
        "src1",
        "span1",
        "version two",
    );

    let entry3 = LedgerEntryBuilder::new(3, Some(entry2.entry_digest.clone())).add_claim(
        "clm_v3",
        "src1",
        "span1",
        "version three",
    );

    // Verify chain links
    assert_eq!(
        entry2.previous_entry_digest,
        Some(entry1.entry_digest.clone())
    );
    assert_eq!(
        entry3.previous_entry_digest,
        Some(entry2.entry_digest.clone())
    );

    // Verify the full ledger
    let verification = verify_ledger(&[entry1, entry2, entry3]);
    assert!(verification.valid);
    assert_eq!(verification.last_sequence, 3);
    assert!(verification.errors.is_empty());
}

/// Test 5: ledger_verification_detects_tampering — modify entry, verification fails.
#[test]
fn ledger_verification_detects_tampering() {
    let entry1 =
        LedgerEntryBuilder::new(1, None).add_claim("clm_abc", "src1", "sp1", "original claim");
    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone())).add_claim(
        "clm_def",
        "src1",
        "sp2",
        "second claim",
    );

    // Valid ledger should pass
    let verification = verify_ledger(&[entry1.clone(), entry2.clone()]);
    assert!(verification.valid);

    // Tamper with entry2's previous digest — chain is broken
    let tampered_entry2 = claim_ledger::LedgerEntry {
        sequence: 2,
        previous_entry_digest: Some("invalid_digest".to_string()),
        event: entry2.event,
        entry_digest: "wrong_digest".to_string(),
    };

    let tampered_verification = verify_ledger(&[entry1, tampered_entry2]);
    assert!(!tampered_verification.valid);
    assert!(!tampered_verification.errors.is_empty());
}

/// Test: ledger parses from JSONL format.
#[test]
fn ledger_jsonl_parsing() {
    let jsonl = r#"{"sequence":1,"previous_entry_digest":null,"event":{"type":"claim_added","claim_id":"clm_1","source_id":"s1","span_id":"sp1","normalized_claim":"test"},"entry_digest":"abc123"}
{"sequence":2,"previous_entry_digest":"abc123","event":{"type":"claim_added","claim_id":"clm_2","source_id":"s1","span_id":"sp2","normalized_claim":"test2"},"entry_digest":"def456"}"#;

    let entries = claim_ledger::parse_ledger_entries(jsonl);
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[0].sequence, 1);
    assert_eq!(entries[1].sequence, 2);
}

/// Test: support judgment event in ledger.
#[test]
fn support_judgment_event_in_ledger() {
    let entry = LedgerEntryBuilder::new(1, None).add_support_judgment(
        "sj_001",
        "clm_abc",
        "evb_123",
        SupportState::Supported,
        "operator_judgment",
    );

    assert_eq!(entry.sequence, 1);
    if let LedgerEvent::SupportJudgment {
        support_judgment_id,
        claim_id,
        evidence_bundle_ref,
        support_state,
        method,
    } = entry.event
    {
        assert_eq!(support_judgment_id, "sj_001");
        assert_eq!(claim_id, "clm_abc");
        assert_eq!(evidence_bundle_ref, "evb_123");
        assert_eq!(support_state, SupportState::Supported);
        assert_eq!(method, "operator_judgment");
    } else {
        panic!("Expected SupportJudgment event");
    }
}
