use crate::bootstrap::types::{
    BootstrapChunkPolicyInfo, BootstrapManifestDelta, BootstrapManifestFile,
    BootstrapManifestSnapshot, BootstrapSourceRichness, BootstrapSourceSkippedFile,
    PreparedSourceFile, BOOTSTRAP_SOURCE_AUTHORITY, BOOTSTRAP_SOURCE_V1_METADATA_KEY,
    BOOTSTRAP_SOURCE_V2_METADATA_KEY, CHUNK_POLICY_VERSION, MAX_CHUNK_BYTES, MAX_CHUNK_LINES,
};
use forge_memory_bridge::{ImportProjectionRecord, ProjectionImportBatchV3};
use semantic_memory::ProjectionImportLogEntry;
use stack_ids::DigestBuilder;
use std::collections::BTreeMap;

pub(crate) fn build_manifest_snapshot(
    namespace: &str,
    files: &[PreparedSourceFile],
    skipped_files: &[BootstrapSourceSkippedFile],
) -> BootstrapManifestSnapshot {
    let files = files
        .iter()
        .map(|file| BootstrapManifestFile {
            path: file.file.relative_path.clone(),
            content_digest: file.file.content_digest.hex().to_string(),
            byte_count: file.file.byte_count,
            line_count: file.file.line_count,
            language: file.file.language.clone(),
            chunk_ids: file
                .chunks
                .iter()
                .map(|chunk| chunk.chunk_id.clone())
                .collect(),
            chunk_count: file.chunks.len(),
            symbol_ids: file
                .symbols
                .iter()
                .map(|symbol| symbol.symbol_id.clone())
                .collect(),
            symbol_count: file.symbols.len(),
            symbol_extraction_status: match file.symbol_capability.status {
                crate::bootstrap::types::BootstrapCapabilityStatus::Supported => "success".into(),
                crate::bootstrap::types::BootstrapCapabilityStatus::Degraded => "degraded".into(),
                crate::bootstrap::types::BootstrapCapabilityStatus::Unavailable => {
                    "unavailable".into()
                }
            },
            symbol_extraction_degradation: file.symbol_capability.degradation_reason.clone(),
            symbol_capability: file.symbol_capability.clone(),
        })
        .collect::<Vec<_>>();

    let file_count = files.len();
    let chunk_count = files.iter().map(|file| file.chunk_count).sum();
    let symbol_count = files.iter().map(|file| file.symbol_count).sum();
    let degraded_symbol_file_count = files
        .iter()
        .filter(|file| file.symbol_extraction_status == "degraded")
        .count();
    let richness = manifest_richness(
        file_count,
        chunk_count,
        symbol_count,
        degraded_symbol_file_count,
    );
    let manifest_seed = digest_text(
        &serde_json::to_string(&serde_json::json!({
            "namespace": namespace,
            "chunk_policy_version": CHUNK_POLICY_VERSION,
            "file_count": file_count,
            "chunk_count": chunk_count,
            "symbol_count": symbol_count,
            "degraded_symbol_file_count": degraded_symbol_file_count,
            "skipped_files": skipped_files,
            "files": files,
        }))
        .unwrap_or_else(|_| namespace.to_string()),
    );

    BootstrapManifestSnapshot {
        manifest_id: format!("workspace-source-manifest:{manifest_seed}"),
        namespace: namespace.to_string(),
        file_count,
        chunk_count,
        symbol_count,
        degraded_symbol_file_count,
        richness,
        chunk_policy: BootstrapChunkPolicyInfo {
            policy_version: CHUNK_POLICY_VERSION.into(),
            max_chunk_bytes: MAX_CHUNK_BYTES,
            max_chunk_lines: MAX_CHUNK_LINES,
            stable_anchor_strategy: "path+policy+boundary_anchor+content_digest".into(),
        },
        skipped_files: skipped_files.to_vec(),
        files,
    }
}

/// Computes the delta between the current bootstrap state and a manifest snapshot.
pub fn compute_manifest_delta(
    previous: Option<&BootstrapManifestSnapshot>,
    current: &BootstrapManifestSnapshot,
) -> BootstrapManifestDelta {
    let current_map = current
        .files
        .iter()
        .cloned()
        .map(|file| (file.path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let previous_map = previous
        .map(|manifest| {
            manifest
                .files
                .iter()
                .cloned()
                .map(|file| (file.path.clone(), file))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut new_files = Vec::new();
    let mut changed_files = Vec::new();
    let mut unchanged_files = Vec::new();
    let mut deleted_files = Vec::new();
    let mut source_unchanged_derived_changed_files = Vec::new();

    for (path, file) in &current_map {
        match previous_map.get(path) {
            None => new_files.push(file.clone()),
            Some(previous_file) if previous_file == file => unchanged_files.push(file.clone()),
            Some(previous_file) if previous_file.content_digest == file.content_digest => {
                source_unchanged_derived_changed_files.push(file.clone())
            }
            Some(_) => changed_files.push(file.clone()),
        }
    }

    for (path, file) in previous_map {
        if !current_map.contains_key(&path) {
            deleted_files.push(file);
        }
    }

    BootstrapManifestDelta {
        new_files,
        changed_files,
        unchanged_files,
        deleted_files,
        source_unchanged_derived_changed_files,
    }
}

pub(crate) fn latest_bootstrap_manifest(
    imports: &[ProjectionImportLogEntry],
) -> Option<BootstrapManifestSnapshot> {
    imports
        .iter()
        .filter(|row| {
            row.status == "complete" && row.source_authority == BOOTSTRAP_SOURCE_AUTHORITY
        })
        .find_map(|row| {
            let batch = row.rebuildable_kernel_batch_v3().ok().flatten()?;
            manifest_from_batch(&batch)
        })
}

/// Rebuilds a manifest snapshot from an imported projection batch when possible.
pub fn manifest_from_batch(batch: &ProjectionImportBatchV3) -> Option<BootstrapManifestSnapshot> {
    for record in &batch.records {
        let ImportProjectionRecord::ClaimVersion(claim) = &record.record else {
            continue;
        };
        let Some(meta) = claim.metadata.as_ref() else {
            continue;
        };
        let Some(v2) = meta.get(BOOTSTRAP_SOURCE_V2_METADATA_KEY) else {
            continue;
        };
        if v2.get("record_kind").and_then(serde_json::Value::as_str) != Some("manifest") {
            continue;
        }
        let snapshot = v2.get("manifest_snapshot")?.clone();
        return serde_json::from_value(snapshot).ok();
    }

    let mut legacy_files = Vec::new();
    for record in &batch.records {
        let ImportProjectionRecord::ClaimVersion(claim) = &record.record else {
            continue;
        };
        let Some(meta) = claim.metadata.as_ref() else {
            continue;
        };
        if let Some(v2) = meta.get(BOOTSTRAP_SOURCE_V2_METADATA_KEY) {
            if v2.get("record_kind").and_then(serde_json::Value::as_str) == Some("file") {
                let path = v2
                    .get("normalized_path")
                    .and_then(serde_json::Value::as_str)?;
                let digest = v2
                    .get("content_digest")
                    .and_then(serde_json::Value::as_str)?;
                legacy_files.push(BootstrapManifestFile {
                    path: path.to_string(),
                    content_digest: digest.to_string(),
                    byte_count: v2
                        .get("byte_count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or_default() as usize,
                    line_count: v2
                        .get("line_count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or_default() as usize,
                    language: v2
                        .get("language")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("generic")
                        .to_string(),
                    chunk_ids: v2
                        .get("chunk_ids")
                        .and_then(serde_json::Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(serde_json::Value::as_str)
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                    chunk_count: v2
                        .get("chunk_count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or_default() as usize,
                    symbol_ids: v2
                        .get("symbol_ids")
                        .and_then(serde_json::Value::as_array)
                        .map(|items| {
                            items
                                .iter()
                                .filter_map(serde_json::Value::as_str)
                                .map(ToString::to_string)
                                .collect::<Vec<_>>()
                        })
                        .unwrap_or_default(),
                    symbol_count: v2
                        .get("symbol_count")
                        .and_then(serde_json::Value::as_u64)
                        .unwrap_or_default() as usize,
                    symbol_extraction_status: v2
                        .get("symbol_extraction_status")
                        .and_then(serde_json::Value::as_str)
                        .unwrap_or("success")
                        .to_string(),
                    symbol_extraction_degradation: v2
                        .get("symbol_extraction_degradation")
                        .and_then(serde_json::Value::as_str)
                        .map(ToString::to_string),
                    symbol_capability: v2
                        .get("symbol_capability")
                        .cloned()
                        .and_then(|value| serde_json::from_value(value).ok())
                        .unwrap_or_else(|| crate::bootstrap::types::BootstrapSymbolCapability {
                            status: crate::bootstrap::types::BootstrapCapabilityStatus::Supported,
                            precision: crate::bootstrap::types::BootstrapSymbolPrecision::Heuristic,
                            extractor: "legacy_v2".into(),
                            policy_version: crate::bootstrap::types::SYMBOL_CAPABILITY_VERSION
                                .into(),
                            degradation_reason: None,
                        }),
                });
            }
            continue;
        }

        let Some(v1) = meta.get(BOOTSTRAP_SOURCE_V1_METADATA_KEY) else {
            continue;
        };
        let path = v1
            .get("normalized_path")
            .and_then(serde_json::Value::as_str)?;
        let digest = v1
            .get("content_digest")
            .and_then(serde_json::Value::as_str)?;
        legacy_files.push(BootstrapManifestFile {
            path: path.to_string(),
            content_digest: digest.to_string(),
            byte_count: v1
                .get("byte_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default() as usize,
            line_count: v1
                .get("line_count")
                .and_then(serde_json::Value::as_u64)
                .unwrap_or_default() as usize,
            language: v1
                .get("language")
                .and_then(serde_json::Value::as_str)
                .unwrap_or("generic")
                .to_string(),
            chunk_ids: Vec::new(),
            chunk_count: 0,
            symbol_ids: Vec::new(),
            symbol_count: 0,
            symbol_extraction_status: "legacy_thin".into(),
            symbol_extraction_degradation: None,
            symbol_capability: crate::bootstrap::types::BootstrapSymbolCapability {
                status: crate::bootstrap::types::BootstrapCapabilityStatus::Unavailable,
                precision: crate::bootstrap::types::BootstrapSymbolPrecision::Heuristic,
                extractor: "legacy_v1".into(),
                policy_version: crate::bootstrap::types::SYMBOL_CAPABILITY_VERSION.into(),
                degradation_reason: Some(
                    "legacy thin import has no symbol capability metadata".into(),
                ),
            },
        });
    }

    if legacy_files.is_empty() {
        return None;
    }

    legacy_files.sort_by(|left, right| left.path.cmp(&right.path));
    let file_count = legacy_files.len();
    let chunk_count = legacy_files.iter().map(|file| file.chunk_count).sum();
    let symbol_count = legacy_files.iter().map(|file| file.symbol_count).sum();
    let degraded_symbol_file_count = legacy_files
        .iter()
        .filter(|file| file.symbol_extraction_status == "degraded")
        .count();
    let richness = manifest_richness(
        file_count,
        chunk_count,
        symbol_count,
        degraded_symbol_file_count,
    );
    let manifest_seed = digest_text(
        &serde_json::to_string(&serde_json::json!({
            "scope": batch.scope_key,
            "files": legacy_files,
        }))
        .ok()?,
    );

    Some(BootstrapManifestSnapshot {
        manifest_id: format!("workspace-source-manifest:{manifest_seed}"),
        namespace: batch.scope_key.namespace.clone(),
        file_count,
        chunk_count,
        symbol_count,
        degraded_symbol_file_count,
        richness,
        chunk_policy: BootstrapChunkPolicyInfo {
            policy_version: CHUNK_POLICY_VERSION.into(),
            max_chunk_bytes: MAX_CHUNK_BYTES,
            max_chunk_lines: MAX_CHUNK_LINES,
            stable_anchor_strategy: "path+policy+boundary_anchor+content_digest".into(),
        },
        skipped_files: Vec::new(),
        files: legacy_files,
    })
}
pub(crate) fn manifest_richness(
    file_count: usize,
    chunk_count: usize,
    symbol_count: usize,
    degraded_symbol_file_count: usize,
) -> BootstrapSourceRichness {
    if file_count == 0 || chunk_count == 0 {
        BootstrapSourceRichness::Thin
    } else if symbol_count == 0 || degraded_symbol_file_count > 0 {
        BootstrapSourceRichness::Chunked
    } else {
        BootstrapSourceRichness::Symbolized
    }
}

pub(crate) fn digest_text(value: &str) -> String {
    let mut builder = DigestBuilder::new();
    builder.update_str(value);
    builder.finalize().hex().to_string()
}
