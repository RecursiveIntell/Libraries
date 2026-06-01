//! Receipt tests: round-trip serialization, validation, content addressing.

use poly_kv::policy::CompressionPolicy;
use poly_kv::receipt::{
    now_unix, BlockInjectionTrace, InjectionReceipt, PoolBuildReceipt, ShellMaterializeReceipt,
};

#[test]
fn test_pool_build_receipt_round_trip_serialize_deserialize() {
    let receipt = PoolBuildReceipt::new(
        "abc123".into(),
        vec!["layer0_digest".into(), "layer1_digest".into()],
        "codebook_digest".into(),
        "rotation_digest".into(),
        100,
        42,
        10_000,
        500_000,
        CompressionPolicy::default_two_tier(),
        42,
        now_unix(),
    );
    assert!(receipt.validate().is_ok());

    let json = serde_json::to_string(&receipt).unwrap();
    let deser: PoolBuildReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.pool_digest, deser.pool_digest);
    assert_eq!(receipt.layer_digests, deser.layer_digests);
    assert_eq!(receipt.total_tokens, deser.total_tokens);
    assert_eq!(receipt.fib_build_ms, deser.fib_build_ms);
    assert_eq!(receipt.pool_size_bytes, deser.pool_size_bytes);
    assert_eq!(receipt.raw_size_bytes, deser.raw_size_bytes);
    assert_eq!(receipt.compression_ratio, deser.compression_ratio);
    assert_eq!(receipt.seeded_with, deser.seeded_with);
    assert_eq!(receipt.built_at_unix, deser.built_at_unix);
    assert_eq!(receipt.schema_version, deser.schema_version);
    assert_eq!(receipt.codebook_digest, deser.codebook_digest);
    assert_eq!(receipt.rotation_digest, deser.rotation_digest);
}

#[test]
fn test_shell_materialize_receipt_round_trip() {
    let receipt = ShellMaterializeReceipt::new(
        "agent_1".into(),
        "pool_abc".into(),
        "shell_xyz".into(),
        50,
        5_000,
        10,
        now_unix(),
    );
    assert!(receipt.validate().is_ok());

    let json = serde_json::to_string(&receipt).unwrap();
    let deser: ShellMaterializeReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.agent_id, deser.agent_id);
    assert_eq!(receipt.pool_digest, deser.pool_digest);
    assert_eq!(receipt.shell_digest, deser.shell_digest);
    assert_eq!(receipt.num_unique_tokens, deser.num_unique_tokens);
    assert_eq!(receipt.shell_size_bytes, deser.shell_size_bytes);
    assert_eq!(receipt.materialize_ms, deser.materialize_ms);
    assert_eq!(receipt.materialized_at_unix, deser.materialized_at_unix);
}

#[test]
fn test_injection_receipt_round_trip() {
    let traces = vec![
        BlockInjectionTrace {
            layer: 0,
            source: "pool".into(),
            source_digest: "abc".into(),
            target_position: 0,
        },
        BlockInjectionTrace {
            layer: 0,
            source: "shell".into(),
            source_digest: "def".into(),
            target_position: 1,
        },
    ];
    let receipt = InjectionReceipt::new(
        "agent_1".into(),
        "pool_abc".into(),
        "shell_xyz".into(),
        2,
        traces,
        now_unix(),
    );
    assert!(receipt.validate().is_ok());

    let json = serde_json::to_string(&receipt).unwrap();
    let deser: InjectionReceipt = serde_json::from_str(&json).unwrap();
    assert_eq!(receipt.agent_id, deser.agent_id);
    assert_eq!(receipt.pool_digest, deser.pool_digest);
    assert_eq!(receipt.shell_digest, deser.shell_digest);
    assert_eq!(receipt.blocks_injected, deser.blocks_injected);
    assert_eq!(receipt.block_traces.len(), deser.block_traces.len());
    assert_eq!(receipt.block_traces[0].source, deser.block_traces[0].source);
}

#[test]
fn test_receipt_digest_is_content_addressed() {
    let receipt1 = PoolBuildReceipt::new(
        "abc".into(),
        vec!["l0".into()],
        "cb".into(),
        "rot".into(),
        10,
        100,
        500,
        5000,
        CompressionPolicy::default_two_tier(),
        42,
        now_unix(),
    );
    let receipt2 = PoolBuildReceipt::new(
        "abc".into(),
        vec!["l0".into()],
        "cb".into(),
        "rot".into(),
        10,
        100,
        500,
        5000,
        CompressionPolicy::default_two_tier(),
        42,
        now_unix(),
    );

    let d1 = receipt1.digest().unwrap();
    let d2 = receipt2.digest().unwrap();
    assert_eq!(d1, d2, "Identical receipts must have identical digests");

    // Different content produces different digest
    let receipt3 = PoolBuildReceipt::new(
        "xyz".into(),
        vec!["l0".into()],
        "cb".into(),
        "rot".into(),
        10,
        100,
        500,
        5000,
        CompressionPolicy::default_two_tier(),
        42,
        now_unix(),
    );
    let d3 = receipt3.digest().unwrap();
    assert_ne!(d1, d3, "Different receipts must have different digests");
}

#[test]
fn test_invalid_receipt_schema_rejected() {
    let mut receipt = PoolBuildReceipt::new(
        "abc".into(),
        vec![],
        "".into(),
        "".into(),
        0,
        0,
        0,
        0,
        CompressionPolicy::default_two_tier(),
        0,
        now_unix(),
    );
    receipt.schema_version = "bad_version".into();
    assert!(receipt.validate().is_err());
}
