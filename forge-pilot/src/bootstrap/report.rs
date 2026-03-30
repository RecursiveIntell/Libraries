use crate::bootstrap::types::{
    BootstrapManifestDelta, BootstrapSourceReport, BootstrapSourceRichness,
    BOOTSTRAP_SOURCE_V2_METADATA_KEY,
};
use semantic_memory::ProjectionClaimVersion;

#[allow(clippy::too_many_arguments)]
pub(crate) fn build_report(
    namespace: &str,
    workspace_path: String,
    scanned_file_count: usize,
    skipped_files: Vec<crate::bootstrap::types::BootstrapSourceSkippedFile>,
    manifest: Option<&crate::bootstrap::types::BootstrapManifestSnapshot>,
    delta: Option<&BootstrapManifestDelta>,
    import_result: Option<&semantic_memory::ProjectionImportResult>,
    imported_chunk_count: usize,
    imported_symbol_count: usize,
    source_envelope_id: Option<String>,
    content_digest: Option<String>,
) -> BootstrapSourceReport {
    let skipped_file_count = skipped_files.len();
    let skipped_large_file_count = skipped_files
        .iter()
        .filter(|file| file.reason.contains("bootstrap limit"))
        .count();
    let (
        manifest_id,
        current_manifest_file_count,
        current_manifest_chunk_count,
        current_manifest_symbol_count,
        degraded_symbol_file_count,
        richness,
        latest_manifest_only,
    ) = manifest
        .map(|manifest| {
            (
                Some(manifest.manifest_id.clone()),
                manifest.file_count,
                manifest.chunk_count,
                manifest.symbol_count,
                manifest.degraded_symbol_file_count,
                manifest.richness,
                true,
            )
        })
        .unwrap_or((None, 0, 0, 0, 0, BootstrapSourceRichness::Thin, false));
    let source_delta = delta
        .map(BootstrapManifestDelta::source_delta_summary)
        .unwrap_or_default();
    let derived_delta = delta
        .map(BootstrapManifestDelta::derived_delta_summary)
        .unwrap_or_default();

    BootstrapSourceReport {
        namespace: namespace.to_string(),
        workspace_path,
        scanned_file_count,
        imported_file_count: delta
            .map(BootstrapManifestDelta::imported_file_count)
            .unwrap_or_default(),
        skipped_file_count,
        source_envelope_id,
        content_digest,
        import_status: import_result.map(|result| result.status.clone()),
        was_duplicate: import_result
            .map(|result| result.was_duplicate)
            .unwrap_or(false),
        skipped_files,
        manifest_id,
        current_manifest_file_count,
        current_manifest_chunk_count,
        current_manifest_symbol_count,
        latest_manifest_only,
        source_delta,
        derived_delta,
        imported_chunk_count,
        imported_symbol_count,
        degraded_symbol_file_count,
        richness,
        skipped_large_file_count,
    }
}

pub(crate) fn is_auxiliary_bootstrap_claim(claim: &ProjectionClaimVersion) -> bool {
    let Some(meta) = claim.metadata.as_ref() else {
        return false;
    };
    let Some(v2) = meta.get(BOOTSTRAP_SOURCE_V2_METADATA_KEY) else {
        return false;
    };
    matches!(
        v2.get("record_kind").and_then(serde_json::Value::as_str),
        Some("manifest" | "chunk" | "symbol" | "deletion")
    )
}
