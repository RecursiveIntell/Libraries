use crate::bootstrap::types::{
    BootstrapManifestDelta, BootstrapManifestFile, BootstrapManifestSnapshot, PreparedSourceFile,
    SymbolRecord, BOOTSTRAP_SOURCE_V2_METADATA_KEY,
};
use crate::error::PilotError;
use semantic_memory_forge::{ExportClaim, ExportEvidenceRef, ExportRecord, ExportRecordV3};
use stack_ids::{ClaimId, ClaimVersionId, EntityId};
use std::collections::BTreeMap;

pub(crate) struct MaterializedBootstrap {
    pub records: Vec<ExportRecordV3>,
    pub imported_chunk_count: usize,
    pub imported_symbol_count: usize,
}

pub(crate) fn materialize_records(
    manifest: &BootstrapManifestSnapshot,
    previous_manifest: Option<&BootstrapManifestSnapshot>,
    delta: &BootstrapManifestDelta,
    files: &[PreparedSourceFile],
) -> Result<MaterializedBootstrap, PilotError> {
    let file_map = files
        .iter()
        .map(|file| (file.file.relative_path.clone(), file))
        .collect::<BTreeMap<_, _>>();
    let previous_map = previous_manifest
        .map(|snapshot| {
            snapshot
                .files
                .iter()
                .map(|file| (file.path.clone(), file))
                .collect::<BTreeMap<_, _>>()
        })
        .unwrap_or_default();

    let mut records = vec![manifest_claim(manifest, delta)?];
    let mut imported_chunk_count = 0usize;
    let mut imported_symbol_count = 0usize;

    for file in delta
        .new_files
        .iter()
        .chain(delta.changed_files.iter())
        .chain(delta.source_unchanged_derived_changed_files.iter())
    {
        let prepared = file_map.get(&file.path).ok_or_else(|| {
            PilotError::Other(format!(
                "missing prepared source file for manifest path {}",
                file.path
            ))
        })?;
        records.push(file_claim(manifest, prepared));
        records.push(file_evidence_ref(manifest, prepared));
        for chunk in &prepared.chunks {
            records.push(chunk_claim(manifest, prepared, chunk));
            imported_chunk_count += 1;
        }
        for symbol in &prepared.symbols {
            records.push(symbol_claim(manifest, prepared, symbol));
            imported_symbol_count += 1;
        }
    }

    for deleted in &delta.deleted_files {
        let previous = previous_map.get(&deleted.path).copied().unwrap_or(deleted);
        records.push(deletion_claim(manifest, previous));
    }

    Ok(MaterializedBootstrap {
        records,
        imported_chunk_count,
        imported_symbol_count,
    })
}

fn manifest_claim(
    manifest: &BootstrapManifestSnapshot,
    delta: &BootstrapManifestDelta,
) -> Result<ExportRecordV3, PilotError> {
    let manifest_meta = serde_json::json!({
        BOOTSTRAP_SOURCE_V2_METADATA_KEY: {
            "record_kind": "manifest",
            "manifest_id": manifest.manifest_id,
            "richness": manifest.richness,
            "manifest_snapshot": manifest,
            "delta": delta,
        },
        "verification_summary": {
            "lifecycle_state": "unverified",
            "notes": ["workspace source bootstrap manifest snapshot"]
        }
    });

    Ok(ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(format!(
                "workspace-source-manifest-claim-{}",
                manifest.manifest_id
            ))),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-manifest-version-{}",
                manifest.manifest_id
            ))),
            subject_entity_id: EntityId::new(format!(
                "workspace-source-manifest-entity-{}",
                manifest.namespace
            )),
            predicate: "describes_workspace_source_manifest".into(),
            object_anchor: serde_json::json!({
                "manifest_id": manifest.manifest_id,
                "kind": "workspace_source_manifest"
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.05,
            content: format!(
                "Workspace source manifest {} for namespace {} with {} file(s).",
                manifest.manifest_id, manifest.namespace, manifest.file_count
            ),
            projection_family: "workspace_source".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(manifest_meta),
        }),
        semantics: None,
    })
}

fn file_claim(
    manifest: &BootstrapManifestSnapshot,
    prepared: &PreparedSourceFile,
) -> ExportRecordV3 {
    let file = &prepared.file;
    let metadata = serde_json::json!({
        BOOTSTRAP_SOURCE_V2_METADATA_KEY: {
            "record_kind": "file",
            "manifest_id": manifest.manifest_id,
            "path": file.relative_path,
            "normalized_path": file.relative_path,
            "content_digest": file.content_digest.hex().to_string(),
            "byte_count": file.byte_count,
            "line_count": file.line_count,
            "file_extension": file.file_extension,
            "language": file.language,
            "chunk_ids": prepared.chunks.iter().map(|chunk| chunk.chunk_id.clone()).collect::<Vec<_>>(),
            "chunk_count": prepared.chunks.len(),
            "symbol_ids": prepared.symbols.iter().map(|symbol| symbol.symbol_id.clone()).collect::<Vec<_>>(),
            "symbol_count": prepared.symbols.len(),
            "symbol_extraction_status": match prepared.symbol_capability.status {
                crate::bootstrap::types::BootstrapCapabilityStatus::Supported => "success",
                crate::bootstrap::types::BootstrapCapabilityStatus::Degraded => "degraded",
                crate::bootstrap::types::BootstrapCapabilityStatus::Unavailable => "unavailable",
            },
            "symbol_extraction_degradation": prepared.symbol_capability.degradation_reason,
            "symbol_capability": prepared.symbol_capability,
        },
        "verification_summary": {
            "lifecycle_state": "unverified",
            "notes": ["workspace source bootstrap import"]
        }
    });
    let path_seed = crate::bootstrap::manifest::digest_text(&file.relative_path);
    let version_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}",
        manifest.manifest_id,
        file.relative_path,
        file.content_digest.hex()
    ));
    let claim_id = format!("workspace-source-file-claim-{path_seed}-{version_seed}");

    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(claim_id.clone())),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-file-version-{version_seed}"
            ))),
            subject_entity_id: EntityId::new(format!("workspace-source-file-{path_seed}")),
            predicate: "describes_workspace_source_file".into(),
            object_anchor: serde_json::json!({
                "path": file.relative_path,
                "kind": "workspace_source_file"
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.05,
            content: format!("File: {}\n\n{}", file.relative_path, file.content),
            projection_family: "workspace_source".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(metadata),
        }),
        semantics: None,
    }
}

fn file_evidence_ref(
    manifest: &BootstrapManifestSnapshot,
    prepared: &PreparedSourceFile,
) -> ExportRecordV3 {
    let file = &prepared.file;
    let path_seed = crate::bootstrap::manifest::digest_text(&file.relative_path);
    let version_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}",
        manifest.manifest_id,
        file.relative_path,
        file.content_digest.hex()
    ));

    ExportRecordV3 {
        record: ExportRecord::EvidenceRef(ExportEvidenceRef {
            claim_id: ClaimId::new(format!(
                "workspace-source-file-claim-{path_seed}-{version_seed}"
            )),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-file-version-{version_seed}"
            ))),
            fetch_handle: format!("workspace-source://{}", file.relative_path),
            source_authority: crate::bootstrap::types::BOOTSTRAP_SOURCE_AUTHORITY.into(),
            metadata: Some(serde_json::json!({
                "normalized_path": file.relative_path,
                "content_digest": file.content_digest.hex().to_string(),
                "language": file.language,
            })),
        }),
        semantics: None,
    }
}

fn chunk_claim(
    manifest: &BootstrapManifestSnapshot,
    prepared: &PreparedSourceFile,
    chunk: &crate::bootstrap::types::ChunkRecord,
) -> ExportRecordV3 {
    let file = &prepared.file;
    let version_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}",
        manifest.manifest_id, file.relative_path, chunk.chunk_id
    ));

    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(format!(
                "{}-{}",
                chunk
                    .chunk_id
                    .as_str()
                    .split_once(":")
                    .map(|(_, p)| p)
                    .unwrap_or(chunk.chunk_id.as_str()),
                manifest.manifest_id
            ))),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-chunk-version-{version_seed}"
            ))),
            subject_entity_id: EntityId::new(format!(
                "workspace-source-chunk-entity-{}",
                chunk.chunk_id
            )),
            predicate: "describes_workspace_source_chunk".into(),
            object_anchor: serde_json::json!({
                "path": file.relative_path,
                "chunk_id": chunk.chunk_id,
                "kind": "workspace_source_chunk"
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.03,
            content: format!(
                "Chunk {} of {}\n\n{}",
                chunk.chunk_index, file.relative_path, chunk.content
            ),
            projection_family: "workspace_source".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                BOOTSTRAP_SOURCE_V2_METADATA_KEY: {
                    "record_kind": "chunk",
                    "manifest_id": manifest.manifest_id,
                    "path": file.relative_path,
                    "normalized_path": file.relative_path,
                    "language": file.language,
                    "chunk_id": chunk.chunk_id,
                    "chunk_index": chunk.chunk_index,
                    "chunk_content_digest": chunk.content_digest.hex().to_string(),
                    "start_line": chunk.start_line,
                    "end_line": chunk.end_line,
                    "byte_count": chunk.byte_count,
                    "stable_anchor": chunk.stable_anchor,
                },
                "verification_summary": {
                    "lifecycle_state": "unverified",
                    "notes": ["workspace source bootstrap chunk import"]
                }
            })),
        }),
        semantics: None,
    }
}

fn symbol_claim(
    manifest: &BootstrapManifestSnapshot,
    prepared: &PreparedSourceFile,
    symbol: &SymbolRecord,
) -> ExportRecordV3 {
    let file = &prepared.file;
    let version_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}",
        manifest.manifest_id, file.relative_path, symbol.symbol_id
    ));

    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(format!(
                "{}-{}",
                symbol
                    .symbol_id
                    .as_str()
                    .split_once(":")
                    .map(|(_, p)| p)
                    .unwrap_or(symbol.symbol_id.as_str()),
                manifest.manifest_id
            ))),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-symbol-version-{version_seed}"
            ))),
            subject_entity_id: EntityId::new(format!(
                "workspace-source-symbol-entity-{}",
                symbol
                    .symbol_id
                    .as_str()
                    .split_once(":")
                    .map(|(_, p)| p)
                    .unwrap_or(symbol.symbol_id.as_str())
            )),
            predicate: "describes_workspace_source_symbol".into(),
            object_anchor: serde_json::json!({
                "path": file.relative_path,
                "symbol_id": symbol.symbol_id,
                "kind": "workspace_source_symbol"
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.02,
            content: format!(
                "Symbol {} ({}) in {}",
                symbol.name, symbol.kind, file.relative_path
            ),
            projection_family: "workspace_source".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                BOOTSTRAP_SOURCE_V2_METADATA_KEY: {
                    "record_kind": "symbol",
                    "manifest_id": manifest.manifest_id,
                    "path": file.relative_path,
                    "normalized_path": file.relative_path,
                    "language": symbol.language,
                    "symbol_id": symbol.symbol_id,
                    "symbol_name": symbol.name,
                    "symbol_kind": symbol.kind,
                    "line_start": symbol.line_start,
                    "line_end": symbol.line_end,
                    "signature": symbol.signature,
                    "parent_chunk_id": symbol.parent_chunk_id,
                    "symbol_capability": prepared.symbol_capability,
                },
                "verification_summary": {
                    "lifecycle_state": "unverified",
                    "notes": ["workspace source bootstrap symbol import"]
                }
            })),
        }),
        semantics: None,
    }
}

fn deletion_claim(
    manifest: &BootstrapManifestSnapshot,
    deleted: &BootstrapManifestFile,
) -> ExportRecordV3 {
    let deletion_seed = crate::bootstrap::manifest::digest_text(&format!(
        "{}:{}:{}",
        manifest.manifest_id, deleted.path, deleted.content_digest
    ));
    ExportRecordV3 {
        record: ExportRecord::Claim(ExportClaim {
            claim_id: Some(ClaimId::new(format!(
                "workspace-source-deletion-claim-{deletion_seed}"
            ))),
            claim_version_id: Some(ClaimVersionId::new(format!(
                "workspace-source-deletion-version-{deletion_seed}"
            ))),
            subject_entity_id: EntityId::new(format!(
                "workspace-source-file-deletion-{}",
                crate::bootstrap::manifest::digest_text(&deleted.path)
            )),
            predicate: "describes_workspace_source_deletion".into(),
            object_anchor: serde_json::json!({
                "path": deleted.path,
                "kind": "workspace_source_deletion"
            }),
            valid_from: None,
            valid_to: None,
            confidence: 0.05,
            content: format!("Deleted file: {}", deleted.path),
            projection_family: "workspace_source".into(),
            supersedes_claim_id: None,
            supersedes_claim_version_id: None,
            metadata: Some(serde_json::json!({
                BOOTSTRAP_SOURCE_V2_METADATA_KEY: {
                    "record_kind": "deletion",
                    "manifest_id": manifest.manifest_id,
                    "path": deleted.path,
                    "normalized_path": deleted.path,
                    "previous_content_digest": deleted.content_digest,
                    "previous_chunk_ids": deleted.chunk_ids,
                    "previous_symbol_ids": deleted.symbol_ids,
                },
                "verification_summary": {
                    "lifecycle_state": "unverified",
                    "notes": ["workspace source bootstrap deletion import"]
                }
            })),
        }),
        semantics: None,
    }
}
