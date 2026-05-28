use verification_policy::{PrivacyRetentionProfileV1, RedactionRuleSetV1, AccessPurposeMatrixV1, AuditExtractionPolicyV1};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p1/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p1_fixture_bundles_parse_into_owned_types() {
    for name in [
        "audit_extraction_escalation",
        "privacy_redaction_happy_path",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("PrivacyRetentionProfileV1") {
            let _: PrivacyRetentionProfileV1 = serde_json::from_value(value.clone()).expect("PrivacyRetentionProfileV1");
        }
        if let Some(value) = artifacts.get("RedactionRuleSetV1") {
            let _: RedactionRuleSetV1 = serde_json::from_value(value.clone()).expect("RedactionRuleSetV1");
        }
        if let Some(value) = artifacts.get("AccessPurposeMatrixV1") {
            let _: AccessPurposeMatrixV1 = serde_json::from_value(value.clone()).expect("AccessPurposeMatrixV1");
        }
        if let Some(value) = artifacts.get("AuditExtractionPolicyV1") {
            let _: AuditExtractionPolicyV1 = serde_json::from_value(value.clone()).expect("AuditExtractionPolicyV1");
        }
    }
}
