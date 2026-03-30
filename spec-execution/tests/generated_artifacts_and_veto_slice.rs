use serde::Deserialize;
use spec_execution::{
    establish_veto_challenge_baseline, generate_companion_bundles, HumanVetoBundleV1,
    MetaChallengeBundleV1, NormativeASTV1, ProofObligationSetV1, SpecBundleV1,
};

#[derive(Debug, Deserialize)]
struct SpecFixture {
    spec_bundle: SpecBundleV1,
    normative_ast: NormativeASTV1,
    proof_obligation_set: ProofObligationSetV1,
}

#[derive(Debug, Deserialize)]
struct GovernanceFixture {
    spec_bundle: SpecBundleV1,
    normative_ast: NormativeASTV1,
    proof_obligation_set: ProofObligationSetV1,
    human_veto_bundle: HumanVetoBundleV1,
    meta_challenge_bundle: MetaChallengeBundleV1,
}

fn load_fixture(name: &str) -> SpecFixture {
    let path = format!(
        "{}/../contracts/fixtures/v20/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

fn load_governance_fixture(name: &str) -> GovernanceFixture {
    let path = format!(
        "{}/../contracts/fixtures/v20/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn companion_generation_emits_self_hosting_receipt_with_backpointers() {
    let fixture = load_fixture("generated-companions.json");
    let bundles = generate_companion_bundles(
        &fixture.spec_bundle,
        &fixture.normative_ast,
        &fixture.proof_obligation_set,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );

    assert_eq!(
        bundles.interpreter_bundle.generated_schema_bundle_id,
        bundles.schema_bundle.generated_schema_bundle_id
    );
    assert_eq!(
        bundles
            .self_hosting_build_receipt
            .generated_migration_plan_id,
        bundles.migration_plan.generated_migration_plan_id
    );
}

#[test]
fn governance_baseline_keeps_veto_and_challenge_rollback_visible() {
    let fixture = load_governance_fixture("human-veto-rollback.json");
    let bundles = generate_companion_bundles(
        &fixture.spec_bundle,
        &fixture.normative_ast,
        &fixture.proof_obligation_set,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );
    let mut veto = fixture.human_veto_bundle;
    veto.generated_schema_bundle_id = bundles.schema_bundle.generated_schema_bundle_id.clone();
    let mut challenge = fixture.meta_challenge_bundle;
    challenge.generated_schema_bundle_id = bundles.schema_bundle.generated_schema_bundle_id.clone();

    let receipt = establish_veto_challenge_baseline(
        &fixture.spec_bundle,
        &bundles.schema_bundle,
        &bundles.interpreter_bundle,
        &bundles.conformance_corpus,
        &bundles.migration_plan,
        &bundles.proof_evaluation_receipt,
        Some(&veto),
        Some(&challenge),
        "2026-03-14T00:00:00Z",
    );

    assert!(receipt.rollback_required);
    assert!(receipt.human_veto_bundle_id.is_some());
    assert!(receipt.meta_challenge_bundle_id.is_some());
}
