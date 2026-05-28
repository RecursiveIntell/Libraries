use continuity_runtime::{IncidentTaxonomyV1, SeverityMatrixV1, PagerRouteProfileV1, EscalationClockPolicyV1};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p7/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p7_fixture_bundles_parse_into_owned_types() {
    for name in [
        "incident_taxonomy_happy_path",
        "pager_route_escalation_timeout",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("IncidentTaxonomyV1") {
            let _: IncidentTaxonomyV1 = serde_json::from_value(value.clone()).expect("IncidentTaxonomyV1");
        }
        if let Some(value) = artifacts.get("SeverityMatrixV1") {
            let _: SeverityMatrixV1 = serde_json::from_value(value.clone()).expect("SeverityMatrixV1");
        }
        if let Some(value) = artifacts.get("PagerRouteProfileV1") {
            let _: PagerRouteProfileV1 = serde_json::from_value(value.clone()).expect("PagerRouteProfileV1");
        }
        if let Some(value) = artifacts.get("EscalationClockPolicyV1") {
            let _: EscalationClockPolicyV1 = serde_json::from_value(value.clone()).expect("EscalationClockPolicyV1");
        }
    }
}
