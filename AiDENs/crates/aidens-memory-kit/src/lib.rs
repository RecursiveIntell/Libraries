//! Thin memory facade over Forge, bridge, semantic-memory, and knowledge-runtime.

use serde::{Deserialize, Serialize};
use std::path::PathBuf;
use thiserror::Error;

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
        MemoryStore as CanonicalMemoryStore, MockEmbedder, ProjectionImportResult, SearchResult,
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
