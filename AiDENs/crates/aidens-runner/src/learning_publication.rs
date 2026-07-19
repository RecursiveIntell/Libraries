//! Separately authorized publication of pending AiDENs evidence through the
//! canonical Forge V3 -> bridge -> semantic-memory projection lane.

use forge_engine::{export_bundle, EpisodeExport, ForgeStore};
use semantic_memory::{MemoryConfig, MemoryStore};
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPublicationRequestV1 {
    pub forge_store: PathBuf,
    pub memory_store: PathBuf,
    pub bundle_id: String,
    pub publication_namespace: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum TerminalPublicationDispositionV1 {
    Published,
    RecoveredIdempotently,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TerminalPublicationOutcomeV1 {
    pub schema: String,
    pub disposition: TerminalPublicationDispositionV1,
    pub bundle_id: String,
    pub envelope_id: String,
    pub content_digest: String,
    pub export_envelope: semantic_memory_forge::ExportEnvelopeV3,
    pub export_receipt: forge_engine::ExperimentExportRecord,
    pub import_result: semantic_memory::ProjectionImportResult,
    pub import_readback: semantic_memory::ProjectionImportLogEntry,
    pub import_status: String,
    pub import_record_count: usize,
    pub import_was_duplicate: bool,
    pub export_was_duplicate: bool,
    pub readback_verified: bool,
}

#[derive(Debug, thiserror::Error)]
pub enum TerminalPublicationError {
    #[error("invalid publication request: {0}")]
    Invalid(String),
    #[error("Forge publication owner failed: {0}")]
    Forge(String),
    #[error("Forge-memory bridge failed: {0}")]
    Bridge(String),
    #[error("semantic-memory projection import failed: {0}")]
    Memory(String),
    #[error("publication readback failed: {0}")]
    Readback(String),
}

pub async fn publish_terminal_evidence(
    request: TerminalPublicationRequestV1,
) -> Result<TerminalPublicationOutcomeV1, TerminalPublicationError> {
    validate_request(&request)?;
    let forge = ForgeStore::open(&request.forge_store)
        .map_err(|error| TerminalPublicationError::Forge(error.to_string()))?;
    let bundle = forge
        .get_canonical_evidence_bundle(&request.bundle_id)
        .map_err(|error| TerminalPublicationError::Forge(error.to_string()))?
        .ok_or_else(|| {
            TerminalPublicationError::Invalid(format!(
                "unknown canonical evidence bundle: {}",
                request.bundle_id
            ))
        })?;
    validate_destination_binding(&request, &bundle)?;

    let export = EpisodeExport::from_bundle(&bundle, &request.publication_namespace);
    let export_was_duplicate = export
        .already_exported(&forge)
        .map_err(|error| TerminalPublicationError::Forge(error.to_string()))?;
    let envelope = export_bundle(&bundle, &request.publication_namespace, &forge)
        .await
        .map_err(|error| TerminalPublicationError::Forge(error.to_string()))?;
    let export_receipt = forge
        .get_export_receipt(&export.export_key)
        .map_err(|error| TerminalPublicationError::Forge(error.to_string()))?
        .ok_or_else(|| {
            TerminalPublicationError::Readback("persisted Forge export receipt missing".into())
        })?;
    if export_receipt.export_key != export.export_key
        || export_receipt.bundle_id != bundle.bundle_id
        || export_receipt.rendering_version != export.rendering_version
        || export_receipt.namespace != request.publication_namespace
    {
        return Err(TerminalPublicationError::Readback(
            "persisted Forge export receipt failed exact identity checks".into(),
        ));
    }
    let batch = forge_memory_bridge::transform_envelope_v3(&envelope)
        .map_err(|error| TerminalPublicationError::Bridge(error.to_string()))?;

    let memory = MemoryStore::open(MemoryConfig {
        base_dir: request.memory_store.clone(),
        ..MemoryConfig::default()
    })
    .map_err(|error| TerminalPublicationError::Memory(error.to_string()))?;
    let imported = memory
        .import_projection_batch(&batch)
        .await
        .map_err(|error| TerminalPublicationError::Memory(error.to_string()))?;
    let envelope_id = envelope.envelope_id.as_str().to_string();
    let content_digest = envelope.content_digest.hex().to_string();
    if imported.source_envelope_id != envelope_id {
        return Err(TerminalPublicationError::Readback(
            "import result envelope identity mismatch".into(),
        ));
    }

    let logs = memory
        .query_projection_imports(Some(&request.publication_namespace), 100)
        .await
        .map_err(|error| TerminalPublicationError::Memory(error.to_string()))?;
    let readback = logs
        .iter()
        .find(|entry| entry.source_envelope_id == envelope_id)
        .ok_or_else(|| {
            TerminalPublicationError::Readback(
                "exact semantic-memory projection import receipt missing".into(),
            )
        })?;
    if readback.status != "complete"
        || readback.content_digest != content_digest
        || readback.evidence_bundle_id.as_deref() != Some(bundle.bundle_id.as_str())
        || readback.direct_write
    {
        return Err(TerminalPublicationError::Readback(
            "semantic-memory projection import receipt failed exact identity checks".into(),
        ));
    }

    let recovered = export_was_duplicate || imported.was_duplicate;
    Ok(TerminalPublicationOutcomeV1 {
        schema: "AiDENsTerminalPublicationOutcomeV1".into(),
        disposition: if recovered {
            TerminalPublicationDispositionV1::RecoveredIdempotently
        } else {
            TerminalPublicationDispositionV1::Published
        },
        bundle_id: bundle.bundle_id,
        envelope_id,
        content_digest,
        export_envelope: envelope,
        export_receipt,
        import_result: imported.clone(),
        import_readback: readback.clone(),
        import_status: imported.status,
        import_record_count: imported.record_count,
        import_was_duplicate: imported.was_duplicate,
        export_was_duplicate,
        readback_verified: true,
    })
}

fn validate_request(
    request: &TerminalPublicationRequestV1,
) -> Result<(), TerminalPublicationError> {
    if request.forge_store.as_os_str().is_empty()
        || request.memory_store.as_os_str().is_empty()
        || request.bundle_id.trim().is_empty()
        || request.publication_namespace.trim().is_empty()
    {
        return Err(TerminalPublicationError::Invalid(
            "Forge store, memory store, bundle ID, and namespace are required".into(),
        ));
    }
    Ok(())
}

fn validate_destination_binding(
    request: &TerminalPublicationRequestV1,
    bundle: &forge_engine::ExperimentEvidenceBundle,
) -> Result<(), TerminalPublicationError> {
    let flags = bundle
        .bundle_scope
        .as_ref()
        .ok_or_else(|| TerminalPublicationError::Invalid("bundle scope is required".into()))?
        .config_flags
        .as_slice();
    let expected = [
        format!("publication_namespace:{}", request.publication_namespace),
        format!(
            "memory_store_owner_digest:{}",
            digest_path(&request.memory_store)
        ),
        format!(
            "forge_store_owner_digest:{}",
            digest_path(&request.forge_store)
        ),
    ];
    if expected.iter().any(|item| !flags.contains(item)) {
        return Err(TerminalPublicationError::Invalid(
            "publication destination does not match preflight-bound bundle".into(),
        ));
    }
    Ok(())
}

fn digest_path(path: &std::path::Path) -> String {
    let json = serde_json::Value::String(path.to_string_lossy().to_string()).to_string();
    blake3::hash(json.as_bytes()).to_hex().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;
    use forge_engine::lab::evaluate::ScoreVector;
    use forge_engine::lab::evidence::{BundleScope, Covariates, Treatment};
    use forge_engine::{ClaimStrength, ExperimentEvidenceBundle};
    use semantic_memory_forge::PromotionState;

    fn pending_bundle(request: &TerminalPublicationRequestV1) -> ExperimentEvidenceBundle {
        let flags = vec![
            format!(
                "forge_store_owner_digest:{}",
                digest_path(&request.forge_store)
            ),
            format!(
                "memory_store_owner_digest:{}",
                digest_path(&request.memory_store)
            ),
            format!("publication_namespace:{}", request.publication_namespace),
        ];
        let mut bundle = ExperimentEvidenceBundle {
            bundle_id: request.bundle_id.clone(),
            candidate_id: "procedure:exact-source".into(),
            eval_id: "effectful:exact-source".into(),
            version_id: "aidens-exact-source-execution-evidence-v1".into(),
            supersedes_claim_version_id: None,
            relation_lineage_hints: Default::default(),
            scores: ScoreVector {
                correctness: 1.0,
                novelty: 0.0,
                stability: 0.0,
                weighted_total: 1.0,
                cea_confidence: Some(1.0),
                cea_predicted_correctness: None,
            },
            hypotheses: Vec::new(),
            verification: None,
            trace_id: Some("0af7651916cd43dd8448eb211c80319c".into()),
            experiment_diff: None,
            attribution_json: None,
            assessment: None,
            warnings: vec!["single-arm execution; no comparative effect claim".into()],
            created_at: "2026-07-18T00:00:00Z".into(),
            run_id: Some("run:publication".into()),
            attempt_id: Some("attempt:publication".into()),
            causal_question: Some(
                "Did the exact source-bound patch complete its declared sealed checks?".into(),
            ),
            unit_definition: Some("one exact patch execution on one frozen source tree".into()),
            bundle_scope: Some(BundleScope {
                workload_id: "source-tree:before".into(),
                backend_family: "sealed-container".into(),
                selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
                timeout_class: "900s".into(),
                config_flags: flags.clone(),
            }),
            pair_comparability: None,
            claim_strength: ClaimStrength::ExecutionVerifiedNoComparison,
            identification_rationale: Some("observed execution only".into()),
            known_threats: vec!["single source tree".into()],
            patch_hash: Some("patch:digest".into()),
            treatment: Some(Treatment {
                kind: "exact_source_bound_patch".into(),
                patch_hash: "patch:digest".into(),
                patch_summary: "one immutable patch".into(),
            }),
            outcome: Some("declared checks passed".into()),
            covariates: Some(Covariates {
                env_fingerprint: "sandbox:digest".into(),
                dependency_fingerprint: Some("source-tree:before".into()),
                config_flags: flags,
                workload_id: "source-tree:before".into(),
                selected_checks: vec!["fmt".into(), "clippy".into(), "test".into()],
                adjacent_edits: false,
                adjacent_edit_signatures: Vec::new(),
            }),
            promotion_state: Some(PromotionState::NotPromoted),
            primary_effect: None,
            all_effects: Vec::new(),
            hypothesis_edges: Vec::new(),
            receipts: Vec::new(),
            verification_trials: Vec::new(),
            refutation_artifacts: Vec::new(),
            sealed: false,
        };
        bundle.seal().unwrap();
        bundle
    }

    #[tokio::test]
    async fn publishes_v3_with_exact_readback_and_recovers_idempotently() {
        let root = tempfile::tempdir().unwrap();
        let request = TerminalPublicationRequestV1 {
            forge_store: root.path().join("forge.sqlite"),
            memory_store: root.path().join("memory"),
            bundle_id: "bundle:publication".into(),
            publication_namespace: "aidens-learning".into(),
        };
        let forge = ForgeStore::open(&request.forge_store).unwrap();
        forge
            .insert_canonical_evidence_bundle(&pending_bundle(&request))
            .unwrap();
        drop(forge);

        let first = publish_terminal_evidence(request.clone()).await.unwrap();
        assert_eq!(
            first.disposition,
            TerminalPublicationDispositionV1::Published
        );
        assert!(!first.export_was_duplicate);
        assert!(!first.import_was_duplicate);
        assert!(first.readback_verified);

        let recovered = publish_terminal_evidence(request).await.unwrap();
        assert_eq!(
            recovered.disposition,
            TerminalPublicationDispositionV1::RecoveredIdempotently
        );
        assert!(recovered.export_was_duplicate);
        assert!(recovered.import_was_duplicate);
        assert_eq!(recovered.envelope_id, first.envelope_id);
        assert_eq!(recovered.content_digest, first.content_digest);
    }

    #[tokio::test]
    async fn rejects_namespace_not_bound_by_preflight_bundle() {
        let root = tempfile::tempdir().unwrap();
        let mut request = TerminalPublicationRequestV1 {
            forge_store: root.path().join("forge.sqlite"),
            memory_store: root.path().join("memory"),
            bundle_id: "bundle:wrong-namespace".into(),
            publication_namespace: "aidens-learning".into(),
        };
        let forge = ForgeStore::open(&request.forge_store).unwrap();
        forge
            .insert_canonical_evidence_bundle(&pending_bundle(&request))
            .unwrap();
        drop(forge);

        request.publication_namespace = "different".into();
        let error = publish_terminal_evidence(request).await.unwrap_err();
        assert!(matches!(error, TerminalPublicationError::Invalid(_)));
    }
}
