use assurance_runtime::{HazardLibraryV1, HazardScenarioV1, MonitorCatalogV1, MitigationPlaybookV1};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/p5/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn profile_p5_fixture_bundles_parse_into_owned_types() {
    for name in [
        "hazard_monitor_happy_path",
        "hazard_playbook_activation",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("HazardLibraryV1") {
            let _: HazardLibraryV1 = serde_json::from_value(value.clone()).expect("HazardLibraryV1");
        }
        if let Some(value) = artifacts.get("HazardScenarioV1") {
            let _: HazardScenarioV1 = serde_json::from_value(value.clone()).expect("HazardScenarioV1");
        }
        if let Some(value) = artifacts.get("MonitorCatalogV1") {
            let _: MonitorCatalogV1 = serde_json::from_value(value.clone()).expect("MonitorCatalogV1");
        }
        if let Some(value) = artifacts.get("MitigationPlaybookV1") {
            let _: MitigationPlaybookV1 = serde_json::from_value(value.clone()).expect("MitigationPlaybookV1");
        }
    }
}
