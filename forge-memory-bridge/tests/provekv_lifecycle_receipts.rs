use forge_memory_bridge::{
    requested_post_import_artifacts, BridgeDerivedArtifactStatusV1, BridgeImportOptions,
    BRIDGE_DERIVED_ARTIFACT_STATUS_V1_SCHEMA, SEMANTIC_MEMORY_PROVEKV_POOL_ARTIFACT_FAMILY,
};

#[test]
fn provekv_lifecycle_receipt_serializes_candidate_only_boundary() {
    let status = BridgeDerivedArtifactStatusV1::provekv_pool_ready(
        "generation-1",
        "blake3:snapshot",
        "blake3:manifest",
    );
    status.validate().unwrap();

    let json = serde_json::to_value(&status).unwrap();
    assert_eq!(
        json["schema_version"],
        BRIDGE_DERIVED_ARTIFACT_STATUS_V1_SCHEMA
    );
    assert_eq!(
        json["artifact_family"],
        SEMANTIC_MEMORY_PROVEKV_POOL_ARTIFACT_FAMILY
    );
    assert_eq!(json["status"], "ready");
    assert_eq!(json["candidate_only"], true);
    assert_eq!(json["exact_f32_rerank_required"], true);
}

#[test]
fn post_import_hook_receipts_requested_rebuild_without_claiming_authority() {
    let options = BridgeImportOptions {
        rebuild_semantic_vector_artifacts: false,
        rebuild_provekv_pool_artifacts: true,
    };

    let artifacts = requested_post_import_artifacts(&options);
    assert_eq!(artifacts.len(), 1);
    assert_eq!(artifacts[0].status, "requested");
    assert!(artifacts[0].requested);
    assert!(artifacts[0].candidate_only);
    assert!(artifacts[0].exact_f32_rerank_required);
    artifacts[0].validate().unwrap();
}

#[test]
fn lifecycle_validator_rejects_authoritative_candidate_lie() {
    let mut status = BridgeDerivedArtifactStatusV1::provekv_pool_requested();
    status.exact_f32_rerank_required = false;

    let err = status.validate().unwrap_err();
    assert!(err.contains("exact f32 rerank"));
}
