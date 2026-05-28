#![allow(clippy::expect_used)]

use verification_control::{
    ComparabilityMatrixV1, DecisionTraceV1, DisputeBundleV1, ExperimentCaseV1, RefuterResultV1,
};

fn load_example<T: serde::de::DeserializeOwned>(stem: &str) -> T {
    let path = format!("{}/../examples/{stem}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn v14_examples_roundtrip_and_validate() {
    let experiment: ExperimentCaseV1 = load_example("ExperimentCaseV1.example.json");
    let comparability: ComparabilityMatrixV1 = load_example("ComparabilityMatrixV1.example.json");
    let refuter: RefuterResultV1 = load_example("RefuterResultV1.example.json");
    let trace: DecisionTraceV1 = load_example("DecisionTraceV1.example.json");
    let dispute: DisputeBundleV1 = load_example("DisputeBundleV1.example.json");

    experiment
        .validate()
        .expect("experiment example should validate");
    comparability
        .validate()
        .expect("comparability example should validate");
    refuter.validate().expect("refuter example should validate");
    trace
        .validate()
        .expect("decision trace example should validate");
    dispute.validate().expect("dispute example should validate");

    let _: ExperimentCaseV1 =
        serde_json::from_str(&serde_json::to_string(&experiment).unwrap()).unwrap();
    let _: ComparabilityMatrixV1 =
        serde_json::from_str(&serde_json::to_string(&comparability).unwrap()).unwrap();
    let _: RefuterResultV1 =
        serde_json::from_str(&serde_json::to_string(&refuter).unwrap()).unwrap();
    let _: DecisionTraceV1 = serde_json::from_str(&serde_json::to_string(&trace).unwrap()).unwrap();
    let _: DisputeBundleV1 =
        serde_json::from_str(&serde_json::to_string(&dispute).unwrap()).unwrap();
}
