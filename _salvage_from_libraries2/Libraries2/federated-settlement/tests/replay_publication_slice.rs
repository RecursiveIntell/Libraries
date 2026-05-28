use federated_settlement::{evaluate_shared_replay, SettlementCaseV1};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct ReplayFixture {
    settlement_case: SettlementCaseV1,
}

fn load_fixture(name: &str) -> ReplayFixture {
    let path = format!(
        "{}/../contracts/fixtures/v18/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn shared_replay_slice_marks_complete_replay_as_non_degraded() {
    let fixture = load_fixture("shared-replay-happy.json");
    let replay = evaluate_shared_replay(&fixture.settlement_case, "2026-03-14T00:00:00Z");

    assert!(replay.all_required_replays_available);
    assert!(!replay.degraded);
    assert!(replay.missing_required_runtime_names.is_empty());
}

#[test]
fn shared_replay_slice_keeps_missing_runtime_visible() {
    let fixture = load_fixture("shared-replay-downgrade.json");
    let replay = evaluate_shared_replay(&fixture.settlement_case, "2026-03-14T00:00:00Z");

    assert!(replay.degraded);
    assert_eq!(replay.missing_required_runtime_names, vec!["runtime-b"]);
}
