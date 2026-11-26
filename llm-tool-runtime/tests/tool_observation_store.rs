#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
//! Durable ToolObservationMemory round-trip + scope isolation.

use llm_tool_runtime::semantic_memory::{ToolObservationMemory, ToolObservationMemoryRecordV1};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};

fn record(scope: &str, summary: &str) -> ToolObservationMemoryRecordV1 {
    ToolObservationMemoryRecordV1 {
        tool_name: "cargo".to_string(),
        invocation_id: "inv-1".to_string(),
        session_id: Some("sess-1".to_string()),
        scope: scope.to_string(),
        summary: summary.to_string(),
        status: "succeeded".to_string(),
        started_at: Some("2026-11-25T12:00:00Z".to_string()),
        completed_at: Some("2026-11-25T12:00:03Z".to_string()),
        receipt_id: Some("rcpt-1".to_string()),
    }
}

#[tokio::test]
async fn store_and_find_roundtrip_with_scope_isolation() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = MemoryConfig {
        base_dir: dir.path().join("mem"),
        ..Default::default()
    };
    let dims = cfg.embedding.dimensions;
    let store = MemoryStore::open_with_embedder(cfg, Box::new(MockEmbedder::new(dims))).unwrap();

    store
        .store_tool_observation(record("scope-a", "ran cargo build successfully"))
        .await
        .unwrap();

    let found = store
        .find_similar_tool_observations("cargo build", "scope-a", 5)
        .await
        .unwrap();
    assert_eq!(
        found.observations.len(),
        1,
        "expected one observation in scope-a"
    );
    assert!(found.observations[0].summary.contains("cargo build"));
    assert_eq!(found.observations[0].tool_name, "cargo");
    assert_eq!(found.observations[0].invocation_id, "inv-1");
    assert_eq!(found.observations[0].session_id.as_deref(), Some("sess-1"));
    assert_eq!(found.observations[0].status, "succeeded");
    assert_eq!(
        found.observations[0].started_at.as_deref(),
        Some("2026-11-25T12:00:00Z")
    );
    assert_eq!(
        found.observations[0].completed_at.as_deref(),
        Some("2026-11-25T12:00:03Z")
    );
    assert_eq!(found.observations[0].receipt_id.as_deref(), Some("rcpt-1"));
    assert_eq!(found.observations[0].scope, "scope-a");

    let other = store
        .find_similar_tool_observations("cargo build", "scope-b", 5)
        .await
        .unwrap();
    assert_eq!(other.observations.len(), 0, "scope-b must be isolated");
}

#[tokio::test]
async fn ordinary_facts_in_the_same_scope_are_not_tool_observations() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = MemoryConfig {
        base_dir: dir.path().join("mem"),
        ..Default::default()
    };
    let dims = cfg.embedding.dimensions;
    let store = MemoryStore::open_with_embedder(cfg, Box::new(MockEmbedder::new(dims))).unwrap();

    store
        .add_fact(
            "scope-a",
            "cargo build command is documented in the handbook",
            Some("handbook"),
            None,
        )
        .await
        .unwrap();
    store
        .add_fact(
            "scope-a",
            "cargo build is mentioned by a non-observation runtime record",
            Some("llm-tool-runtime"),
            Some(serde_json::json!({"kind": "other_runtime_record_v1"})),
        )
        .await
        .unwrap();
    store
        .store_tool_observation(record("scope-a", "ran cargo build successfully"))
        .await
        .unwrap();

    let found = store
        .find_similar_tool_observations("cargo build", "scope-a", 10)
        .await
        .unwrap();
    assert_eq!(found.observations.len(), 1);
    assert_eq!(found.observations[0].tool_name, "cargo");
    assert_eq!(found.observations[0].invocation_id, "inv-1");
}

#[tokio::test]
async fn zero_limit_is_rejected() {
    let dir = tempfile::tempdir().unwrap();
    let cfg = MemoryConfig {
        base_dir: dir.path().join("mem"),
        ..Default::default()
    };
    let dims = cfg.embedding.dimensions;
    let store = MemoryStore::open_with_embedder(cfg, Box::new(MockEmbedder::new(dims))).unwrap();
    let err = store
        .find_similar_tool_observations("x", "scope-a", 0)
        .await
        .unwrap_err();
    assert!(format!("{err}").contains("limit"));
}
