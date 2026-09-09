use llm_pipeline::ToolLoopRequest;
use stack_ids::AttemptId;

#[test]
fn pipeline_request_accepts_canonical_authority_attempt_identity() {
    let attempt = AttemptId::generate();
    let mut request = ToolLoopRequest::new("fixture-no-inference", "identity-only");
    request.attempt_id = Some(attempt.clone());
    let returned: Option<AttemptId> = request.attempt_id;
    assert_eq!(returned, Some(attempt));
}
