use crate::codecs::q8_keys::Q8KeyBlock;
use crate::{KvPoolManifestV1, PolyKvError, SharedKvPool};
use quant_codec_core::ArtifactDigest;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::fs::{self, File, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
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

#[derive(Debug, Serialize, Deserialize)]
enum StoredBlock {
    Q8(Q8KeyBlock),
    Raw(Vec<u8>),
}

impl KvPoolStore {
    pub fn open(root: impl Into<PathBuf>) -> Result<Self, PolyKvError> {
        let store = Self { root: root.into() };
        for dir in ["manifests", "blocks", "fallbacks", "journal"] {
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
                crate::pool::EncodedBlock::Q8Key(value) => StoredBlock::Q8(value.clone()),
                crate::pool::EncodedBlock::Value(value) => StoredBlock::Raw(value.clone()),
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
            blocks.push(data);
        }
        Ok((manifest, blocks))
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
