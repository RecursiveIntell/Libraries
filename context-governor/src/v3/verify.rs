use super::*;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(deny_unknown_fields)]
pub struct V3VerificationReportV1 {
    pub schema: String,
    pub input_receipts: usize,
    pub verified_v2_receipts: usize,
    pub skipped_v1_receipts: usize,
    pub exact_evidence_items: usize,
    pub unique_evidence_blobs: usize,
    pub plaintext_bytes: u64,
    pub blob_bytes: u64,
    pub source_receipts_sha256: String,
    pub complete: bool,
}

/// Verify an existing V3 projection read-only against the authoritative V2
/// source snapshot. This does not activate V3 or permit migration resume.
pub fn verify_v3_projection(
    store: &FileContextStore,
    output_root: impl AsRef<Path>,
    encryption_key: Option<&[u8]>,
) -> Result<V3VerificationReportV1, ContextGovernorError> {
    let output_root = output_root.as_ref();
    check_path(output_root)?;
    let binding = descriptor(output_root)?;
    let (ids, source_digest) = source_snapshot(store)?;
    if binding.input_receipts != ids.len() || binding.source_receipts_sha256 != source_digest {
        return Err(v3_error(V3ProjectionError::SourceChanged));
    }
    if let Some(expected) = binding.encryption_key_id.as_deref() {
        let key = encryption_key.ok_or_else(|| v3_error(V3ProjectionError::SourceMismatch))?;
        if key.len() != 32 || key_id(key) != expected {
            return Err(v3_error(V3ProjectionError::SourceMismatch));
        }
    }

    let migration_report: V3MigrationReportV2 = serde_json::from_slice(&read_bounded(
        &v3_root(output_root).join("migration-report.json"),
        MAX_MANIFEST_BYTES,
    )?)?;
    if migration_report.schema != "V3MigrationReportV2"
        || migration_report.input_receipts != ids.len()
        || migration_report.source_receipts_sha256 != source_digest
        || migration_report.encrypted != binding.encryption_key_id.is_some()
        || !migration_report.source_scan_complete
    {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }

    let mut verified = V3VerificationReportV1 {
        schema: "V3VerificationReportV1".into(),
        input_receipts: ids.len(),
        source_receipts_sha256: source_digest.clone(),
        ..Default::default()
    };
    let mut expected_manifests = BTreeSet::new();
    let mut unique_blobs: BTreeMap<String, (String, bool)> = BTreeMap::new();
    for receipt_id in &ids {
        match store.load_versioned(receipt_id)? {
            VersionedCompactResponse::V1(_) => {
                verified.skipped_v1_receipts += 1;
            }
            VersionedCompactResponse::V2(_) => {
                let manifest = read_v3_manifest(output_root, receipt_id)?;
                verify_v3_manifest_against_v2(store, output_root, &manifest)?;
                expected_manifests.insert(format!("{receipt_id}.json"));
                verified.verified_v2_receipts += 1;
                for reference in &manifest.evidence {
                    let message = read_blob_evidence(
                        output_root,
                        &manifest,
                        &reference.source_id,
                        encryption_key,
                    )?;
                    verified.exact_evidence_items += 1;
                    unique_blobs
                        .entry(reference.message_sha256.clone())
                        .or_insert_with(|| (reference.source_id.clone(), reference.encrypted));
                    if sha256_bytes(&serde_json::to_vec(&message)?) != reference.message_sha256 {
                        return Err(v3_error(V3ProjectionError::SourceMismatch));
                    }
                }
            }
        }
    }

    let manifest_root = v3_root(output_root).join(MANIFEST_ROOT);
    let mut actual_manifests = BTreeSet::new();
    if expected_manifests.is_empty() {
        if manifest_root.exists() && fs::read_dir(&manifest_root)?.next().is_some() {
            return Err(v3_error(V3ProjectionError::UnsafePath));
        }
    } else {
        check_path(&manifest_root)?;
        for entry in fs::read_dir(&manifest_root)? {
            let entry = entry?;
            let metadata = fs::symlink_metadata(entry.path())?;
            if !metadata.is_file() || metadata.file_type().is_symlink() {
                return Err(v3_error(V3ProjectionError::UnsafePath));
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| v3_error(V3ProjectionError::UnsafePath))?;
            actual_manifests.insert(name);
        }
        if actual_manifests != expected_manifests {
            return Err(v3_error(V3ProjectionError::SourceMismatch));
        }
    }

    for (digest, (source_id, encrypted)) in &unique_blobs {
        let path = blob_path(output_root, digest, *encrypted);
        verified.blob_bytes += fs::symlink_metadata(&path)?.len();
        let manifest = ids
            .iter()
            .filter_map(|receipt_id| read_v3_manifest(output_root, receipt_id).ok())
            .find(|manifest| {
                manifest.evidence.iter().any(|reference| {
                    &reference.message_sha256 == digest && &reference.source_id == source_id
                })
            })
            .ok_or_else(|| v3_error(V3ProjectionError::SourceMismatch))?;
        let message = read_blob_evidence(output_root, &manifest, source_id, encryption_key)?;
        verified.plaintext_bytes += serde_json::to_vec(&message)?.len() as u64;
    }
    verified.unique_evidence_blobs = unique_blobs.len();

    if migration_report.migrated_receipts != verified.verified_v2_receipts
        || migration_report.skipped_v1_receipts != verified.skipped_v1_receipts
        || migration_report.exact_evidence_items != verified.exact_evidence_items
        || migration_report.unique_evidence_blobs != verified.unique_evidence_blobs
        || migration_report.plaintext_bytes != verified.plaintext_bytes
        || migration_report.blob_bytes != verified.blob_bytes
        || migration_report.v2_projection_complete
            != (verified.verified_v2_receipts + verified.skipped_v1_receipts == ids.len())
        || migration_report.full_corpus_migrated
            != (verified.skipped_v1_receipts == 0 && !ids.is_empty())
        || migration_report.complete != migration_report.full_corpus_migrated
    {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }

    let (_, after_digest) = source_snapshot(store)?;
    if after_digest != source_digest {
        return Err(v3_error(V3ProjectionError::SourceChanged));
    }
    verified.complete = migration_report.complete;
    Ok(verified)
}
