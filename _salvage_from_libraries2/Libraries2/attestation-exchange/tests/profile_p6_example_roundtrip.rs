use attestation_exchange::{
    VendorCertificationAdapterV1, VendorEvidenceTranslationV1, VendorRevocationHandlingV1,
    VendorTrustRootBindingV1,
};

#[test]
fn vendor_certification_adapter_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/vendor-certification-adapter-v1.example.json")
        .expect("read example");
    let value: VendorCertificationAdapterV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: VendorCertificationAdapterV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn vendor_evidence_translation_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/vendor-evidence-translation-v1.example.json")
        .expect("read example");
    let value: VendorEvidenceTranslationV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: VendorEvidenceTranslationV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn vendor_trust_root_binding_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/vendor-trust-root-binding-v1.example.json")
        .expect("read example");
    let value: VendorTrustRootBindingV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: VendorTrustRootBindingV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn vendor_revocation_handling_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/vendor-revocation-handling-v1.example.json")
        .expect("read example");
    let value: VendorRevocationHandlingV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: VendorRevocationHandlingV1 = serde_json::from_str(&encoded).expect("deserialize");
}
