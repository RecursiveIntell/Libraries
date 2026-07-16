use claim_ledger::{SimilarClaimCandidateV1, SIMILAR_CLAIM_CANDIDATE_V1_SCHEMA};

#[test]
fn similar_claim_candidate_is_candidate_only_and_exact_reranked() {
    let candidate = SimilarClaimCandidateV1 {
        claim_id: "claim-1".into(),
        claim_version_id: Some("claim-version-1".into()),
        retrieval_backend: "semantic-memory".into(),
        candidate_backend: Some("provekv_pool_candidate_then_exact_f32".into()),
        generation_id: Some("generation-1".into()),
        exact_rerank: true,
        candidate_only: true,
        mutates_verification_state: false,
    };

    candidate.validate_boundary().unwrap();
    let json = serde_json::to_value(&candidate).unwrap();
    assert_eq!(json["candidate_only"], true);
    assert_eq!(
        SIMILAR_CLAIM_CANDIDATE_V1_SCHEMA,
        "similar_claim_candidate_v1"
    );
}

#[test]
fn similar_claim_candidate_cannot_mutate_verification_state() {
    let candidate = SimilarClaimCandidateV1 {
        claim_id: "claim-1".into(),
        claim_version_id: None,
        retrieval_backend: "semantic-memory".into(),
        candidate_backend: None,
        generation_id: None,
        exact_rerank: true,
        candidate_only: true,
        mutates_verification_state: true,
    };

    let err = candidate.validate_boundary().unwrap_err();
    assert!(err.contains("cannot mutate verification state"));
}
