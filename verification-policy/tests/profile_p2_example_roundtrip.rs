use verification_policy::{
    CrossBoundaryTransferClassV1, LocalityExceptionV1, ResidencyPolicyProfileV1,
    TenantBoundaryProfileV1,
};

#[test]
fn residency_policy_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/residency-policy-profile-v1.example.json")
        .expect("read example");
    let value: ResidencyPolicyProfileV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ResidencyPolicyProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn tenant_boundary_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/tenant-boundary-profile-v1.example.json")
        .expect("read example");
    let value: TenantBoundaryProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: TenantBoundaryProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn cross_boundary_transfer_class_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/cross-boundary-transfer-class-v1.example.json")
        .expect("read example");
    let value: CrossBoundaryTransferClassV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: CrossBoundaryTransferClassV1 =
        serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn locality_exception_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/locality-exception-v1.example.json")
        .expect("read example");
    let value: LocalityExceptionV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: LocalityExceptionV1 = serde_json::from_str(&encoded).expect("deserialize");
}
