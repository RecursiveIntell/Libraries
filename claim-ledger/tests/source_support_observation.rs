//! A source observation binds reported support fields without promoting them to a judgment.
use claim_ledger::{
    compact_ledger, verify_ledger, CompactionPolicy, ExpectedLedgerHead, LedgerEntryBuilder,
    LedgerEvent, ProofDebt, SourceSupportObservationV1, SupportState, UnprojectableEventPolicy,
};

fn observation() -> SourceSupportObservationV1 {
    SourceSupportObservationV1 {
        source_bundle_digest: "sha256:bundle".into(),
        source_claim_ref: "python:clm:1".into(),
        source_evidence_bundle_ref: "python:evb:1".into(),
        source_admission_ref: "python:sar:1".into(),
        source_previous_judgment_ref: "python:sup:0".into(),
        source_new_judgment_ref: "python:sup:1".into(),
        reported_state: SupportState::Supported,
        method: "test_fixture_admitted".into(),
        rationale: "first admission".into(),
        proof_payload_digest: Some("sha256:proof".into()),
        reported_proof_debt: vec![ProofDebt::None],
    }
}

#[test]
fn source_observation_binds_every_reported_field_without_minting_support() {
    let first = LedgerEntryBuilder::new(1, None)
        .add_source_support_observation_v1(observation())
        .unwrap();
    let head = ExpectedLedgerHead::new(1, first.entry_digest.clone());
    verify_ledger(std::slice::from_ref(&first), &head).unwrap();
    assert_eq!(first.event.type_name(), "source_support_observation_v1");
    assert!(matches!(
        first.event,
        LedgerEvent::SourceSupportObservationV1 { .. }
    ));
    let mut encoded = serde_json::to_value(&first.event).unwrap();
    encoded["observation"]["unknown"] = serde_json::Value::Bool(true);
    assert!(serde_json::from_value::<LedgerEvent>(encoded).is_err());

    let mut mutations: Vec<SourceSupportObservationV1> = Vec::new();
    let mut next = observation();
    next.source_bundle_digest.push('x');
    mutations.push(next);
    let mut next = observation();
    next.source_claim_ref.push('x');
    mutations.push(next);
    let mut next = observation();
    next.source_evidence_bundle_ref.push('x');
    mutations.push(next);
    let mut next = observation();
    next.source_admission_ref.push('x');
    mutations.push(next);
    let mut next = observation();
    next.source_previous_judgment_ref.push('x');
    mutations.push(next);
    let mut next = observation();
    next.source_new_judgment_ref.push('x');
    mutations.push(next);
    let mut next = observation();
    next.reported_state = SupportState::PartiallySupported;
    mutations.push(next);
    let mut next = observation();
    next.method.push('x');
    mutations.push(next);
    let mut next = observation();
    next.rationale.push('x');
    mutations.push(next);
    let mut next = observation();
    next.proof_payload_digest = None;
    mutations.push(next);
    let mut next = observation();
    next.proof_payload_digest = Some(String::new());
    mutations.push(next);
    let mut next = observation();
    next.reported_proof_debt = vec![ProofDebt::MissingSourceBasis];
    mutations.push(next);
    for changed in mutations {
        let mut tampered = first.clone();
        tampered.event = LedgerEvent::SourceSupportObservationV1 {
            observation: changed,
        };
        assert!(verify_ledger(&[tampered], &head).is_err());
    }

    let mut repeated = observation();
    repeated.source_admission_ref = "python:sar:2".into();
    repeated.source_previous_judgment_ref = "python:sup:1".into();
    repeated.source_new_judgment_ref = "python:sup:2".into();
    repeated.rationale = "second admission".into();
    let second = LedgerEntryBuilder::new(2, Some(first.entry_digest.clone()))
        .add_source_support_observation_v1(repeated)
        .unwrap();
    assert_ne!(first.entry_digest, second.entry_digest);
    verify_ledger(
        &[first.clone(), second.clone()],
        &ExpectedLedgerHead::new(2, second.entry_digest.clone()),
    )
    .unwrap();

    // Snapshot V1 cannot interpret this source observation as authoritative support.
    let policy = CompactionPolicy {
        retain_tail_entries: 0,
        unprojectable_events: UnprojectableEventPolicy::FailClosed,
    };
    assert!(compact_ledger(&[first.clone(), second.clone()], &policy).is_err());
    let retained = compact_ledger(
        &[first, second],
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::Retain,
        },
    )
    .unwrap();
    assert!(retained.snapshot.claim_support.is_empty());
    assert_eq!(retained.retained_tail.len(), 2);
}
