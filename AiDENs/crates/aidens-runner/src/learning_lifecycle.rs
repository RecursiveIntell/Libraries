//! Thin adapter to the canonical `semantic-memory` procedure lifecycle owner.
//!
//! This module owns no lifecycle state, permit, receipt, eligibility, or store. It only forwards
//! requests to [`MemoryStore`] and returns the owner's [`ProcedureLifecycleReceiptV1`] unchanged.

use crate::learning_effectful::EffectfulEvaluationReportV1;
use aidens_contracts::CanonicalBackpointerV1;
use forge_memory_bridge::ForgeAdjudicationStore;
use semantic_memory::{
    MemoryError, MemoryStore, ProceduralMemoryArtifactV1, ProcedureEffectfulEvaluationReceiptV1,
    ProcedureLifecyclePermitV1, ProcedureLifecycleReceiptV1,
};
use serde::Serialize;
use verification_adjudication::VerificationDisposition;

#[derive(Debug, Clone, PartialEq, Eq, Serialize)]
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

    /// Bind a publication-complete real evaluation to the canonical procedure owner.
    ///
    /// The adapter stores only owner-native receipt IDs and digests. Sandbox,
    /// verification, check, and CEA payload truth remains in those owner crates.
    pub async fn record_effectful_evaluation(
        &self,
        artifact_id: &str,
        artifact_digest: &str,
        report: &EffectfulEvaluationReportV1,
        caller_idempotency_key: impl Into<String>,
    ) -> Result<ProcedureEffectfulEvaluationReceiptV1, MemoryError> {
        let checks_complete = report.checks.fmt_executed
            && report.checks.fmt_passed
            && report.checks.clippy_executed
            && report.checks.clippy_passed
            && report.checks.test_executed
            && report.checks.test_passed
            && !report.checks.fmt_output_digest.is_empty()
            && !report.checks.clippy_output_digest.is_empty()
            && !report.checks.test_output_digest.is_empty();
        if report.execution_mode != "real_sandbox"
            || !report.verified
            || !report.cea_persisted
            || report.verification.disposition != VerificationDisposition::EligibleForPromotion
            || !report.sandbox_capability.verify()
            || !checks_complete
            || report.before_tree_digest.is_empty()
            || report.after_tree_digest.is_empty()
            || !report.rollback_verified
            || report.rollback_tree_digest != report.before_tree_digest
            || report.cea_run_hash.is_empty()
        {
            return Err(MemoryError::ProceduralMemoryRejected {
                reason: "effectful evaluation requires publication-complete real sandbox evidence"
                    .into(),
            });
        }
        let receipt = ProcedureEffectfulEvaluationReceiptV1::verified(
            artifact_id,
            artifact_digest,
            report.sandbox_capability.content_digest.clone(),
            vec![
                report.checks.fmt_output_digest.clone(),
                report.checks.clippy_output_digest.clone(),
                report.checks.test_output_digest.clone(),
            ],
            report
                .verification
                .promotion_decision
                .decision_id
                .to_string(),
            report.cea_run_hash.clone(),
            report.before_tree_digest.clone(),
            report.after_tree_digest.clone(),
        )?;
        self.store
            .record_effectful_procedure_evaluation(receipt, caller_idempotency_key)
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

    /// Promote a procedure only after a bridge-verified single-adjudication.
    pub async fn promote_adjudicated(
        &self,
        forge: &dyn ForgeAdjudicationStore,
        permit: ProcedureLifecyclePermitV1,
        adjudication_id: &str,
        caller_idempotency_key: impl Into<String>,
    ) -> Result<ProcedureLifecycleReceiptV1, MemoryError> {
        let adjudication = forge
            .read_verified_adjudication(adjudication_id)
            .map_err(|error| MemoryError::ProceduralMemoryRejected {
                reason: error.to_string(),
            })?;
        self.store
            .promote_adjudicated_procedure(
                forge,
                permit,
                adjudication_id,
                &adjudication.candidate_id,
                adjudication.candidate_digest.as_str(),
                &adjudication.evidence_bundle_id,
                adjudication.evidence_bundle_digest.as_str(),
                caller_idempotency_key,
            )
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
    use forge_engine::ForgeStore;
    use semantic_memory::{
        verify_procedure_lifecycle_receipt_v1, AllowedProcedureToolV1, ApplicabilityPredicateV1,
        AuthorityScopeV1, AuthorityScopesV1, ElevationRequirementV1, MemoryConfig, MemoryStore,
        NamespaceScopeV1, OriginAuthorityLabelV1, OriginClassV1, OriginRiskV1,
        ProceduralMemoryArtifactV1, ProcedureActionV1, ProcedureCapabilityV1, ProcedureEffectV1,
        ProcedureEffectfulEvaluationReceiptV1, ProcedureEvidenceTestEnvelopeV1, ProcedureFixtureV1,
        ProcedureLifecycleDispositionV1, ProcedureLifecyclePermitV1, ProcedurePreconditionV1,
        ProcedureRiskV1, ProcedureStepV1, RevocationStatusV1, SubjectPrincipalV1,
    };
    use serde_json::json;
    use tempfile::tempdir;
    use verification_adjudication::{
        adjudicate_candidate, CandidatePromotionInput, FamilyGateV1, FrozenPromotionThresholdsV1,
        HoldoutGateV1, IdentityDigest, ReceiptRef, UncertaintyV1,
    };

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
            adjudication_digest: None,
            permit_digest: None,
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

    #[tokio::test]
    async fn promote_adjudicated_receipt_is_linked_to_adjudication() {
        let dir = tempdir().expect("temporary canonical memory directory");
        let config = MemoryConfig {
            base_dir: dir.path().to_path_buf(),
            ..MemoryConfig::default()
        };
        let store = MemoryStore::open(config).expect("canonical store");
        let adapter = ProcedureLifecycleAdapter::new(&store);
        let artifact = artifact("procedure:adjudicated", 1, None);
        let permit = lifecycle_permit("promote", &artifact.artifact_id);
        let adjudication = adjudication_for(&artifact).unwrap();
        let forge = ForgeStore::open(&dir.path().join("forge.sqlite")).unwrap();
        forge.persist_adjudication(&adjudication).unwrap();

        store
            .compile_procedure(artifact.clone(), "compile:adjudicated")
            .await
            .unwrap();
        store
            .test_procedure(&artifact.artifact_id, "test:adjudicated")
            .await
            .unwrap();
        store
            .record_effectful_procedure_evaluation(
                effectful_receipt(&artifact),
                "effectful:adjudicated",
            )
            .await
            .unwrap();
        let receipt = adapter
            .promote_adjudicated(
                &forge,
                permit,
                adjudication.adjudication_id.as_str(),
                "promote:adjudicated",
            )
            .await
            .unwrap();

        assert_eq!(
            receipt.disposition,
            ProcedureLifecycleDispositionV1::Promoted
        );
        assert!(verify_procedure_lifecycle_receipt_v1(&receipt));
        assert_eq!(
            receipt.adjudication_digest.as_deref(),
            Some(adjudication.adjudication_digest.as_str())
        );
    }

    fn lifecycle_permit(operation: &str, artifact_id: &str) -> ProcedureLifecyclePermitV1 {
        ProcedureLifecyclePermitV1::elevated_for(
            "principal:alice",
            "operator:test",
            operation,
            artifact_id,
            "2999-01-01T00:00:00Z",
        )
    }

    fn artifact(key: &str, version: u64, supersedes: Option<String>) -> ProceduralMemoryArtifactV1 {
        ProceduralMemoryArtifactV1::new(
            key,
            ProcedureCapabilityV1::new("repository", "format"),
            ProcedureActionV1::new(
                "format_rust",
                "format Rust sources with the approved formatter",
            ),
            vec![ApplicabilityPredicateV1::equals("language", json!("rust"))],
            vec![ProcedurePreconditionV1::equals(
                "working_tree",
                json!("available"),
            )],
            vec![ProcedureStepV1::tool(
                "format",
                "rustfmt",
                json!({"check": true}),
                Some("no_write".into()),
            )],
            vec![AllowedProcedureToolV1::new(
                "rustfmt",
                json!({
                    "type": "object",
                    "properties": {"check": {"type": "boolean"}},
                    "required": ["check"],
                    "additionalProperties": false
                }),
            )],
            vec![ProcedureEffectV1::new("format_checked", json!(true))],
            vec![ProcedureEffectV1::new("network_access", json!(true))],
            ProcedureRiskV1::Low,
            origin("principal:alice"),
            "principal:alice",
            vec!["principal:alice".into()],
            NamespaceScopeV1::exact("repo:alpha"),
            version,
            supersedes,
            ProcedureEvidenceTestEnvelopeV1::new(
                "sandbox-v1",
                vec![ProcedureFixtureV1::new(
                    "rust-project",
                    json!({"language": "rust", "working_tree": "available"}),
                    vec!["rustfmt".into()],
                    vec![ProcedureEffectV1::new("format_checked", json!(true))],
                    vec![],
                )],
                vec![],
            ),
            Some("2999-01-01T00:00:00Z".into()),
        )
        .unwrap()
    }

    fn origin(principal: &str) -> OriginAuthorityLabelV1 {
        OriginAuthorityLabelV1::new(
            OriginClassV1::OperatorSystem,
            principal,
            "procedure-compiler",
            "blake3:procedure-source",
            OriginRiskV1::Low,
            AuthorityScopesV1 {
                recall: AuthorityScopeV1::Audience,
                assertion: AuthorityScopeV1::Denied,
                action: AuthorityScopeV1::Audience,
            },
            ElevationRequirementV1::ExplicitOperatorApproval,
            None,
            RevocationStatusV1::Active,
            vec![principal.into()],
        )
        .unwrap()
        .with_subject_principal(SubjectPrincipalV1::new(principal).unwrap())
        .with_resource_scope(NamespaceScopeV1::exact("repo:alpha"))
    }

    fn effectful_receipt(
        artifact: &ProceduralMemoryArtifactV1,
    ) -> ProcedureEffectfulEvaluationReceiptV1 {
        ProcedureEffectfulEvaluationReceiptV1::verified(
            artifact.artifact_id.clone(),
            artifact.artifact_digest.clone(),
            "sandbox-capability:owner-receipt",
            vec![
                "check:fmt".into(),
                "check:clippy".into(),
                "check:test".into(),
            ],
            "verification:owner-receipt",
            "cea:owner-receipt",
            "tree:before",
            "tree:after",
        )
        .unwrap()
    }

    fn adjudication_for(
        artifact: &ProceduralMemoryArtifactV1,
    ) -> Result<verification_adjudication::CandidatePromotionAdjudicationV1, &'static str> {
        adjudicate_candidate(CandidatePromotionInput {
            adjudication_id: "adjudication:procedure-adjudicated".into(),
            candidate_id: artifact.artifact_id.clone(),
            candidate_digest: IdentityDigest::new(
                "aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa",
            )
            .map_err(|_| "candidate digest must be valid blake3 hex")?,
            patch_digest: IdentityDigest::new(
                "1111111111111111111111111111111111111111111111111111111111111111",
            )
            .map_err(|_| "patch digest must be valid")?,
            source_tree_digest: IdentityDigest::new(
                "2222222222222222222222222222222222222222222222222222222222222222",
            )
            .map_err(|_| "source tree digest must be valid")?,
            verifier_digest: IdentityDigest::new(
                "3333333333333333333333333333333333333333333333333333333333333333",
            )
            .map_err(|_| "verifier digest must be valid")?,
            check_policy_digest: IdentityDigest::new(
                "4444444444444444444444444444444444444444444444444444444444444444",
            )
            .map_err(|_| "check policy digest must be valid")?,
            environment_digest: IdentityDigest::new(
                "5555555555555555555555555555555555555555555555555555555555555555",
            )
            .map_err(|_| "environment digest must be valid")?,
            image_digest: IdentityDigest::new(
                "6666666666666666666666666666666666666666666666666666666666666666",
            )
            .map_err(|_| "image digest must be valid")?,
            experiment_id: "experiment:procedure-adjudicated".into(),
            evidence_bundle_id: "evidence:bundle".into(),
            evidence_bundle_digest: IdentityDigest::new(
                "7777777777777777777777777777777777777777777777777777777777777777",
            )
            .map_err(|_| "evidence digest must be valid")?,
            assignment_digest: IdentityDigest::new(
                "8888888888888888888888888888888888888888888888888888888888888888",
            )
            .map_err(|_| "assignment digest must be valid")?,
            paired_denominator: 5,
            admissible_pairs: 5,
            excluded_pairs: 0,
            uncertainty: UncertaintyV1 {
                estimate: 0.02,
                lower_bound: 0.01,
                upper_bound: 0.03,
            },
            family_results: vec![FamilyGateV1 {
                family: "format".into(),
                score: 0.98,
                passed: true,
                admissible_pairs: 5,
            }],
            holdout_result: HoldoutGateV1 {
                score: 0.99,
                passed: true,
                admissible_pairs: 1,
            },
            thresholds: FrozenPromotionThresholdsV1 {
                minimum_admissible_pairs: 1,
                minimum_family_score: 0.8,
                minimum_holdout_score: 0.8,
                maximum_uncertainty: 0.25,
            },
            source_receipt_refs: vec![ReceiptRef {
                receipt_id: "source:receipt".into(),
                receipt_digest: IdentityDigest::new(
                    "9999999999999999999999999999999999999999999999999999999999999999",
                )
                .map_err(|_| "source receipt digest must be valid")?,
            }],
            created_at: "2026-01-01T00:00:00Z".into(),
        })
        .map_err(|_| "failed to build adjudication")
    }
}
