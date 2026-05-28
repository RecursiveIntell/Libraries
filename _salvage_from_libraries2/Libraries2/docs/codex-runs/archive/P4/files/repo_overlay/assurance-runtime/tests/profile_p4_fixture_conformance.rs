use assurance_runtime::{RegulatoryRegimeProfileV1, RequirementControlMapV1, EvidenceCollectionPlanV1, RecertificationScheduleV1};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p4/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p4_fixture_bundles_parse_into_owned_types() {
    for name in [
        "recertification_overdue_blocked",
        "regulated_release_happy_path",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("RegulatoryRegimeProfileV1") {
            let _: RegulatoryRegimeProfileV1 = serde_json::from_value(value.clone()).expect("RegulatoryRegimeProfileV1");
        }
        if let Some(value) = artifacts.get("RequirementControlMapV1") {
            let _: RequirementControlMapV1 = serde_json::from_value(value.clone()).expect("RequirementControlMapV1");
        }
        if let Some(value) = artifacts.get("EvidenceCollectionPlanV1") {
            let _: EvidenceCollectionPlanV1 = serde_json::from_value(value.clone()).expect("EvidenceCollectionPlanV1");
        }
        if let Some(value) = artifacts.get("RecertificationScheduleV1") {
            let _: RecertificationScheduleV1 = serde_json::from_value(value.clone()).expect("RecertificationScheduleV1");
        }
    }
}
