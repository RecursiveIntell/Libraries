#[test]
fn stack_import_smoke() {
    let artifact_id = stack_ids::ArtifactId::new("aidens-stack-smoke");
    let digest = stack_ids::ContentDigest::compute_str("aidens-stack-smoke");
    let tool_result = llm_tool_runtime::ToolResult::text("ok");
    let operator = recursive_kernel_core::constraint_compiler_operator();
    let compiler_policy = constraint_compiler::CompilerPolicy {
        policy_version: "aidens-smoke".into(),
        include_hyperedges: true,
    };
    let citation = verification_control::V25CitationContext::missing();

    assert_eq!(artifact_id.as_str(), "artifact:aidens-stack-smoke");
    assert_eq!(digest.hex().len(), 64);
    assert_eq!(tool_result.to_model_output(), "ok");
    assert!(operator.validate().is_ok());
    assert!(compiler_policy.include_hyperedges);
    assert_eq!(
        semantic_memory_forge::EXPORT_ENVELOPE_V3_SCHEMA,
        "export_envelope_v3"
    );
    assert_eq!(
        forge_memory_bridge::PROJECTION_IMPORT_BATCH_V3_SCHEMA,
        "projection_import_batch_v3"
    );
    assert_eq!(
        citation.context_status(),
        verification_control::ConstitutionalContextStatus::Missing
    );
    assert!(std::mem::size_of::<semantic_memory::MemoryStore>() > 0);
    assert!(std::mem::size_of::<knowledge_runtime::KnowledgeRuntime>() > 0);
    assert!(std::mem::size_of::<kernel_execution::ExecutionReport>() > 0);
    assert!(std::mem::size_of::<kernel_oracles::OracleAssessment>() > 0);
    assert!(std::mem::size_of::<verification_policy::ExecutionPermit>() > 0);
    assert!(std::mem::size_of::<verification_adjudication::EffectAdjudicationReceiptV1>() > 0);
    assert!(std::mem::size_of::<authority_delegation::AuthorityChainV1>() > 0);
    assert!(std::mem::size_of::<assurance_runtime::AssuranceCaseV1>() > 0);
}
