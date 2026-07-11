//! Integration tests for proof-debt budget accounting (FEUT-004).
//!
//! These tests verify the end-to-end proof-debt lifecycle:
//! 1. Create a budget for a claim
//! 2. Consume debt when a claim lacks proof
//! 3. Gate fires when budget is exhausted
//! 4. Replenish when evidence is added
//! 5. Gate allows proceed after replenishment
//! 6. Operator waiver authorizes bounded proceeding without reducing debt
//! 7. Ledger records all proof-debt events

use claim_ledger::{
    budget_for_claim, evaluate_proof_debt_gate, evaluate_proof_debt_gate_with_config,
    evaluate_proof_debt_gate_with_waiver, proof_debt_weight, total_proof_debt_weight,
    verify_proof_debt_waiver, ExpectedLedgerHead, LedgerEntryBuilder, ProofDebt,
    ProofDebtBudgetConfig, ProofDebtBudgetV1, ProofDebtGateDecision, ProofDebtSummaryV1,
    ProofDebtWaiverReceipt,
};

#[test]
fn full_proof_debt_lifecycle() {
    // 1. Create budget for a claim with missing source basis
    let mut budget = ProofDebtBudgetV1::new("claim:clm_test_001", 500_000);
    assert!(!budget.is_exhausted());
    assert_eq!(budget.available_micros(), 500_000);

    // 2. Consume debt: claim has missing source basis (heaviest debt)
    let debit = budget
        .consume_debt(
            &ProofDebt::MissingSourceBasis,
            "claim:clm_test_001",
            "claim extracted but no source artifact found",
            false,
        )
        .expect("consume should succeed in non-strict mode");

    assert_eq!(debit.amount_micros, 250_000);
    assert_eq!(budget.consumed_micros, 250_000);
    assert_eq!(budget.consumed_pct(), 50);

    // 3. Gate should allow proceeding at 50%
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Proceed);

    // 4. Add more debt: missing repro
    budget
        .consume_debt(
            &ProofDebt::MissingRepro,
            "claim:clm_test_001",
            "no reproduction case available",
            false,
        )
        .expect("consume should succeed");

    // 250k + 150k = 400k = 80% -> Warn
    assert_eq!(budget.consumed_micros, 400_000);
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Warn);

    // 5. Add missing benchmark -> 500k = 100% -> Degrade
    budget
        .consume_debt(
            &ProofDebt::MissingBenchmark,
            "claim:clm_test_001",
            "no benchmark reference",
            false,
        )
        .expect("consume should succeed");

    assert_eq!(budget.consumed_micros, 500_000);
    assert!(budget.is_exhausted());
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Degrade);

    // 6. Evidence found: source basis resolved -> replenish 250k
    budget.resolve_debt(
        &ProofDebt::MissingSourceBasis,
        "evidence:evb_source_found",
        "source artifact located and verified",
    );

    assert_eq!(budget.consumed_micros, 250_000);
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Proceed);
}

#[test]
fn strict_mode_blocks_overdraw() {
    let mut budget = ProofDebtBudgetV1::new("claim:clm_strict", 100_000);

    // Strict mode: consuming more than budget returns None
    let result = budget.consume(200_000, "test", "overdraw", true);
    assert!(result.is_none());
    assert_eq!(budget.consumed_micros, 0); // unchanged
}

#[test]
fn operator_waiver_preserves_debt() {
    let mut budget = ProofDebtBudgetV1::new("claim:clm_waiver", 500_000);
    budget.consume(500_000, "claim", "full debt", false);
    assert!(budget.is_exhausted());

    // Operator waives the debt
    let waiver = ProofDebtWaiverReceipt::new(
        &budget,
        500_000,
        "operator:josh",
        "accepting risk: claim is advisory-only and won't be used for public assertions",
    );

    assert_eq!(budget.consumed_micros, 500_000);
    assert!(budget.is_exhausted());

    let verified =
        verify_proof_debt_waiver(&budget, waiver, |operator| operator == "operator:josh")
            .expect("authorized operator should produce a verified waiver");
    let gate = evaluate_proof_debt_gate_with_waiver(&budget, Some(&verified));
    assert_eq!(gate.decision, ProofDebtGateDecision::Waived);
}

#[test]
fn ledger_records_proof_debt_events() {
    let mut budget = ProofDebtBudgetV1::new("claim:clm_ledger_test", 500_000);

    // Build ledger entries for proof-debt events
    let entry1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_ledger_test", "src1", "sp1", "test claim")
        .unwrap();

    // Consume debt
    let debit = budget
        .consume_debt(
            &ProofDebt::MissingSourceBasis,
            "claim:clm_ledger_test",
            "missing source",
            false,
        )
        .expect("consume should succeed");

    let entry2 = LedgerEntryBuilder::new(2, Some(entry1.entry_digest.clone()))
        .add_proof_debt_consumed(
            &budget.budget_id,
            &debit.debit_id,
            debit.amount_micros,
            "claim:clm_ledger_test",
            debit.overdrawn,
        )
        .unwrap();

    // Replenish debt
    let credit = budget.resolve_debt(
        &ProofDebt::MissingSourceBasis,
        "evidence:evb_found",
        "source found",
    );

    let entry3 = LedgerEntryBuilder::new(3, Some(entry2.entry_digest.clone()))
        .add_proof_debt_replenished(
            &budget.budget_id,
            &credit.as_ref().expect("credit should exist").credit_id,
            credit.as_ref().expect("credit should exist").amount_micros,
            "evidence:evb_found",
        )
        .unwrap();

    // Verify ledger chain
    let entries = vec![entry1, entry2, entry3];
    let verification = claim_ledger::verify_ledger(
        &entries,
        &ExpectedLedgerHead::new(3, entries[2].entry_digest.clone()),
    )
    .unwrap();
    assert_eq!(verification.last_sequence, 3);
}

#[test]
fn retract_fires_on_severe_overdraw() {
    let mut budget = ProofDebtBudgetV1::new("claim:clm_retract", 100_000);
    // Consume 120k in non-strict mode
    budget.consume(120_000, "test", "severe overdraw", false);

    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Retract);
    assert!(gate.decision.blocks());
    assert!(!gate.decision.allows_proceed());
}

#[test]
fn debt_weights_are_ordered_by_danger() {
    // MissingSourceBasis should be the heaviest
    let source = proof_debt_weight(&ProofDebt::MissingSourceBasis);
    let repro = proof_debt_weight(&ProofDebt::MissingRepro);
    let benchmark = proof_debt_weight(&ProofDebt::MissingBenchmark);
    let external = proof_debt_weight(&ProofDebt::MissingExternalValidation);

    assert!(source > repro, "source basis should be heavier than repro");
    assert!(repro > benchmark, "repro should be heavier than benchmark");
    assert!(
        benchmark > external,
        "benchmark should be heavier than external validation"
    );
    assert_eq!(proof_debt_weight(&ProofDebt::None), 0);
}

#[test]
fn budget_id_is_deterministic() {
    let b1 = ProofDebtBudgetV1::new("claim:clm_abc", 500_000);
    let b2 = ProofDebtBudgetV1::new("claim:clm_abc", 1_000_000);
    // Same scope -> same budget_id regardless of budget amount
    assert_eq!(b1.budget_id, b2.budget_id);
    assert!(b1.budget_id.starts_with("pdb_"));
}

#[test]
fn automated_pipeline_budget_for_claim() {
    // The standard entry point: create a budget from a claim's proof_debt vec
    // and immediately consume all debts. This is the automated path.
    let debts = vec![
        ProofDebt::MissingSourceBasis,
        ProofDebt::MissingRepro,
        ProofDebt::MissingBenchmark,
    ];

    // Total weight: 250k + 150k + 100k = 500k
    assert_eq!(total_proof_debt_weight(&debts), 500_000);

    let (budget, debits) = budget_for_claim("claim:clm_auto_pipe", &debts, 500_000);

    // All three debts consumed
    assert_eq!(debits.len(), 3);
    assert_eq!(budget.consumed_micros, 500_000);
    assert!(budget.is_exhausted());

    // Gate should fire Degrade
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Degrade);
}

#[test]
fn automated_pipeline_custom_config() {
    // Operator tunes weights to be more conservative on source basis
    let config = ProofDebtBudgetConfig {
        weight_missing_source_basis: 400_000, // heavier than default 250k
        weight_missing_repro: 200_000,
        weight_missing_benchmark: 100_000,
        warn_threshold_pct: 60, // warn earlier
        ..ProofDebtBudgetConfig::default()
    };

    let mut budget = ProofDebtBudgetV1::new("claim:clm_config_pipe", 700_000);

    // Consume with custom weights
    budget
        .consume_debt_with_config(
            &ProofDebt::MissingSourceBasis,
            &config,
            "claim:clm_config_pipe",
            "no source",
            false,
        )
        .expect("consume should succeed");

    // 400k / 700k = 57% -> Proceed with default, but custom warn at 60%...
    // 57% < 60% so still Proceed
    assert_eq!(budget.consumed_micros, 400_000);
    let gate = evaluate_proof_debt_gate_with_config(&budget, &config);
    assert_eq!(gate.decision, ProofDebtGateDecision::Proceed);

    // Add repro debt: 400k + 200k = 600k / 700k = 85% -> Warn (>= 60%)
    budget
        .consume_debt_with_config(
            &ProofDebt::MissingRepro,
            &config,
            "claim:clm_config_pipe",
            "no repro",
            false,
        )
        .expect("consume should succeed");

    assert_eq!(budget.consumed_micros, 600_000);
    let gate = evaluate_proof_debt_gate_with_config(&budget, &config);
    assert_eq!(gate.decision, ProofDebtGateDecision::Warn);
}

#[test]
fn automated_pipeline_summary_serializable() {
    let mut budget = ProofDebtBudgetV1::new("claim:clm_summary_test", 500_000);
    budget.consume(350_000, "test", "70%", false);

    let summary = ProofDebtSummaryV1::from_budget(&budget);

    // Verify it's serializable (this is what context packs and reports need)
    let json = serde_json::to_string(&summary).expect("summary should serialize");
    assert!(json.contains("ProofDebtSummaryV1"));
    assert!(json.contains("claim:clm_summary_test"));
    assert!(json.contains("proceed")); // gate decision at 70%

    // And deserializable
    let restored: ProofDebtSummaryV1 =
        serde_json::from_str(&json).expect("summary should deserialize");
    assert_eq!(restored, summary);
}

#[test]
fn automated_pipeline_full_lifecycle_with_ledger() {
    // Full automated lifecycle: create budget -> consume -> waive authorization -> ledger
    let debts = vec![ProofDebt::MissingSourceBasis, ProofDebt::MissingRepro];
    let (budget, debits) = budget_for_claim("claim:clm_full_cycle", &debts, 300_000);

    // Budget exhausted (250k + 150k = 400k > 300k budget = 133%)
    assert!(budget.is_exhausted());

    // Gate fires Retract (consumed >= 120% of budget)
    let gate = evaluate_proof_debt_gate(&budget);
    assert_eq!(gate.decision, ProofDebtGateDecision::Retract);

    // Operator waives the overdrawn amount
    let waiver = ProofDebtWaiverReceipt::new(
        &budget,
        budget.consumed_micros,
        "operator:josh",
        "advisory-only claim, accepting risk",
    );
    assert_eq!(budget.consumed_micros, 400_000);
    let verified =
        verify_proof_debt_waiver(&budget, waiver, |operator| operator == "operator:josh")
            .expect("authorized operator should produce a verified waiver");
    assert_eq!(
        evaluate_proof_debt_gate_with_waiver(&budget, Some(&verified)).decision,
        ProofDebtGateDecision::Waived
    );

    // Build ledger entries for the full lifecycle
    let entry1 = LedgerEntryBuilder::new(1, None)
        .add_claim("clm_full_cycle", "src1", "sp1", "test claim")
        .unwrap();

    let mut entries = vec![entry1.clone()];
    let mut prev_digest = entry1.entry_digest.clone();

    for debit in &debits {
        let entry = LedgerEntryBuilder::new(entries.len() as u64 + 1, Some(prev_digest.clone()))
            .add_proof_debt_consumed(
                &budget.budget_id,
                &debit.debit_id,
                debit.amount_micros,
                &debit.source,
                debit.overdrawn,
            )
            .unwrap();
        prev_digest = entry.entry_digest.clone();
        entries.push(entry);
    }

    // Waivers are separate authorizations; they are not replenishment events.
    let verification = claim_ledger::verify_ledger(
        &entries,
        &ExpectedLedgerHead::new(
            entries.len() as u64,
            entries.last().unwrap().entry_digest.clone(),
        ),
    )
    .unwrap();
    assert_eq!(verification.last_sequence, entries.len() as u64);
}

#[test]
fn consumed_pct_handles_u64_max_without_wrapping_and_gates_correctly() {
    let mut exact = ProofDebtBudgetV1::new("claim:percent-max", u64::MAX);
    exact.consumed_micros = u64::MAX;
    assert_eq!(exact.consumed_pct(), 100);
    assert_eq!(
        evaluate_proof_debt_gate(&exact).decision,
        ProofDebtGateDecision::Degrade
    );

    let mut near_overflow = ProofDebtBudgetV1::new("claim:percent-near-max", u64::MAX - 1);
    near_overflow.consumed_micros = u64::MAX;
    assert_eq!(near_overflow.consumed_pct(), 100);
    assert_eq!(
        evaluate_proof_debt_gate(&near_overflow).decision,
        ProofDebtGateDecision::Degrade
    );

    let mut doubled = ProofDebtBudgetV1::new("claim:percent-double", u64::MAX / 2);
    doubled.consumed_micros = u64::MAX;
    assert_eq!(doubled.consumed_pct(), 200);
    assert_eq!(
        evaluate_proof_debt_gate(&doubled).decision,
        ProofDebtGateDecision::Retract
    );

    let zero_budget = ProofDebtBudgetV1::new("claim:percent-zero", 0);
    assert_eq!(zero_budget.consumed_pct(), 100);
    assert_eq!(
        evaluate_proof_debt_gate(&zero_budget).decision,
        ProofDebtGateDecision::Degrade
    );
}

#[test]
fn waiver_requires_integrity_authority_and_an_exact_budget_debt_snapshot() {
    let time = chrono::Utc::now();
    let mut budget = ProofDebtBudgetV1::new_at("claim:waiver-integrity", 500_000, time);
    budget.consume_at(500_000, "claim", "unproven", false, time);
    let waiver = ProofDebtWaiverReceipt::new_at(
        &budget,
        500_000,
        "operator:authorized",
        "bounded advisory exception",
        time,
    );

    let serialized = serde_json::to_string(&waiver).unwrap();
    let deserialized: ProofDebtWaiverReceipt = serde_json::from_str(&serialized).unwrap();
    assert!(verify_proof_debt_waiver(&budget, deserialized, |_| false).is_err());
    assert_eq!(
        evaluate_proof_debt_gate(&budget).decision,
        ProofDebtGateDecision::Degrade
    );

    let forged = ProofDebtWaiverReceipt::new_at(
        &budget,
        500_000,
        "operator:forged",
        "attacker-created public data",
        time,
    );
    assert!(forged.has_valid_integrity());
    assert!(verify_proof_debt_waiver(&budget, forged, |_| false).is_err());

    let verified = verify_proof_debt_waiver(&budget, waiver.clone(), |operator| {
        operator == "operator:authorized"
    })
    .expect("explicit operator authorization should verify an intact receipt");
    assert_eq!(
        evaluate_proof_debt_gate_with_waiver(&budget, Some(&verified)).decision,
        ProofDebtGateDecision::Waived
    );
    assert_eq!(budget.consumed_micros, 500_000, "waivers never reduce debt");

    let mut cases = Vec::new();
    let mut wrong_schema = waiver.clone();
    wrong_schema.schema_version = "ProofDebtWaiverReceiptV0".into();
    cases.push(wrong_schema);
    let mut wrong_domain = waiver.clone();
    wrong_domain.authorization_domain = "other.domain".into();
    cases.push(wrong_domain);
    let mut changed_budget_id = waiver.clone();
    changed_budget_id.budget_id = "pdb_other".into();
    cases.push(changed_budget_id);
    let mut changed_scope = waiver.clone();
    changed_scope.scope = "claim:other-scope".into();
    cases.push(changed_scope);
    let mut changed_ceiling = waiver.clone();
    changed_ceiling.budget_micros = 500_001;
    cases.push(changed_ceiling);
    let mut wrong_id = waiver.clone();
    wrong_id.waiver_id = "pdw_forged".into();
    cases.push(wrong_id);
    let mut changed_amount = waiver.clone();
    changed_amount.waived_amount_micros = 1;
    cases.push(changed_amount);
    let mut changed_rationale = waiver.clone();
    changed_rationale.rationale = "tampered rationale".into();
    cases.push(changed_rationale);
    let mut changed_time = waiver.clone();
    changed_time.recorded_time = time + chrono::Duration::seconds(1);
    cases.push(changed_time);
    let mut changed_snapshot = waiver.clone();
    changed_snapshot.outstanding_debt_micros = 499_999;
    cases.push(changed_snapshot);

    for tampered in cases {
        assert!(
            verify_proof_debt_waiver(&budget, tampered, |operator| operator
                == "operator:authorized")
            .is_err(),
            "tampered public waiver data must not become a verified authorization"
        );
    }

    let mut changed_budget = budget.clone();
    changed_budget.consume_at(1, "claim", "new debt", false, time);
    assert!(
        verify_proof_debt_waiver(&changed_budget, waiver, |operator| {
            operator == "operator:authorized"
        })
        .is_err()
    );
    assert_eq!(
        evaluate_proof_debt_gate_with_waiver(&changed_budget, Some(&verified)).decision,
        ProofDebtGateDecision::Degrade
    );

    let mut changed_scope = budget.clone();
    changed_scope.scope = "claim:waiver-integrity-other-scope".into();
    assert_ne!(
        evaluate_proof_debt_gate_with_waiver(&changed_scope, Some(&verified)).decision,
        ProofDebtGateDecision::Waived
    );

    let mut changed_ceiling = budget;
    changed_ceiling.budget_micros = 500_001;
    assert_ne!(
        evaluate_proof_debt_gate_with_waiver(&changed_ceiling, Some(&verified)).decision,
        ProofDebtGateDecision::Waived
    );
}
