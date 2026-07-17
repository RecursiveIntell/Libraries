//! Thin adapter to the canonical `semantic-memory` procedure lifecycle owner.
//!
//! This module owns no lifecycle state, permit, receipt, eligibility, or store. It only forwards
//! requests to [`MemoryStore`] and returns the owner's [`ProcedureLifecycleReceiptV1`] unchanged.

use aidens_contracts::CanonicalBackpointerV1;
use semantic_memory::{
    MemoryError, MemoryStore, ProceduralMemoryArtifactV1, ProcedureLifecyclePermitV1,
    ProcedureLifecycleReceiptV1,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcedureLifecycleProjectionV1 {
    pub receipt_id: String,
    pub artifact_id: String,
    pub artifact_digest: String,
    pub event_digest: String,
    pub canonical_backpointer: CanonicalBackpointerV1,
}

pub fn project_lifecycle_receipt(
    receipt: &ProcedureLifecycleReceiptV1,
) -> Result<ProcedureLifecycleProjectionV1, &'static str> {
    if receipt.receipt_id.trim().is_empty()
        || receipt.artifact_id.trim().is_empty()
        || receipt.artifact_digest.trim().is_empty()
        || receipt.event_digest.trim().is_empty()
    {
        return Err("lifecycle receipt has no durable identity");
    }
    Ok(ProcedureLifecycleProjectionV1 {
        receipt_id: receipt.receipt_id.clone(),
        artifact_id: receipt.artifact_id.clone(),
        artifact_digest: receipt.artifact_digest.clone(),
        event_digest: receipt.event_digest.clone(),
        canonical_backpointer: CanonicalBackpointerV1::external(
            "semantic-memory",
            "ProcedureLifecycleReceiptV1",
            "procedure-lifecycle-receipt",
            receipt.receipt_id.clone(),
        ),
    })
}

/// Borrowed adapter over the canonical semantic-memory store.
pub struct ProcedureLifecycleAdapter<'a> {
    store: &'a MemoryStore,
}

impl<'a> ProcedureLifecycleAdapter<'a> {
    pub fn new(store: &'a MemoryStore) -> Self {
        Self { store }
    }

    pub async fn compile(
        &self,
        artifact: ProceduralMemoryArtifactV1,
        caller_idempotency_key: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .compile_procedure(artifact, caller_idempotency_key)
            .await
    }

    pub async fn test(
        &self,
        artifact_id: &str,
        caller_idempotency_key: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .test_procedure(artifact_id, caller_idempotency_key)
            .await
    }

    pub async fn promote(
        &self,
        permit: ProcedureLifecyclePermitV1,
        artifact_id: &str,
        caller_idempotency_key: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .promote_procedure(permit, artifact_id, caller_idempotency_key)
            .await
    }

    pub async fn quarantine(
        &self,
        permit: ProcedureLifecyclePermitV1,
        artifact_id: &str,
        caller_idempotency_key: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .quarantine_procedure(permit, artifact_id, caller_idempotency_key, reason)
            .await
    }

    pub async fn revoke(
        &self,
        permit: ProcedureLifecyclePermitV1,
        artifact_id: &str,
        caller_idempotency_key: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .revoke_procedure(permit, artifact_id, caller_idempotency_key, reason)
            .await
    }

    pub async fn rollback(
        &self,
        permit: ProcedureLifecyclePermitV1,
        artifact_id: &str,
        caller_idempotency_key: impl Into<String>,
        reason: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        self.store
            .rollback_procedure(permit, artifact_id, caller_idempotency_key, reason)
            .await
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use semantic_memory::{MemoryConfig, ProcedureLifecycleDispositionV1};
    use tempfile::tempdir;

    #[tokio::test]
    async fn forwards_test_to_canonical_store_and_preserves_owner_receipt() {
        let dir = tempdir().expect("temporary canonical memory directory");
        let config = MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..MemoryConfig::default()
        };
        let store = MemoryStore::open(config).expect("canonical store");
        let adapter = ProcedureLifecycleAdapter::new(&store);

        let error = adapter
            .test("missing-artifact", "test-key")
            .await
            .unwrap_err();
        assert!(matches!(
            error,
            MemoryError::ProceduralMemoryNotFound { .. }
        ));
    }

    #[tokio::test]
    async fn canonical_owner_rejects_invalid_permit_fail_closed() {
        let dir = tempdir().expect("temporary canonical memory directory");
        let config = MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..MemoryConfig::default()
        };
        let store = MemoryStore::open(config).expect("canonical store");
        let adapter = ProcedureLifecycleAdapter::new(&store);
        let mut permit = ProcedureLifecyclePermitV1::elevated("principal:alice", "operator:test");
        permit.capability = "invalid".into();

        let error = adapter
            .promote(permit, "missing-artifact", "promote-key")
            .await
            .unwrap_err();
        assert!(
            matches!(error, MemoryError::ProceduralMemoryUnauthorized { .. }),
            "unexpected canonical owner error: {error:?}"
        );
    }

    #[test]
    fn adapter_exposes_owner_receipt_type_only() {
        let _: Option<ProcedureLifecycleReceiptV1> = None;
        let _ = ProcedureLifecycleDispositionV1::Compiled;
    }

    #[test]
    fn lifecycle_projection_contains_durable_canonical_owner_backpointer() {
        let receipt = ProcedureLifecycleReceiptV1 {
            schema_version: "procedure-lifecycle-receipt-v1".into(),
            receipt_id: "receipt-1".into(),
            caller_idempotency_key: "key-1".into(),
            operation: "compile".into(),
            artifact_id: "artifact-1".into(),
            artifact_digest: "digest-1".into(),
            principal: "principal:alice".into(),
            disposition: ProcedureLifecycleDispositionV1::Compiled,
            reason_codes: Vec::new(),
            test_receipt: None,
            prior_event_digest: None,
            event_id: "event-1".into(),
            event_digest: "event-digest-1".into(),
            receipt_digest: "receipt-digest-1".into(),
            committed_at: "2026-01-01T00:00:00Z".into(),
        };
        let projection = project_lifecycle_receipt(&receipt).expect("durable projection");
        assert_eq!(projection.receipt_id, receipt.receipt_id);
        assert_eq!(projection.artifact_digest, receipt.artifact_digest);
        assert_eq!(projection.event_digest, receipt.event_digest);
        assert_eq!(
            projection.canonical_backpointer.owner_crate,
            "semantic-memory"
        );
        assert_eq!(
            projection.canonical_backpointer.owner_type,
            "ProcedureLifecycleReceiptV1"
        );
        assert_eq!(
            projection.canonical_backpointer.external_id.as_deref(),
            Some(receipt.receipt_id.as_str())
        );
    }
}
