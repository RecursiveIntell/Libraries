use authority_delegation::{
    ApprovalMatrixV1, ConflictClassCatalogV1, DelegationMatrixV1, RoleCatalogV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p3/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p3_fixture_bundles_parse_into_owned_types() {
    for name in ["conflict_recusal_blocked", "delegation_matrix_happy_path"] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("RoleCatalogV1") {
            let _: RoleCatalogV1 = serde_json::from_value(value.clone()).expect("RoleCatalogV1");
        }
        if let Some(value) = artifacts.get("DelegationMatrixV1") {
            let _: DelegationMatrixV1 =
                serde_json::from_value(value.clone()).expect("DelegationMatrixV1");
        }
        if let Some(value) = artifacts.get("ApprovalMatrixV1") {
            let _: ApprovalMatrixV1 =
                serde_json::from_value(value.clone()).expect("ApprovalMatrixV1");
        }
        if let Some(value) = artifacts.get("ConflictClassCatalogV1") {
            let _: ConflictClassCatalogV1 =
                serde_json::from_value(value.clone()).expect("ConflictClassCatalogV1");
        }
    }
}
