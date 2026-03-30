use crate::error::PilotError;
use forge_engine::{export_bundle, ExperimentEvidenceBundle, ForgeStore};
use forge_memory_bridge::transform_envelope_v3;
use semantic_memory::{MemoryStore, ProjectionImportResult};
use semantic_memory_forge::ExportEnvelopeV3;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoundtripResult {
    pub envelope: ExportEnvelopeV3,
    pub import_result: ProjectionImportResult,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImportBootstrapReport {
    pub namespace: String,
    pub forge_bundle_count: usize,
    pub imported_bundle_ids: Vec<String>,
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
    Ok(RoundtripResult {
        envelope,
        import_result,
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
    for bundle_id in bundle_ids {
        let bundle_row = forge_store
            .get_evidence_bundle(&bundle_id)?
            .ok_or_else(|| PilotError::Other(format!("missing Forge bundle row {bundle_id}")))?;
        let bundle = bundle_row.local_bundle()?;
        canonical_roundtrip(&bundle, namespace, forge_store, memory_store).await?;
        imported_bundle_ids.push(bundle_id);
    }

    Ok(ImportBootstrapReport {
        namespace: namespace.into(),
        forge_bundle_count,
        imported_bundle_ids,
    })
}
