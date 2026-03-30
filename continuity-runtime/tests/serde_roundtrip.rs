use continuity_runtime::{IncidentSeverityV1, ServiceLevelProfileV1};

#[test]
fn serde_roundtrip_example() {
    let value = ServiceLevelProfileV1::new(
        "slp_001",
        "alpha-service",
        "99.9%",
        "p95<250ms",
        "team:reliability",
        "ebl_001",
    )
    .expect("service level profile");
    let json = serde_json::to_string(&value).expect("serialize");
    let decoded: ServiceLevelProfileV1 = serde_json::from_str(&json).expect("deserialize");
    decoded.validate().expect("profile validates");

    let severity_json = serde_json::to_string(&IncidentSeverityV1::Sev2).expect("serialize");
    assert_eq!(severity_json, "\"sev2\"");
}
