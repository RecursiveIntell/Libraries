use claim_ledger::{
    compact_ledger, compact_ledger_from_snapshot, compute_entry_digest, verify_compaction,
    verify_snapshot, CompactionPolicy, LedgerEntry, LedgerEvent, SupportState,
    UnprojectableEventPolicy,
};

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

fn projected_entries() -> Vec<LedgerEntry> {
    let mut entries = Vec::new();
    append(
        &mut entries,
        LedgerEvent::ClaimAdded {
            claim_id: "claim-1".into(),
            source_id: "semantic-memory:fact:fact-1".into(),
            span_id: "span-1".into(),
            normalized_claim: "the claim".into(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::SupportJudgment {
            support_judgment_id: "judgment-1".into(),
            claim_id: "claim-1".into(),
            evidence_bundle_ref: "bundle-1".into(),
            support_state: SupportState::Supported,
            method: "operator".into(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ContradictionCandidate {
            contradiction_id: "contradiction-1".into(),
            claim_refs: vec!["claim-1".into()],
            pattern: "test".into(),
            rationale: "conflict".into(),
        },
    );
    append(
        &mut entries,
        LedgerEvent::ContradictionResolved {
            contradiction_resolution_receipt_id: "resolution-1".into(),
            contradiction_id: "contradiction-1".into(),
            resolution: "confirmed".into(),
            affected_claim_refs: vec!["claim-1".into()],
        },
    );
    entries
}

#[test]
fn snapshot_round_trip_and_receipt_verify() {
    let entries = projected_entries();
    let compacted = compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 1,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap();
    let json = serde_json::to_string(&compacted.snapshot).unwrap();
    let decoded = serde_json::from_str(&json).unwrap();
    assert_eq!(compacted.snapshot, decoded);
    verify_snapshot(&decoded).unwrap();
    verify_compaction(&decoded, &compacted.retained_tail, &compacted.receipt).unwrap();
}

#[test]
fn tampered_snapshot_is_rejected() {
    let mut snapshot = compact_ledger(
        &projected_entries(),
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap()
    .snapshot;
    snapshot.claims[0].normalized_claim = "tampered".into();
    assert!(verify_snapshot(&snapshot).is_err());
}

#[test]
fn compacted_replay_equals_full_replay() {
    let entries = projected_entries();
    let full = compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap();
    let first = compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 2,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap();
    let replayed = compact_ledger_from_snapshot(
        Some(&first.snapshot),
        &first.retained_tail,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap();
    assert_eq!(full.snapshot.claims, replayed.snapshot.claims);
    assert_eq!(
        full.snapshot.fact_to_claim_links,
        replayed.snapshot.fact_to_claim_links
    );
    assert_eq!(
        full.snapshot.content_to_claim_links,
        replayed.snapshot.content_to_claim_links
    );
    assert_eq!(
        full.snapshot.support_judgments,
        replayed.snapshot.support_judgments
    );
    assert_eq!(full.snapshot.claim_support, replayed.snapshot.claim_support);
    assert_eq!(
        full.snapshot.contradiction_states,
        replayed.snapshot.contradiction_states
    );
    assert_eq!(
        full.snapshot.last_compacted_sequence,
        replayed.snapshot.last_compacted_sequence
    );
    assert_eq!(
        full.snapshot.last_compacted_entry_digest,
        replayed.snapshot.last_compacted_entry_digest
    );
}

#[test]
fn unprojectable_events_are_retained_or_fail_closed() {
    let mut entries = projected_entries();
    entries.insert(
        1,
        LedgerEntry {
            sequence: 0,
            previous_entry_digest: None,
            event: LedgerEvent::BundleExported {
                bundle_id: "bundle".into(),
                export_receipt_id: "receipt".into(),
                output_ref: "output".into(),
                output_digest: "digest".into(),
            },
            entry_digest: String::new(),
        },
    );
    let events: Vec<_> = entries.into_iter().map(|entry| entry.event).collect();
    let mut entries = Vec::new();
    for event in events {
        append(&mut entries, event);
    }

    let retained = compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::Retain,
        },
    )
    .unwrap();
    assert_eq!(retained.snapshot.last_compacted_sequence, 1);
    assert!(matches!(
        retained.retained_tail[0].event,
        LedgerEvent::BundleExported { .. }
    ));
    verify_compaction(
        &retained.snapshot,
        &retained.retained_tail,
        &retained.receipt,
    )
    .unwrap();

    assert!(compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .is_err());
}

#[test]
fn unknown_event_types_fail_closed_before_compaction() {
    let entry = &projected_entries()[0];
    let mut value = serde_json::to_value(entry).unwrap();
    value["event"]["type"] = serde_json::json!("future_unknown_event");
    let jsonl = format!("{}\n", serde_json::to_string(&value).unwrap());
    assert!(claim_ledger::parse_ledger_entries(&jsonl).is_err());
}

#[test]
fn snapshot_preserves_last_writer_content_link() {
    let mut entries = Vec::new();
    for claim_id in ["claim-z", "claim-a"] {
        append(
            &mut entries,
            LedgerEvent::ClaimAdded {
                claim_id: claim_id.into(),
                source_id: format!("source:{claim_id}"),
                span_id: "span".into(),
                normalized_claim: "duplicate content".into(),
            },
        );
    }
    let compacted = compact_ledger(
        &entries,
        &CompactionPolicy {
            retain_tail_entries: 0,
            unprojectable_events: UnprojectableEventPolicy::FailClosed,
        },
    )
    .unwrap();
    assert_eq!(compacted.snapshot.content_to_claim_links.len(), 1);
    assert_eq!(
        compacted.snapshot.content_to_claim_links[0].claim_id,
        "claim-a"
    );
}
