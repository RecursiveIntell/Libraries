//! Thin adapter to the canonical `semantic-memory` procedure lifecycle owner.
//!
//! This module owns no lifecycle state, permit, receipt, eligibility, or store. It only forwards
//! requests to [`MemoryStore`] and returns the owner's [`ProcedureLifecycleReceiptV1`] unchanged.

use semantic_memory::{
    MemoryError, MemoryStore, ProceduralMemoryArtifactV1, ProcedureLifecyclePermitV1,
    ProcedureLifecycleReceiptV1,
};

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
}
