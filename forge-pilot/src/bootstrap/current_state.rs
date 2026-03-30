use crate::bootstrap::manifest::manifest_from_batch;
use crate::bootstrap::types::{
    BootstrapCurrentState, BootstrapManifestDelta, BootstrapManifestSnapshot,
};
use forge_memory_bridge::ImportProjectionRecord;
use semantic_memory::ProjectionImportLogEntry;

pub(crate) fn current_state_from_latest_manifest(
    recent_imports: &[ProjectionImportLogEntry],
) -> BootstrapCurrentState {
    let Some(latest) = latest_manifest_import(recent_imports) else {
        return BootstrapCurrentState::default();
    };
    let Some(batch) = latest.rebuildable_kernel_batch_v3().ok().flatten() else {
        return BootstrapCurrentState {
            source_envelope_id: Some(latest.source_envelope_id.clone()),
            imported_at: Some(latest.imported_at.clone()),
            import_disposition: "missing_rebuildable_batch".into(),
            ..BootstrapCurrentState::default()
        };
    };
    let Some(manifest) = manifest_from_batch(&batch) else {
        return BootstrapCurrentState {
            source_envelope_id: Some(latest.source_envelope_id.clone()),
            imported_at: Some(latest.imported_at.clone()),
            import_disposition: "missing_manifest".into(),
            ..BootstrapCurrentState::default()
        };
    };

    BootstrapCurrentState {
        source_envelope_id: Some(latest.source_envelope_id.clone()),
        imported_at: Some(latest.imported_at.clone()),
        import_disposition: "latest_manifest".into(),
        ..BootstrapCurrentState::from_manifest(&manifest)
    }
}

pub(crate) fn latest_manifest_import(
    imports: &[ProjectionImportLogEntry],
) -> Option<&ProjectionImportLogEntry> {
    imports.iter().find(|row| {
        row.status == "complete"
            && row.source_authority == crate::bootstrap::types::BOOTSTRAP_SOURCE_AUTHORITY
            && row
                .rebuildable_kernel_batch_v3()
                .ok()
                .flatten()
                .and_then(|batch| manifest_from_batch(&batch))
                .is_some()
    })
}

pub(crate) fn manifest_from_current_state(
    current_state: &BootstrapCurrentState,
    imports: &[ProjectionImportLogEntry],
) -> Option<BootstrapManifestSnapshot> {
    let manifest_id = current_state.manifest_id.as_deref()?;
    imports
        .iter()
        .filter(|row| row.status == "complete")
        .find_map(|row| {
            let batch = row.rebuildable_kernel_batch_v3().ok().flatten()?;
            let manifest = manifest_from_batch(&batch)?;
            (manifest.manifest_id == manifest_id).then_some(manifest)
        })
}

pub(crate) fn manifest_delta_from_current_state(
    current_state: &BootstrapCurrentState,
    imports: &[ProjectionImportLogEntry],
) -> Option<BootstrapManifestDelta> {
    let manifest_id = current_state.manifest_id.as_deref()?;
    imports
        .iter()
        .filter(|row| row.status == "complete")
        .find_map(|row| {
            let batch = row.rebuildable_kernel_batch_v3().ok().flatten()?;
            manifest_delta_from_batch(&batch, manifest_id)
        })
}

fn manifest_delta_from_batch(
    batch: &forge_memory_bridge::ProjectionImportBatchV3,
    manifest_id: &str,
) -> Option<BootstrapManifestDelta> {
    for record in &batch.records {
        let ImportProjectionRecord::ClaimVersion(claim) = &record.record else {
            continue;
        };
        let meta = claim
            .metadata
            .as_ref()?
            .get(crate::bootstrap::types::BOOTSTRAP_SOURCE_V2_METADATA_KEY)?;
        if meta.get("record_kind").and_then(serde_json::Value::as_str) != Some("manifest") {
            continue;
        }
        if meta.get("manifest_id").and_then(serde_json::Value::as_str) != Some(manifest_id) {
            continue;
        }
        return meta
            .get("delta")
            .cloned()
            .and_then(|value| serde_json::from_value(value).ok());
    }
    None
}
