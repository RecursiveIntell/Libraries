use crate::bootstrap::types::{BootstrapManifestSnapshot, BOOTSTRAP_SOURCE_AUTHORITY};
use crate::error::PilotError;
use forge_memory_bridge::transform_envelope_v3;
use semantic_memory::{MemoryStore, ProjectionImportResult};
use semantic_memory_forge::{ExportEnvelopeV3, ExportRecordV3, EXPORT_ENVELOPE_V3_SCHEMA};
use stack_ids::{EnvelopeId, ScopeKey, TraceCtx};

pub(crate) fn build_source_envelope(
    namespace: &str,
    manifest: &BootstrapManifestSnapshot,
    records: Vec<ExportRecordV3>,
) -> Result<ExportEnvelopeV3, PilotError> {
    let scope_key = ScopeKey::namespace_only(namespace);
    let exported_at = chrono::Utc::now().to_rfc3339();
    let trace_ctx = Some(TraceCtx::generate());
    let content_digest = ExportEnvelopeV3::compute_digest(
        BOOTSTRAP_SOURCE_AUTHORITY,
        &scope_key,
        &records,
        None,
        None,
    )
    .map_err(|error| {
        PilotError::Other(format!(
            "failed to compute bootstrap envelope digest: {error}"
        ))
    })?;

    Ok(ExportEnvelopeV3 {
        envelope_id: EnvelopeId::new(format!(
            "workspace-source-envelope-{}",
            manifest.manifest_id
        )),
        schema_version: EXPORT_ENVELOPE_V3_SCHEMA.into(),
        content_digest,
        source_authority: BOOTSTRAP_SOURCE_AUTHORITY.into(),
        scope_key,
        trace_ctx,
        exported_at,
        export_meta: None,
        evidence_bundle: None,
        support_sets: vec![],
        contradiction_witnesses: vec![],
        retraction_records: vec![],
        claim_states_v13: vec![],
        intervention_bundles_v14: vec![],
        outcome_schemas_v14: vec![],
        cohort_contracts_v14: vec![],
        counterfactual_slices_v14: vec![],
        experiment_cases_v14: vec![],
        comparability_matrices_v14: vec![],
        decision_traces_v14: vec![],
        refuter_suites_v14: vec![],
        refuter_results_v14: vec![],
        experiment_budgets_v14: vec![],
        rollout_decisions_v14: vec![],
        rollback_decisions_v14: vec![],
        attestation_envelopes_v15: vec![],
        trust_root_sets_v15: vec![],
        artifact_admission_policies_v15: vec![],
        transparency_receipts_v15: vec![],
        attestation_revocations_v15: vec![],
        attestation_supersessions_v15: vec![],
        remote_oracle_leases_v15: vec![],
        remote_slice_requests_v15: vec![],
        remote_slice_results_v15: vec![],
        cross_runtime_replay_tickets_v15: vec![],
        dispute_bundles_v15: vec![],
        disclosure_policies_v15: vec![],
        disclosure_budgets_v15: vec![],
        records,
    })
}

pub(crate) async fn import_envelope(
    memory_store: &MemoryStore,
    envelope: &ExportEnvelopeV3,
) -> Result<ProjectionImportResult, PilotError> {
    let batch = transform_envelope_v3(envelope)?;
    Ok(memory_store.import_projection_batch(&batch).await?)
}
