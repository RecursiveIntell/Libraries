use profile_runtime::{
    ApplicabilityContextV1, CompiledObligationSetV1, CompositionConflictSetV1,
    CompositionReceiptV1, EffectiveConstitutionV1, PolicyImpactDiffV1, ProfileExceptionBundleV1,
    ProfileSetV1,
};
use serde_json::Value;

fn load_bundle(name: &str) -> Value {
    let path = format!("../contracts/fixtures/v25/{name}.bundle.json");
    let body = std::fs::read_to_string(path).expect("read fixture bundle");
    serde_json::from_str(&body).expect("parse fixture bundle")
}

#[test]
fn v25_fixture_bundles_parse_into_owned_types() {
    for name in [
        "blocked_locality_without_exception",
        "locality_exception_admitted",
        "disclosure_conflict",
        "policy_impact_diff",
        "delegation_break_glass_depth",
        "release_readiness_blocked",
        "continuity_incident_mode_diff",
        "vendor_translation_caveat",
    ] {
        let bundle = load_bundle(name);
        let artifacts = bundle["artifacts"].as_object().expect("artifacts object");

        if let Some(value) = artifacts.get("ApplicabilityContextV1") {
            let _: ApplicabilityContextV1 =
                serde_json::from_value(value.clone()).expect("ApplicabilityContextV1");
        }
        if let Some(value) = artifacts.get("ProfileSetV1") {
            let _: ProfileSetV1 = serde_json::from_value(value.clone()).expect("ProfileSetV1");
        }
        if let Some(value) = artifacts.get("ProfileExceptionBundleV1") {
            let _: ProfileExceptionBundleV1 =
                serde_json::from_value(value.clone()).expect("ProfileExceptionBundleV1");
        }
        if let Some(value) = artifacts.get("EffectiveConstitutionV1") {
            let _: EffectiveConstitutionV1 =
                serde_json::from_value(value.clone()).expect("EffectiveConstitutionV1");
        }
        if let Some(value) = artifacts.get("EffectiveConstitutionFromV1") {
            let _: EffectiveConstitutionV1 =
                serde_json::from_value(value.clone()).expect("EffectiveConstitutionFromV1");
        }
        if let Some(value) = artifacts.get("EffectiveConstitutionToV1") {
            let _: EffectiveConstitutionV1 =
                serde_json::from_value(value.clone()).expect("EffectiveConstitutionToV1");
        }
        if let Some(value) = artifacts.get("CompiledObligationSetV1") {
            let _: CompiledObligationSetV1 =
                serde_json::from_value(value.clone()).expect("CompiledObligationSetV1");
        }
        if let Some(value) = artifacts.get("CompiledObligationSetFromV1") {
            let _: CompiledObligationSetV1 =
                serde_json::from_value(value.clone()).expect("CompiledObligationSetFromV1");
        }
        if let Some(value) = artifacts.get("CompiledObligationSetToV1") {
            let _: CompiledObligationSetV1 =
                serde_json::from_value(value.clone()).expect("CompiledObligationSetToV1");
        }
        if let Some(value) = artifacts.get("CompositionConflictSetV1") {
            let _: CompositionConflictSetV1 =
                serde_json::from_value(value.clone()).expect("CompositionConflictSetV1");
        }
        if let Some(value) = artifacts.get("CompositionReceiptV1") {
            let _: CompositionReceiptV1 =
                serde_json::from_value(value.clone()).expect("CompositionReceiptV1");
        }
        if let Some(value) = artifacts.get("PolicyImpactDiffV1") {
            let _: PolicyImpactDiffV1 =
                serde_json::from_value(value.clone()).expect("PolicyImpactDiffV1");
        }
    }
}
