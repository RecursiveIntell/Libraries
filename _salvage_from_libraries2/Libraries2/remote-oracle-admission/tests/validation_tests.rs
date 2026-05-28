use remote_oracle_admission::{
    AttestationReplayImpactV1, AttestationSupersessionV1, CrossRuntimeReplayTicketV1,
    LocalAdmissionRecommendationV1, RemoteDisclosureClassV1, RemoteExactnessClassV1,
    RemoteOracleLeaseV1, RemoteReplayObligationV1, RemoteSliceRequestV1, RemoteSliceResultV1,
    ReplayFailureBehaviorV1, V25ConstitutionCitation,
};
use stack_ids::{
    ApplicabilityContextId, AttestationEnvelopeId, AttestationSupersessionId,
    CompiledObligationSetId, CompositionReceiptId, CrossRuntimeReplayTicketId,
    EffectiveConstitutionId, ProfileSetId, RemoteOracleLeaseId, RemoteSliceRequestId,
    RemoteSliceResultId, TrustRootSetId,
};

fn citation() -> V25ConstitutionCitation {
    V25ConstitutionCitation::new(
        ApplicabilityContextId::new("applicability-context-v25"),
        ProfileSetId::new("profile-set-v25"),
        CompositionReceiptId::new("composition-receipt-v25"),
        EffectiveConstitutionId::new("effective-constitution-v25"),
        CompiledObligationSetId::new("compiled-obligation-set-v25"),
        None,
        Vec::new(),
    )
}

#[test]
fn lease_requires_owner_refs() {
    let err = RemoteOracleLeaseV1::new(
        RemoteOracleLeaseId::new("lease_001"),
        "oracle:trusted-remote",
        vec!["AttestationEnvelopeV1".to_string()],
        vec!["bounded_replay_slice".to_string()],
        RemoteExactnessClassV1::BoundedExact,
        "daily_cost<=5",
        RemoteDisclosureClassV1::RedactedStructuredOnly,
        RemoteReplayObligationV1::MustReturnReplayTicketOrNonreplayableReason,
        "2026-03-31T00:00:00Z",
        Vec::new(),
    )
    .expect_err("lease without owner refs must fail");
    assert_eq!(err, "policy_owner_refs");
}

#[test]
fn request_requires_artifact_refs() {
    let err = RemoteSliceRequestV1::new(
        RemoteSliceRequestId::new("request_001"),
        "bounded replay slice",
        Vec::new(),
        "no_pii",
        RemoteExactnessClassV1::Exact,
        TrustRootSetId::new("trs_001"),
        citation(),
        vec!["expectation:boundary-proof".to_string()],
        RemoteOracleLeaseId::new("lease_001"),
    )
    .expect_err("request without required artifact refs must fail");
    assert_eq!(err, "required_artifact_refs");
}

#[test]
fn result_requires_returned_artifact_refs() {
    let err = RemoteSliceResultV1::new(
        RemoteSliceResultId::new("result_001"),
        RemoteSliceRequestId::new("request_001"),
        citation(),
        Vec::new(),
        RemoteExactnessClassV1::Exact,
        "replay-ready",
        vec!["marker:policy".to_string()],
        "replay-handle-001",
        LocalAdmissionRecommendationV1::Eligible,
        AttestationEnvelopeId::new("attestation-envelope-001"),
    )
    .expect_err("result without returned artifacts must fail");
    assert_eq!(err, "returned_artifact_refs");
}

#[test]
fn replay_ticket_requires_trust_roots() {
    let err = CrossRuntimeReplayTicketV1::new(
        CrossRuntimeReplayTicketId::new("ticket_001"),
        citation(),
        vec!["artifact:replay".to_string()],
        "2026-03-17T00:00:00Z",
        Vec::new(),
        RemoteDisclosureClassV1::NonSensitive,
        "2026-03-17T00:00:00Z/2026-03-18T00:00:00Z",
        vec!["expectation:complete".to_string()],
        ReplayFailureBehaviorV1::Retry,
    )
    .expect_err("ticket without trust roots must fail");
    assert_eq!(err, "required_trust_roots");
}

#[test]
fn supersession_rejects_self_replacement() {
    let err = AttestationSupersessionV1::new(
        AttestationSupersessionId::new("sup_001"),
        "attestation:old",
        "attestation:old",
        "same artifact cannot supersede itself",
        "2026-03-17T00:00:00Z",
        AttestationReplayImpactV1::ReplayTicketUnchanged,
        false,
    )
    .expect_err("self replacement must fail");
    assert_eq!(err, "replacement_ref");
}
