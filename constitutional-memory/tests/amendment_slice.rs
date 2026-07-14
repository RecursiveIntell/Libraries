use constitutional_memory::{evaluate_amendment, evaluate_archive_compaction, AmendmentProposalV1};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct AmendmentFixture {
    amendment_proposal: AmendmentProposalV1,
    migration_obligations_satisfied: bool,
}

#[derive(Debug, Deserialize)]
struct ArchiveFixture {
    preserved_refs: Vec<String>,
    dropped_detail_refs: Vec<String>,
    guaranteed_query_modes: Vec<String>,
}

fn load_fixture(name: &str) -> AmendmentFixture {
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
fn amendment_requires_rollback_handle_before_approval() {
    let fixture = load_fixture("degraded-path.json");
    let decision = evaluate_amendment(
        &fixture.amendment_proposal,
        fixture.migration_obligations_satisfied,
        None,
        None,
        "2026-03-14T00:00:00Z",
    );

    assert!(decision.advisory_only);
    assert!(!decision.rollback_ready);
}

#[test]
fn happy_amendment_fixture_retains_rollback_lane() {
    let fixture = load_fixture("happy-path.json");
    let archive = load_archive_fixture("archive-compaction.json");
    let (archive_manifest, guarantee, _) = evaluate_archive_compaction(
        fixture.amendment_proposal.charter_bundle_id.clone(),
        archive.preserved_refs,
        archive.dropped_detail_refs,
        archive.guaranteed_query_modes,
    );
    let decision = evaluate_amendment(
        &fixture.amendment_proposal,
        fixture.migration_obligations_satisfied,
        Some(&archive_manifest),
        Some(&guarantee),
        "2026-03-14T00:00:00Z",
    );

    assert!(decision.rollback_ready);
    assert!(!decision.advisory_only);
    assert!(decision.semantic_diff_id.is_some());
}
