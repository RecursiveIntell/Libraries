#![allow(clippy::expect_used)]

//! LIB-HA-002: Verify that the hybrid search path uses RRF fusion (BM25 + HNSW)
//! when embeddings are available, rather than falling through to projection-only
//! substring matching.

use knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter;
use knowledge_runtime::config::ProjectionConfig;
use knowledge_runtime::{KnowledgeRuntime, RuntimeConfig, Scope};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder, SearchConfig};
use tempfile::TempDir;

fn open_store(dir: &TempDir) -> MemoryStore {
    let config = MemoryConfig {
        base_dir: dir.path().to_path_buf(),
        search: SearchConfig {
            min_similarity: -1.0,
            ..Default::default()
        },
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).expect("open store")
}

fn test_runtime_config() -> RuntimeConfig {
    RuntimeConfig {
        default_scope: Scope::new("test-ns"),
        query: Default::default(),
        entity: Default::default(),
        projection: ProjectionConfig {
            staleness_threshold_secs: 3600,
            import_staleness_threshold_secs: 0,
            persist: false,
        },
        strict_temporal: false,
        strict_scope: false,
    }
}

/// Ingest facts through semantic-memory (which indexes in both FTS5 and HNSW),
/// then query through the runtime and verify that results have RRF fusion
/// provenance (bm25_rank and/or vector_rank populated), not just projection
/// substring matches.
#[tokio::test]
async fn hybrid_search_uses_rrf_fusion_not_projection_only() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    // Ingest 3 facts through semantic-memory (adds to FTS5 + HNSW)
    store
        .add_fact(
            "test-ns",
            "Rust programming language is systems-level",
            None,
            None,
        )
        .await
        .unwrap();
    store
        .add_fact(
            "test-ns",
            "Rust has a borrow checker for memory safety",
            None,
            None,
        )
        .await
        .unwrap();
    store
        .add_fact(
            "test-ns",
            "Cargo is the Rust package manager and build tool",
            None,
            None,
        )
        .await
        .unwrap();

    let adapter = SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, _trace) = runtime
        .query("Rust programming", Some(&scope))
        .await
        .unwrap();

    assert!(!results.is_empty(), "query should return results");

    // Results from hybrid search (RRF fusion) have bm25_rank and/or vector_rank.
    // Results from projection-only substring matching have neither.
    let has_rrf_provenance = results
        .iter()
        .any(|r| r.bm25_rank.is_some() || r.vector_rank.is_some());

    assert!(
        has_rrf_provenance,
        "At least one result must have bm25_rank or vector_rank set, \
         proving hybrid RRF fusion was used instead of projection-only. \
         Got results: {:?}",
        results
            .iter()
            .map(|r| (&r.content, r.bm25_rank, r.vector_rank))
            .collect::<Vec<_>>()
    );
}

/// Verify that score ordering is correct: higher-scored results come first.
#[tokio::test]
async fn hybrid_search_results_are_score_ordered() {
    let dir = TempDir::new().unwrap();
    let store = open_store(&dir);

    store
        .add_fact("test-ns", "Rust programming language overview", None, None)
        .await
        .unwrap();
    store
        .add_fact(
            "test-ns",
            "Python is a dynamic programming language",
            None,
            None,
        )
        .await
        .unwrap();
    store
        .add_fact("test-ns", "Cooking recipes for pasta dishes", None, None)
        .await
        .unwrap();

    let adapter = SemanticMemoryAdapter::new(store.clone());
    let config = test_runtime_config();
    let runtime = KnowledgeRuntime::new(config, adapter).unwrap();

    let scope = Scope::new("test-ns");
    let (results, _trace) = runtime
        .query("Rust programming", Some(&scope))
        .await
        .unwrap();

    assert!(!results.is_empty(), "query should return results");

    // Verify descending score order
    for window in results.windows(2) {
        assert!(
            window[0].score >= window[1].score,
            "Results must be in descending score order: {} >= {} failed for '{}' vs '{}'",
            window[0].score,
            window[1].score,
            window[0].content,
            window[1].content,
        );
    }
}
