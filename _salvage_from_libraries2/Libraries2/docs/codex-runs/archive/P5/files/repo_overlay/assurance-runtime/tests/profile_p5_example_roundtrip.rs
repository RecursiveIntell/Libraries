use assurance_runtime::{HazardLibraryV1, HazardScenarioV1, MonitorCatalogV1, MitigationPlaybookV1};

#[test]
fn hazard_library_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/hazard-library-v1.example.json").expect("read example");
    let value: HazardLibraryV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: HazardLibraryV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn hazard_scenario_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/hazard-scenario-v1.example.json").expect("read example");
    let value: HazardScenarioV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: HazardScenarioV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn monitor_catalog_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/monitor-catalog-v1.example.json").expect("read example");
    let value: MonitorCatalogV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: MonitorCatalogV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn mitigation_playbook_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/mitigation-playbook-v1.example.json").expect("read example");
    let value: MitigationPlaybookV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: MitigationPlaybookV1 = serde_json::from_str(&encoded).expect("deserialize");
}
