use assurance_runtime::{
    AssuranceCaseV1, CertificationBundleV1, ControlMappingV1, DeploymentProfileV1,
    FieldMonitoringPlanV1, HazardRegisterV1, OperatingEnvelopeV1, RecertificationTriggerV1,
    ReleaseReadinessDecisionV1, ResidualRiskAcceptanceV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/v23/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn v23_fixture_bundles_parse_into_owned_types() {
    for name in [
        "release_happy_path",
        "blocked_release_due_to_gap",
        "residual_risk_accepted",
        "certification_advisory_only",
        "recertification_triggered",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("DeploymentProfileV1") {
            let artifact: DeploymentProfileV1 =
                serde_json::from_value(value.clone()).expect("DeploymentProfileV1");
            artifact.validate().expect("DeploymentProfileV1 valid");
        }
        if let Some(value) = artifacts.get("OperatingEnvelopeV1") {
            let artifact: OperatingEnvelopeV1 =
                serde_json::from_value(value.clone()).expect("OperatingEnvelopeV1");
            artifact.validate().expect("OperatingEnvelopeV1 valid");
        }
        if let Some(value) = artifacts.get("AssuranceCaseV1") {
            let artifact: AssuranceCaseV1 =
                serde_json::from_value(value.clone()).expect("AssuranceCaseV1");
            artifact.validate().expect("AssuranceCaseV1 valid");
        }
        if let Some(value) = artifacts.get("HazardRegisterV1") {
            let artifact: HazardRegisterV1 =
                serde_json::from_value(value.clone()).expect("HazardRegisterV1");
            artifact.validate().expect("HazardRegisterV1 valid");
        }
        if let Some(value) = artifacts.get("ControlMappingV1") {
            let artifact: ControlMappingV1 =
                serde_json::from_value(value.clone()).expect("ControlMappingV1");
            artifact.validate().expect("ControlMappingV1 valid");
        }
        if let Some(value) = artifacts.get("ResidualRiskAcceptanceV1") {
            let artifact: ResidualRiskAcceptanceV1 =
                serde_json::from_value(value.clone()).expect("ResidualRiskAcceptanceV1");
            artifact.validate().expect("ResidualRiskAcceptanceV1 valid");
        }
        if let Some(value) = artifacts.get("ReleaseReadinessDecisionV1") {
            let artifact: ReleaseReadinessDecisionV1 =
                serde_json::from_value(value.clone()).expect("ReleaseReadinessDecisionV1");
            artifact
                .validate()
                .expect("ReleaseReadinessDecisionV1 valid");
        }
        if let Some(value) = artifacts.get("FieldMonitoringPlanV1") {
            let artifact: FieldMonitoringPlanV1 =
                serde_json::from_value(value.clone()).expect("FieldMonitoringPlanV1");
            artifact.validate().expect("FieldMonitoringPlanV1 valid");
        }
        if let Some(value) = artifacts.get("CertificationBundleV1") {
            let artifact: CertificationBundleV1 =
                serde_json::from_value(value.clone()).expect("CertificationBundleV1");
            artifact.validate().expect("CertificationBundleV1 valid");
        }
        if let Some(value) = artifacts.get("RecertificationTriggerV1") {
            let artifact: RecertificationTriggerV1 =
                serde_json::from_value(value.clone()).expect("RecertificationTriggerV1");
            artifact.validate().expect("RecertificationTriggerV1 valid");
        }
    }
}
