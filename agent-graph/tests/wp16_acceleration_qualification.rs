use agent_graph::acceleration_qualification::{
    qualify_kv_reuse, qualify_representation, AccelerationMetricsV1, FidelityClassV1, KvIdentityV1,
};

fn identity() -> KvIdentityV1 {
    KvIdentityV1 {
        model_revision: "model:r1".into(),
        layer: 3,
        head: 7,
        token_span_digest: "sha256:tokens".into(),
        position_encoding: "rope-v1".into(),
        kv_format: "fp16-kv".into(),
        precision: "fp16".into(),
    }
}

#[test]
fn adv_01_incompatible_kv_identity_rejects_exact_reuse_but_keeps_semantic_assessment_separate() {
    let stored = identity();
    let mut requested = stored.clone();
    requested.position_encoding = "rope-v2".into();
    requested.precision = "int8".into();
    let decision = qualify_kv_reuse(&stored, &requested, true);
    assert!(!decision.exact_kv_reuse);
    assert!(decision.semantic_artifact_reuse);
    assert_eq!(
        decision.mismatched_fields,
        ["position_encoding", "precision"]
    );
}

#[test]
fn adv_02_storage_active_memory_and_latency_metrics_remain_distinct() {
    let metrics = AccelerationMetricsV1 {
        storage_bytes: 10_000,
        active_ram_bytes: 2_000,
        active_vram_bytes: 4_000,
        latency_ms: 3.5,
        measured_active_runtime: true,
    };
    let encoded = serde_json::to_value(&metrics).unwrap();
    assert_eq!(encoded["storage_bytes"], 10_000);
    assert_eq!(encoded["active_ram_bytes"], 2_000);
    assert_eq!(encoded["active_vram_bytes"], 4_000);
    assert_eq!(encoded["latency_ms"], 3.5);
    assert_ne!(metrics.storage_bytes, metrics.active_ram_bytes);
    assert_ne!(metrics.active_ram_bytes, metrics.active_vram_bytes);
}

#[test]
fn rep_01_representation_change_never_upgrades_semantics_and_runtime_gain_needs_measurement() {
    let unmeasured = qualify_representation(
        "fibquant-v1",
        FidelityClassV1::LossyBounded,
        Some("artifact:exact-retained".into()),
        AccelerationMetricsV1 {
            storage_bytes: 100,
            active_ram_bytes: 200,
            active_vram_bytes: 300,
            latency_ms: 1.0,
            measured_active_runtime: false,
        },
        true,
    );
    assert_eq!(unmeasured.codec, "fibquant-v1");
    assert_eq!(unmeasured.fidelity, FidelityClassV1::LossyBounded);
    assert_eq!(
        unmeasured.exact_evidence_ref.as_deref(),
        Some("artifact:exact-retained")
    );
    assert!(!unmeasured.semantic_guarantee_upgraded);
    assert!(!unmeasured.serving_gain_supported);

    let measured = qualify_representation(
        "fibquant-v1",
        FidelityClassV1::LossyBounded,
        Some("artifact:exact-retained".into()),
        AccelerationMetricsV1 {
            storage_bytes: 100,
            active_ram_bytes: 200,
            active_vram_bytes: 300,
            latency_ms: 0.8,
            measured_active_runtime: true,
        },
        true,
    );
    assert!(measured.serving_gain_supported);
    assert!(!measured.semantic_guarantee_upgraded);
}
