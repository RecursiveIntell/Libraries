use continuity_runtime::{
    EscalationClockPolicyV1, IncidentTaxonomyV1, PagerRouteProfileV1, SeverityMatrixV1,
};
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
            let artifact: IncidentTaxonomyV1 =
                serde_json::from_value(value.clone()).expect("IncidentTaxonomyV1");
            artifact.validate().expect("IncidentTaxonomyV1 valid");
        }
        if let Some(value) = artifacts.get("SeverityMatrixV1") {
            let artifact: SeverityMatrixV1 =
                serde_json::from_value(value.clone()).expect("SeverityMatrixV1");
            artifact.validate().expect("SeverityMatrixV1 valid");
        }
        if let Some(value) = artifacts.get("PagerRouteProfileV1") {
            let artifact: PagerRouteProfileV1 =
                serde_json::from_value(value.clone()).expect("PagerRouteProfileV1");
            artifact.validate().expect("PagerRouteProfileV1 valid");
        }
        if let Some(value) = artifacts.get("EscalationClockPolicyV1") {
            let artifact: EscalationClockPolicyV1 =
                serde_json::from_value(value.clone()).expect("EscalationClockPolicyV1");
            artifact.validate().expect("EscalationClockPolicyV1 valid");
        }
    }
}
