use crate::error::PilotError;
#[cfg(feature = "governance")]
use claim_ledger::{ids, ExportReceipt};
use forge_engine::{export_bundle, ExperimentEvidenceBundle, ForgeStore};
use forge_memory_bridge::transform_envelope_v3;
use semantic_memory::{MemoryStore, ProjectionImportResult};
use semantic_memory_forge::ExportEnvelopeV3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtripResult {
    pub envelope: ExportEnvelopeV3,
    pub import_result: ProjectionImportResult,
    /// `claim_ledger::ExportReceipt` recording the canonical export/import roundtrip.
    /// Captures the bundle digest, envelope digest, and import receipt linkage.
    /// `None` when the `governance` feature is disabled.
    #[cfg(feature = "governance")]
    pub export_receipt: Option<ExportReceipt>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBootstrapReport {
    pub namespace: String,
    pub forge_bundle_count: usize,
    pub imported_bundle_ids: Vec<String>,
    /// `claim_ledger::ExportReceipt` per imported bundle, in import order.
    /// `None` when the `governance` feature is disabled.
    #[cfg(feature = "governance")]
    pub export_receipts: Vec<ExportReceipt>,
}

/// Build a `claim_ledger::ExportReceipt` that binds the bundle hash and
/// envelope hash of a canonical roundtrip. No-op when the `governance`
/// feature is disabled — call site must not rely on the receipt.
#[cfg(feature = "governance")]
fn build_export_receipt(
    bundle_id: &str,
    envelope: &ExportEnvelopeV3,
    import_result: &ProjectionImportResult,
) -> ExportReceipt {
    let envelope_json = serde_json::to_string(envelope).unwrap_or_default();
    let envelope_digest = ids::sha256_text(&envelope_json);
    let mut receipt = ExportReceipt::new(
        "forge_pilot_canonical_roundtrip",
        vec![bundle_id.to_string()],
        envelope.envelope_id.to_string(),
    );
    receipt.input_digests.insert("bundle".to_string(), envelope_digest.clone());
    receipt.bind_output(
        format!(
            "projection_import:{}",
            import_result.source_envelope_id
        ),
        ids::sha256_text(&envelope_json),
    );
    receipt.mark_success();
    receipt
}

pub async fn canonical_roundtrip(
    bundle: &ExperimentEvidenceBundle,
    namespace: &str,
    forge_store: &ForgeStore,
    memory_store: &MemoryStore,
) -> Result<RoundtripResult, PilotError> {
    let envelope = export_bundle(bundle, namespace, forge_store).await?;
    let batch = transform_envelope_v3(&envelope)?;
    let import_result = memory_store.import_projection_batch(&batch).await?;
    #[cfg(feature = "governance")]
    let export_receipt = Some(build_export_receipt(
        &bundle.bundle_id,
        &envelope,
        &import_result,
    ));
    Ok(RoundtripResult {
        envelope,
        import_result,
        #[cfg(feature = "governance")]
        export_receipt,
    })
}

pub async fn import_recent_forge_bundles(
    namespace: &str,
    forge_store: &ForgeStore,
    memory_store: &MemoryStore,
    limit: usize,
) -> Result<ImportBootstrapReport, PilotError> {
    let mut bundle_ids = forge_store.list_recent_evidence_bundle_ids(limit)?;
    let forge_bundle_count = bundle_ids.len();
    bundle_ids.reverse();

    let mut imported_bundle_ids = Vec::with_capacity(bundle_ids.len());
    #[cfg(feature = "governance")]
    let mut export_receipts = Vec::with_capacity(bundle_ids.len());
    for bundle_id in bundle_ids {
        let bundle_row = forge_store
            .get_evidence_bundle(&bundle_id)?
            .ok_or_else(|| PilotError::Other(format!("missing Forge bundle row {bundle_id}")))?;
        let bundle = bundle_row.local_bundle()?;
        let result = canonical_roundtrip(&bundle, namespace, forge_store, memory_store).await?;
        #[cfg(feature = "governance")]
        if let Some(receipt) = result.export_receipt.clone() {
            export_receipts.push(receipt);
        }
        imported_bundle_ids.push(bundle_id);
    }

    Ok(ImportBootstrapReport {
        namespace: namespace.into(),
        forge_bundle_count,
        imported_bundle_ids,
        #[cfg(feature = "governance")]
        export_receipts,
    })
}
