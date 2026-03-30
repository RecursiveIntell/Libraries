use constitutional_memory::{
    evaluate_amendment, evaluate_archive_compaction, AmendmentProposalV1, CharterBundleV1,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AmendmentFixture {
    amendment_proposal: AmendmentProposalV1,
    migration_obligations_satisfied: bool,
}

#[derive(Debug, Deserialize)]
struct ArchiveFixture {
    charter_bundle: CharterBundleV1,
    preserved_refs: Vec<String>,
    dropped_detail_refs: Vec<String>,
    guaranteed_query_modes: Vec<String>,
}

fn load_amendment_fixture(name: &str) -> AmendmentFixture {
    let path = format!(
        "{}/../contracts/fixtures/v19/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn load_archive_fixture(name: &str) -> ArchiveFixture {
    let path = format!(
        "{}/../contracts/fixtures/v19/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn archive_compaction_keeps_query_guarantees_explicit() {
    let fixture = load_archive_fixture("archive-compaction.json");
    let (archive_manifest, guarantee, receipt) = evaluate_archive_compaction(
        fixture.charter_bundle.charter_bundle_id,
        fixture.preserved_refs,
        fixture.dropped_detail_refs,
        fixture.guaranteed_query_modes,
    );

    assert_eq!(
        guarantee.archive_manifest_id,
        archive_manifest.archive_manifest_id
    );
    assert!(receipt
        .queryable_degradation_note
        .contains("historical query guarantee"));
}

#[test]
fn amendment_approval_requires_archive_outputs_and_semantic_diff() {
    let amendment = load_amendment_fixture("semantic-diff-linked-amendment.json");
    let archive = load_archive_fixture("archive-compaction.json");
    let (archive_manifest, guarantee, _) = evaluate_archive_compaction(
        archive.charter_bundle.charter_bundle_id,
        archive.preserved_refs,
        archive.dropped_detail_refs,
        archive.guaranteed_query_modes,
    );
    let decision = evaluate_amendment(
        &amendment.amendment_proposal,
        amendment.migration_obligations_satisfied,
        Some(&archive_manifest),
        Some(&guarantee),
        "2026-03-14T00:00:00Z",
    );

    assert!(decision.semantic_diff_id.is_some());
    assert!(decision.archive_manifest_id.is_some());
    assert!(!decision.advisory_only);
}
