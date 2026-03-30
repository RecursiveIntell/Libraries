use verification_adjudication::{RollbackDecisionV1, RolloutDecisionV1};

fn load_example<T: serde::de::DeserializeOwned>(stem: &str) -> T {
    let path = format!("{}/../examples/{stem}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn v14_rollout_examples_roundtrip_and_validate() {
    let rollout: RolloutDecisionV1 = load_example("RolloutDecisionV1.example.json");
    let rollback: RollbackDecisionV1 = load_example("RollbackDecisionV1.example.json");

    rollout.validate().expect("rollout example should validate");
    rollback
        .validate()
        .expect("rollback example should validate");

    let _: RolloutDecisionV1 =
        serde_json::from_str(&serde_json::to_string(&rollout).unwrap()).unwrap();
    let _: RollbackDecisionV1 =
        serde_json::from_str(&serde_json::to_string(&rollback).unwrap()).unwrap();
}
