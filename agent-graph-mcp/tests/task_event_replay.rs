#![allow(clippy::expect_used)]

use agent_collaboration_contract::{
    ArtifactRefV1, ResourceLimitsV1, TaskEnvelopeV1, TaskEventKindV1, TaskEventV1, TaskStatusV1,
    CONTRACT_SCHEMA_VERSION,
};
use agent_graph_mcp::collaboration_store::CollaborationStoreError;
use agent_graph_mcp::store::PersistentStore;
use rusqlite::Connection;
use stack_ids::{AgentId, CapabilityManifestId, ContentDigest, GraphRunId, TaskEventId, TaskId};
use tempfile::tempdir;

fn envelope() -> TaskEnvelopeV1 {
    TaskEnvelopeV1 {
        schema_version: CONTRACT_SCHEMA_VERSION.into(),
        task_id: TaskId::new("replay-task"),
        owner_agent_id: AgentId::new("owner"),
        producer_agent_id: AgentId::new("producer"),
        capability_manifest_id: CapabilityManifestId::new("capability"),
        capability_manifest_digest: ContentDigest::compute(b"capability"),
        graph_id: GraphRunId::new("graph"),
        graph_version: "v1".into(),
        graph_digest: ContentDigest::compute(b"graph"),
        idempotency_key: "replay-key".into(),
        request_digest: ContentDigest::compute(b"request"),
        input_artifacts: Vec::<ArtifactRefV1>::new(),
        resource_limits: ResourceLimitsV1 {
            max_input_bytes: 1024,
            max_output_bytes: 1024,
            max_runtime_secs: 60,
            max_artifacts: 1,
        },
        submitted_at: "2026-09-08T00:00:00Z".into(),
    }
}

#[test]
fn replay_rejects_tampered_event_bytes() {
    let temp = tempdir().expect("temporary data root");
    let persistent = PersistentStore::open(temp.path()).expect("store");
    let collaboration = persistent.collaboration();
    let task = envelope();
    collaboration.append_task(&task).expect("task");
    collaboration
        .append_event(&TaskEventV1 {
            schema_version: CONTRACT_SCHEMA_VERSION.into(),
            event_id: TaskEventId::new("replay-event"),
            task_id: task.task_id.clone(),
            owner_agent_id: AgentId::new("owner"),
            kind: TaskEventKindV1::Accepted,
            status: TaskStatusV1::Accepted,
            attempt_id: None,
            trial_id: None,
            occurred_at: "2026-09-08T00:00:01Z".into(),
            recorded_at: "2026-09-08T00:00:02Z".into(),
            previous_event_digest: None,
            reason_code: None,
        })
        .expect("event");
    let connection = Connection::open(temp.path().join("agent-graph.db")).expect("db");
    connection
        .execute(
            "UPDATE collaboration_task_events SET event_json = ?1 WHERE task_id = ?2",
            rusqlite::params!["{\"tampered\":true}", task.task_id.to_string()],
        )
        .expect("tamper event");
    assert!(matches!(
        collaboration.rebuild_projection(task.task_id.as_str()),
        Err(CollaborationStoreError::Serialization(_))
            | Err(CollaborationStoreError::ProjectionIntegrity(_))
    ));
}
