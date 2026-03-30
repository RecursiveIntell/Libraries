use verification_policy::{
    ContinuityPolicyProfileV1, DelegationPolicyProfileV1, EffectPolicyProfileV1,
    ReleasePolicyProfileV1,
};

#[test]
fn effect_policy_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/effect-policy-profile-v1.example.json")
        .expect("read example");
    let value: EffectPolicyProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: EffectPolicyProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn delegation_policy_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/delegation-policy-profile-v1.example.json")
        .expect("read example");
    let value: DelegationPolicyProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: DelegationPolicyProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn release_policy_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/release-policy-profile-v1.example.json")
        .expect("read example");
    let value: ReleasePolicyProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: ReleasePolicyProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}

#[test]
fn continuity_policy_profile_v1_roundtrips() {
    let body = std::fs::read_to_string("../examples/continuity-policy-profile-v1.example.json")
        .expect("read example");
    let value: ContinuityPolicyProfileV1 = serde_json::from_str(&body).expect("parse example");
    value.validate().expect("validate example");
    let encoded = serde_json::to_string(&value).expect("serialize");
    let roundtrip: ContinuityPolicyProfileV1 = serde_json::from_str(&encoded).expect("deserialize");
    roundtrip.validate().expect("validate roundtrip");
}
