use assurance_runtime::{AssurancePublicationStatusV1, DeploymentProfileV1};

#[test]
fn serde_roundtrip_example() {
    let value = DeploymentProfileV1::new(
        "dpf_001",
        "alpha-service",
        "prod-us-central",
        "high",
        "oen_001",
        "asc_001",
        AssurancePublicationStatusV1::Required,
    )
    .expect("valid deployment profile");
    let json = serde_json::to_string(&value).expect("serialize");
    let value: DeploymentProfileV1 = serde_json::from_str(&json).expect("deserialize");
    value.validate().expect("deployment profile validates");
}
