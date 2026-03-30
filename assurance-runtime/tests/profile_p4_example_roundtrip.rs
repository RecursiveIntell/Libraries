use assurance_runtime::{
    EvidenceCollectionPlanV1, RecertificationScheduleV1, RegulatoryRegimeProfileV1,
    RequirementControlMapV1,
};

#[test]
fn regulatory_regime_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/regulatory-regime-profile-v1.example.json")
        .expect("read example");
    let value: RegulatoryRegimeProfileV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: RegulatoryRegimeProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn requirement_control_map_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/requirement-control-map-v1.example.json")
        .expect("read example");
    let value: RequirementControlMapV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: RequirementControlMapV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn evidence_collection_plan_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/evidence-collection-plan-v1.example.json")
        .expect("read example");
    let value: EvidenceCollectionPlanV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: EvidenceCollectionPlanV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn recertification_schedule_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/recertification-schedule-v1.example.json")
        .expect("read example");
    let value: RecertificationScheduleV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: RecertificationScheduleV1 = serde_json::from_str(&encoded).expect("deserialize");
}
