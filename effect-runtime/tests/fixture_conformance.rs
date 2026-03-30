use effect_runtime::{
    CompensationExecutionReceiptV1, CompensationPlanV1, EffectCommitDecisionV1,
    EffectExecutionReceiptV1, EffectIntentV1, EffectObservationBundleV1, EffectPreflightReportV1,
    EffectWindowV1, ExternalEffectLedgerEntryV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/v21/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn v21_fixture_bundles_parse_into_owned_types() {
    for name in [
        "effect_happy_path",
        "effect_blocked_preflight",
        "effect_partial_with_compensation",
        "effect_advisory_only",
        "effect_replay_ready",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("EffectWindowV1") {
            let artifact: EffectWindowV1 =
                serde_json::from_value(value.clone()).expect("EffectWindowV1");
            artifact.validate().expect("EffectWindowV1 valid");
        }
        if let Some(value) = artifacts.get("EffectIntentV1") {
            let artifact: EffectIntentV1 =
                serde_json::from_value(value.clone()).expect("EffectIntentV1");
            artifact.validate().expect("EffectIntentV1 valid");
        }
        if let Some(value) = artifacts.get("EffectPreflightReportV1") {
            let artifact: EffectPreflightReportV1 =
                serde_json::from_value(value.clone()).expect("EffectPreflightReportV1");
            artifact.validate().expect("EffectPreflightReportV1 valid");
        }
        if let Some(value) = artifacts.get("EffectCommitDecisionV1") {
            let artifact: EffectCommitDecisionV1 =
                serde_json::from_value(value.clone()).expect("EffectCommitDecisionV1");
            artifact.validate().expect("EffectCommitDecisionV1 valid");
        }
        if let Some(value) = artifacts.get("EffectExecutionReceiptV1") {
            let artifact: EffectExecutionReceiptV1 =
                serde_json::from_value(value.clone()).expect("EffectExecutionReceiptV1");
            artifact.validate().expect("EffectExecutionReceiptV1 valid");
        }
        if let Some(value) = artifacts.get("EffectObservationBundleV1") {
            let artifact: EffectObservationBundleV1 =
                serde_json::from_value(value.clone()).expect("EffectObservationBundleV1");
            artifact
                .validate()
                .expect("EffectObservationBundleV1 valid");
        }
        if let Some(value) = artifacts.get("CompensationPlanV1") {
            let artifact: CompensationPlanV1 =
                serde_json::from_value(value.clone()).expect("CompensationPlanV1");
            artifact.validate().expect("CompensationPlanV1 valid");
        }
        if let Some(value) = artifacts.get("CompensationExecutionReceiptV1") {
            let artifact: CompensationExecutionReceiptV1 =
                serde_json::from_value(value.clone()).expect("CompensationExecutionReceiptV1");
            artifact
                .validate()
                .expect("CompensationExecutionReceiptV1 valid");
        }
        if let Some(value) = artifacts.get("ExternalEffectLedgerEntryV1") {
            let artifact: ExternalEffectLedgerEntryV1 =
                serde_json::from_value(value.clone()).expect("ExternalEffectLedgerEntryV1");
            artifact
                .validate()
                .expect("ExternalEffectLedgerEntryV1 valid");
        }
    }
}
