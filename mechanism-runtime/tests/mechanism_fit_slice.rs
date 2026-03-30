use mechanism_runtime::{
    evaluate_fit_run, MechanismBundleV1, RolloutStabilityReportV1, SimulationContractV1,
    TheoryRefuterSuiteV1, TheoryVersionV1,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct FitFixture {
    mechanism_bundle: MechanismBundleV1,
    theory_version: TheoryVersionV1,
    simulation_contract: SimulationContractV1,
    theory_refuter_suite: TheoryRefuterSuiteV1,
    rollout_stability_report: RolloutStabilityReportV1,
    fit_score: f64,
}

fn load_fixture(name: &str) -> FitFixture {
    let path = format!(
        "{}/../contracts/fixtures/v17/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn fit_fixture_remains_advisory_without_refuter_suite() {
    let fixture = load_fixture("degraded-path.json");
    let fit = evaluate_fit_run(
        &fixture.mechanism_bundle,
        &fixture.theory_version,
        &fixture.simulation_contract,
        &fixture.theory_refuter_suite,
        &fixture.rollout_stability_report,
        fixture.fit_score,
        "2026-03-14T00:00:00Z",
    );

    assert!(fit.advisory_only);
    assert!(fit.degraded);
    assert!(!fit.refuter_ready);
    assert!(!fit.stability_clear);
}

#[test]
fn happy_fixture_can_reach_local_review_but_not_authority() {
    let fixture = load_fixture("happy-path.json");
    let fit = evaluate_fit_run(
        &fixture.mechanism_bundle,
        &fixture.theory_version,
        &fixture.simulation_contract,
        &fixture.theory_refuter_suite,
        &fixture.rollout_stability_report,
        fixture.fit_score,
        "2026-03-14T00:00:00Z",
    );

    assert!(!fit.degraded);
    assert!(fit.refuter_ready);
    assert!(fit.stability_clear);
}
