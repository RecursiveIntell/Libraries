#![allow(clippy::expect_used)]

use verification_control::{
    ConstitutionalContextStatus, ContinuityReviewCaseV1, ControlReceipt, DelegationReviewCaseV1,
    EffectBlockReceiptV1, EffectReviewCaseV1, ReleaseGateCaseV1,
};

fn load_example<T: serde::de::DeserializeOwned>(stem: &str) -> T {
    let path = format!("{}/../examples/{stem}", env!("CARGO_MANIFEST_DIR"));
    serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap()
}

#[test]
fn v25_review_case_artifacts_roundtrip_from_examples() {
    let control: ControlReceipt = load_example("control-receipt-v1.example.json");
    let effect: EffectReviewCaseV1 = load_example("effect-review-case-v1.example.json");
    let block: EffectBlockReceiptV1 = load_example("effect-block-receipt-v1.example.json");
    let delegation: DelegationReviewCaseV1 = load_example("delegation-review-case-v1.example.json");
    let release: ReleaseGateCaseV1 = load_example("release-gate-case-v1.example.json");
    let continuity: ContinuityReviewCaseV1 = load_example("continuity-review-case-v1.example.json");

    control.validate().expect("control example should validate");
    effect
        .validate()
        .expect("effect review example should validate");
    block
        .validate()
        .expect("effect block example should validate");
    delegation
        .validate()
        .expect("delegation review example should validate");
    release
        .validate()
        .expect("release gate example should validate");
    continuity
        .validate()
        .expect("continuity review example should validate");

    assert_eq!(
        control
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        effect
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        block
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        delegation
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        release
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );
    assert_eq!(
        continuity
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context:applicability-context-v25"
    );

    let roundtrip =
        serde_json::from_str::<ControlReceipt>(&serde_json::to_string(&control).unwrap()).unwrap();
    assert_eq!(
        roundtrip.citation.applicability_context_id,
        control.citation.applicability_context_id
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        roundtrip.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );

    let roundtrip =
        serde_json::from_str::<EffectReviewCaseV1>(&serde_json::to_string(&effect).unwrap())
            .unwrap();
    assert_eq!(
        roundtrip
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        effect
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );

    let roundtrip =
        serde_json::from_str::<EffectBlockReceiptV1>(&serde_json::to_string(&block).unwrap())
            .unwrap();
    assert_eq!(
        roundtrip
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        block
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );

    let roundtrip = serde_json::from_str::<DelegationReviewCaseV1>(
        &serde_json::to_string(&delegation).unwrap(),
    )
    .unwrap();
    assert_eq!(
        roundtrip
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        delegation
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );

    let roundtrip =
        serde_json::from_str::<ReleaseGateCaseV1>(&serde_json::to_string(&release).unwrap())
            .unwrap();
    assert_eq!(
        roundtrip
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        release
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );

    let roundtrip = serde_json::from_str::<ContinuityReviewCaseV1>(
        &serde_json::to_string(&continuity).unwrap(),
    )
    .unwrap();
    assert_eq!(
        roundtrip
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        continuity
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string()
    );
    assert_eq!(
        roundtrip.citation_status,
        ConstitutionalContextStatus::Complete
    );
}

#[test]
fn control_receipt_retains_v25_obligation_ref_shape() {
    let control: ControlReceipt = load_example("control-receipt-v1.example.json");
    assert_eq!(
        control.obligation_refs.required_obligation_refs,
        vec!["obligation:review:required"]
    );
    assert_eq!(
        control.obligation_refs.blocking_obligation_refs,
        vec!["obligation:blocking:policy"]
    );
    assert_eq!(
        control.obligation_refs.monitoring_obligation_refs,
        vec!["obligation:monitoring:continuous"]
    );
    assert_eq!(
        control.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        control.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );
}
