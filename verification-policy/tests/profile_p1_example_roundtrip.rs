#![allow(clippy::expect_used)]

use verification_policy::{
    AccessPurposeMatrixV1, AuditExtractionPolicyV1, PrivacyRetentionProfileV1, RedactionRuleSetV1,
};

#[test]
fn privacy_retention_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/privacy-retention-profile-v1.example.json")
        .expect("read example");
    let value: PrivacyRetentionProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: PrivacyRetentionProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn redaction_rule_set_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/redaction-rule-set-v1.example.json")
        .expect("read example");
    let value: RedactionRuleSetV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: RedactionRuleSetV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn access_purpose_matrix_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/access-purpose-matrix-v1.example.json")
        .expect("read example");
    let value: AccessPurposeMatrixV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: AccessPurposeMatrixV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn audit_extraction_policy_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/audit-extraction-policy-v1.example.json")
        .expect("read example");
    let value: AuditExtractionPolicyV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: AuditExtractionPolicyV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}
