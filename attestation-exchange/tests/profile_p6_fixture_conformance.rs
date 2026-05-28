#![allow(clippy::expect_used)]

use attestation_exchange::{
    VendorCertificationAdapterV1, VendorEvidenceTranslationV1, VendorRevocationHandlingV1,
    VendorTrustRootBindingV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p6/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p6_fixture_bundles_parse_into_owned_types() {
    for name in ["vendor_adapter_happy_path", "vendor_revocation_downgrade"] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("VendorCertificationAdapterV1") {
            let artifact: VendorCertificationAdapterV1 =
                serde_json::from_value(value.clone()).expect("VendorCertificationAdapterV1");
            artifact
                .validate()
                .expect("VendorCertificationAdapterV1 valid");
        }
        if let Some(value) = artifacts.get("VendorEvidenceTranslationV1") {
            let artifact: VendorEvidenceTranslationV1 =
                serde_json::from_value(value.clone()).expect("VendorEvidenceTranslationV1");
            artifact
                .validate()
                .expect("VendorEvidenceTranslationV1 valid");
        }
        if let Some(value) = artifacts.get("VendorTrustRootBindingV1") {
            let artifact: VendorTrustRootBindingV1 =
                serde_json::from_value(value.clone()).expect("VendorTrustRootBindingV1");
            artifact.validate().expect("VendorTrustRootBindingV1 valid");
        }
        if let Some(value) = artifacts.get("VendorRevocationHandlingV1") {
            let artifact: VendorRevocationHandlingV1 =
                serde_json::from_value(value.clone()).expect("VendorRevocationHandlingV1");
            artifact
                .validate()
                .expect("VendorRevocationHandlingV1 valid");
        }
    }
}
