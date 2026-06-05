use semantic_memory_forge::{
    ForgeAuditCandidateSearchRequestV1, ForgeAuditCandidateSearchResultV1,
};

#[test]
fn audit_candidate_search_requires_explain_only() {
    let request = ForgeAuditCandidateSearchRequestV1 {
        query: "similar evidence".into(),
        scope: "repo".into(),
        limit: 8,
        explain_only: false,
    };

    let err = request.validate().unwrap_err();
    assert!(err.contains("explain_only"));
}

#[test]
fn audit_candidate_result_cannot_claim_verification() {
    let mut result = ForgeAuditCandidateSearchResultV1 {
        evidence_ref: "evidence:1".into(),
        summary: "candidate only".into(),
        retrieval_backend: "semantic-memory".into(),
        candidate_backend: Some("provekv_pool_candidate_then_exact_f32".into()),
        candidate_only: true,
        exact_rerank: true,
        verified_by_forge: false,
    };
    result.validate_candidate_boundary().unwrap();

    result.verified_by_forge = true;
    let err = result.validate_candidate_boundary().unwrap_err();
    assert!(err.contains("cannot claim Forge verification"));
}
