use crate::bootstrap::capability::symbol_capability;
use crate::bootstrap::chunk::chunk_source_file;
use crate::bootstrap::extract::extract_symbols;
use crate::bootstrap::manifest::{build_manifest_snapshot, latest_bootstrap_manifest};
use crate::bootstrap::scan::{scan_workspace_source_files, WorkspaceScan};
use crate::bootstrap::types::{
    BootstrapManifestDelta, BootstrapManifestSnapshot, PreparedSourceFile,
};
use crate::config::LoopConfig;
use crate::error::PilotError;
use semantic_memory::MemoryStore;
use std::path::Path;

pub(crate) struct BootstrapPipeline {
    pub scan: WorkspaceScan,
    pub prepared_files: Vec<PreparedSourceFile>,
    pub previous_manifest: Option<BootstrapManifestSnapshot>,
    pub current_manifest: BootstrapManifestSnapshot,
    pub delta: BootstrapManifestDelta,
}

pub(crate) async fn build_bootstrap_pipeline(
    memory_store: &MemoryStore,
    config: &LoopConfig,
    workspace_path: &Path,
) -> Result<BootstrapPipeline, PilotError> {
    let scan = scan_workspace_source_files(workspace_path)?;
    let prepared_files = scan
        .files
        .iter()
        .cloned()
        .map(|file| {
            let chunks = chunk_source_file(&file);
            let extraction = extract_symbols(&file, &chunks);
            let capability = symbol_capability(&file.language, extraction.degraded_reason);
            PreparedSourceFile {
                file,
                chunks,
                symbols: extraction.symbols,
                symbol_capability: capability,
            }
        })
        .collect::<Vec<_>>();
    let previous_imports = memory_store
        .query_projection_imports(Some(&config.scope.namespace), 64)
        .await?;
    let previous_manifest = latest_bootstrap_manifest(&previous_imports);
    let current_manifest = build_manifest_snapshot(
        &config.scope.namespace,
        &prepared_files,
        &scan.skipped_files,
    );
    let delta = crate::bootstrap::delta::compute_manifest_delta(
        previous_manifest.as_ref(),
        &current_manifest,
    );

    Ok(BootstrapPipeline {
        scan,
        prepared_files,
        previous_manifest,
        current_manifest,
        delta,
    })
}
