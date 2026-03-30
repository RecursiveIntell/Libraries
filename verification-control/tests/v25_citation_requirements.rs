use verification_control::{
    ConstitutionalContextStatus, ControlReceipt, EffectBlockReceiptV1, EffectReviewCaseV1,
    CONTROL_RECEIPT_V1_SCHEMA,
};

#[test]
fn review_case_citation_fields_are_present_in_fixture_shapes() {
    let path = format!(
        "{}/../examples/effect-review-case-v1.example.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let effect: EffectReviewCaseV1 =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert_eq!(
        effect
            .citation
            .applicability_context_id
            .as_ref()
            .unwrap()
            .to_string(),
        "applicability-context-v25"
    );
    assert_eq!(
        effect.citation.profile_set_id.as_ref().unwrap().to_string(),
        "profile-set-v25"
    );
    assert_eq!(
        effect
            .citation
            .composition_receipt_id
            .as_ref()
            .unwrap()
            .to_string(),
        "composition-receipt-v25"
    );
    assert_eq!(
        effect
            .citation
            .compiled_obligation_set_id
            .as_ref()
            .unwrap()
            .to_string(),
        "compiled-obligation-set-v25"
    );
    assert_eq!(
        effect.obligation_refs.required_obligation_refs,
        vec!["obligation:required:policy"]
    );
    assert_eq!(
        effect.obligation_refs.blocking_obligation_refs,
        vec!["obligation:blocking:policy"]
    );
    assert_eq!(
        effect.obligation_refs.monitoring_obligation_refs,
        vec!["obligation:monitoring:continuous"]
    );
    assert_eq!(
        effect.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        effect.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );

    let path = format!(
        "{}/../examples/effect-block-receipt-v1.example.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let block: EffectBlockReceiptV1 =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert_eq!(
        block.citation.profile_set_id.as_ref().unwrap().to_string(),
        "profile-set-v25"
    );
    assert_eq!(
        block
            .citation
            .composition_receipt_id
            .as_ref()
            .unwrap()
            .to_string(),
        "composition-receipt-v25"
    );
}

#[test]
fn control_receipt_schema_is_roundtrip_safe() {
    let path = format!(
        "{}/../examples/control-receipt-v1.example.json",
        env!("CARGO_MANIFEST_DIR")
    );
    let receipt: ControlReceipt =
        serde_json::from_str(&std::fs::read_to_string(path).unwrap()).unwrap();

    assert_eq!(receipt.schema_version, CONTROL_RECEIPT_V1_SCHEMA);
    assert_eq!(
        receipt.citation_status,
        ConstitutionalContextStatus::Complete
    );
    assert_eq!(
        receipt.obligation_refs_status,
        ConstitutionalContextStatus::Complete
    );
    let text = serde_json::to_string_pretty(&receipt).unwrap();
    let reparsed: ControlReceipt = serde_json::from_str(&text).unwrap();
    assert_eq!(receipt.receipt_id, reparsed.receipt_id);
    assert_eq!(
        receipt.citation.applicability_context_id,
        reparsed.citation.applicability_context_id
    );
}
