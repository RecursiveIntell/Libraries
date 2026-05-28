use remote_oracle_admission::{
    CrossRuntimeReplayTicketV1, LocalAdmissionRecommendationV1, RemoteDisclosureClassV1,
    RemoteExactnessClassV1, RemoteSliceRequestV1, RemoteSliceResultV1, ReplayFailureBehaviorV1,
};

#[test]
fn remote_slice_artifacts_preserve_composite_constitutional_citation() {
    let citation = remote_oracle_admission::V25ConstitutionCitation {
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
    };

    let request = RemoteSliceRequestV1 {
        schema_version: remote_oracle_admission::REMOTE_SLICE_REQUEST_V1_SCHEMA.into(),
        remote_slice_request_id: stack_ids::RemoteSliceRequestId::new("remote-slice-request-v25"),
        requested_slice_definition: "bounded replay slice".into(),
        required_artifact_refs: vec!["artifact:claim:v25".into()],
        allowed_disclosure_policy: "no_pii".into(),
        exactness_target: RemoteExactnessClassV1::Exact,
        trust_root_set_id: stack_ids::TrustRootSetId::new("trust-root-set-v25"),
        citation: citation.clone(),
        challenge_expectations: vec!["expected:proof-of-boundary".into()],
        remote_oracle_lease_id: stack_ids::RemoteOracleLeaseId::new("remote-oracle-lease-v25"),
    };

    let result = RemoteSliceResultV1 {
        schema_version: remote_oracle_admission::REMOTE_SLICE_RESULT_V1_SCHEMA.into(),
        remote_slice_result_id: stack_ids::RemoteSliceResultId::new("remote-slice-result-v25"),
        remote_slice_request_id: request.remote_slice_request_id.clone(),
        citation: request.citation.clone(),
        returned_artifact_refs: vec!["artifact:replay:v25".into()],
        exactness_class: RemoteExactnessClassV1::Exact,
        remote_execution_evidence: "replay-ready".into(),
        disclosure_markers: vec!["marker:policy".into()],
        replay_handle: "replay-handle-v25".into(),
        local_admission_recommendation: LocalAdmissionRecommendationV1::Eligible,
        attestation_envelope_id: stack_ids::AttestationEnvelopeId::new("attestation-envelope-v25"),
    };

    let ticket = CrossRuntimeReplayTicketV1 {
        schema_version: remote_oracle_admission::CROSS_RUNTIME_REPLAY_TICKET_V1_SCHEMA.into(),
        cross_runtime_replay_ticket_id: stack_ids::CrossRuntimeReplayTicketId::new(
            "cross-runtime-ticket-v25",
        ),
        citation: request.citation.clone(),
        artifact_refs: vec!["artifact:replay:v25".into()],
        time_coordinates: "2026-03-17T00:00:00Z".into(),
        required_trust_roots: vec![stack_ids::TrustRootSetId::new("trust-root-set-v25")],
        allowed_disclosure: RemoteDisclosureClassV1::NonSensitive,
        lease_window: "2026-03-17T00:00:00Z/2026-03-18T00:00:00Z".into(),
        replay_expectations: vec!["expectation:complete".into()],
        failure_behavior: ReplayFailureBehaviorV1::Retry,
    };

    request.validate().expect("request validates");
    result.validate().expect("result validates");
    ticket.validate().expect("ticket validates");

    assert_eq!(
        request.citation.applicability_context_id,
        citation.applicability_context_id
    );
    assert_eq!(
        result.citation.composition_receipt_id,
        request.citation.composition_receipt_id
    );
    assert_eq!(
        ticket.citation.compiled_obligation_set_id,
        request.citation.compiled_obligation_set_id
    );
}
