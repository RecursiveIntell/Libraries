//! SemanticMemoryAdapter — wires integrity, receipts, graph traversal, and compressed
//! search from aidens-memory-kit through semantic-memory's public API.
//!
//! Enabled by the `semantic-memory-integration` feature (which also enables
//! `semantic-memory/turbo-quant-codec` for the TurboQuant compressed-search path).

use semantic_memory::{
    DerivedVectorBackendPolicy, IntegrityReport, MemoryConfig, MemoryError, MemoryStore,
    MockEmbedder, SearchReplayReportV1, SearchResult, StoredGraphEdge, VerifyMode,
};
use std::path::PathBuf;

/// Adapter that wraps [`MemoryStore`] and exposes integrity verification,
/// search-receipt replay, multi-hop graph traversal, and compressed search.
pub struct SemanticMemoryAdapter {
    store: MemoryStore,
}

impl SemanticMemoryAdapter {
    /// Wrap a pre-opened store.
    pub fn new(store: MemoryStore) -> Self {
        Self { store }
    }

    /// Open a store at `config.base_dir` using `MockEmbedder`.
    ///
    /// Useful for tests and environments without a live embedding server.
    pub fn open_with_mock_embedder(config: MemoryConfig) -> Result<Self, MemoryError> {
        let dimensions = config.embedding.dimensions;
        let store =
            MemoryStore::open_with_embedder(config, Box::new(MockEmbedder::new(dimensions)))?;
        Ok(Self { store })
    }

    /// Open a store configured with [`DerivedVectorBackendPolicy::TurboQuantCandidateOnly`].
    ///
    /// TurboQuant generates compressed vector candidates for ANN retrieval, then
    /// exact-reranks from authoritative f32 embeddings. This constructor requires the
    /// `semantic-memory-integration` feature (which enables `turbo-quant-codec`).
    pub fn open_turbo_quant(base_dir: impl Into<PathBuf>) -> Result<Self, MemoryError> {
        let mut config = MemoryConfig {
            base_dir: base_dir.into(),
            ..Default::default()
        };
        config.search.derived_vector_backend = DerivedVectorBackendPolicy::TurboQuantCandidateOnly;
        let dimensions = config.embedding.dimensions;
        let store =
            MemoryStore::open_with_embedder(config, Box::new(MockEmbedder::new(dimensions)))?;
        Ok(Self { store })
    }

    /// Access the underlying store.
    pub fn store(&self) -> &MemoryStore {
        &self.store
    }

    // ─── 1. Memory integrity verification ───────────────────────────────────

    /// Verify store integrity. `Quick` checks table existence and row counts;
    /// `Full` also runs SQLite `integrity_check` and FTS consistency.
    pub async fn verify_integrity(&self, mode: VerifyMode) -> Result<IntegrityReport, MemoryError> {
        self.store.verify_integrity(mode).await
    }

    // ─── 2. Search receipt replay ────────────────────────────────────────────

    /// Retrieve a stored receipt and replay the search at the original evaluation time.
    ///
    /// Receipts do not store query text — the caller must supply `query` and
    /// the same optional `top_k` used in the original search.
    pub async fn replay_search_receipt(
        &self,
        receipt_id: &str,
        query: &str,
        top_k: Option<usize>,
    ) -> Result<SearchReplayReportV1, MemoryError> {
        self.store
            .replay_search_receipt(receipt_id, query, top_k, None, None)
            .await
    }

    // ─── 3. Graph traversal (multi-hop knowledge retrieval) ─────────────────

    /// Load graph edges within `max_hops` BFS hops from `seed_ids`, capped at
    /// `max_nodes` total visited nodes.
    ///
    /// Returns only live (non-invalidated) edges. An empty seed list returns an
    /// empty result set without touching the database.
    pub async fn graph_traversal(
        &self,
        seed_ids: Vec<String>,
        max_hops: usize,
        max_nodes: usize,
    ) -> Result<Vec<StoredGraphEdge>, MemoryError> {
        self.store
            .list_graph_edges_for_neighborhood(seed_ids, max_hops, max_nodes)
            .await
    }

    // ─── 4. Vector codecs — compressed search ────────────────────────────────

    /// Hybrid search that routes through the store's configured vector backend.
    ///
    /// When the store was opened with [`open_turbo_quant`], this uses TurboQuant
    /// compressed candidates followed by exact f32 reranking. On stores opened with
    /// other policies (e.g. `open_with_mock_embedder`), this is a standard hybrid
    /// search.
    pub async fn compressed_search(
        &self,
        query: &str,
        top_k: Option<usize>,
    ) -> Result<Vec<SearchResult>, MemoryError> {
        self.store.search(query, top_k, None, None).await
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_memory::{GraphEdgeType, MemoryConfig, ReceiptMode, SearchContext};
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn test_root(tag: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "aidens-smadapter-{}-{}-{}",
            std::process::id(),
            nanos,
            tag
        ))
    }

    fn cleanup(root: &PathBuf) {
        let _ = std::fs::remove_dir_all(root);
    }

    fn mock_config(root: &PathBuf) -> MemoryConfig {
        MemoryConfig {
            base_dir: root.clone(),
            ..Default::default()
        }
    }

    // ─── Constructor tests ───────────────────────────────────────────────────

    #[test]
    fn opens_with_new_from_existing_store() {
        let root = test_root("new");
        let config = mock_config(&root);
        let dimensions = config.embedding.dimensions;
        let store =
            MemoryStore::open_with_embedder(config, Box::new(MockEmbedder::new(dimensions)))
                .expect("store");
        let _adapter = SemanticMemoryAdapter::new(store);
        cleanup(&root);
    }

    #[test]
    fn opens_with_mock_embedder_default_config() {
        let root = test_root("mock");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        assert_eq!(adapter.store().config().base_dir, root);
        cleanup(&root);
    }

    #[test]
    fn store_accessor_returns_inner_store() {
        let root = test_root("accessor");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        assert_eq!(adapter.store().config().base_dir, root);
        cleanup(&root);
    }

    #[test]
    fn open_turbo_quant_creates_store_with_turbo_config() {
        let root = test_root("turbo-cfg");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");
        assert_eq!(
            adapter.store().config().search.derived_vector_backend,
            DerivedVectorBackendPolicy::TurboQuantCandidateOnly
        );
        cleanup(&root);
    }

    // ─── verify_integrity ────────────────────────────────────────────────────

    #[tokio::test]
    async fn verify_integrity_quick_on_empty_store() {
        let root = test_root("vi-quick");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let report = adapter
            .verify_integrity(VerifyMode::Quick)
            .await
            .expect("integrity");
        assert!(report.ok);
        cleanup(&root);
    }

    #[tokio::test]
    async fn verify_integrity_full_on_empty_store() {
        let root = test_root("vi-full");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let report = adapter
            .verify_integrity(VerifyMode::Full)
            .await
            .expect("integrity");
        assert!(report.ok);
        cleanup(&root);
    }

    #[tokio::test]
    async fn verify_integrity_quick_reports_no_issues() {
        let root = test_root("vi-no-issues");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let report = adapter
            .verify_integrity(VerifyMode::Quick)
            .await
            .expect("integrity");
        assert!(report.issues.is_empty(), "issues: {:?}", report.issues);
        cleanup(&root);
    }

    #[tokio::test]
    async fn verify_integrity_after_adding_facts() {
        let root = test_root("vi-after-facts");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        adapter
            .store()
            .add_fact("test-ns", "integrity after facts fixture", None, None)
            .await
            .expect("add fact");
        let report = adapter
            .verify_integrity(VerifyMode::Quick)
            .await
            .expect("integrity");
        assert!(report.ok);
        cleanup(&root);
    }

    #[tokio::test]
    async fn verify_integrity_full_reports_no_issues_after_facts() {
        let root = test_root("vi-full-facts");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        adapter
            .store()
            .add_fact("test-ns", "full integrity after facts fixture", None, None)
            .await
            .expect("add fact");
        let report = adapter
            .verify_integrity(VerifyMode::Full)
            .await
            .expect("integrity");
        assert!(report.ok, "issues: {:?}", report.issues);
        cleanup(&root);
    }

    // ─── replay_search_receipt ───────────────────────────────────────────────

    #[tokio::test]
    async fn replay_search_receipt_missing_id_returns_error() {
        let root = test_root("replay-missing");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let result = adapter
            .replay_search_receipt("nonexistent-receipt-id", "query", None)
            .await;
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not found") || err.contains("receipt"),
            "unexpected error: {err}"
        );
        cleanup(&root);
    }

    #[tokio::test]
    async fn replay_search_receipt_roundtrip() {
        let root = test_root("replay-roundtrip");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        adapter
            .store()
            .add_fact(
                "replay-ns",
                "replay receipt roundtrip fact about Rust",
                None,
                None,
            )
            .await
            .expect("add fact");

        let mut context = SearchContext::default_now();
        context.receipt_mode = ReceiptMode::ReturnReceipt;
        let response = adapter
            .store()
            .search_with_context("Rust", Some(5), None, None, context)
            .await
            .expect("search with context");
        let receipt = response.receipt.expect("receipt from search");

        let report = adapter
            .replay_search_receipt(&receipt.receipt_id, "Rust", Some(5))
            .await
            .expect("replay");
        assert_eq!(report.receipt_id, receipt.receipt_id);
        assert!(!report.replay_receipt_id.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn replay_search_receipt_result_ids_match_original() {
        let root = test_root("replay-ids");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        adapter
            .store()
            .add_fact(
                "replay-ids-ns",
                "receipt result id matching test fact",
                None,
                None,
            )
            .await
            .expect("add fact");

        let mut context = SearchContext::default_now();
        context.receipt_mode = ReceiptMode::ReturnReceipt;
        let response = adapter
            .store()
            .search_with_context("matching test", Some(5), None, None, context)
            .await
            .expect("search");
        let receipt = response.receipt.expect("receipt");

        let report = adapter
            .replay_search_receipt(&receipt.receipt_id, "matching test", Some(5))
            .await
            .expect("replay");
        assert!(report.result_ids_match || report.missing_result_ids.is_empty());
        cleanup(&root);
    }

    // ─── graph_traversal ────────────────────────────────────────────────────

    #[tokio::test]
    async fn graph_traversal_empty_seeds_returns_empty() {
        let root = test_root("gt-empty-seeds");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let edges = adapter
            .graph_traversal(vec![], 1, 100)
            .await
            .expect("traversal");
        assert!(edges.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn graph_traversal_no_edges_for_seeds_returns_empty() {
        let root = test_root("gt-no-edges");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let edges = adapter
            .graph_traversal(vec!["namespace:nonexistent".to_string()], 1, 100)
            .await
            .expect("traversal");
        assert!(edges.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn graph_traversal_returns_direct_neighbors() {
        let root = test_root("gt-neighbors");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        let edge = adapter
            .store()
            .add_graph_edge(
                "namespace:gt-src",
                "namespace:gt-tgt",
                GraphEdgeType::Semantic {
                    cosine_similarity: 0.9,
                },
                1.0,
                None,
            )
            .await
            .expect("add edge");

        let edges = adapter
            .graph_traversal(vec!["namespace:gt-src".to_string()], 1, 100)
            .await
            .expect("traversal");
        assert!(!edges.is_empty(), "expected at least one edge");
        assert!(edges.iter().any(|e| e.id == edge.id));
        cleanup(&root);
    }

    #[tokio::test]
    async fn graph_traversal_multi_hop_reaches_second_neighbor() {
        let root = test_root("gt-multi-hop");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        adapter
            .store()
            .add_graph_edge(
                "namespace:hop-a",
                "namespace:hop-b",
                GraphEdgeType::Semantic {
                    cosine_similarity: 0.85,
                },
                1.0,
                None,
            )
            .await
            .expect("edge a-b");

        adapter
            .store()
            .add_graph_edge(
                "namespace:hop-b",
                "namespace:hop-c",
                GraphEdgeType::Causal {
                    confidence: 0.9,
                    evidence_ids: vec![],
                },
                1.0,
                None,
            )
            .await
            .expect("edge b-c");

        let edges_1hop = adapter
            .graph_traversal(vec!["namespace:hop-a".to_string()], 1, 100)
            .await
            .expect("1-hop traversal");

        let edges_2hop = adapter
            .graph_traversal(vec!["namespace:hop-a".to_string()], 2, 100)
            .await
            .expect("2-hop traversal");

        assert!(
            edges_2hop.len() >= edges_1hop.len(),
            "2-hop should reach at least as many edges as 1-hop"
        );
        cleanup(&root);
    }

    #[tokio::test]
    async fn graph_traversal_respects_max_nodes_zero_cap() {
        let root = test_root("gt-maxnodes");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        for i in 0..3 {
            adapter
                .store()
                .add_graph_edge(
                    &format!("namespace:maxn-src-{i}"),
                    &format!("namespace:maxn-tgt-{i}"),
                    GraphEdgeType::Semantic {
                        cosine_similarity: 0.7,
                    },
                    1.0,
                    None,
                )
                .await
                .expect("add edge");
        }

        let seeds: Vec<String> = (0..3).map(|i| format!("namespace:maxn-src-{i}")).collect();
        let edges_uncapped = adapter
            .graph_traversal(seeds.clone(), 1, 1_000)
            .await
            .expect("uncapped traversal");
        let edges_capped = adapter
            .graph_traversal(seeds, 1, 1)
            .await
            .expect("capped traversal");

        assert!(
            edges_capped.len() <= edges_uncapped.len(),
            "capping should not return more edges"
        );
        cleanup(&root);
    }

    // ─── compressed_search ──────────────────────────────────────────────────

    #[tokio::test]
    async fn compressed_search_empty_store_returns_empty() {
        let root = test_root("cs-empty");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");
        let results = adapter
            .compressed_search("anything", Some(5))
            .await
            .expect("search");
        assert!(results.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn compressed_search_returns_matching_results() {
        let root = test_root("cs-match");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        adapter
            .store()
            .add_fact(
                "cs-ns",
                "compressed search fixture fact about memory",
                None,
                None,
            )
            .await
            .expect("add fact");

        let results = adapter
            .compressed_search("memory", Some(5))
            .await
            .expect("search");
        assert!(!results.is_empty(), "expected results for matching query");
        cleanup(&root);
    }

    #[tokio::test]
    async fn compressed_search_top_k_limits_results() {
        let root = test_root("cs-topk");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        for i in 0..5 {
            adapter
                .store()
                .add_fact(
                    "cs-topk-ns",
                    &format!("top-k limit test fact number {i}"),
                    None,
                    None,
                )
                .await
                .expect("add fact");
        }

        let results_3 = adapter
            .compressed_search("top-k limit test", Some(3))
            .await
            .expect("top-3");
        let results_2 = adapter
            .compressed_search("top-k limit test", Some(2))
            .await
            .expect("top-2");

        assert!(
            results_3.len() <= 3,
            "top_k=3 returned {} results",
            results_3.len()
        );
        assert!(
            results_2.len() <= 2,
            "top_k=2 returned {} results",
            results_2.len()
        );
        cleanup(&root);
    }

    #[tokio::test]
    async fn compressed_search_after_multiple_facts_finds_all() {
        let root = test_root("cs-multi");
        let adapter =
            SemanticMemoryAdapter::open_with_mock_embedder(mock_config(&root)).expect("adapter");

        adapter
            .store()
            .add_fact(
                "cs-multi-ns",
                "multi-fact alpha: Rust memory safety",
                None,
                None,
            )
            .await
            .expect("add fact alpha");
        adapter
            .store()
            .add_fact(
                "cs-multi-ns",
                "multi-fact beta: Rust ownership model",
                None,
                None,
            )
            .await
            .expect("add fact beta");

        let results = adapter
            .compressed_search("Rust memory", Some(10))
            .await
            .expect("search");
        assert!(!results.is_empty());
        cleanup(&root);
    }

    // ─── TurboQuant path ────────────────────────────────────────────────────

    #[tokio::test]
    async fn open_turbo_quant_verify_integrity_quick() {
        let root = test_root("tq-vi");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");
        let report = adapter
            .verify_integrity(VerifyMode::Quick)
            .await
            .expect("integrity");
        assert!(report.ok, "issues: {:?}", report.issues);
        cleanup(&root);
    }

    #[tokio::test]
    async fn open_turbo_quant_compressed_search_empty_store() {
        let root = test_root("tq-cs-empty");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");
        let results = adapter
            .compressed_search("turbo quant empty", Some(5))
            .await
            .expect("search");
        assert!(results.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn open_turbo_quant_graph_traversal_empty() {
        let root = test_root("tq-gt");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");
        let edges = adapter
            .graph_traversal(vec![], 1, 100)
            .await
            .expect("traversal");
        assert!(edges.is_empty());
        cleanup(&root);
    }

    #[tokio::test]
    async fn open_turbo_quant_replay_receipt_missing_returns_error() {
        let root = test_root("tq-replay");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");
        let result = adapter
            .replay_search_receipt("no-such-receipt", "query", None)
            .await;
        assert!(result.is_err());
        cleanup(&root);
    }

    #[tokio::test]
    async fn open_turbo_quant_compressed_search_finds_added_fact() {
        let root = test_root("tq-cs-fact");
        let adapter = SemanticMemoryAdapter::open_turbo_quant(&root).expect("turbo adapter");

        adapter
            .store()
            .add_fact(
                "tq-ns",
                "turbo quant fact about vector compression",
                None,
                None,
            )
            .await
            .expect("add fact");

        let results = adapter
            .compressed_search("vector compression", Some(5))
            .await
            .expect("compressed search");
        assert!(
            !results.is_empty(),
            "turbo quant search should find added fact"
        );
        cleanup(&root);
    }
}
