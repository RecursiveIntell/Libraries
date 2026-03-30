use authority_delegation::{
    ActingOnBehalfReceiptV1, AuthorityChainV1, AuthorityLeaseV1, BreakGlassGrantV1,
    CapabilityClassV1, ConflictDisclosureV1, DelegationBundleV1, DelegationRevocationV1,
    DualControlApprovalV1, SeparationOfDutiesPolicyV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/v22/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn v22_fixture_bundles_parse_into_owned_types() {
    for name in [
        "delegated_effect_happy_path",
        "delegation_revoked",
        "conflict_disclosed_and_mitigated",
        "break_glass_path",
        "out_of_scope_delegation_failure",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("CapabilityClassV1") {
            let artifact: CapabilityClassV1 =
                serde_json::from_value(value.clone()).expect("CapabilityClassV1");
            artifact.validate().expect("CapabilityClassV1 valid");
        }
        if let Some(value) = artifacts.get("AuthorityLeaseV1") {
            let artifact: AuthorityLeaseV1 =
                serde_json::from_value(value.clone()).expect("AuthorityLeaseV1");
            artifact.validate().expect("AuthorityLeaseV1 valid");
        }
        if let Some(value) = artifacts.get("DelegationBundleV1") {
            let artifact: DelegationBundleV1 =
                serde_json::from_value(value.clone()).expect("DelegationBundleV1");
            artifact.validate().expect("DelegationBundleV1 valid");
        }
        if let Some(value) = artifacts.get("AuthorityChainV1") {
            let artifact: AuthorityChainV1 =
                serde_json::from_value(value.clone()).expect("AuthorityChainV1");
            artifact.validate().expect("AuthorityChainV1 valid");
        }
        if let Some(value) = artifacts.get("SeparationOfDutiesPolicyV1") {
            let artifact: SeparationOfDutiesPolicyV1 =
                serde_json::from_value(value.clone()).expect("SeparationOfDutiesPolicyV1");
            artifact
                .validate()
                .expect("SeparationOfDutiesPolicyV1 valid");
        }
        if let Some(value) = artifacts.get("DualControlApprovalV1") {
            let artifact: DualControlApprovalV1 =
                serde_json::from_value(value.clone()).expect("DualControlApprovalV1");
            artifact.validate().expect("DualControlApprovalV1 valid");
        }
        if let Some(value) = artifacts.get("BreakGlassGrantV1") {
            let artifact: BreakGlassGrantV1 =
                serde_json::from_value(value.clone()).expect("BreakGlassGrantV1");
            artifact.validate().expect("BreakGlassGrantV1 valid");
        }
        if let Some(value) = artifacts.get("DelegationRevocationV1") {
            let artifact: DelegationRevocationV1 =
                serde_json::from_value(value.clone()).expect("DelegationRevocationV1");
            artifact.validate().expect("DelegationRevocationV1 valid");
        }
        if let Some(value) = artifacts.get("ActingOnBehalfReceiptV1") {
            let artifact: ActingOnBehalfReceiptV1 =
                serde_json::from_value(value.clone()).expect("ActingOnBehalfReceiptV1");
            artifact.validate().expect("ActingOnBehalfReceiptV1 valid");
        }
        if let Some(value) = artifacts.get("ConflictDisclosureV1") {
            let artifact: ConflictDisclosureV1 =
                serde_json::from_value(value.clone()).expect("ConflictDisclosureV1");
            artifact.validate().expect("ConflictDisclosureV1 valid");
        }
    }
}
