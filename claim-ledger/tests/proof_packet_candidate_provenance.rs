use claim_ledger::{
    ProofPacketCandidateProvenanceV1, SimilarClaimCandidateV1,
    PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA,
};

fn candidate() -> SimilarClaimCandidateV1 {
    SimilarClaimCandidateV1 {
        claim_id: "claim-1".into(),
        claim_version_id: Some("claim-version-1".into()),
        retrieval_backend: "semantic-memory".into(),
        candidate_backend: Some("provekv_pool_candidate_then_exact_f32".into()),
        generation_id: Some("generation-1".into()),
        exact_rerank: true,
        candidate_only: true,
        mutates_verification_state: false,
    }
}

#[test]
fn candidate_only_proof_packet_does_not_pass_verification_gate() {
    let packet = ProofPacketCandidateProvenanceV1 {
        schema_version: PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA.into(),
        candidate_discovery: vec![candidate()],
        verified_evidence_refs: vec![],
    };

    let err = packet.validate_for_verification_gate().unwrap_err();
    assert!(err.contains("no verified evidence refs"));
}

#[test]
fn proof_packet_with_verified_evidence_keeps_candidate_provenance_separate() {
    let packet = ProofPacketCandidateProvenanceV1 {
        schema_version: PROOF_PACKET_CANDIDATE_PROVENANCE_V1_SCHEMA.into(),
        candidate_discovery: vec![candidate()],
        verified_evidence_refs: vec!["evidence:verified:1".into()],
    };

    packet.validate_for_verification_gate().unwrap();
}
