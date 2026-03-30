use serde_json::Value;
use verification_policy::{
    CrossBoundaryTransferClassV1, LocalityExceptionV1, ResidencyPolicyProfileV1,
    TenantBoundaryProfileV1,
};

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p2/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p2_fixture_bundles_parse_into_owned_types() {
    for name in [
        "locality_exception_happy_path",
        "residency_transfer_blocked",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("ResidencyPolicyProfileV1") {
            let _: ResidencyPolicyProfileV1 =
                serde_json::from_value(value.clone()).expect("ResidencyPolicyProfileV1");
        }
        if let Some(value) = artifacts.get("TenantBoundaryProfileV1") {
            let _: TenantBoundaryProfileV1 =
                serde_json::from_value(value.clone()).expect("TenantBoundaryProfileV1");
        }
        if let Some(value) = artifacts.get("CrossBoundaryTransferClassV1") {
            let _: CrossBoundaryTransferClassV1 =
                serde_json::from_value(value.clone()).expect("CrossBoundaryTransferClassV1");
        }
        if let Some(value) = artifacts.get("LocalityExceptionV1") {
            let _: LocalityExceptionV1 =
                serde_json::from_value(value.clone()).expect("LocalityExceptionV1");
        }
    }
}
