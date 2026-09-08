#![allow(clippy::expect_used)]

use agent_collaboration_contract::{
    ArtifactRefV1, ResourceLimitsV1, TaskEnvelopeV1, TaskEventKindV1, TaskEventV1, TaskStatusV1,
    CONTRACT_SCHEMA_VERSION,
};
use agent_graph_mcp::collaboration_store::CollaborationStoreError;
use agent_graph_mcp::store::PersistentStore;
use stack_ids::{
    AgentId, ArtifactId, CapabilityManifestId, ContentDigest, GraphRunId, TaskEventId, TaskId,
};
use tempfile::tempdir;

fn envelope(key: &str, value: u8) -> TaskEnvelopeV1 {
    TaskEnvelopeV1 {
        schema_version: CONTRACT_SCHEMA_VERSION.into(),
        task_id: TaskId::new(format!("task-{key}")),
        owner_agent_id: AgentId::new("owner-1"),
        producer_agent_id: AgentId::new("producer-1"),
        capability_manifest_id: CapabilityManifestId::new("capability-1"),
        capability_manifest_digest: ContentDigest::compute(b"capability"),
        graph_id: GraphRunId::new("graph-1"),
        graph_version: "graph-v1".into(),
        graph_digest: ContentDigest::compute(b"graph"),
        idempotency_key: key.into(),
        request_digest: ContentDigest::compute(&[value]),
        input_artifacts: Vec::<ArtifactRefV1>::new(),
        resource_limits: ResourceLimitsV1 {
            max_input_bytes: 1024,
            max_output_bytes: 1024,
            max_runtime_secs: 30,
            max_artifacts: 2,
        },
        submitted_at: "2026-09-08T00:00:00Z".into(),
    }
}

fn event(
    task_id: &TaskId,
    event_id: &str,
    status: TaskStatusV1,
    previous: Option<ContentDigest>,
) -> TaskEventV1 {
    TaskEventV1 {
        schema_version: CONTRACT_SCHEMA_VERSION.into(),
        event_id: TaskEventId::new(event_id),
        task_id: task_id.clone(),
        owner_agent_id: AgentId::new("owner-1"),
        kind: match status {
            TaskStatusV1::Accepted => TaskEventKindV1::Accepted,
            TaskStatusV1::Completed => TaskEventKindV1::Completed,
            _ => TaskEventKindV1::Failed,
        },
        status,
        attempt_id: None,
        trial_id: None,
        occurred_at: "2026-09-08T00:00:01Z".into(),
        recorded_at: "2026-09-08T00:00:02Z".into(),
        previous_event_digest: previous,
        reason_code: None,
    }
}

#[test]
fn collaboration_migration_is_idempotent_and_owned_by_persistent_store() {
    let temp = tempdir().expect("temporary data root");
    let first = PersistentStore::open(temp.path()).expect("first open");
    let _collaboration = first.collaboration();
    drop(first);
    let second = PersistentStore::open(temp.path()).expect("second open");
    let connection = rusqlite::Connection::open(temp.path().join("agent-graph.db")).expect("db");
    let count: i64 = connection
        .query_row(
            "SELECT count(*) FROM collaboration_schema_migrations WHERE version = 1",
            [],
            |row| row.get(0),
        )
        .expect("migration row");
    assert_eq!(count, 1);
    for table in [
        "collaboration_tasks",
        "collaboration_task_events",
        "collaboration_attempts",
        "collaboration_leases",
        "collaboration_artifacts",
        "collaboration_deliveries",
        "collaboration_conflicts",
    ] {
        let present: i64 = connection
            .query_row(
                "SELECT count(*) FROM sqlite_master WHERE type='table' AND name=?1",
                [table],
                |row| row.get(0),
            )
            .expect("collaboration table");
        assert_eq!(present, 1, "missing table {table}");
    }
    let projection = second
        .collaboration()
        .append_task(&envelope("migration-task", 1));
    assert!(projection.is_ok());
}

#[test]
fn task_events_are_idempotent_replayable_and_conflict_safe() {
    let temp = tempdir().expect("temporary data root");
    let store = PersistentStore::open(temp.path())
        .expect("store")
        .collaboration();
    let submitted = envelope("same-key", 1);
    let first = store.append_task(&submitted).expect("task accepted");
    let artifact = ArtifactRefV1 {
        artifact_id: ArtifactId::new("artifact-1"),
        digest: ContentDigest::compute(b"abc"),
        size_bytes: 3,
        media_type: "text/plain".into(),
    };
    store
        .record_artifact(submitted.task_id.as_str(), "owner-1", &artifact)
        .expect("artifact metadata");
    store
        .record_artifact(submitted.task_id.as_str(), "owner-1", &artifact)
        .expect("idempotent artifact metadata");
    let accepted = event(&submitted.task_id, "event-1", TaskStatusV1::Accepted, None);
    let after_accept = store.append_event(&accepted).expect("accepted event");
    assert_eq!(after_accept.status, TaskStatusV1::Accepted);
    assert_eq!(after_accept.sequence, 1);
    let duplicate = store.append_event(&accepted).expect("duplicate replay");
    assert_eq!(duplicate, after_accept);
    let replayed = store
        .rebuild_projection(submitted.task_id.as_str())
        .expect("replay");
    assert_eq!(replayed, after_accept);
    let bad_transition = event(
        &submitted.task_id,
        "event-2",
        TaskStatusV1::Leased,
        Some(ContentDigest::compute(b"wrong-previous")),
    );
    assert!(matches!(
        store.append_event(&bad_transition),
        Err(CollaborationStoreError::EventConflict(_))
    ));
    let mut conflict = envelope("same-key", 2);
    conflict.task_id = TaskId::new("different-task");
    assert!(matches!(
        store.append_task(&conflict),
        Err(CollaborationStoreError::IdempotencyConflict { .. })
    ));
    assert_eq!(first.status, TaskStatusV1::Submitted);
}
