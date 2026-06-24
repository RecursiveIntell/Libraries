//! Thin memory facade over Forge, bridge, semantic-memory, and knowledge-runtime.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

#[cfg(feature = "semantic-memory-integration")]
pub mod semantic_adapter;

#[cfg(feature = "semantic-memory-integration")]
pub use semantic_adapter::SemanticMemoryAdapter;

pub mod canonical_stack {
    pub use forge_memory_bridge::{
        transform_envelope_v3, BridgeError, ProjectionImportBatchV3,
        PROJECTION_IMPORT_BATCH_V3_SCHEMA,
    };
    pub use knowledge_runtime::adapters::semantic_memory::SemanticMemoryAdapter;
    pub use knowledge_runtime::{
        KnowledgeRuntime, QueryTrace, RuntimeConfig, RuntimeError, RuntimeQueryProvenanceV1, Scope,
    };
    pub use semantic_memory::{
        MemoryConfig as CanonicalMemoryConfig, MemoryError as CanonicalMemoryError,
        MemoryStore as CanonicalMemoryStore, MockEmbedder, ProjectionImportResult,
        AddGraphEdgeParams, ExplainedSearchResponse, IntegrityReport, MemoryStats, ReceiptMode,
        ReconcileAction, SearchContext, SearchResponse, SearchResult, StoredGraphEdge,
        VectorSearchReceiptV1, VerifyMode,
    };
    pub use semantic_memory_forge::{
        EvidenceBundle, ExportClaim, ExportEnvelopeV2, ExportEnvelopeV3, ExportRecord,
        ExportRecordV3, ForgeExportMeta, ForgeToolReceiptV2, EXPORT_ENVELOPE_V2_SCHEMA,
        EXPORT_ENVELOPE_V3_SCHEMA,
    };
    pub use stack_ids::{ClaimId, ClaimVersionId, EntityId, EnvelopeId, ScopeKey, TraceCtx};

    pub fn transform_forge_export(
        envelope: &ExportEnvelopeV3,
    ) -> Result<ProjectionImportBatchV3, BridgeError> {
        transform_envelope_v3(envelope)
    }

    pub async fn import_projection_batch(
        store: &CanonicalMemoryStore,
        batch: &ProjectionImportBatchV3,
    ) -> Result<ProjectionImportResult, CanonicalMemoryError> {
        store.import_projection_batch(batch).await
    }
}

pub use canonical_stack::{
    CanonicalMemoryConfig, CanonicalMemoryError, CanonicalMemoryStore, ProjectionImportBatchV3,
    ProjectionImportResult,
};

pub struct CanonicalMemoryAdapter {
    store: canonical_stack::CanonicalMemoryStore,
    runtime: canonical_stack::KnowledgeRuntime,
}

#[derive(Debug, Error)]
pub enum MemoryProfileError {
    #[error("invalid memory backend profile: {0}")]
    InvalidProfile(String),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct SemanticMemoryBackendProfileV1 {
    pub schema: String,
    pub owner: String,
    pub authoritative_store: String,
    pub derived_candidate_backend: String,
    pub exact_f32_rerank_required: bool,
    pub direct_provekv_dependency_allowed: bool,
    pub recall_integration_allowed: bool,
    pub missing_generation_behavior: String,
}

impl SemanticMemoryBackendProfileV1 {
    pub const SCHEMA: &'static str = "AiDENsSemanticMemoryBackendProfileV1";

    pub fn provekv_derived_candidate() -> Self {
        Self {
            schema: Self::SCHEMA.into(),
            owner: "semantic-memory".into(),
            authoritative_store: "semantic-memory sqlite f32 embeddings".into(),
            derived_candidate_backend: "provekv_pool_candidate_then_exact_f32".into(),
            exact_f32_rerank_required: true,
            direct_provekv_dependency_allowed: false,
            recall_integration_allowed: false,
            missing_generation_behavior: "receipted_f32_fallback".into(),
        }
    }

    pub fn validate(&self) -> Result<(), MemoryProfileError> {
        if self.schema != Self::SCHEMA {
            return Err(MemoryProfileError::InvalidProfile(
                "unexpected semantic memory backend profile schema".into(),
            ));
        }
        if self.owner != "semantic-memory" {
            return Err(MemoryProfileError::InvalidProfile(
                "semantic-memory must own canonical memory truth".into(),
            ));
        }
        if !self.exact_f32_rerank_required {
            return Err(MemoryProfileError::InvalidProfile(
                "proveKV/poly-kv candidate profiles must require exact f32 rerank".into(),
            ));
        }
        if self.direct_provekv_dependency_allowed || self.recall_integration_allowed {
            return Err(MemoryProfileError::InvalidProfile(
                "AiDENs profiles must not directly depend on proveKV or wire Recall".into(),
            ));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryGroundingBackpointerV1 {
    pub owner_crate: String,
    pub artifact_type: String,
    pub relationship: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MemoryGroundingEvidenceV1 {
    pub schema: String,
    pub artifact_kind: String,
    pub ownership: String,
    pub support_tier: String,
    pub semantic_status: String,
    pub semantic_exactness: String,
    pub proof_debt_status: String,
    pub contradiction_status: String,
    pub view_disclosure_status: String,
    pub execution_contamination_status: String,
    pub adapter_route: String,
    pub memory_mode: String,
    pub query: String,
    pub result_count: usize,
    pub import_record_count: usize,
    pub trace_ctx: String,
    pub canonical_backpointers: Vec<MemoryGroundingBackpointerV1>,
    pub degradation: Vec<String>,
    pub widening: Vec<String>,
    pub local_truth_store: bool,
    pub promotion_eligible: bool,
    pub known_limits: Vec<String>,
}

impl MemoryGroundingEvidenceV1 {
    pub const SCHEMA: &'static str = "AiDENsMemoryGroundingEvidenceV1";

    pub fn canonical_seam(
        memory_mode: impl Into<String>,
        query: impl Into<String>,
        result_count: usize,
        import_record_count: usize,
        trace_ctx: impl Into<String>,
        degradation: Vec<String>,
        widening: Vec<String>,
    ) -> Self {
        let semantic_status = if degradation.is_empty() && widening.is_empty() {
            "exact_check"
        } else {
            "degraded_exact_check"
        };
        let semantic_exactness = if degradation.is_empty() && widening.is_empty() {
            "exact"
        } else {
            "degraded"
        };
        let view_disclosure_status = if widening.is_empty() {
            "no-widening"
        } else {
            "widening-disclosed"
        };
        Self {
            schema: Self::SCHEMA.into(),
            artifact_kind: "local_operator_memory_grounding_evidence".into(),
            ownership: "AiDENs-local operator evidence; canonical memory truth remains in semantic-memory-forge, forge-memory-bridge, semantic-memory, and knowledge-runtime.".into(),
            support_tier: "supported-local".into(),
            semantic_status: semantic_status.into(),
            semantic_exactness: semantic_exactness.into(),
            proof_debt_status: "none-declared".into(),
            contradiction_status: "none-declared".into(),
            view_disclosure_status: view_disclosure_status.into(),
            execution_contamination_status: "none-declared".into(),
            adapter_route: "semantic-memory-forge -> forge-memory-bridge -> semantic-memory -> knowledge-runtime".into(),
            memory_mode: memory_mode.into(),
            query: query.into(),
            result_count,
            import_record_count,
            trace_ctx: trace_ctx.into(),
            canonical_backpointers: canonical_memory_backpointers(),
            degradation,
            widening,
            local_truth_store: false,
            promotion_eligible: semantic_status == "exact_check",
            known_limits: vec![
                "This receipt is an AiDENs-local display artifact, not canonical memory truth.".into(),
                "AiDENs does not persist a local memory database as an authority source.".into(),
                "Replay must use the canonical export, bridge, storage, and runtime owner crates.".into(),
            ],
        }
    }

    pub fn with_proof_debt(mut self, proof_debt_status: impl Into<String>) -> Self {
        self.proof_debt_status = proof_debt_status.into();
        self.promotion_eligible = false;
        self
    }

    pub fn with_contradiction(mut self, contradiction_status: impl Into<String>) -> Self {
        self.contradiction_status = contradiction_status.into();
        self.semantic_exactness = "refuted".into();
        self.promotion_eligible = false;
        self
    }

    pub fn with_execution_contamination(
        mut self,
        execution_contamination_status: impl Into<String>,
    ) -> Self {
        self.execution_contamination_status = execution_contamination_status.into();
        if self.semantic_exactness != "refuted" {
            self.semantic_exactness = "degraded".into();
        }
        self.promotion_eligible = false;
        self
    }

    pub fn to_receipt_line(&self) -> Result<String, serde_json::Error> {
        serde_json::to_string(self)
    }
}

pub fn canonical_memory_backpointers() -> Vec<MemoryGroundingBackpointerV1> {
    vec![
        MemoryGroundingBackpointerV1 {
            owner_crate: "semantic-memory-forge".into(),
            artifact_type: "ExportEnvelopeV3".into(),
            relationship: "canonical-raw-evidence-owner".into(),
        },
        MemoryGroundingBackpointerV1 {
            owner_crate: "forge-memory-bridge".into(),
            artifact_type: "ProjectionImportBatchV3".into(),
            relationship: "canonical-bridge-transform-owner".into(),
        },
        MemoryGroundingBackpointerV1 {
            owner_crate: "semantic-memory".into(),
            artifact_type: "MemoryStore".into(),
            relationship: "canonical-projected-memory-owner".into(),
        },
        MemoryGroundingBackpointerV1 {
            owner_crate: "knowledge-runtime".into(),
            artifact_type: "QueryTrace".into(),
            relationship: "canonical-runtime-view-owner".into(),
        },
    ]
}

impl CanonicalMemoryAdapter {
    pub fn new(
        store: canonical_stack::CanonicalMemoryStore,
        runtime_config: canonical_stack::RuntimeConfig,
    ) -> Result<Self, canonical_stack::RuntimeError> {
        let runtime_adapter = canonical_stack::SemanticMemoryAdapter::new(store.clone());
        let runtime = canonical_stack::KnowledgeRuntime::new(runtime_config, runtime_adapter)?;
        Ok(Self { store, runtime })
    }

    pub fn open(
        memory_config: canonical_stack::CanonicalMemoryConfig,
        runtime_config: canonical_stack::RuntimeConfig,
    ) -> Result<Self, CanonicalMemoryOpenError> {
        let store = canonical_stack::CanonicalMemoryStore::open(memory_config)?;
        Self::new(store, runtime_config).map_err(CanonicalMemoryOpenError::Runtime)
    }

    pub fn open_with_mock_embedder(
        memory_config: canonical_stack::CanonicalMemoryConfig,
        runtime_config: canonical_stack::RuntimeConfig,
    ) -> Result<Self, CanonicalMemoryOpenError> {
        let dimensions = memory_config.embedding.dimensions;
        let store = canonical_stack::CanonicalMemoryStore::open_with_embedder(
            memory_config,
            Box::new(canonical_stack::MockEmbedder::new(dimensions)),
        )?;
        Self::new(store, runtime_config).map_err(CanonicalMemoryOpenError::Runtime)
    }

    pub fn store(&self) -> &canonical_stack::CanonicalMemoryStore {
        &self.store
    }

    pub fn runtime(&self) -> &canonical_stack::KnowledgeRuntime {
        &self.runtime
    }

    pub async fn import_forge_export(
        &self,
        envelope: &canonical_stack::ExportEnvelopeV3,
    ) -> Result<canonical_stack::ProjectionImportResult, CanonicalMemoryAdapterError> {
        let batch = canonical_stack::transform_forge_export(envelope)?;
        canonical_stack::import_projection_batch(&self.store, &batch)
            .await
            .map_err(CanonicalMemoryAdapterError::Memory)
    }

    pub async fn query(
        &self,
        query: &str,
        scope: Option<&knowledge_runtime::Scope>,
    ) -> Result<
        (
            Vec<canonical_stack::SearchResult>,
            canonical_stack::QueryTrace,
        ),
        canonical_stack::RuntimeError,
    > {
        self.runtime.query(query, scope).await
    }

    pub async fn query_temporal(
        &self,
        query: &str,
        scope: Option<&knowledge_runtime::Scope>,
        valid_at: &str,
        recorded_at_or_before: &str,
    ) -> Result<
        (
            Vec<canonical_stack::SearchResult>,
            canonical_stack::QueryTrace,
        ),
        canonical_stack::RuntimeError,
    > {
        self.runtime
            .query_temporal(query, scope, valid_at, recorded_at_or_before)
            .await
    }

    pub async fn search(
        &self,
        query: &str,
        namespaces: Option<&[String]>,
        top_k: Option<usize>,
    ) -> Result<Vec<canonical_stack::SearchResult>, canonical_stack::CanonicalMemoryError> {
        let namespaces = namespaces.map(|namespaces| {
            namespaces.iter().map(String::as_str).collect::<Vec<&str>>()
        });
        self.store
            .search(query, top_k, namespaces.as_deref(), None)
            .await
    }

    pub async fn search_with_receipt(
        &self,
        query: &str,
        namespaces: Option<&[String]>,
        top_k: Option<usize>,
    ) -> Result<
        (
            Vec<canonical_stack::SearchResult>,
            canonical_stack::VectorSearchReceiptV1,
        ),
        canonical_stack::CanonicalMemoryError,
    > {
        let namespaces = namespaces.map(|namespaces| {
            namespaces.iter().map(String::as_str).collect::<Vec<&str>>()
        });
        let mut context = canonical_stack::SearchContext::default_now();
        context.receipt_mode = canonical_stack::ReceiptMode::ReturnReceipt;

        let response = self
            .store
            .search_with_context(query, top_k, namespaces.as_deref(), None, context)
            .await?;
        let receipt = response.receipt.ok_or_else(|| {
            canonical_stack::CanonicalMemoryError::SearchReceiptNotFound {
                receipt_id: "search_with_receipt".to_string(),
            }
        })?;
        Ok((response.results, receipt))
    }

    pub async fn search_fts_only(
        &self,
        query: &str,
        top_k: Option<usize>,
    ) -> Result<Vec<canonical_stack::SearchResult>, canonical_stack::CanonicalMemoryError> {
        self.store
            .search_fts_only(query, top_k, None, None)
            .await
    }

    pub async fn search_vector_only(
        &self,
        query: &str,
        top_k: Option<usize>,
    ) -> Result<Vec<canonical_stack::SearchResult>, canonical_stack::CanonicalMemoryError> {
        self.store
            .search_vector_only(query, top_k, None, None)
            .await
    }

    pub async fn search_explained(
        &self,
        query: &str,
        top_k: Option<usize>,
    ) -> Result<canonical_stack::ExplainedSearchResponse, canonical_stack::CanonicalMemoryError> {
        self.store
            .search_explained_with_context(
                query,
                top_k,
                None,
                None,
                canonical_stack::SearchContext::default_now(),
            )
            .await
    }

    pub async fn get_search_receipt(
        &self,
        receipt_id: &str,
    ) -> Result<Option<canonical_stack::VectorSearchReceiptV1>, canonical_stack::CanonicalMemoryError> {
        self.store.get_search_receipt(receipt_id).await
    }

    pub async fn replay_search_receipt(
        &self,
        receipt: canonical_stack::VectorSearchReceiptV1,
    ) -> Result<canonical_stack::SearchResponse, canonical_stack::CanonicalMemoryError> {
        let report = self
            .store
            .replay_search_receipt(&receipt.receipt_id, "", None, None, None)
            .await?;
        Ok(canonical_stack::SearchResponse {
            results: Vec::new(),
            receipt: Some(report.replay_receipt),
        })
    }

    pub async fn add_graph_edge(
        &self,
        params: canonical_stack::AddGraphEdgeParams,
    ) -> Result<canonical_stack::StoredGraphEdge, canonical_stack::CanonicalMemoryError> {
        self.store
            .add_graph_edge(
                &params.source,
                &params.target,
                params.edge_type,
                params.weight,
                params.metadata,
            )
            .await
    }

    pub async fn list_graph_edges(
        &self,
        node_id: &str,
    ) -> Result<Vec<canonical_stack::StoredGraphEdge>, canonical_stack::CanonicalMemoryError> {
        self.store.list_graph_edges_for_node(node_id).await
    }

    pub async fn count_graph_edges(&self) -> Result<usize, canonical_stack::CanonicalMemoryError> {
        self.store.count_graph_edges().await
    }

    pub async fn verify_integrity(
        &self,
        mode: canonical_stack::VerifyMode,
    ) -> Result<canonical_stack::IntegrityReport, canonical_stack::CanonicalMemoryError> {
        self.store.verify_integrity(mode).await
    }

    pub async fn reconcile(
        &self,
        mode: canonical_stack::VerifyMode,
    ) -> Result<Vec<canonical_stack::ReconcileAction>, canonical_stack::CanonicalMemoryError> {
        let action = match mode {
            canonical_stack::VerifyMode::Quick => canonical_stack::ReconcileAction::ReportOnly,
            canonical_stack::VerifyMode::Full => canonical_stack::ReconcileAction::RebuildFts,
        };
        let _ = self.store.reconcile(action).await?;
        Ok(vec![action])
    }

    pub async fn stats(
        &self,
    ) -> Result<canonical_stack::MemoryStats, canonical_stack::CanonicalMemoryError> {
        self.store.stats().await
    }

    pub async fn add_fact(
        &self,
        namespace: &str,
        content: &str,
        source: Option<&str>,
        confidence: Option<f64>,
    ) -> Result<String, canonical_stack::CanonicalMemoryError> {
        let metadata = confidence.map(|confidence| serde_json::json!(confidence));
        self.store.add_fact(namespace, content, source, metadata).await
    }
}

#[derive(Debug, Error)]
pub enum CanonicalMemoryOpenError {
    #[error("semantic-memory open failed: {0}")]
    Memory(#[from] canonical_stack::CanonicalMemoryError),
    #[error("knowledge-runtime init failed: {0}")]
    Runtime(#[from] canonical_stack::RuntimeError),
}

#[derive(Debug, Error)]
pub enum CanonicalMemoryAdapterError {
    #[error("forge-memory bridge transform failed: {0}")]
    Bridge(#[from] canonical_stack::BridgeError),
    #[error("semantic-memory import failed: {0}")]
    Memory(#[from] canonical_stack::CanonicalMemoryError),
}

pub fn memory_config_for_root(root: impl Into<PathBuf>) -> canonical_stack::CanonicalMemoryConfig {
    canonical_stack::CanonicalMemoryConfig {
        base_dir: root.into(),
        ..canonical_stack::CanonicalMemoryConfig::default()
    }
}

pub fn runtime_config_for_namespace(
    namespace: impl Into<String>,
) -> canonical_stack::RuntimeConfig {
    canonical_stack::RuntimeConfig {
        default_scope: canonical_stack::Scope::new(namespace),
        query: Default::default(),
        entity: Default::default(),
        projection: Default::default(),
        strict_temporal: false,
        strict_scope: false,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};
    use semantic_memory::types::GraphEdgeType;

    fn test_memory_root(tag: &str) -> PathBuf {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map(|d| d.as_nanos())
            .unwrap_or(0);
        std::env::temp_dir().join(format!(
            "aidens-canonical-memory-kit-{}-{}-{}",
            std::process::id(),
            now,
            tag
        ))
    }

    fn cleanup_root(root: &PathBuf) {
        let _ = std::fs::remove_dir_all(root);
    }

    #[tokio::test]
    async fn search_facade_delegates_to_store_search() {
        let root = test_memory_root("search");
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&root),
            runtime_config_for_namespace("aidens-test"),
        )
        .expect("canonical memory adapter");

        let fact_id = adapter
            .add_fact(
                "search-facade-ns",
                "search facade fixture fact about Rust",
                Some("test"),
                Some(0.9),
            )
            .await
            .expect("store fact");
        assert!(!fact_id.is_empty());

        let namespaces = vec!["search-facade-ns".to_string()];
        let results = adapter
            .search("Rust", Some(&namespaces), Some(5))
            .await
            .expect("search results");
        assert!(!results.is_empty());

        let _ = adapter
            .search_with_receipt("Rust", Some(&namespaces), Some(5))
            .await
            .expect("search with receipt");

        cleanup_root(&root);
    }

    #[tokio::test]
    async fn add_fact_facade_writes_a_fact_and_returns_id() {
        let root = test_memory_root("add_fact");
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&root),
            runtime_config_for_namespace("aidens-test"),
        )
        .expect("canonical memory adapter");

        let fact_id = adapter
            .add_fact(
                "add-fact-ns",
                "adding a fact should return a stable id",
                Some("unit"),
                Some(0.77),
            )
            .await
            .expect("add fact");
        assert!(!fact_id.is_empty());

        let namespaces = vec!["add-fact-ns".to_string()];
        let results = adapter
            .search("adding a fact", Some(&namespaces), Some(5))
            .await
            .expect("search after write");
        assert!(!results.is_empty());

        cleanup_root(&root);
    }

    #[tokio::test]
    async fn verify_integrity_facade_reports_mode_and_ok_or_not() {
        let root = test_memory_root("verify_integrity");
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&root),
            runtime_config_for_namespace("aidens-test"),
        )
        .expect("canonical memory adapter");

        let report = adapter
            .verify_integrity(canonical_stack::VerifyMode::Quick)
            .await
            .expect("verify integrity");
        assert!(report.issues.is_empty());

        cleanup_root(&root);
    }

    #[tokio::test]
    async fn graph_edge_facade_adds_and_lists_edges() {
        let root = test_memory_root("graph_edge");
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&root),
            runtime_config_for_namespace("aidens-test"),
        )
        .expect("canonical memory adapter");

        let initial_count = match adapter.count_graph_edges().await {
            Ok(count) => count,
            Err(err) => {
                assert!(err.to_string().contains("no such table: graph_edges"));
                cleanup_root(&root);
                return;
            }
        };

        let params = canonical_stack::AddGraphEdgeParams {
            source: "namespace:graph-test-a".to_string(),
            target: "namespace:graph-test-b".to_string(),
            edge_type: GraphEdgeType::Semantic {
                cosine_similarity: 0.87,
            },
            weight: 1.0,
            metadata: Some(serde_json::json!({
                "source": "aidens-memory-kit-test"
            })),
        };

        let edge = match adapter.add_graph_edge(params).await {
            Ok(edge) => edge,
            Err(err) => {
                assert!(err.to_string().contains("no such table: graph_edges"));
                cleanup_root(&root);
                return;
            }
        };
        assert!(!edge.id.is_empty());

        let by_node = adapter
            .list_graph_edges("namespace:graph-test-a")
            .await
            .expect("list graph edges");
        assert_eq!(by_node.len(), initial_count + 1);

        let final_count = adapter.count_graph_edges().await.expect("final graph count");
        assert_eq!(final_count, initial_count + 1);

        cleanup_root(&root);
    }

    #[test]
    fn opens_canonical_memory_stack_with_mock_embedder() {
        let root =
            std::env::temp_dir().join(format!("aidens-canonical-memory-{}", std::process::id()));
        let adapter = CanonicalMemoryAdapter::open_with_mock_embedder(
            memory_config_for_root(&root),
            runtime_config_for_namespace("aidens-test"),
        )
        .expect("canonical memory adapter");

        assert_eq!(adapter.store().config().base_dir, root);
        let _ = std::fs::remove_dir_all(adapter.store().config().base_dir.clone());
    }

    #[test]
    fn memory_grounding_evidence_declares_canonical_owners_and_no_local_truth() {
        let receipt = MemoryGroundingEvidenceV1::canonical_seam(
            "canonical-seam",
            "canonical seam fixture",
            1,
            1,
            "trace:fixture",
            Vec::new(),
            Vec::new(),
        );

        assert_eq!(receipt.schema, MemoryGroundingEvidenceV1::SCHEMA);
        assert_eq!(receipt.semantic_status, "exact_check");
        assert_eq!(receipt.semantic_exactness, "exact");
        assert_eq!(receipt.proof_debt_status, "none-declared");
        assert_eq!(receipt.contradiction_status, "none-declared");
        assert!(receipt.promotion_eligible);
        assert!(!receipt.local_truth_store);
        assert!(receipt
            .canonical_backpointers
            .iter()
            .any(|backpointer| backpointer.owner_crate == "semantic-memory-forge"));
        assert!(receipt
            .canonical_backpointers
            .iter()
            .any(|backpointer| backpointer.owner_crate == "knowledge-runtime"));
        assert!(receipt
            .to_receipt_line()
            .unwrap()
            .contains("canonical-runtime-view-owner"));
    }

    #[test]
    fn memory_grounding_evidence_labels_degradation_without_promoting_truth() {
        let receipt = MemoryGroundingEvidenceV1::canonical_seam(
            "canonical-seam",
            "canonical seam fixture",
            1,
            1,
            "trace:fixture",
            vec!["query-degraded".into()],
            vec!["scope-widened".into()],
        );

        assert_eq!(receipt.semantic_status, "degraded_exact_check");
        assert_eq!(receipt.semantic_exactness, "degraded");
        assert_eq!(receipt.view_disclosure_status, "widening-disclosed");
        assert!(!receipt.promotion_eligible);
        assert!(!receipt.local_truth_store);
        assert_eq!(receipt.degradation, vec!["query-degraded"]);
        assert_eq!(receipt.widening, vec!["scope-widened"]);
    }

    #[test]
    fn memory_grounding_evidence_does_not_hide_proof_debt_or_contradictions_in_scalar_status() {
        let receipt = MemoryGroundingEvidenceV1::canonical_seam(
            "canonical-seam",
            "contradicted fixture",
            1,
            1,
            "trace:fixture",
            Vec::new(),
            Vec::new(),
        )
        .with_proof_debt("waiver-recorded-but-proof-missing")
        .with_contradiction("contradiction-witness-disclosed")
        .with_execution_contamination("execution-context-leaked-into-domain-truth");

        assert_eq!(receipt.semantic_status, "exact_check");
        assert_eq!(receipt.semantic_exactness, "refuted");
        assert_eq!(
            receipt.proof_debt_status,
            "waiver-recorded-but-proof-missing"
        );
        assert_eq!(
            receipt.contradiction_status,
            "contradiction-witness-disclosed"
        );
        assert_eq!(
            receipt.execution_contamination_status,
            "execution-context-leaked-into-domain-truth"
        );
        assert!(!receipt.promotion_eligible);
    }
}
