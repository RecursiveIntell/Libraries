use verification_adjudication::{PromotionDecision, RefutationDecision, RollbackPlan};

fn load_example<T: serde::de::DeserializeOwned>(stem: &str) -> T {
    let path = format!("{}/../examples/{stem}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn decision_examples_roundtrip_and_validate() {
    let promotion: PromotionDecision = load_example("promotion-decision-v1.example.json");
    let refutation: RefutationDecision = load_example("refutation-decision-v1.example.json");
    let rollback: RollbackPlan = load_example("rollback-plan-v1.example.json");

    promotion
        .validate()
        .expect("promotion example should validate");
    refutation
        .validate()
        .expect("refutation example should validate");
    rollback
        .validate()
        .expect("rollback example should validate");

    let _: PromotionDecision =
        serde_json::from_str(&serde_json::to_string(&promotion).unwrap()).unwrap();
    let _: RefutationDecision =
        serde_json::from_str(&serde_json::to_string(&refutation).unwrap()).unwrap();
    let _: RollbackPlan = serde_json::from_str(&serde_json::to_string(&rollback).unwrap()).unwrap();
}
