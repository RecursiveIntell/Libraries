use crate::codecs::q8_keys::{Q8KeyBlock, Q8KeyCodec};
use crate::codecs::value::ValueCodec;
use crate::pool::{EncodedBlock, EncodedPoolBlock, SharedKvPoolInner};
use crate::{KvPoolManifestV1, PolyKvError, PoolBuildReceiptV1, SharedKvPool};
use quant_codec_core::{ArtifactDigest, CodecProfile, KvRole, LayerId};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::Arc;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone)]
pub struct KvPoolStore {
    root: PathBuf,
}

#[derive(Debug, Clone)]
pub struct PersistedPool {
    pub manifest: KvPoolManifestV1,
    pub blocks: Vec<PathBuf>,
    pub exact_fallback_path: Option<PathBuf>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
enum StoredBlock {
    Q8 {
        role: KvRole,
        layer: LayerId,
        block: Q8KeyBlock,
    },
    Raw {
        role: KvRole,
        layer: LayerId,
        bytes: Vec<u8>,
    },
}

const POOL_BUNDLE_SCHEMA_V1: u16 = 1;
const MAX_POOL_BUNDLE_BYTES: usize = 1 << 30;

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolBundleCoreV1 {
    schema_version: u16,
    manifest: KvPoolManifestV1,
    build_receipt: PoolBuildReceiptV1,
    fallback: crate::ExactFallback,
    blocks: Vec<StoredBlock>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct PoolBundleEnvelopeV1 {
    core_bytes: Vec<u8>,
    bundle_digest: ArtifactDigest,
}

/// Encode one complete owner-controlled pool artifact bundle for durable projection storage.
pub fn encode_pool_bundle(pool: &SharedKvPool) -> Result<Vec<u8>, PolyKvError> {
    let blocks = pool
        .inner
        .encoded_blocks
        .iter()
        .map(|block| match &block.encoded {
            EncodedBlock::Q8Key(value) => StoredBlock::Q8 {
                role: block.role,
                layer: block.layer,
                block: value.clone(),
            },
            EncodedBlock::Value(value) => StoredBlock::Raw {
                role: block.role,
                layer: block.layer,
                bytes: value.clone(),
            },
        })
        .collect();
    let core = PoolBundleCoreV1 {
        schema_version: POOL_BUNDLE_SCHEMA_V1,
        manifest: pool.manifest().clone(),
        build_receipt: pool.build_receipt().clone(),
        fallback: pool.inner.fallback.clone(),
        blocks,
    };
    let core_bytes = bincode::serde::encode_to_vec(&core, bincode::config::standard())
        .map_err(|error| PolyKvError::Serialization(error.to_string()))?;
    let envelope = PoolBundleEnvelopeV1 {
        core_bytes: core_bytes.clone(),
        bundle_digest: ArtifactDigest::from_canonical_bytes(&core_bytes),
    };
    serde_json::to_vec(&envelope).map_err(|error| PolyKvError::Serialization(error.to_string()))
}

/// Decode and admit an owner-controlled pool artifact bundle with the supplied value codec.
pub fn decode_pool_bundle_with_value_codec<V>(
    bytes: &[u8],
    value_codec: V,
) -> Result<SharedKvPool, PolyKvError>
where
    V: ValueCodec + 'static,
{
    let core = decode_pool_bundle_core(bytes)?;
    pool_from_components(core, Arc::new(value_codec))
}

/// Decode and admit a FibQuant pool bundle using the profile embedded in its value wire.
#[cfg(feature = "fibquant-adapter")]
pub fn decode_fibquant_pool_bundle(
    bytes: &[u8],
    max_mse: f64,
) -> Result<SharedKvPool, PolyKvError> {
    let core = decode_pool_bundle_core(bytes)?;
    let value_wire = core
        .blocks
        .iter()
        .find_map(|block| match block {
            StoredBlock::Raw {
                role: KvRole::Value,
                bytes,
                ..
            } => Some(bytes.as_slice()),
            _ => None,
        })
        .ok_or_else(|| PolyKvError::Manifest("FibQuant value block is missing".into()))?;
    let codec =
        crate::adapters::fibquant::FibQuantValueCodec::from_encoded_wire(value_wire, max_mse)?;
    pool_from_components(core, Arc::new(codec))
}

fn decode_pool_bundle_core(bytes: &[u8]) -> Result<PoolBundleCoreV1, PolyKvError> {
    if bytes.len() > MAX_POOL_BUNDLE_BYTES {
        return Err(PolyKvError::Serialization(format!(
            "pool bundle exceeds {MAX_POOL_BUNDLE_BYTES} byte limit"
        )));
    }
    let envelope: PoolBundleEnvelopeV1 = serde_json::from_slice(bytes)
        .map_err(|error| PolyKvError::Serialization(error.to_string()))?;
    if ArtifactDigest::from_canonical_bytes(&envelope.core_bytes) != envelope.bundle_digest {
        return Err(PolyKvError::Manifest("pool bundle digest mismatch".into()));
    }
    let (core, consumed): (PoolBundleCoreV1, usize) =
        bincode::serde::decode_from_slice(&envelope.core_bytes, bincode::config::standard())
            .map_err(|error| PolyKvError::Serialization(error.to_string()))?;
    if consumed != envelope.core_bytes.len() {
        return Err(PolyKvError::Serialization(
            "pool bundle core contains trailing bytes".into(),
        ));
    }
    if core.schema_version != POOL_BUNDLE_SCHEMA_V1 {
        return Err(PolyKvError::Manifest(format!(
            "unsupported pool bundle schema {}",
            core.schema_version
        )));
    }
    Ok(core)
}

impl KvPoolStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, PolyKvError> {
        let store = Self { root: root.into() };
        for dir in ["manifests", "blocks", "fallbacks", "receipts", "journal"] {
            fs::create_dir_all(store.root.join(dir)).map_err(io_err)?;
        }
        Ok(store)
    }

    pub fn persist(&self, pool: &SharedKvPool) -> Result<PersistedPool, PolyKvError> {
        let manifest = pool.manifest().clone();
        let mut paths = Vec::new();
        let mut block_names: Vec<String> = Vec::with_capacity(pool.inner.encoded_blocks.len());
        for block in &pool.inner.encoded_blocks {
            let stored = match &block.encoded {
                crate::pool::EncodedBlock::Q8Key(value) => StoredBlock::Q8 {
                    role: block.role,
                    layer: block.layer,
                    block: value.clone(),
                },
                crate::pool::EncodedBlock::Value(value) => StoredBlock::Raw {
                    role: block.role,
                    layer: block.layer,
                    bytes: value.clone(),
                },
            };
            let bytes = serde_json::to_vec(&stored)
                .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
            let digest = blake3::hash(&bytes).to_hex().to_string();
            let path = self.root.join("blocks").join(&digest);
            atomic_write(&path, &bytes)?;
            paths.push(path);
            block_names.push(digest);
        }
        // Write block index alongside manifest for O(1) load.
        let block_index_path = self
            .root
            .join("manifests")
            .join(format!("{}.blocks.json", manifest.manifest_digest));
        let block_index_bytes = serde_json::to_vec(&block_names)
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        atomic_write(&block_index_path, &block_index_bytes)?;

        let fallback_path = self
            .root
            .join("fallbacks")
            .join(manifest.manifest_digest.to_string());
        let fallback = serde_json::to_vec(&pool.inner.fallback)
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        atomic_write(&fallback_path, &fallback)?;
        let receipt_path = self
            .root
            .join("receipts")
            .join(format!("{}.json", manifest.manifest_digest));
        let receipt_bytes = serde_json::to_vec_pretty(pool.build_receipt())
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        atomic_write(&receipt_path, &receipt_bytes)?;
        let manifest_path = self.manifest_path(&manifest.manifest_digest);
        let manifest_bytes = serde_json::to_vec_pretty(&manifest)
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        atomic_write(&manifest_path, &manifest_bytes)?;
        self.journal("create", &manifest.manifest_digest)?;
        Ok(PersistedPool {
            manifest,
            blocks: paths,
            exact_fallback_path: Some(fallback_path),
        })
    }

    pub fn load(
        &self,
        digest: &ArtifactDigest,
    ) -> Result<(KvPoolManifestV1, Vec<Vec<u8>>), PolyKvError> {
        let path = self.manifest_path(digest);
        let bytes = fs::read(&path).map_err(io_err)?;
        let manifest: KvPoolManifestV1 = serde_json::from_slice(&bytes)
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        if manifest.manifest_digest != *digest
            || manifest.canonical_digest_without_self() != *digest
        {
            return Err(PolyKvError::Manifest("manifest digest mismatch".into()));
        }
        // Load block index for direct O(1) lookup.
        let block_index_path = self
            .root
            .join("manifests")
            .join(format!("{}.blocks.json", digest));
        let block_names: Vec<String> =
            serde_json::from_slice(&fs::read(&block_index_path).map_err(io_err)?)
                .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        if block_names.len() != manifest.blocks.len() {
            return Err(PolyKvError::Manifest(format!(
                "block index count {} != manifest entry count {}",
                block_names.len(),
                manifest.blocks.len()
            )));
        }
        let mut blocks = Vec::with_capacity(block_names.len());
        for name in &block_names {
            let block_path = self.root.join("blocks").join(name);
            let data = fs::read(&block_path).map_err(io_err)?;
            let actual_digest = blake3::hash(&data).to_hex().to_string();
            if &actual_digest != name {
                return Err(PolyKvError::Manifest(format!(
                    "block digest mismatch: expected {name}, got {actual_digest}"
                )));
            }
            let stored: StoredBlock = serde_json::from_slice(&data)
                .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
            let (role, layer, encoded_bytes) = match stored {
                StoredBlock::Q8 { role, layer, block } => (role, layer, block.encoded_bytes()),
                StoredBlock::Raw { role, layer, bytes } => (role, layer, bytes.len() as u64),
            };
            let entry = &manifest.blocks[blocks.len()];
            if entry.role != role || entry.layer != layer.0 || entry.encoded_bytes != encoded_bytes
            {
                return Err(PolyKvError::Manifest(
                    "block key-map metadata mismatch".into(),
                ));
            }
            blocks.push(data);
        }
        Ok((manifest, blocks))
    }

    /// Load and authenticate the exact fallback for a persisted pool.
    pub fn load_fallback(
        &self,
        digest: &ArtifactDigest,
    ) -> Result<crate::ExactFallback, PolyKvError> {
        let path = self.root.join("fallbacks").join(digest.to_string());
        let bytes = fs::read(path).map_err(io_err)?;
        let fallback: crate::ExactFallback = serde_json::from_slice(&bytes)
            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        if fallback.blocks.is_empty() {
            return Err(PolyKvError::MissingFallback);
        }
        Ok(fallback)
    }

    /// Load a persisted pool using an explicitly supplied admitted value codec.
    pub fn load_pool_with_value_codec<V>(
        &self,
        digest: &ArtifactDigest,
        value_codec: V,
    ) -> Result<SharedKvPool, PolyKvError>
    where
        V: ValueCodec + 'static,
    {
        let (manifest, block_files) = self.load(digest)?;
        self.load_pool_from_parts(digest, manifest, block_files, Arc::new(value_codec))
    }

    /// Load a persisted FibQuant pool by deriving the exact codec profile from the
    /// authenticated stored value wire rather than reconstructing fixed parameters.
    #[cfg(feature = "fibquant-adapter")]
    pub fn load_fibquant_pool(
        &self,
        digest: &ArtifactDigest,
        max_mse: f64,
    ) -> Result<SharedKvPool, PolyKvError> {
        let (manifest, block_files) = self.load(digest)?;
        let value_wire = block_files
            .iter()
            .find_map(
                |bytes| match serde_json::from_slice::<StoredBlock>(bytes).ok()? {
                    StoredBlock::Raw {
                        role: KvRole::Value,
                        bytes,
                        ..
                    } => Some(bytes),
                    _ => None,
                },
            )
            .ok_or_else(|| {
                PolyKvError::Manifest("persisted FibQuant value block is missing".into())
            })?;
        let codec =
            crate::adapters::fibquant::FibQuantValueCodec::from_encoded_wire(&value_wire, max_mse)?;
        self.load_pool_from_parts(digest, manifest, block_files, Arc::new(codec))
    }

    fn load_pool_from_parts(
        &self,
        digest: &ArtifactDigest,
        manifest: KvPoolManifestV1,
        block_files: Vec<Vec<u8>>,
        value_codec: Arc<dyn ValueCodec>,
    ) -> Result<SharedKvPool, PolyKvError> {
        let fallback = self.load_fallback(digest)?;
        let receipt_path = self.root.join("receipts").join(format!("{digest}.json"));
        let build_receipt: PoolBuildReceiptV1 =
            serde_json::from_slice(&fs::read(receipt_path).map_err(io_err)?)
                .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
        let blocks = block_files
            .iter()
            .map(|bytes| {
                serde_json::from_slice(bytes)
                    .map_err(|error| PolyKvError::Serialization(error.to_string()))
            })
            .collect::<Result<Vec<StoredBlock>, PolyKvError>>()?;
        pool_from_components(
            PoolBundleCoreV1 {
                schema_version: POOL_BUNDLE_SCHEMA_V1,
                manifest,
                build_receipt,
                fallback,
                blocks,
            },
            value_codec,
        )
    }

    pub fn list_pools(&self) -> Result<Vec<KvPoolManifestV1>, PolyKvError> {
        let mut result = Vec::new();
        for entry in fs::read_dir(self.root.join("manifests")).map_err(io_err)? {
            let path = entry.map_err(io_err)?.path();
            let fname = path.file_name().and_then(|s| s.to_str()).unwrap_or("");
            // Skip block-index files and non-JSON files.
            if fname.ends_with(".blocks.json")
                || path.extension().and_then(|s| s.to_str()) != Some("json")
            {
                continue;
            }
            let bytes = fs::read(path).map_err(io_err)?;
            let manifest: KvPoolManifestV1 = serde_json::from_slice(&bytes)
                .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
            if manifest.canonical_digest_without_self() != manifest.manifest_digest {
                return Err(PolyKvError::Manifest("manifest digest mismatch".into()));
            }
            result.push(manifest);
        }
        Ok(result)
    }

    pub fn delete_pool(&self, digest: &ArtifactDigest) -> Result<(), PolyKvError> {
        let path = self.manifest_path(digest);
        if path.exists() {
            fs::remove_file(&path).map_err(io_err)?;
        }
        let block_index = self
            .root
            .join("manifests")
            .join(format!("{}.blocks.json", digest));
        if block_index.exists() {
            fs::remove_file(&block_index).map_err(io_err)?;
        }
        let fallback = self.root.join("fallbacks").join(digest.to_string());
        if fallback.exists() {
            fs::remove_file(&fallback).map_err(io_err)?;
        }
        let receipt = self.root.join("receipts").join(format!("{digest}.json"));
        if receipt.exists() {
            fs::remove_file(&receipt).map_err(io_err)?;
        }
        self.journal("delete", digest)
    }

    pub fn gc_unreferenced(&self, keep: &HashSet<ArtifactDigest>) -> Result<u64, PolyKvError> {
        let mut referenced: HashSet<String> = HashSet::new();
        for manifest in self.list_pools()? {
            if keep.contains(&manifest.manifest_digest) {
                // Load the block index to find which files this pool references.
                let block_index_path = self
                    .root
                    .join("manifests")
                    .join(format!("{}.blocks.json", manifest.manifest_digest));
                if block_index_path.exists() {
                    let names: Vec<String> =
                        serde_json::from_slice(&fs::read(&block_index_path).map_err(io_err)?)
                            .map_err(|e| PolyKvError::Serialization(e.to_string()))?;
                    referenced.extend(names);
                }
            }
        }
        let mut removed = 0u64;
        let blocks_dir = self.root.join("blocks");
        if blocks_dir.exists() {
            for entry in fs::read_dir(&blocks_dir).map_err(io_err)? {
                let path = entry.map_err(io_err)?.path();
                let fname = path
                    .file_name()
                    .and_then(|s| s.to_str())
                    .unwrap_or("")
                    .to_string();
                if !referenced.contains(&fname) {
                    fs::remove_file(&path).map_err(io_err)?;
                    removed += 1;
                }
            }
        }
        Ok(removed)
    }

    fn manifest_path(&self, digest: &ArtifactDigest) -> PathBuf {
        self.root.join("manifests").join(format!("{digest}.json"))
    }
    fn journal(&self, action: &str, digest: &ArtifactDigest) -> Result<(), PolyKvError> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|e| PolyKvError::Manifest(e.to_string()))?
            .as_secs();
        let line = format!(
            "{{\"action\":\"{action}\",\"manifest_digest\":\"{digest}\",\"timestamp\":{now}}}\n"
        );
        let path = self.root.join("journal").join("actions.log");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(path)
            .map_err(io_err)?;
        file.write_all(line.as_bytes()).map_err(io_err)?;
        file.sync_all().map_err(io_err)
    }
}

fn pool_from_components(
    core: PoolBundleCoreV1,
    value_codec: Arc<dyn ValueCodec>,
) -> Result<SharedKvPool, PolyKvError> {
    let PoolBundleCoreV1 {
        schema_version: _,
        manifest,
        build_receipt,
        fallback,
        blocks,
    } = core;
    if manifest.schema_version != 1
        || manifest.canonical_digest_without_self() != manifest.manifest_digest
    {
        return Err(PolyKvError::Manifest(
            "persisted manifest schema/digest mismatch".into(),
        ));
    }
    crate::pool::validate_block_set(&manifest.shape, &fallback.blocks)?;
    if fallback
        .blocks
        .iter()
        .flat_map(|block| block.data.iter())
        .any(|value| !value.is_finite())
    {
        return Err(PolyKvError::Manifest(
            "persisted exact fallback contains non-finite values".into(),
        ));
    }
    let encoded_bytes = manifest.blocks.iter().try_fold(0u64, |total, entry| {
        total
            .checked_add(entry.encoded_bytes)
            .ok_or_else(|| PolyKvError::Manifest("encoded byte accounting overflow".into()))
    })?;
    if build_receipt.manifest_digest != manifest.manifest_digest
        || build_receipt.input_digest != fallback.input_digest()
        || build_receipt.block_count != manifest.blocks.len() as u64
        || blocks.len() != manifest.blocks.len()
        || build_receipt.encoded_bytes != manifest.encoded_bytes
        || encoded_bytes != manifest.encoded_bytes
        || build_receipt.exact_fallback_bytes != manifest.exact_fallback_bytes
        || build_receipt.quality_gate != manifest.policy.quality_gate
        || fallback.exact_bytes() != manifest.exact_fallback_bytes
        || build_receipt.memory.encoded_shared_bytes != manifest.encoded_bytes
        || build_receipt.memory.exact_fallback_bytes != manifest.exact_fallback_bytes
    {
        return Err(PolyKvError::Manifest(
            "persisted build receipt/fallback/accounting does not match manifest".into(),
        ));
    }

    let key_codec = Q8KeyCodec::symmetric_per_block();
    if manifest.policy.key_codec_id != key_codec.codec_id()
        || manifest.policy.value_codec_id != value_codec.codec_id()
        || manifest.policy.profile_digest
            != crate::pool::combined_profile_digest(&key_codec, value_codec.as_ref())
    {
        return Err(PolyKvError::Manifest(
            "persisted codec/profile identity does not match admitted codecs".into(),
        ));
    }

    let mut encoded_blocks = Vec::with_capacity(blocks.len());
    for (index, stored) in blocks.into_iter().enumerate() {
        let entry = manifest.blocks.get(index).ok_or_else(|| {
            PolyKvError::Manifest("persisted block index exceeds manifest".into())
        })?;
        let exact = fallback
            .blocks
            .iter()
            .find(|block| block.role == entry.role && block.layer.0 == entry.layer)
            .ok_or_else(|| {
                PolyKvError::Manifest("persisted block has no matching exact fallback block".into())
            })?;
        let (encoded, actual_digest) = match stored {
            StoredBlock::Q8 { role, layer, block }
                if role == entry.role
                    && layer.0 == entry.layer
                    && entry.codec_id == key_codec.codec_id() =>
            {
                let digest = crate::pool::digest_encoded_q8(exact, &block);
                (EncodedBlock::Q8Key(block), digest)
            }
            StoredBlock::Raw { role, layer, bytes }
                if role == entry.role
                    && layer.0 == entry.layer
                    && entry.codec_id == value_codec.codec_id() =>
            {
                let mut decoded = vec![0.0f32; exact.data.len()];
                value_codec.decode_values(&bytes, &mut decoded)?;
                if decoded.iter().any(|value| !value.is_finite()) {
                    return Err(PolyKvError::Codec(
                        "persisted value block decoded non-finite values".into(),
                    ));
                }
                let digest = crate::pool::digest_encoded_raw(exact, &bytes);
                (EncodedBlock::Value(bytes), digest)
            }
            _ => {
                return Err(PolyKvError::Manifest(
                    "persisted block role/layer/codec/encoding mismatch".into(),
                ));
            }
        };
        if actual_digest != entry.artifact_digest {
            return Err(PolyKvError::Manifest(
                "persisted encoded block artifact digest mismatch".into(),
            ));
        }
        encoded_blocks.push(EncodedPoolBlock {
            role: entry.role,
            layer: LayerId(entry.layer),
            encoded,
            exact_len: exact.data.len(),
            encoded_bytes: entry.encoded_bytes,
        });
    }

    Ok(SharedKvPool {
        inner: Arc::new(SharedKvPoolInner {
            manifest,
            build_receipt,
            fallback,
            encoded_blocks,
            value_codec,
            active_readers: AtomicUsize::new(0),
            active_reader_scratch_bytes: AtomicU64::new(0),
            next_reader_id: AtomicU64::new(1),
        }),
    })
}

fn atomic_write(path: &Path, bytes: &[u8]) -> Result<(), PolyKvError> {
    let tmp = path.with_extension(format!("tmp-{}", std::process::id()));
    let mut file = File::create(&tmp).map_err(io_err)?;
    file.write_all(bytes).map_err(io_err)?;
    file.sync_all().map_err(io_err)?;
    fs::rename(&tmp, path).map_err(io_err)
}

fn io_err(error: std::io::Error) -> PolyKvError {
    PolyKvError::Manifest(error.to_string())
}
