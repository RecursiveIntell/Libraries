use continuity_runtime::{
    EscalationClockPolicyV1, IncidentTaxonomyV1, PagerRouteProfileV1, SeverityMatrixV1,
};

#[test]
fn incident_taxonomy_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/incident-taxonomy-v1.example.json")
        .expect("read example");
    let value: IncidentTaxonomyV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("taxonomy validates");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let decoded: IncidentTaxonomyV1 = serde_json::from_str(&encoded).expect("deserialize");
    decoded.validate().expect("taxonomy roundtrip validates");
}

#[test]
fn severity_matrix_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/severity-matrix-v1.example.json")
        .expect("read example");
    let value: SeverityMatrixV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("severity matrix validates");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let decoded: SeverityMatrixV1 = serde_json::from_str(&encoded).expect("deserialize");
    decoded
        .validate()
        .expect("severity matrix roundtrip validates");
}

#[test]
fn pager_route_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/pager-route-profile-v1.example.json")
        .expect("read example");
    let value: PagerRouteProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("pager route validates");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let decoded: PagerRouteProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    decoded.validate().expect("pager route roundtrip validates");
}

#[test]
fn escalation_clock_policy_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/escalation-clock-policy-v1.example.json")
        .expect("read example");
    let value: EscalationClockPolicyV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("clock policy validates");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let decoded: EscalationClockPolicyV1 = serde_json::from_str(&encoded).expect("deserialize");
    decoded
        .validate()
        .expect("clock policy roundtrip validates");
}
