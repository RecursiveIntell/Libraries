use federated_settlement::{
    evaluate_divergence_or_suspension, evaluate_settlement, evaluate_shared_replay,
    SettlementCaseV1, SettlementDisposition, SettlementReceiptV1, SharedDispositionV1,
    SharedDivergenceReportV1, SharedReplaySliceV1,
};

#[test]
fn settlement_outputs_preserve_local_constitutional_citation() {
    let case = SettlementCaseV1 {
        schema_version: federated_settlement::SETTLEMENT_CASE_V1_SCHEMA.into(),
        settlement_case_id: stack_ids::SettlementCaseId::new("settlement-case-v25"),
        treaty_bundle_id: stack_ids::TreatyBundleId::new("treaty-v25"),
        equivalence_bundle_id: stack_ids::CrossRuntimeEquivalenceBundleId::new("equivalence-v25"),
        citation: federated_settlement::V25ConstitutionCitation {
            applicability_context_id: stack_ids::ApplicabilityContextId::new(
                "applicability-context-v25",
            ),
            profile_set_id: stack_ids::ProfileSetId::new("profile-set-v25"),
            composition_receipt_id: stack_ids::CompositionReceiptId::new("composition-receipt-v25"),
            effective_constitution_id: stack_ids::EffectiveConstitutionId::new(
                "effective-constitution-v25",
            ),
            compiled_obligation_set_id: stack_ids::CompiledObligationSetId::new(
                "compiled-obligation-set-v25",
            ),
            composition_conflict_set_id: None,
            profile_exception_bundle_ids: vec![],
        },
        shared_disposition: SharedDispositionV1 {
            schema_version: federated_settlement::SHARED_DISPOSITION_V1_SCHEMA.into(),
            shared_disposition_id: stack_ids::SharedDispositionId::new("shared-disposition-v25"),
            disposition_label: "bounded_share".into(),
            treatment: "bounded claim parity".into(),
            advisory_only: false,
        },
        replay_requirements: vec![
            federated_settlement::ReplayRequirementV1 {
                runtime_name: "runtime-a".into(),
                required: true,
                replay_available: true,
            },
            federated_settlement::ReplayRequirementV1 {
                runtime_name: "runtime-b".into(),
                required: true,
                replay_available: true,
            },
        ],
        local_dissent: vec![],
        advisory_only: false,
        non_admitted: false,
    };

    let replay: SharedReplaySliceV1 = evaluate_shared_replay(&case, "2026-03-17T00:00:00Z");
    let (divergence, suspension): (
        SharedDivergenceReportV1,
        Option<federated_settlement::TreatySuspensionV1>,
    ) = evaluate_divergence_or_suspension(&case, &replay, "no divergence", "2026-03-17T00:00:00Z");
    let receipt: SettlementReceiptV1 = evaluate_settlement(&case, "2026-03-17T00:00:00Z");

    assert_eq!(
        replay.citation.applicability_context_id,
        case.citation.applicability_context_id
    );
    assert_eq!(
        divergence.citation.applicability_context_id,
        case.citation.applicability_context_id
    );
    assert_eq!(
        receipt.citation.applicability_context_id,
        case.citation.applicability_context_id
    );
    assert_eq!(
        receipt.citation.compiled_obligation_set_id,
        case.citation.compiled_obligation_set_id
    );
    assert_eq!(replay.citation.profile_set_id, case.citation.profile_set_id);
    assert_eq!(divergence.blocking_runtime_names, Vec::<String>::new());
    assert_eq!(
        receipt.disposition,
        SettlementDisposition::SharedDispositionIssued
    );
    assert!(receipt.shared_disposition_id.is_some());
    assert!(suspension.is_none());
}
