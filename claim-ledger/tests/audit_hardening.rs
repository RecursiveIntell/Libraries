use chrono::{TimeZone, Utc};
use claim_ledger::{
    compute_entry_digest, evaluate_proof_debt_gate_with_waiver, parse_ledger_entries,
    serialize_entry, stable_id, verify_ledger, ExpectedLedgerHead, ExportReceipt,
    LedgerAppendReceipt, LedgerEntryBuilder, ProofDebtBudgetV1, ProofDebtGateDecision,
    ProofDebtSummaryV1, ProofDebtWaiverReceipt,
};

fn entry(sequence: u64, previous: Option<String>) -> claim_ledger::LedgerEntry {
    LedgerEntryBuilder::new(sequence, previous)
        .add_claim("clm_1", "source", "span", "hello")
        .expect("entry construction should succeed")
}

#[test]
fn stable_ids_bind_version_prefix_and_part_boundaries() {
    assert_ne!(
        stable_id("x", &["a\nb"], 20),
        stable_id("x", &["a", "b"], 20)
    );
    assert_ne!(stable_id("x", &[""], 20), stable_id("x", &[], 20));
    assert_ne!(
        stable_id("x", &["", "a"], 20),
        stable_id("x", &["a", ""], 20)
    );
    assert_ne!(
        stable_id("left", &["same"], 20),
        stable_id("right", &["same"], 20)
    );
    assert_eq!(
        stable_id("uni", &["東京🙂"], 20),
        stable_id("uni", &["東京🙂"], 20)
    );
}

#[test]
fn canonical_digest_has_a_cross_language_golden_vector() {
    let entry = entry(1, None);
    assert_eq!(
        entry.entry_digest, "4169125ca0feadd0730051ab7cce353ee2c15ac1cf02407c2c88265134c98a75",
        "SHA-256 of the documented canonical binary preimage"
    );
    assert_eq!(
        compute_entry_digest(entry.sequence, None, &entry.event).unwrap(),
        entry.entry_digest
    );
}

#[test]
fn strict_jsonl_rejects_malformed_nonblank_lines_with_line_number() {
    let first = entry(1, None);
    let second = entry(2, Some(first.entry_digest.clone()));
    let first_json = serialize_entry(&first).unwrap();
    let second_json = serialize_entry(&second).unwrap();
    let middle_error =
        parse_ledger_entries(&format!("{first_json}\nnot-json\n{second_json}\n")).unwrap_err();
    assert!(middle_error.to_string().contains("line 2"));
    let tail_error =
        parse_ledger_entries(&format!("{first_json}\n{second_json}\nnot-json\n")).unwrap_err();
    assert!(tail_error.to_string().contains("line 3"));
    assert!(parse_ledger_entries("\n \t\n").unwrap().is_empty());
}

#[test]
fn verification_requires_the_expected_head_and_rejects_a_valid_prefix() {
    let first = entry(1, None);
    let second = entry(2, Some(first.entry_digest.clone()));
    let head = ExpectedLedgerHead::new(second.sequence, second.entry_digest.clone());

    verify_ledger(&[first.clone(), second.clone()], &head).unwrap();
    assert!(verify_ledger(std::slice::from_ref(&first), &head).is_err());
    assert!(verify_ledger(
        &[first.clone(), second.clone()],
        &ExpectedLedgerHead::new(second.sequence, "wrong"),
    )
    .is_err());
    assert!(verify_ledger(
        &[first, second.clone()],
        &ExpectedLedgerHead::new(second.sequence + 1, second.entry_digest.clone()),
    )
    .is_err());
    assert!(verify_ledger(&[], &ExpectedLedgerHead::empty()).is_ok());
    assert!(verify_ledger(&[], &head).is_err());
}

#[test]
fn explicit_time_constructors_make_receipts_byte_identical() {
    let time = Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap();
    let first = ExportReceipt::new_at("export", vec!["in".into()], "attempt".into(), time);
    let second = ExportReceipt::new_at("export", vec!["in".into()], "attempt".into(), time);
    assert_eq!(
        serde_json::to_vec(&first).unwrap(),
        serde_json::to_vec(&second).unwrap()
    );

    let append_one = LedgerAppendReceipt::new_at("ledger", 1, None, "digest".into(), time);
    let append_two = LedgerAppendReceipt::new_at("ledger", 1, None, "digest".into(), time);
    assert_eq!(
        serde_json::to_vec(&append_one).unwrap(),
        serde_json::to_vec(&append_two).unwrap()
    );
}

#[test]
fn waiver_authorizes_bounded_proceeding_without_erasing_debt() {
    let time = Utc.with_ymd_and_hms(2026, 7, 10, 12, 0, 0).unwrap();
    let mut budget = ProofDebtBudgetV1::new_at("claim:waived", 500_000, time);
    budget.consume(500_000, "claim", "unproven", false);
    let waiver = ProofDebtWaiverReceipt::new_at(
        &budget,
        500_000,
        "operator:test",
        "advisory-only scope",
        time,
    );

    assert_eq!(budget.consumed_micros, 500_000);
    let gate = evaluate_proof_debt_gate_with_waiver(&budget, Some(&waiver));
    assert_eq!(gate.decision, ProofDebtGateDecision::Waived);
    let summary = ProofDebtSummaryV1::from_budget_with_waiver(&budget, Some(&waiver));
    assert_eq!(summary.consumed_micros, 500_000);
    assert_eq!(
        summary.waiver_id.as_deref(),
        Some(waiver.waiver_id.as_str())
    );
}
