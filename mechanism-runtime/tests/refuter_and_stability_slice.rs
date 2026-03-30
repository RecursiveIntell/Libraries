use mechanism_runtime::{
    evaluate_fit_run, FitDisposition, MechanismBundleV1, RolloutStabilityReportV1,
    SimulationContractV1, TheoryRefuterSuiteV1, TheoryVersionV1,
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
fn fit_run_blocks_when_required_refuters_are_missing() {
    let fixture = load_fixture("refuter-gated-fit.json");
    let fit = evaluate_fit_run(
        &fixture.mechanism_bundle,
        &fixture.theory_version,
        &fixture.simulation_contract,
        &fixture.theory_refuter_suite,
        &fixture.rollout_stability_report,
        fixture.fit_score,
        "2026-03-14T00:00:00Z",
    );

    assert_eq!(
        fit.disposition,
        FitDisposition::PromotionBlockedMissingRefuter
    );
    assert!(fit
        .notes
        .iter()
        .any(|note| note.contains("missing required refuters")));
}

#[test]
fn fit_run_blocks_when_stability_report_is_not_review_clear() {
    let fixture = load_fixture("stability-report-block.json");
    let fit = evaluate_fit_run(
        &fixture.mechanism_bundle,
        &fixture.theory_version,
        &fixture.simulation_contract,
        &fixture.theory_refuter_suite,
        &fixture.rollout_stability_report,
        fixture.fit_score,
        "2026-03-14T00:00:00Z",
    );

    assert_eq!(
        fit.disposition,
        FitDisposition::PromotionBlockedStabilityRisk
    );
    assert!(fit
        .notes
        .iter()
        .any(|note| note.contains("stability blockers")));
}
