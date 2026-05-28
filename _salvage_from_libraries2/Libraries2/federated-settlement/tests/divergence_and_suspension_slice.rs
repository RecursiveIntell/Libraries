use federated_settlement::{
    evaluate_divergence_or_suspension, evaluate_shared_replay, SettlementCaseV1,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct DivergenceFixture {
    settlement_case: SettlementCaseV1,
}

fn load_fixture(name: &str) -> DivergenceFixture {
    let path = format!(
        "{}/../contracts/fixtures/v18/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn divergence_report_keeps_blocking_runtime_and_local_dissent_visible() {
    let fixture = load_fixture("divergence-report.json");
    let replay = evaluate_shared_replay(&fixture.settlement_case, "2026-03-14T00:00:00Z");
    let (report, suspension) = evaluate_divergence_or_suspension(
        &fixture.settlement_case,
        &replay,
        "shared and local readings diverged",
        "2026-03-14T00:00:00Z",
    );

    assert!(report.requires_local_dissent_preservation);
    assert!(report.recommends_suspension);
    assert!(report
        .blocking_runtime_names
        .contains(&"runtime-a".to_string()));
    assert!(suspension.is_some());
}

#[test]
fn treaty_suspension_demands_resumption_requirements() {
    let fixture = load_fixture("treaty-suspension.json");
    let replay = evaluate_shared_replay(&fixture.settlement_case, "2026-03-14T00:00:00Z");
    let (_, suspension) = evaluate_divergence_or_suspension(
        &fixture.settlement_case,
        &replay,
        "non-admitted surfaces force suspension",
        "2026-03-14T00:00:00Z",
    );

    let suspension = suspension.expect("suspension should be emitted");
    assert!(!suspension.resumption_requirements.is_empty());
    assert!(suspension.advisory_only);
}
