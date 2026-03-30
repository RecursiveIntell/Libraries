use authority_delegation::{
    CapabilityBlastRadiusCeilingV1, CapabilityClassV1, CapabilityDisclosureCeilingV1,
};

#[test]
fn serde_roundtrip_example() {
    let value = CapabilityClassV1::new(
        "cap_001",
        vec!["external_write".to_string(), "deploy_gate".to_string()],
        CapabilityBlastRadiusCeilingV1::SingleService,
        CapabilityDisclosureCeilingV1::Internal,
        "dual_control",
        true,
        1,
    )
    .expect("valid capability class");
    let json = serde_json::to_string(&value).expect("serialize");
    let value: CapabilityClassV1 = serde_json::from_str(&json).expect("deserialize");
    value.validate().expect("capability class validates");
}
