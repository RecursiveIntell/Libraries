use continuity_runtime::{IncidentTaxonomyV1, SeverityMatrixV1, PagerRouteProfileV1, EscalationClockPolicyV1};

#[test]
fn incident_taxonomy_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/incident-taxonomy-v1.example.json").expect("read example");
    let value: IncidentTaxonomyV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: IncidentTaxonomyV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn severity_matrix_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/severity-matrix-v1.example.json").expect("read example");
    let value: SeverityMatrixV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: SeverityMatrixV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn pager_route_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/pager-route-profile-v1.example.json").expect("read example");
    let value: PagerRouteProfileV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: PagerRouteProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn escalation_clock_policy_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/escalation-clock-policy-v1.example.json").expect("read example");
    let value: EscalationClockPolicyV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: EscalationClockPolicyV1 = serde_json::from_str(&encoded).expect("deserialize");
}
