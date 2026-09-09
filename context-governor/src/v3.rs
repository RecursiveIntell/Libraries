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
use std::fs::{self, File, OpenOptions};
use std::io::{Error, ErrorKind};
use std::io::{Read, Write};
use std::path::{Component, Path, PathBuf};

const V3_ROOT: &str = ".v3";
const MAX_MESSAGE_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RECEIPT_BYTES: u64 = 512 * 1024 * 1024;
const MAX_MANIFEST_BYTES: u64 = 64 * 1024 * 1024;
const MAX_RECEIPTS: usize = 100_000;
const MANIFEST_SCHEMA: &str = "ContextCompactionReceiptV3ManifestV1";

/// A migration is fresh-only. An interrupted or existing projection is never
/// silently resumed. Verify it read-only or explicitly quarantine it and use a
/// fresh output root. This is separate from CAS reuse within a single migration.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct V3ProjectionDescriptorV1 {
    pub schema: String,
    pub source_receipts_sha256: String,
    pub input_receipts: usize,
    pub compression_level: i32,
    pub encryption_key_id: Option<String>,
    pub exactness_scope: String,
}

#[derive(Debug, Clone, thiserror::Error, PartialEq, Eq)]
pub enum V3ProjectionError {
    #[error("V3 source directory is missing or invalid")]
    InvalidSource,
    #[error(
        "V3 requires a fresh projection root; verify or quarantine existing output explicitly"
    )]
    ExistingProjection,
    #[error("V3 source and output scopes must be disjoint")]
    OverlappingScope,
    #[error("V3 path contains a symlink, traversal, or unexpected artifact")]
    UnsafePath,
    #[error("V3 artifact exceeds the declared resource limit")]
    ResourceLimit,
    #[error("V3 source snapshot changed during the operation")]
    SourceChanged,
    #[error("V3 projection does not match its authenticated V2 source")]
    SourceMismatch,
    #[error("V3 schema, codec, or encryption contract is unsupported")]
    UnsupportedContract,
    #[error("V3 projection options are invalid")]
    InvalidOptions,
    #[error("V3 artifact publication collided with an existing path")]
    PublicationCollision,
    #[error("V3 legacy V1 ancestor exact-text sources require an explicit migration contract")]
    LegacyAncestorUnsupported,
}

fn v3_error(error: V3ProjectionError) -> ContextGovernorError {
    ContextGovernorError::V3Projection(error)
}

const MANIFEST_ROOT: &str = "manifests";
const EVIDENCE_ROOT: &str = "evidence/sha256";

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
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
#[serde(deny_unknown_fields)]
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

#[derive(Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(deny_unknown_fields)]
pub struct V3MigrationOptions {
    #[serde(default, skip_serializing, skip_deserializing)]
    pub encryption_key: Option<Vec<u8>>,
    #[serde(default)]
    pub require_encryption: bool,
    #[serde(default = "default_compression_level")]
    pub compression_level: i32,
}

impl std::fmt::Debug for V3MigrationOptions {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("V3MigrationOptions")
            .field(
                "encryption_key",
                &self.encryption_key.as_ref().map(|_| "[REDACTED]"),
            )
            .field("require_encryption", &self.require_encryption)
            .field("compression_level", &self.compression_level)
            .finish()
    }
}

fn validate_options(options: &V3MigrationOptions) -> Result<(), ContextGovernorError> {
    if options
        .encryption_key
        .as_ref()
        .is_some_and(|key| key.len() != 32)
        || (options.require_encryption && options.encryption_key.is_none())
        || !(-7..=22).contains(&options.compression_level)
    {
        return Err(v3_error(V3ProjectionError::InvalidOptions));
    }
    Ok(())
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
pub struct V3MigrationReportV2 {
    pub schema: String,
    pub input_receipts: usize,
    pub migrated_receipts: usize,
    pub skipped_v1_receipts: usize,
    pub mismatched_receipts: Vec<String>,
    pub exact_evidence_items: usize,
    pub plaintext_bytes: u64,
    pub blob_bytes: u64,
    pub encrypted: bool,
    /// True only for a nonempty, fully verified corpus with no V1 skips.
    pub complete: bool,
    pub source_scan_complete: bool,
    pub v2_projection_complete: bool,
    pub full_corpus_migrated: bool,
    pub unique_evidence_blobs: usize,
    pub source_receipts_sha256: String,
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

fn check_path(path: &Path) -> Result<(), ContextGovernorError> {
    let mut current = PathBuf::new();
    for part in path.components() {
        if matches!(part, Component::ParentDir) {
            return Err(v3_error(V3ProjectionError::UnsafePath));
        }
        current.push(part.as_os_str());
        match fs::symlink_metadata(&current) {
            Ok(metadata) if metadata.file_type().is_symlink() => {
                return Err(v3_error(V3ProjectionError::UnsafePath));
            }
            Ok(_) => {}
            Err(error) if error.kind() == ErrorKind::NotFound => {}
            Err(error) => return Err(error.into()),
        }
    }
    Ok(())
}

fn read_bounded(path: &Path, maximum: u64) -> Result<Vec<u8>, ContextGovernorError> {
    check_path(path)?;
    let metadata = fs::symlink_metadata(path)?;
    if !metadata.is_file() {
        return Err(v3_error(V3ProjectionError::UnsafePath));
    }
    if metadata.len() > maximum {
        return Err(v3_error(V3ProjectionError::ResourceLimit));
    }
    let file = File::open(path)?;
    let mut bytes = Vec::new();
    file.take(maximum + 1).read_to_end(&mut bytes)?;
    if bytes.len() as u64 > maximum {
        return Err(v3_error(V3ProjectionError::ResourceLimit));
    }
    Ok(bytes)
}

// Exclusive fresh-root ownership prevents legitimate concurrent writers. The
// no-clobber hard link additionally prevents check-then-rename replacement.
fn write_atomic(path: &Path, bytes: &[u8]) -> Result<(), ContextGovernorError> {
    check_path(path)?;
    let parent = path
        .parent()
        .ok_or_else(|| v3_error(V3ProjectionError::UnsafePath))?;
    fs::create_dir_all(parent)?;
    check_path(parent)?;
    let tmp = parent.join(format!(".{}.tmp", uuid::Uuid::new_v4().simple()));
    let mut options = OpenOptions::new();
    options.write(true).create_new(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.mode(0o600);
    }
    let result = (|| -> Result<(), ContextGovernorError> {
        let mut file = options.open(&tmp)?;
        file.write_all(bytes)?;
        file.sync_all()?;
        drop(file);
        fs::hard_link(&tmp, path).map_err(|error| {
            if error.kind() == ErrorKind::AlreadyExists {
                v3_error(V3ProjectionError::PublicationCollision)
            } else {
                ContextGovernorError::Io(error)
            }
        })?;
        Ok(())
    })();
    let cleanup = fs::remove_file(&tmp);
    result?;
    cleanup?;
    #[cfg(unix)]
    File::open(parent)?.sync_all()?;
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
    let decoder = zstd::stream::read::Decoder::new(compressed.as_slice())
        .map_err(|_| io_error("V3 decompression failed"))?;
    let mut plaintext = Vec::new();
    decoder
        .take(MAX_MESSAGE_BYTES + 1)
        .read_to_end(&mut plaintext)?;
    if plaintext.len() as u64 > MAX_MESSAGE_BYTES {
        return Err(v3_error(V3ProjectionError::ResourceLimit));
    }
    Ok(plaintext)
}

fn read_v2_ids(root: &Path) -> Result<Vec<String>, ContextGovernorError> {
    let mut ids = Vec::new();
    check_path(root)?;
    if !root.is_dir() {
        return Err(v3_error(V3ProjectionError::InvalidSource));
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
            let id = name.trim_end_matches(".json");
            validate_receipt_id(id)?;
            read_bounded(&path, MAX_RECEIPT_BYTES)?;
            ids.push(id.to_string());
            if ids.len() > MAX_RECEIPTS {
                return Err(v3_error(V3ProjectionError::ResourceLimit));
            }
        }
    }
    ids.sort();
    Ok(ids)
}

fn validate_receipt_id(id: &str) -> Result<(), ContextGovernorError> {
    if !id.starts_with("ctxr_")
        || id.len() > 256
        || !id
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        return Err(v3_error(V3ProjectionError::UnsafePath));
    }
    Ok(())
}

fn valid_digest(value: &str) -> bool {
    value.len() == 64
        && value
            .bytes()
            .all(|b| b.is_ascii_digit() || (b'a'..=b'f').contains(&b))
}

fn source_snapshot(
    store: &FileContextStore,
) -> Result<(Vec<String>, String), ContextGovernorError> {
    let ids = read_v2_ids(store.root_path())?;
    let mut entries = Vec::new();
    for id in &ids {
        let bytes = read_bounded(&store.path_for_receipt(id)?, MAX_RECEIPT_BYTES)?;
        entries.push((id, bytes.len(), sha256_bytes(&bytes)));
    }
    Ok((ids.clone(), sha256_bytes(&serde_json::to_vec(&entries)?)))
}

fn v2_chain_sources(
    store: &FileContextStore,
    receipt_id: &str,
) -> Result<(crate::lineage::CompactResponseV2, BTreeMap<String, Message>), ContextGovernorError> {
    // Use the canonical owner once, not a fresh full-chain load for every parent.
    let chain = store.collect_lineage(receipt_id, true)?;
    let Some(VersionedCompactResponse::V2(head)) = chain.first() else {
        return Err(v3_error(V3ProjectionError::UnsupportedContract));
    };
    let mut sources = BTreeMap::new();
    for response in &chain {
        let VersionedCompactResponse::V2(response) = response else {
            // Legacy exact text is not a Message. Never invent role/metadata.
            return Err(v3_error(V3ProjectionError::LegacyAncestorUnsupported));
        };
        for source in &response.source_evidence {
            if sources
                .insert(source.source_id.clone(), source.message.clone())
                .is_some()
            {
                return Err(v3_error(V3ProjectionError::SourceMismatch));
            }
        }
    }
    Ok(((**head).clone(), sources))
}

fn validate_manifest(manifest: &V3ReceiptManifestV1) -> Result<(), ContextGovernorError> {
    validate_receipt_id(&manifest.receipt_id)?;
    if let Some(parent) = &manifest.parent_receipt_id {
        validate_receipt_id(parent)?;
    }
    if manifest.schema != MANIFEST_SCHEMA
        || manifest.compression != "zstd-v1"
        || !matches!(manifest.encryption.as_str(), "none" | "aes-256-gcm-v1")
        || !valid_digest(&manifest.covered_source_ids_sha256)
        || !valid_digest(&manifest.compacted_transcript_sha256)
    {
        return Err(v3_error(V3ProjectionError::UnsupportedContract));
    }
    let encrypted = manifest.encryption == "aes-256-gcm-v1";
    if encrypted != manifest.encryption_key_id.is_some()
        || manifest
            .encryption_key_id
            .as_deref()
            .is_some_and(|id| !valid_digest(id))
        || manifest
            .evidence
            .windows(2)
            .any(|pair| pair[0].source_id >= pair[1].source_id)
    {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }
    for reference in &manifest.evidence {
        if !valid_digest(&reference.message_sha256)
            || !valid_digest(&reference.content_sha256)
            || reference.encrypted != encrypted
            || reference.blob_relpath != blob_relpath(&reference.message_sha256, encrypted)
            || (encrypted
                && reference
                    .nonce_hex
                    .as_deref()
                    .map_or(true, |n| n.len() != 24 || hex::decode(n).is_err()))
            || (!encrypted && reference.nonce_hex.is_some())
        {
            return Err(v3_error(V3ProjectionError::UnsafePath));
        }
    }
    Ok(())
}

fn descriptor(output_root: &Path) -> Result<V3ProjectionDescriptorV1, ContextGovernorError> {
    let value: V3ProjectionDescriptorV1 = serde_json::from_slice(&read_bounded(
        &v3_root(output_root).join("projection.json"),
        16_384,
    )?)?;
    if value.schema != "ContextGovernorV3ProjectionV1"
        || value.exactness_scope != "serde_message_json_v1"
        || !valid_digest(&value.source_receipts_sha256)
    {
        return Err(v3_error(V3ProjectionError::UnsupportedContract));
    }
    Ok(value)
}

/// Verify receipt/source membership against authoritative V2, not manifest claims.
/// This is a snapshot check, not permission to promote V3 into authority.
pub fn verify_v3_manifest_against_v2(
    store: &FileContextStore,
    output_root: impl AsRef<Path>,
    manifest: &V3ReceiptManifestV1,
) -> Result<(), ContextGovernorError> {
    validate_manifest(manifest)?;
    let binding = descriptor(output_root.as_ref())?;
    let (ids, digest) = source_snapshot(store)?;
    if digest != binding.source_receipts_sha256 || ids.len() != binding.input_receipts {
        return Err(v3_error(V3ProjectionError::SourceChanged));
    }
    if manifest.encryption_key_id != binding.encryption_key_id {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }
    let (head, sources) = v2_chain_sources(store, &manifest.receipt_id)?;
    let mut local = head.receipt.local_source_ids.clone();
    local.sort();
    if manifest.session_id != head.receipt.session_id
        || manifest.generation != head.receipt.generation
        || manifest.parent_receipt_id
            != head
                .receipt
                .parent_receipt
                .as_ref()
                .map(|p| p.receipt_id.clone())
        || manifest.local_source_ids != local
        || manifest.covered_source_ids_sha256
            != sha256_bytes(&serde_json::to_vec(&head.receipt.covered_original_sources)?)
        || manifest.compacted_transcript_sha256 != head.receipt.compacted_transcript_sha256
        || manifest.evidence.len() != sources.len()
    {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }
    for reference in &manifest.evidence {
        let message = sources
            .get(&reference.source_id)
            .ok_or_else(|| v3_error(V3ProjectionError::SourceMismatch))?;
        if reference.message_sha256 != sha256_bytes(&serde_json::to_vec(message)?)
            || reference.content_sha256 != hash_text_sha256(&message.content)
        {
            return Err(v3_error(V3ProjectionError::SourceMismatch));
        }
    }
    Ok(())
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
) -> Result<V3MigrationReportV2, ContextGovernorError> {
    let output_root = output_root.as_ref();
    validate_options(options)?;
    let (ids, source_digest) = source_snapshot(store)?;
    check_path(output_root)?;
    // Existing ancestors must be real directories; an output within the source
    // would mutate the source scope even though it does not rewrite receipts.
    let absolute_output = if output_root.is_absolute() {
        output_root.to_path_buf()
    } else {
        std::env::current_dir()?.join(output_root)
    };
    let absolute_source = fs::canonicalize(store.root_path())?;
    if absolute_output.starts_with(&absolute_source)
        || absolute_source.starts_with(&absolute_output)
    {
        return Err(v3_error(V3ProjectionError::OverlappingScope));
    }
    fs::create_dir_all(output_root)?;
    let projection_root = v3_root(output_root);
    fs::create_dir(&projection_root).map_err(|error| {
        if error.kind() == ErrorKind::AlreadyExists {
            v3_error(V3ProjectionError::ExistingProjection)
        } else {
            ContextGovernorError::Io(error)
        }
    })?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(&projection_root, fs::Permissions::from_mode(0o700))?;
    }
    let binding = V3ProjectionDescriptorV1 {
        schema: "ContextGovernorV3ProjectionV1".into(),
        source_receipts_sha256: source_digest.clone(),
        input_receipts: ids.len(),
        compression_level: options.compression_level,
        encryption_key_id: options.encryption_key.as_deref().map(key_id),
        exactness_scope: "serde_message_json_v1".into(),
    };
    write_atomic(
        &projection_root.join("projection.json"),
        &serde_json::to_vec_pretty(&binding)?,
    )?;
    let mut blobs: BTreeMap<String, (bool, Option<String>)> = BTreeMap::new();
    let mut report = V3MigrationReportV2 {
        schema: "V3MigrationReportV2".to_string(),
        input_receipts: ids.len(),
        encrypted: options.encryption_key.is_some(),
        source_receipts_sha256: source_digest.clone(),
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
                if message_bytes.len() as u64 > MAX_MESSAGE_BYTES {
                    return Err(v3_error(V3ProjectionError::ResourceLimit));
                }
                let (encrypted, nonce) = if let Some(published) = blobs.get(&message_digest) {
                    // Reuse the actual winning ciphertext AND its original nonce.
                    published.clone()
                } else {
                    let (blob, encrypted, nonce) = encode_blob(&message_bytes, options)?;
                    write_atomic(&blob_path(output_root, &message_digest, encrypted), &blob)?;
                    let decoded = decode_blob(
                        &blob,
                        encrypted,
                        nonce.as_deref(),
                        options.encryption_key.as_deref(),
                    )?;
                    if decoded != message_bytes {
                        return Err(v3_error(V3ProjectionError::SourceMismatch));
                    }
                    report.plaintext_bytes += message_bytes.len() as u64;
                    report.blob_bytes += blob.len() as u64;
                    report.unique_evidence_blobs += 1;
                    blobs.insert(message_digest.clone(), (encrypted, nonce.clone()));
                    (encrypted, nonce)
                };
                report.exact_evidence_items += 1;
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
                schema: MANIFEST_SCHEMA.to_string(),
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
            validate_manifest(&manifest)?;
            let manifest_bytes = serde_json::to_vec_pretty(&manifest)?;
            if manifest_bytes.len() as u64 > MAX_MANIFEST_BYTES {
                return Err(v3_error(V3ProjectionError::ResourceLimit));
            }
            write_atomic(&manifest_path(output_root, &receipt_id), &manifest_bytes)?;
            // The source membership was derived above from the canonical chain.
            // Independently read the published bytes, not the generated buffer.
            for reference in &manifest.evidence {
                read_blob_evidence(
                    output_root,
                    &manifest,
                    &reference.source_id,
                    options.encryption_key.as_deref(),
                )?;
            }
            report.migrated_receipts += 1;
            Ok(())
        })();
        if let Err(error) = result {
            report
                .mismatched_receipts
                .push(format!("{receipt_id}: {error}"));
        }
    }
    let (_, after_digest) = source_snapshot(store)?;
    if after_digest != source_digest {
        return Err(v3_error(V3ProjectionError::SourceChanged));
    }
    report.source_scan_complete = true;
    report.v2_projection_complete = report.mismatched_receipts.is_empty()
        && report.migrated_receipts + report.skipped_v1_receipts == report.input_receipts;
    report.full_corpus_migrated = report.v2_projection_complete
        && report.skipped_v1_receipts == 0
        && report.input_receipts > 0;
    report.complete = report.full_corpus_migrated;
    write_atomic(
        &projection_root.join("migration-report.json"),
        &serde_json::to_vec_pretty(&report)?,
    )?;
    Ok(report)
}

pub fn read_v3_manifest(
    output_root: impl AsRef<Path>,
    receipt_id: &str,
) -> Result<V3ReceiptManifestV1, ContextGovernorError> {
    validate_receipt_id(receipt_id)?;
    let bytes = read_bounded(
        &manifest_path(output_root.as_ref(), receipt_id),
        MAX_MANIFEST_BYTES,
    )?;
    let manifest: V3ReceiptManifestV1 = serde_json::from_slice(&bytes)?;
    validate_manifest(&manifest)?;
    if manifest.receipt_id != receipt_id {
        return Err(v3_error(V3ProjectionError::SourceMismatch));
    }
    Ok(manifest)
}

/// Read evidence only after verifying its receipt/source binding against V2.
/// The added store parameter is an intentional source API change for the
/// unactivated V3 prototype; old unverified reads are not a compatibility mode.
pub fn read_v3_evidence(
    store: &FileContextStore,
    output_root: impl AsRef<Path>,
    manifest: &V3ReceiptManifestV1,
    source_id: &str,
    encryption_key: Option<&[u8]>,
) -> Result<Message, ContextGovernorError> {
    verify_v3_manifest_against_v2(store, &output_root, manifest)?;
    read_blob_evidence(output_root.as_ref(), manifest, source_id, encryption_key)
}

fn read_blob_evidence(
    output_root: &Path,
    manifest: &V3ReceiptManifestV1,
    source_id: &str,
    encryption_key: Option<&[u8]>,
) -> Result<Message, ContextGovernorError> {
    validate_manifest(manifest)?;
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
    let bytes = read_bounded(
        &blob_path(output_root, &reference.message_sha256, reference.encrypted),
        MAX_MESSAGE_BYTES + 1024 * 1024,
    )?;
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
        let recovered =
            read_v3_evidence(&store, &output, &manifest, &source_id, Some(&key)).unwrap();
        assert!(recovered.content.contains("exact marker"));
        assert!(read_v3_evidence(&store, &output, &manifest, &source_id, None).is_err());
    }
}
