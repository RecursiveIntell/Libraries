use profile_runtime::{
    ApplicabilityContextV1, CompiledObligationSetV1, CompositionConflictSetV1,
    CompositionReceiptV1, CompositionRuleSetV1, EffectiveConstitutionV1, PolicyImpactDiffV1,
    ProfileExceptionBundleV1, ProfileSetV1,
};

#[test]
fn applicability_context_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/applicability-context-v1.example.json")
        .expect("read example");
    let value: ApplicabilityContextV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ApplicabilityContextV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn profile_set_v1_roundtrips() {
    let body =
        std::fs::read_to_string("../examples/profile-set-v1.example.json").expect("read example");
    let value: ProfileSetV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ProfileSetV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn composition_rule_set_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/composition-rule-set-v1.example.json")
        .expect("read example");
    let value: CompositionRuleSetV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: CompositionRuleSetV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn composition_receipt_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/composition-receipt-v1.example.json")
        .expect("read example");
    let value: CompositionReceiptV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: CompositionReceiptV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn effective_constitution_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/effective-constitution-v1.example.json")
        .expect("read example");
    let value: EffectiveConstitutionV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: EffectiveConstitutionV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn compiled_obligation_set_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/compiled-obligation-set-v1.example.json")
        .expect("read example");
    let value: CompiledObligationSetV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: CompiledObligationSetV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn composition_conflict_set_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/composition-conflict-set-v1.example.json")
        .expect("read example");
    let value: CompositionConflictSetV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: CompositionConflictSetV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn profile_exception_bundle_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/profile-exception-bundle-v1.example.json")
        .expect("read example");
    let value: ProfileExceptionBundleV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: ProfileExceptionBundleV1 = serde_json::from_str(&encoded).expect("deserialize");
}

#[test]
fn policy_impact_diff_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/policy-impact-diff-v1.example.json")
        .expect("read example");
    let value: PolicyImpactDiffV1 = serde_json::from_str(&body).expect("parse example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let _: PolicyImpactDiffV1 = serde_json::from_str(&encoded).expect("deserialize");
}
