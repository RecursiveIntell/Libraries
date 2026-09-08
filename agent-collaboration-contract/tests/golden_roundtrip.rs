use agent_collaboration_contract::{
    ArtifactRefV1, ResourceLimitsV1, TaskEnvelopeV1, CONTRACT_SCHEMA_VERSION,
};
use serde_json::json;
use stack_ids::{AgentId, ArtifactId, CapabilityManifestId, ContentDigest, GraphRunId, TaskId};

fn digest(label: &str) -> ContentDigest {
    ContentDigest::compute(label.as_bytes())
}

fn artifact() -> ArtifactRefV1 {
    ArtifactRefV1 {
        artifact_id: ArtifactId::new("artifact-1"),
        digest: digest("input"),
        size_bytes: 5,
        media_type: "application/octet-stream".into(),
    }
}

#[test]
fn task_envelope_roundtrips_and_validates() {
    let task = TaskEnvelopeV1 {
        schema_version: CONTRACT_SCHEMA_VERSION.into(),
        task_id: TaskId::new("task-1"),
        owner_agent_id: AgentId::new("controller"),
        producer_agent_id: AgentId::new("producer"),
        capability_manifest_id: CapabilityManifestId::new("capabilities-1"),
        capability_manifest_digest: digest("capabilities"),
        graph_id: GraphRunId::new("graph-1"),
        graph_version: "graph-v1".into(),
        graph_digest: digest("graph"),
        idempotency_key: "request-1".into(),
        request_digest: digest("request"),
        input_artifacts: vec![artifact()],
        resource_limits: ResourceLimitsV1 {
            max_input_bytes: 1024,
            max_output_bytes: 2048,
            max_runtime_secs: 30,
            max_artifacts: 4,
        },
        submitted_at: "2026-09-08T04:00:00Z".into(),
    };
    task.validate().unwrap();
    let encoded = serde_json::to_value(&task).unwrap();
    assert_eq!(encoded["schema_version"], json!(CONTRACT_SCHEMA_VERSION));
    let decoded: TaskEnvelopeV1 = serde_json::from_value(encoded).unwrap();
    assert_eq!(decoded, task);
}
