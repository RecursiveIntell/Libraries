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
/// envelope hash of a canonical roundtrip. The receipt starts in
/// `pending` state; callers must update `status` to `"success"` or
/// `"failure"` based on the operation outcome. The receipt must be
/// emitted on every path — including failures — so audit consumers can
/// reconstruct the full history.
///
/// No-op when the `governance` feature is disabled — call site must not
/// rely on the receipt.
#[cfg(feature = "governance")]
fn build_pending_export_receipt(
    bundle_id: &str,
    envelope: &ExportEnvelopeV3,
) -> ExportReceipt {
    let envelope_json = serde_json::to_string(envelope).unwrap_or_default();
    let envelope_digest = ids::sha256_text(&envelope_json);
    let mut receipt = ExportReceipt::new(
        "forge_pilot_canonical_roundtrip",
        vec![bundle_id.to_string()],
        envelope.envelope_id.to_string(),
    );
    receipt.input_digests.insert("bundle".to_string(), envelope_digest.clone());
    // The output is the envelope itself, which exists before the
    // import step. Bind it here so both success and failure receipts
    // carry the envelope as a recoverable artifact.
    receipt.bind_output(
        format!("export_envelope:{}", envelope.envelope_id),
        envelope_digest,
    );
    receipt.status = "pending".to_string();
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

    // Build the receipt in `pending` state *before* the import so we
    // can mark it `success` or `failure` based on the outcome, and
    // return it on either path. This is the doctrinal fix for the
    // audit-trail gap: every material operation must produce a
    // receipt, including failures.
    #[cfg(feature = "governance")]
    let mut export_receipt = Some(build_pending_export_receipt(
        &bundle.bundle_id,
        &envelope,
    ));

    let import_outcome = memory_store.import_projection_batch(&batch).await;
    let import_result = match import_outcome {
        Ok(r) => {
            #[cfg(feature = "governance")]
            if let Some(ref mut r) = export_receipt {
                r.status = "success".to_string();
            }
            r
        }
        Err(e) => {
            #[cfg(feature = "governance")]
            if let Some(ref mut r) = export_receipt {
                r.status = "failure".to_string();
                r.degradation.push(format!("import_error: {e}"));
            }
            return Err(PilotError::from(e));
        }
    };
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
