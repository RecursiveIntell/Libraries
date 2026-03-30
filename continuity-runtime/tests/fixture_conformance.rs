use continuity_runtime::{
    ContainmentDecisionV1, ContinuityExceptionV1, ErrorBudgetLedgerV1, ForensicFreezeV1,
    IncidentCaseV1, PostmortemBundleV1, RecoveryPlanV1, RecoveryReplaySliceV1,
    ResilienceExerciseV1, ServiceLevelProfileV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/v24/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn v24_fixture_bundles_parse_into_owned_types() {
    for name in [
        "incident_happy_path",
        "forensic_freeze_path",
        "continuity_exception",
        "error_budget_exhausted",
        "resilience_exercise",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("ServiceLevelProfileV1") {
            let artifact: ServiceLevelProfileV1 =
                serde_json::from_value(value.clone()).expect("ServiceLevelProfileV1");
            artifact.validate().expect("ServiceLevelProfileV1 valid");
        }
        if let Some(value) = artifacts.get("ErrorBudgetLedgerV1") {
            let artifact: ErrorBudgetLedgerV1 =
                serde_json::from_value(value.clone()).expect("ErrorBudgetLedgerV1");
            artifact.validate().expect("ErrorBudgetLedgerV1 valid");
        }
        if let Some(value) = artifacts.get("IncidentCaseV1") {
            let artifact: IncidentCaseV1 =
                serde_json::from_value(value.clone()).expect("IncidentCaseV1");
            artifact.validate().expect("IncidentCaseV1 valid");
        }
        if let Some(value) = artifacts.get("ContainmentDecisionV1") {
            let artifact: ContainmentDecisionV1 =
                serde_json::from_value(value.clone()).expect("ContainmentDecisionV1");
            artifact.validate().expect("ContainmentDecisionV1 valid");
        }
        if let Some(value) = artifacts.get("ForensicFreezeV1") {
            let artifact: ForensicFreezeV1 =
                serde_json::from_value(value.clone()).expect("ForensicFreezeV1");
            artifact.validate().expect("ForensicFreezeV1 valid");
        }
        if let Some(value) = artifacts.get("RecoveryPlanV1") {
            let artifact: RecoveryPlanV1 =
                serde_json::from_value(value.clone()).expect("RecoveryPlanV1");
            artifact.validate().expect("RecoveryPlanV1 valid");
        }
        if let Some(value) = artifacts.get("RecoveryReplaySliceV1") {
            let artifact: RecoveryReplaySliceV1 =
                serde_json::from_value(value.clone()).expect("RecoveryReplaySliceV1");
            artifact.validate().expect("RecoveryReplaySliceV1 valid");
        }
        if let Some(value) = artifacts.get("ContinuityExceptionV1") {
            let artifact: ContinuityExceptionV1 =
                serde_json::from_value(value.clone()).expect("ContinuityExceptionV1");
            artifact.validate().expect("ContinuityExceptionV1 valid");
        }
        if let Some(value) = artifacts.get("PostmortemBundleV1") {
            let artifact: PostmortemBundleV1 =
                serde_json::from_value(value.clone()).expect("PostmortemBundleV1");
            artifact.validate().expect("PostmortemBundleV1 valid");
        }
        if let Some(value) = artifacts.get("ResilienceExerciseV1") {
            let artifact: ResilienceExerciseV1 =
                serde_json::from_value(value.clone()).expect("ResilienceExerciseV1");
            artifact.validate().expect("ResilienceExerciseV1 valid");
        }
    }
}
