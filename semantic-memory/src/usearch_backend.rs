//! UsearchBackend — stub for the usearch 2.25 implementation of the
//! [`VectorBackend`] trait.
//!
//! ## Status: NOT IMPLEMENTED
//!
//! This file is a placeholder. The full implementation is the destination of
//! the HNSW migration (see `HNSW_RESEARCH_2026-06-02.md`) but is out of
//! scope for the bounded trait-introduction commit. Every method returns
//! `MemoryError::NotImplemented` so the trait surface can be exercised
//! against a stub before the real implementation lands.
//!
//! ## What the real implementation needs
//!
//! - Build a `usearch::Index` with `MetricKind::Cosine` + `ScalarKind::F32`
//!   (or `F16`/`F8` once those are validated for recall).
//! - Maintain a key-to-u64 id map. semantic-memory's HnswIndex currently
//!   uses String keys ("fact:123", "chunk:abc"); usearch's `Index::add`
//!   takes `Key = u64`. A `HashMap<String, u64>` and a reverse map are
//!   required (this is similar to the existing `KeyMapState` in hnsw.rs).
//! - Translate search: usearch returns `(Key, Distance)` tuples; convert
//!   to `Vec<VectorHit>`.
//! - Persistence: usearch has its own save/load format. The existing
//!   sidecar format (`HnswSidecarManifestV1` with `HNSW_SIDECAR_VERSION = 1`)
//!   is custom to hnsw.rs. The usearch backend should either:
//!     (a) wrap usearch's bytes inside a new `HnswSidecarManifestV2`, or
//!     (b) use usearch's save directly with a new manifest field
//!         `backend_kind = "usearch"`.
//!   Option (b) is simpler; option (a) preserves the existing manifest
//!   schema. This is a design decision deferred to the full migration.
//! - Handle the `catch_unwind` removal: hnsw_ops.rs:49 wraps hnsw_rs's
//!   save in catch_unwind because the upstream save has been known to
//!   panic. usearch's save is stable; this band-aid can be removed once
//!   usearch is the default.
//!
//! ## cxx bridge
//!
//! usearch 2.25 uses a C++ bridge (cxx 1.0). The `usearch` crate pulls
//! in `cxx-build` and `cxx` as transitive deps; `semantic-memory` declares
//! them as direct `[build-dependencies]` so the build script can compile
//! the bridge if needed. For this stub, no actual usearch call is made
//! so the C++ side is never instantiated — the cxx deps are present
//! for downstream use but not used here.

use std::path::Path;
use std::sync::Arc;

use crate::error::MemoryError;
use crate::vector_backend::{VectorBackend, VectorHit, VectorIndexConfig};

/// Stub backend. Returns `MemoryError::NotImplemented` for every operation.
pub struct UsearchBackend {
    config: VectorIndexConfig,
}

impl UsearchBackend {
    pub fn new(config: VectorIndexConfig) -> Result<Self, MemoryError> {
        Ok(Self { config })
    }

    pub fn load(
        _dir: &Path,
        _basename: &str,
        config: VectorIndexConfig,
    ) -> Result<Self, MemoryError> {
        Ok(Self { config })
    }

    pub fn config(&self) -> &VectorIndexConfig {
        &self.config
    }

    /// Build a `usearch::Index` with the active config. Stub returns
    /// `MemoryError::NotImplemented`.
    fn _build_index(&self) -> Result<Arc<()>, MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::new — full implementation deferred. See HNSW_RESEARCH_2026-06-02.md §10.".to_string(),
        ))
    }
}

impl VectorBackend for UsearchBackend {
    fn insert(&self, _key: String, _vector: &[f32]) -> Result<(), MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::insert — full implementation deferred".to_string(),
        ))
    }

    fn delete(&self, _key: &str) -> Result<(), MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::delete — full implementation deferred".to_string(),
        ))
    }

    fn update(&self, _key: String, _vector: &[f32]) -> Result<(), MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::update — full implementation deferred".to_string(),
        ))
    }

    fn search(&self, _query: &[f32], _top_k: usize) -> Result<Vec<VectorHit>, MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::search — full implementation deferred".to_string(),
        ))
    }

    fn len(&self) -> usize {
        0
    }

    fn is_empty(&self) -> bool {
        true
    }

    fn save(&self, _dir: &Path, _basename: &str) -> Result<(), MemoryError> {
        Err(MemoryError::NotImplemented(
            "UsearchBackend::save — full implementation deferred".to_string(),
        ))
    }

    fn backend_name(&self) -> &'static str {
        "usearch 2.25 (STUB — full implementation deferred)"
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn stub_backend_returns_not_implemented_for_every_op() {
        let cfg = VectorIndexConfig::default();
        let b = UsearchBackend::new(cfg).unwrap();
        assert!(b.insert("k".to_string(), &[1.0, 0.0]).is_err());
        assert!(b.delete("k").is_err());
        assert!(b.update("k".to_string(), &[1.0, 0.0]).is_err());
        assert!(b.search(&[1.0, 0.0], 1).is_err());
        assert_eq!(b.len(), 0);
        assert!(b.is_empty());
    }

    #[test]
    fn stub_backend_name_reflects_status() {
        let cfg = VectorIndexConfig::default();
        let b = UsearchBackend::new(cfg).unwrap();
        assert!(b.backend_name().contains("STUB"));
    }
}
