use discovery_portfolio::{
    evaluate_portfolio_plan, DiscoveryProgramV1, ExperimentCampaignV1, InformationValueEstimateV1,
    PortfolioPlanV1, ProgramHypothesisSetV1, VerificationLoadBudgetV1,
};
use serde::Deserialize;

#[derive(Debug, Deserialize)]
struct PortfolioFixture {
    discovery_program: DiscoveryProgramV1,
    program_hypothesis_set: ProgramHypothesisSetV1,
    portfolio_plan: PortfolioPlanV1,
    campaigns: Vec<ExperimentCampaignV1>,
    information_value_estimates: Vec<InformationValueEstimateV1>,
    verification_load_budget: VerificationLoadBudgetV1,
}

fn load_fixture(name: &str) -> PortfolioFixture {
    let path = format!(
        "{}/../contracts/fixtures/v18/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn portfolio_trace_pauses_explicitly_when_budget_is_exhausted() {
    let fixture = load_fixture("degraded-path.json");
    let trace = evaluate_portfolio_plan(
        &fixture.discovery_program,
        &fixture.program_hypothesis_set,
        &fixture.portfolio_plan,
        &fixture.campaigns,
        &fixture.information_value_estimates,
        &fixture.verification_load_budget,
        "2026-03-14T00:00:00Z",
    );

    assert!(trace.degraded);
    assert!(trace
        .decisions
        .iter()
        .any(|line| line.rationale.contains("budget")));
}

#[test]
fn happy_portfolio_fixture_launches_at_least_one_campaign() {
    let fixture = load_fixture("happy-path.json");
    let trace = evaluate_portfolio_plan(
        &fixture.discovery_program,
        &fixture.program_hypothesis_set,
        &fixture.portfolio_plan,
        &fixture.campaigns,
        &fixture.information_value_estimates,
        &fixture.verification_load_budget,
        "2026-03-14T00:00:00Z",
    );

    assert!(trace
        .decisions
        .iter()
        .any(|line| format!("{:?}", line.decision).contains("Launch")));
    assert!(trace
        .decisions
        .iter()
        .all(|line| !line.hypothesis_refs.is_empty()));
}
