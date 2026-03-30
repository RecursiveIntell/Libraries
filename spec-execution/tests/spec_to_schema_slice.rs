use serde::Deserialize;
use spec_execution::{generate_schema_bundle, NormativeASTV1, ProofObligationSetV1, SpecBundleV1};

#[derive(Debug, Deserialize)]
struct SpecFixture {
    spec_bundle: SpecBundleV1,
    normative_ast: NormativeASTV1,
    proof_obligation_set: ProofObligationSetV1,
}

fn load_fixture(name: &str) -> SpecFixture {
    let path = format!(
        "{}/../contracts/fixtures/v20/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn generated_schema_bundle_keeps_backpointers_and_receipt() {
    let fixture = load_fixture("happy-path.json");
    let (bundle, receipt) = generate_schema_bundle(
        &fixture.spec_bundle,
        &fixture.normative_ast,
        &fixture.proof_obligation_set,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );

    assert_eq!(bundle.spec_bundle_id, fixture.spec_bundle.spec_bundle_id);
    assert_eq!(
        bundle.normative_ast_id,
        fixture.normative_ast.normative_ast_id
    );
    assert_eq!(
        receipt.proof_obligation_set_id,
        fixture.proof_obligation_set.proof_obligation_set_id
    );
    assert!(receipt.generated_schema_bundle_id.is_some());
}

#[test]
fn unsatisfied_blocking_obligations_remain_visible() {
    let fixture = load_fixture("degraded-path.json");
    let (_, receipt) = generate_schema_bundle(
        &fixture.spec_bundle,
        &fixture.normative_ast,
        &fixture.proof_obligation_set,
        "demo-family",
        "2026-03-14T00:00:00Z",
    );

    assert!(!receipt.admission_allowed);
    assert!(!receipt.unsatisfied_blocking_obligations.is_empty());
}
