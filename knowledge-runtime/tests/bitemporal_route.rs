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

#[test]
fn explicit_reference_snapshot_respects_both_time_coordinates() {
    let dir = tempfile::tempdir().unwrap();
    let rt = runtime(
        dir.path(),
        vec![rec("x", 100, 100, "old"), rec("x", 200, 200, "new")],
    );
    let cutoff = |seconds| chrono::Utc.timestamp_opt(seconds, 0).unwrap();
    for (valid, recorded, expected) in [(150, 250, "old"), (250, 150, "old"), (250, 250, "new")] {
        let hits = rt.reference_bitemporal_as_of(cutoff(valid), cutoff(recorded));
        assert_eq!(hits.len(), 1);
        assert_eq!(hits[0].id, "x");
        assert_eq!(hits[0].value, expected);
    }
}

#[tokio::test]
async fn reference_snapshot_does_not_fabricate_search_hits_across_scopes() {
    let dir = tempfile::tempdir().unwrap();
    let rt = runtime(
        dir.path(),
        vec![
            rec("amber", 100, 100, "amber deployment status"),
            rec("indigo", 100, 100, "indigo inventory status"),
            rec("amber", 90, 200, "amber retroactive correction"),
        ],
    );
    let default = Scope::new("ns");
    let other = Scope::new("unrelated-namespace");

    // The adapter has no imports or facts in either scope. A reference vector
    // has no scope, search ranking or dereferenceable fact IDs.
    for (scope, query, recorded) in [
        (&default, "amber deployment", "1970-01-01T00:02:30Z"),
        (&other, "indigo inventory", "1970-01-01T00:04:10Z"),
    ] {
        let (hits, trace) = rt
            .query_temporal_with_trace(query, Some(scope), None, "1970-01-01T00:02:30Z", recorded)
            .await
            .expect("permissive route explicitly degrades when imports are absent");
        assert_eq!(
            hits.len(),
            trace.merged_result_count,
            "trace must describe the results actually returned"
        );
        assert!(
            hits.is_empty(),
            "unscoped reference records are not search hits"
        );
        assert_eq!(trace.temporal_mode.as_deref(), Some("downgraded"));
    }
}
