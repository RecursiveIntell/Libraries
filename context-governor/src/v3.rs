//! Side-by-side Context Governor V3 evidence migration.
//!
//! V2 JSON receipts remain authoritative and immutable. This module creates a
//! rebuildable V3 projection containing content-addressed exact evidence,
//! parent/local-source delta metadata, and optional authenticated encryption.
//! It never changes the active writer or treats the V3 projection as lineage
//! authority.

use crate::{
    hash_text_sha256, ContextGovernorError, FileContextStore, Message, VersionedCompactResponse,
};
use aes_gcm::{aead::Aead, Aes256Gcm, KeyInit, Nonce};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
use std::fs;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};

const V3_ROOT: &str = ".v3";
const MANIFEST_ROOT: &str = "manifests";
const EVIDENCE_ROOT: &str = "evidence/sha256";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V3EvidenceRefV1 {
    pub source_id: String,
    pub message_sha256: String,
    pub content_sha256: String,
    pub blob_relpath: String,
    pub encrypted: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub nonce_hex: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V3ReceiptManifestV1 {
    pub schema: String,
    pub receipt_id: String,
    pub session_id: String,
    pub generation: u32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parent_receipt_id: Option<String>,
    pub local_source_ids: Vec<String>,
    pub covered_source_ids_sha256: String,
    pub compacted_transcript_sha256: String,
    pub evidence: Vec<V3EvidenceRefV1>,
    pub compression: String,
    pub encryption: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub encryption_key_id: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct V3MigrationOptions {
    #[serde(default, skip_serializing)]
    pub encryption_key: Option<Vec<u8>>,
    #[serde(default)]
    pub require_encryption: bool,
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,
}

fn default_compression_level() -> i32 {
    3
}

impl Default for V3MigrationOptions {
    fn default() -> Self {
        Self {
            encryption_key: None,
            require_encryption: false,
            compression_level: default_compression_level(),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
pub struct V3MigrationReportV1 {
    pub schema: String,
    pub input_receipts: usize,
    pub migrated_receipts: usize,
    pub skipped_v1_receipts: usize,
    pub mismatched_receipts: Vec<String>,
    pub exact_evidence_items: usize,
    pub plaintext_bytes: u64,
    pub blob_bytes: u64,
    pub encrypted: bool,
    pub complete: bool,
}

fn io_error(message: impl Into<String>) -> ContextGovernorError {
    ContextGovernorError::Io(Error::new(ErrorKind::InvalidData, message.into()))
}

fn sha256_bytes(bytes: &[u8]) -> String {
    format!("{:x}", Sha256::digest(bytes))
}

fn key_id(key: &[u8]) -> String {
    sha256_bytes(key)
}

fn v3_root(output_root: &Path) -> PathBuf {
    output_root.join(V3_ROOT)
}

fn manifest_path(output_root: &Path, receipt_id: &str) -> PathBuf {
    v3_root(output_root)
        .join(MANIFEST_ROOT)
        .join(format!("{receipt_id}.json"))
}

fn blob_path(output_root: &Path, digest: &str, encrypted: bool) -> PathBuf {
    let suffix = if encrypted { ".zst.enc" } else { ".zst" };
    v3_root(output_root)
        .join(EVIDENCE_ROOT)
        .join(&digest[..2])
        .join(&digest[2..4])
        .join(format!("{digest}{suffix}"))
}

fn blob_relpath(digest: &str, encrypted: bool) -> String {
    let suffix = if encrypted { ".zst.enc" } else { ".zst" };
    format!(
        "{EVIDENCE_ROOT}/{}/{}/{}{}",
        &digest[..2],
        &digest[2..4],
        digest,
        suffix
    )
}

fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ContextGovernorError> {
    if path.exists() {
        return Ok(());
    }
    let parent = path
        .parent()
        .ok_or_else(|| io_error("V3 path has no parent"))?;
    fs::create_dir_all(parent)?;
    let tmp = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
    fs::write(&tmp, bytes)?;
    fs::rename(&tmp, path)?;
    Ok(())
}

fn encode_blob(
    message_bytes: &[u8],
    options: &V3MigrationOptions,
) -> Result<(Vec<u8>, bool, Option<String>), ContextGovernorError> {
    let compressed = zstd::stream::encode_all(message_bytes, options.compression_level)
        .map_err(|error| io_error(format!("V3 compression failed: {error}")))?;
    let Some(key) = options.encryption_key.as_deref() else {
        if options.require_encryption {
            return Err(io_error("V3 migration requires an encryption key"));
        }
        return Ok((compressed, false, None));
    };
    if key.len() != 32 {
        return Err(io_error(format!(
            "V3 encryption key must be 32 bytes, got {}",
            key.len()
        )));
    }
    let cipher = Aes256Gcm::new_from_slice(key)
        .map_err(|_| io_error("V3 encryption key could not initialize AES-256-GCM"))?;
    let mut nonce = [0u8; 12];
    rand::thread_rng().fill_bytes(&mut nonce);
    let encrypted = cipher
        .encrypt(Nonce::from_slice(&nonce), compressed.as_ref())
        .map_err(|_| io_error("V3 evidence encryption failed"))?;
    Ok((encrypted, true, Some(hex::encode(nonce))))
}

fn decode_blob(
    bytes: &[u8],
    encrypted: bool,
    nonce_hex: Option<&str>,
    key: Option<&[u8]>,
) -> Result<Vec<u8>, ContextGovernorError> {
    let compressed = if encrypted {
        let key = key.ok_or_else(|| io_error("encrypted V3 evidence requires a key"))?;
        if key.len() != 32 {
            return Err(io_error("V3 decryption key must be 32 bytes"));
        }
        let nonce_hex = nonce_hex.ok_or_else(|| io_error("encrypted V3 evidence has no nonce"))?;
        let nonce = hex::decode(nonce_hex).map_err(|_| io_error("invalid V3 nonce encoding"))?;
        if nonce.len() != 12 {
            return Err(io_error("V3 nonce must be 12 bytes"));
        }
        let cipher = Aes256Gcm::new_from_slice(key)
            .map_err(|_| io_error("V3 decryption key could not initialize AES-256-GCM"))?;
        cipher
            .decrypt(Nonce::from_slice(&nonce), bytes)
            .map_err(|_| io_error("V3 evidence authentication failed"))?
    } else {
        bytes.to_vec()
    };
    zstd::stream::decode_all(compressed.as_slice())
        .map_err(|error| io_error(format!("V3 decompression failed: {error}")))
}

fn read_v2_ids(root: &Path) -> Result<Vec<String>, ContextGovernorError> {
    let mut ids = Vec::new();
    if !root.exists() {
        return Ok(ids);
    }
    for entry in fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().and_then(|value| value.to_str()) != Some("json") {
            continue;
        }
        let Some(name) = path.file_name().and_then(|value| value.to_str()) else {
            continue;
        };
        if name.starts_with("ctxr_") && name.ends_with(".json") {
            ids.push(name.trim_end_matches(".json").to_string());
        }
    }
    ids.sort();
    Ok(ids)
}

fn v2_chain_sources(
    store: &FileContextStore,
    receipt_id: &str,
) -> Result<(crate::lineage::CompactResponseV2, BTreeMap<String, Message>), ContextGovernorError> {
    let head = store.load_v2(receipt_id)?;
    let mut current = head.clone();
    let mut sources = BTreeMap::new();
    loop {
        for source in &current.source_evidence {
            sources.insert(source.source_id.clone(), source.message.clone());
        }
        let Some(parent) = current.receipt.parent_receipt.as_ref() else {
            break;
        };
        current = store.load_v2(&parent.receipt_id)?;
    }
    Ok((head, sources))
}

/// Migrate all verified V2 receipts into a disposable V3 projection.
///
/// V2 JSON remains the authority. Any receipt that fails verification is
/// recorded in `mismatched_receipts` and blocks `complete`; it is never silently
/// omitted from a successful migration.
pub fn migrate_v2_store(
    store: &FileContextStore,
    output_root: impl AsRef<Path>,
    options: &V3MigrationOptions,
) -> Result<V3MigrationReportV1, ContextGovernorError> {
    let output_root = output_root.as_ref();
    let root = store.root_path();
    let ids = read_v2_ids(root)?;
    let mut report = V3MigrationReportV1 {
        schema: "V3MigrationReportV1".to_string(),
        input_receipts: ids.len(),
        encrypted: options.encryption_key.is_some(),
        ..Default::default()
    };
    for receipt_id in ids {
        let result = (|| -> Result<(), ContextGovernorError> {
            if matches!(
                store.load_versioned(&receipt_id)?,
                VersionedCompactResponse::V1(_)
            ) {
                report.skipped_v1_receipts += 1;
                return Ok(());
            }
            let (head, sources) = v2_chain_sources(store, &receipt_id)?;
            let mut evidence = Vec::new();
            let mut local_source_ids = head.receipt.local_source_ids.clone();
            local_source_ids.sort();
            let covered_json = serde_json::to_vec(&head.receipt.covered_original_sources)?;
            let covered_digest = sha256_bytes(&covered_json);
            for (source_id, message) in sources {
                let message_bytes = serde_json::to_vec(&message)?;
                let message_digest = sha256_bytes(&message_bytes);
                let (blob, encrypted, nonce) = encode_blob(&message_bytes, options)?;
                let blob_path = blob_path(output_root, &message_digest, encrypted);
                write_atomic(&blob_path, &blob)?;
                report.exact_evidence_items += 1;
                report.plaintext_bytes += message_bytes.len() as u64;
                report.blob_bytes += blob.len() as u64;
                evidence.push(V3EvidenceRefV1 {
                    source_id,
                    message_sha256: message_digest.clone(),
                    content_sha256: hash_text_sha256(&message.content),
                    blob_relpath: blob_relpath(&message_digest, encrypted),
                    encrypted,
                    nonce_hex: nonce,
                });
            }
            evidence.sort_by(|left, right| left.source_id.cmp(&right.source_id));
            let manifest = V3ReceiptManifestV1 {
                schema: "ContextCompactionReceiptV3ManifestV1".to_string(),
                receipt_id: head.receipt.receipt_id.clone(),
                session_id: head.receipt.session_id.clone(),
                generation: head.receipt.generation,
                parent_receipt_id: head
                    .receipt
                    .parent_receipt
                    .as_ref()
                    .map(|parent| parent.receipt_id.clone()),
                local_source_ids,
                covered_source_ids_sha256: covered_digest,
                compacted_transcript_sha256: head.receipt.compacted_transcript_sha256.clone(),
                evidence,
                compression: "zstd-v1".to_string(),
                encryption: if options.encryption_key.is_some() {
                    "aes-256-gcm-v1".to_string()
                } else {
                    "none".to_string()
                },
                encryption_key_id: options.encryption_key.as_deref().map(key_id),
            };
            let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
            write_atomic(&manifest_path(output_root, &receipt_id), &manifest_bytes)?;
            report.migrated_receipts += 1;
            Ok(())
        })();
        if let Err(error) = result {
            report
                .mismatched_receipts
                .push(format!("{receipt_id}: {error}"));
        }
    }
    report.complete = report.mismatched_receipts.is_empty()
        && report.migrated_receipts + report.skipped_v1_receipts == report.input_receipts;
    Ok(report)
}

pub fn read_v3_manifest(
    output_root: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<V3ReceiptManifestV1, ContextGovernorError> {
    let bytes = fs::read(manifest_path(output_root.as_ref(), receipt_id))?;
    Ok(serde_json::from_slice(&bytes)?)
}

pub fn read_v3_evidence(
    output_root: impl AsRef<Path>,
    manifest: &V3ReceiptManifestV1,
    source_id: &str,
    encryption_key: Option<&[u8]>,
) -> Result<Message, ContextGovernorError> {
    if manifest.encryption == "aes-256-gcm-v1" {
        let key = encryption_key.ok_or_else(|| io_error("encrypted V3 evidence requires a key"))?;
        if manifest.encryption_key_id.as_deref() != Some(key_id(key).as_str()) {
            return Err(io_error("V3 encryption key ID does not match manifest"));
        }
    }
    let reference = manifest
        .evidence
        .iter()
        .find(|reference| reference.source_id == source_id)
        .ok_or_else(|| ContextGovernorError::ReceiptNotFound(source_id.to_string()))?;
    let bytes = fs::read(v3_root(output_root.as_ref()).join(&reference.blob_relpath))?;
    let plaintext = decode_blob(
        &bytes,
        reference.encrypted,
        reference.nonce_hex.as_deref(),
        encryption_key,
    )?;
    if sha256_bytes(&plaintext) != reference.message_sha256 {
        return Err(io_error("V3 message digest mismatch"));
    }
    let message: Message = serde_json::from_slice(&plaintext)?;
    if hash_text_sha256(&message.content) != reference.content_sha256 {
        return Err(io_error("V3 message content digest mismatch"));
    }
    Ok(message)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{CompactRequest, CompactionPolicy};

    fn message(role: &str, content: &str) -> Message {
        Message {
            id: None,
            role: role.to_string(),
            content: content.to_string(),
            name: None,
            metadata: Default::default(),
        }
    }

    #[test]
    fn encrypted_blob_round_trip_is_authenticated() {
        let temp = tempfile::tempdir().unwrap();
        let output = temp.path().join("projection");
        let key = vec![7u8; 32];
        let store = FileContextStore::with_hmac_key(temp.path().join("store"), &[3u8; 32]);
        let response = store
            .compact_next_v2(
                CompactRequest {
                    session_id: "v3-test".to_string(),
                    messages: vec![
                        message("tool", &"exact marker ".repeat(500)),
                        message("user", "latest task"),
                    ],
                    policy: CompactionPolicy {
                        target_tokens: 260,
                        protect_first_n: 0,
                        protect_last_n: 1,
                        ..Default::default()
                    },
                    focus: None,
                    hmac_key_path: None,
                },
                None,
            )
            .unwrap();
        let receipt_id = response.receipt.receipt_id.clone();
        store.save_v2(&response).unwrap();
        let report = migrate_v2_store(
            &store,
            &output,
            &V3MigrationOptions {
                encryption_key: Some(key.clone()),
                require_encryption: true,
                ..Default::default()
            },
        )
        .unwrap();
        assert!(report.complete);
        let manifest = read_v3_manifest(&output, &receipt_id).unwrap();
        let source_id = manifest.evidence[0].source_id.clone();
        let recovered = read_v3_evidence(&output, &manifest, &source_id, Some(&key)).unwrap();
        assert!(recovered.content.contains("exact marker"));
        assert!(read_v3_evidence(&output, &manifest, &source_id, None).is_err());
    }
}
