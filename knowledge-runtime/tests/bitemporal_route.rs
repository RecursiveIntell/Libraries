#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::field_reassign_with_default
)]
use bitemporal_runtime::BitemporalRecord;
use chrono::TimeZone;
use knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter;
use knowledge_runtime::{KnowledgeRuntime, RuntimeConfig, Scope};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};

fn rec(id: &str, valid: i64, recorded: i64, v: &str) -> BitemporalRecord<String> {
    BitemporalRecord {
        id: id.to_string(),
        valid_time: chrono::Utc.timestamp_opt(valid, 0).unwrap(),
        recorded_time: chrono::Utc.timestamp_opt(recorded, 0).unwrap(),
        value: v.to_string(),
    }
}

fn runtime(dir: &std::path::Path, records: Vec<BitemporalRecord<String>>) -> KnowledgeRuntime {
    let cfg = MemoryConfig {
        base_dir: dir.join("mem"),
        ..Default::default()
    };
    let dims = cfg.embedding.dimensions;
    let store = MemoryStore::open_with_embedder(cfg, Box::new(MockEmbedder::new(dims))).unwrap();
    let adapter = SemanticMemoryAdapter::new(store);
    let config = RuntimeConfig {
        default_scope: Scope::new("ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: Default::default(),
        strict_temporal: false,
        strict_scope: false,
    };
    KnowledgeRuntime::new(config, adapter)
        .unwrap()
        .with_bitemporal_records(records)
}

#[tokio::test]
async fn temporal_route_consumes_bitemporal_records() {
    let dir = tempfile::tempdir().unwrap();
    let rt = runtime(
        dir.path(),
        vec![rec("x", 100, 100, "old"), rec("x", 200, 200, "new")],
    );

    // As of valid=150, recorded=250 -> only "old" was valid.
    let (early, trace) = rt
        .query_temporal_with_trace(
            "what changed",
            None,
            None,
            "1970-01-01T00:02:30Z", // 150s
            "1970-01-01T00:04:10Z", // 250s
        )
        .await
        .expect("temporal query should succeed");
    assert!(!early.is_empty(), "expected a bitemporal result");
    assert_eq!(
        early[0].content, "old",
        "as-of must select the older version"
    );
    assert!(trace.temporal_mode.as_deref().is_some());

    // As of valid=250 -> "new".
    let (late, _) = rt
        .query_temporal_with_trace(
            "what changed",
            None,
            None,
            "1970-01-01T00:04:10Z",
            "1970-01-01T00:04:10Z",
        )
        .await
        .expect("temporal query should succeed");
    assert_eq!(
        late[0].content, "new",
        "as-of must select the newer version"
    );
}
