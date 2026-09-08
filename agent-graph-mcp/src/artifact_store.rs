//! Content-addressed immutable artifact storage owned by the daemon data root.

use stack_ids::{ArtifactId, ContentDigest};
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::{Path, PathBuf};
use thiserror::Error;

use agent_collaboration_contract::ArtifactRefV1;

#[derive(Debug, Error)]
pub enum ArtifactStoreError {
    #[error("artifact exceeds configured size limit")]
    TooLarge,
    #[error("artifact bytes do not match declared digest")]
    DigestMismatch,
    #[error("artifact is empty")]
    Empty,
    #[error("artifact media type is empty")]
    EmptyMediaType,
    #[error("artifact filesystem error: {0}")]
    Io(String),
    #[error("artifact blob is corrupt and was quarantined")]
    Quarantined,
}

#[derive(Clone)]
pub struct ArtifactStore {
    root: PathBuf,
    max_bytes: u64,
}

impl ArtifactStore {
    pub fn new(data_dir: &Path, max_bytes: u64) -> Result<Self, ArtifactStoreError> {
        let root = data_dir.join("artifacts");
        fs::create_dir_all(root.join("blake3")).map_err(io_error)?;
        fs::create_dir_all(root.join("quarantine")).map_err(io_error)?;
        Ok(Self { root, max_bytes })
    }

    pub fn put(
        &self,
        artifact_id: ArtifactId,
        bytes: &[u8],
        media_type: &str,
        expected_digest: &ContentDigest,
    ) -> Result<ArtifactRefV1, ArtifactStoreError> {
        if bytes.is_empty() {
            return Err(ArtifactStoreError::Empty);
        }
        if bytes.len() as u64 > self.max_bytes {
            return Err(ArtifactStoreError::TooLarge);
        }
        if media_type.trim().is_empty() {
            return Err(ArtifactStoreError::EmptyMediaType);
        }
        let actual = ContentDigest::compute(bytes);
        if &actual != expected_digest {
            return Err(ArtifactStoreError::DigestMismatch);
        }
        let target = self.blob_path(&actual);
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent).map_err(io_error)?;
        }
        if target.exists() {
            self.verify_path(&target, &actual)?;
        } else {
            let temp = target.with_file_name(format!(
                ".tmp-{}-{}",
                std::process::id(),
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .map_err(|error| io_error(std::io::Error::other(error)))?
                    .as_nanos()
            ));
            let mut file = OpenOptions::new()
                .write(true)
                .create_new(true)
                .open(&temp)
                .map_err(io_error)?;
            file.write_all(bytes).map_err(io_error)?;
            file.sync_all().map_err(io_error)?;
            drop(file);
            if let Err(error) = fs::rename(&temp, &target) {
                let _ = fs::remove_file(&temp);
                if !target.exists() {
                    return Err(io_error(error));
                }
            }
            self.verify_path(&target, &actual)?;
        }
        Ok(ArtifactRefV1 {
            artifact_id,
            digest: actual,
            size_bytes: bytes.len() as u64,
            media_type: media_type.to_owned(),
        })
    }

    pub fn get(&self, digest: &ContentDigest) -> Result<Vec<u8>, ArtifactStoreError> {
        let path = self.blob_path(digest);
        self.verify_path(&path, digest)?;
        fs::read(path).map_err(io_error)
    }

    pub fn blob_path(&self, digest: &ContentDigest) -> PathBuf {
        let hex = digest.hex();
        self.root.join("blake3").join(&hex[..2]).join(&hex[2..])
    }

    fn verify_path(&self, path: &Path, expected: &ContentDigest) -> Result<(), ArtifactStoreError> {
        let metadata = fs::symlink_metadata(path).map_err(io_error)?;
        if !metadata.file_type().is_file() {
            return Err(ArtifactStoreError::DigestMismatch);
        }
        if metadata.len() > self.max_bytes {
            return self.quarantine(path);
        }
        let bytes = fs::read(path).map_err(io_error)?;
        if ContentDigest::compute(&bytes) != *expected {
            return self.quarantine(path);
        }
        Ok(())
    }

    fn quarantine(&self, path: &Path) -> Result<(), ArtifactStoreError> {
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or("unknown");
        let target = self.root.join("quarantine").join(format!(
            "{}-{}",
            name,
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|error| io_error(std::io::Error::other(error)))?
                .as_nanos()
        ));
        fs::rename(path, target).map_err(io_error)?;
        Err(ArtifactStoreError::Quarantined)
    }
}

fn io_error(error: std::io::Error) -> ArtifactStoreError {
    ArtifactStoreError::Io(error.to_string())
}
