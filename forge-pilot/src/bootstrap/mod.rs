mod capability;
mod chunk;
mod classify;
mod current_state;
mod delta;
mod extract;
mod ignore;
mod import;
pub(crate) mod manifest;
mod materialize;
mod pipeline;
pub(crate) mod report;
mod scan;
pub(crate) mod types;

use crate::config::LoopConfig;
use crate::error::PilotError;
use semantic_memory::MemoryStore;
use std::path::PathBuf;

pub(crate) use classify::is_supported_source_file_path;
pub(crate) use current_state::{
    current_state_from_latest_manifest, manifest_delta_from_current_state,
    manifest_from_current_state,
};
pub(crate) use ignore::should_skip_workspace_dir;
pub use manifest::{compute_manifest_delta, manifest_from_batch};
pub(crate) use report::is_auxiliary_bootstrap_claim;
pub use types::{
    BootstrapCurrentState, BootstrapDeltaSummary, BootstrapManifestDelta,
    BootstrapManifestSnapshot, BootstrapRichness, BootstrapSourceObservation,
    BootstrapSourceReport, BootstrapSourceRichness, BootstrapSourceSkippedFile, SymbolRecord,
};

pub async fn bootstrap_source_workspace(
    memory_store: &MemoryStore,
    config: &LoopConfig,
) -> Result<BootstrapSourceReport, PilotError> {
    let workspace_path = resolve_runtime_path(&config.workspace_path);
    let pipeline =
        pipeline::build_bootstrap_pipeline(memory_store, config, &workspace_path).await?;
    if pipeline.prepared_files.is_empty() && pipeline.previous_manifest.is_none() {
        return Ok(report::build_report(
            &config.scope.namespace,
            workspace_path.to_string_lossy().to_string(),
            0,
            pipeline.scan.skipped_files,
            None,
            None,
            None,
            0,
            0,
            None,
            None,
        ));
    }

    if pipeline.delta.is_noop() {
        let import_result = semantic_memory::ProjectionImportResult {
            source_envelope_id: format!(
                "workspace-source-envelope-{}",
                pipeline.current_manifest.manifest_id
            ),
            status: "already_imported".into(),
            record_count: 0,
            was_duplicate: true,
        };
        return Ok(report::build_report(
            &config.scope.namespace,
            workspace_path.to_string_lossy().to_string(),
            pipeline.prepared_files.len(),
            pipeline.scan.skipped_files,
            Some(&pipeline.current_manifest),
            Some(&pipeline.delta),
            Some(&import_result),
            0,
            0,
            Some(import_result.source_envelope_id.clone()),
            None,
        ));
    }
    let materialized = materialize::materialize_records(
        &pipeline.current_manifest,
        pipeline.previous_manifest.as_ref(),
        &pipeline.delta,
        &pipeline.prepared_files,
    )?;
    let envelope = import::build_source_envelope(
        &config.scope.namespace,
        &pipeline.current_manifest,
        materialized.records,
    )?;
    let import_result = import::import_envelope(memory_store, &envelope).await?;

    Ok(report::build_report(
        &config.scope.namespace,
        workspace_path.to_string_lossy().to_string(),
        pipeline.prepared_files.len(),
        pipeline.scan.skipped_files,
        Some(&pipeline.current_manifest),
        Some(&pipeline.delta),
        Some(&import_result),
        materialized.imported_chunk_count,
        materialized.imported_symbol_count,
        Some(envelope.envelope_id.as_str().to_string()),
        Some(envelope.content_digest.hex().to_string()),
    ))
}
fn resolve_runtime_path(raw: &str) -> PathBuf {
    let path = PathBuf::from(raw);
    if path.is_absolute() {
        path
    } else {
        std::env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(path)
    }
}
