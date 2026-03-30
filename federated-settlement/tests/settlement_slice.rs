use federated_settlement::{
    evaluate_settlement, CrossRuntimeEquivalenceBundleV1, RuntimeIdentitySetV1, SettlementCaseV1,
    SettlementDisposition, TreatyBundleV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!(
        "{}/../contracts/fixtures/v16/{name}",
        env!("CARGO_MANIFEST_DIR")
    );
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn happy_path_fixture_proves_lawful_settlement_slice() {
    let fixture = load_bundle("happy-path.json");
    let _treaty: TreatyBundleV1 = serde_json::from_value(fixture["treaty_bundle"].clone()).unwrap();
    let _identity: RuntimeIdentitySetV1 =
        serde_json::from_value(fixture["runtime_identity_set"].clone()).unwrap();
    let _equivalence: CrossRuntimeEquivalenceBundleV1 =
        serde_json::from_value(fixture["equivalence_bundle"].clone()).unwrap();
    let case: SettlementCaseV1 =
        serde_json::from_value(fixture["settlement_case"].clone()).unwrap();

    let receipt = evaluate_settlement(&case, "2026-03-14T00:00:00Z");
    assert_eq!(
        receipt.disposition,
        SettlementDisposition::SharedDispositionIssued
    );
    assert!(receipt.settlement_allowed);
    assert!(!receipt.degraded);
    assert_eq!(receipt.local_dissent.len(), 1);
}

#[test]
fn degraded_fixture_forces_explicit_shared_view_downgrade() {
    let fixture = load_bundle("degraded-path.json");
    let case: SettlementCaseV1 =
        serde_json::from_value(fixture["settlement_case"].clone()).unwrap();

    let receipt = evaluate_settlement(&case, "2026-03-14T00:00:00Z");
    assert_eq!(
        receipt.disposition,
        SettlementDisposition::DegradedSharedView
    );
    assert!(receipt.degraded);
    assert!(!receipt.settlement_allowed);
    assert!(receipt.downgrade.is_some());
}
