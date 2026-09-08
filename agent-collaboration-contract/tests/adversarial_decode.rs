use agent_collaboration_contract::{ResourceLimitsV1, TaskEnvelopeV1, CONTRACT_SCHEMA_VERSION};
use serde_json::json;
use stack_ids::{AgentId, CapabilityManifestId, ContentDigest, GraphRunId, TaskId};

fn minimal() -> serde_json::Value {
    json!({
        "schema_version": CONTRACT_SCHEMA_VERSION,
        "task_id": "task-1",
        "owner_agent_id": "controller",
        "producer_agent_id": "producer",
        "capability_manifest_id": "capabilities-1",
        "capability_manifest_digest": ContentDigest::compute(b"capabilities").hex(),
        "graph_id": "graph-1",
        "graph_version": "graph-v1",
        "graph_digest": ContentDigest::compute(b"graph").hex(),
        "idempotency_key": "request-1",
        "request_digest": ContentDigest::compute(b"request").hex(),
        "input_artifacts": [],
        "resource_limits": {
            "max_input_bytes": 1024,
            "max_output_bytes": 2048,
            "max_runtime_secs": 30,
            "max_artifacts": 4
        },
        "submitted_at": "2026-09-08T04:00:00Z"
    })
}

#[test]
fn unknown_fields_are_rejected() {
    let mut value = minimal();
    value["untrusted_authority"] = json!(true);
    assert!(serde_json::from_value::<TaskEnvelopeV1>(value).is_err());
}

#[test]
fn zero_resource_ceiling_is_rejected() {
    let mut value = minimal();
    value["resource_limits"]["max_runtime_secs"] = json!(0);
    let decoded: TaskEnvelopeV1 = serde_json::from_value(value).unwrap();
    assert!(decoded.validate().is_err());
}

#[test]
fn typed_ids_and_digests_decode_without_string_aliases() {
    let decoded: TaskEnvelopeV1 = serde_json::from_value(minimal()).unwrap();
    assert_eq!(decoded.task_id, TaskId::new("task-1"));
    assert_eq!(decoded.owner_agent_id, AgentId::new("controller"));
    assert_eq!(decoded.graph_id, GraphRunId::new("graph-1"));
    assert_eq!(
        decoded.capability_manifest_id,
        CapabilityManifestId::new("capabilities-1")
    );
    assert_eq!(
        decoded.resource_limits,
        ResourceLimitsV1 {
            max_input_bytes: 1024,
            max_output_bytes: 2048,
            max_runtime_secs: 30,
            max_artifacts: 4,
        }
    );
}
