# semantic-memory v0.2.0 Upgrade Specification

## Document Purpose

This is the complete, authoritative specification for upgrading `semantic-memory` from v0.1.0 to v0.2.0. It covers every change: HNSW integration, scalar quantization, feature flag restructuring, bug fixes, API refinements, and the new storage architecture. This spec is designed to be consumed by Claude Code to execute the upgrade with zero ambiguity.

**Target version:** 0.2.0
**Breaking changes:** Yes (pre-release, no users, acceptable)
**HNSW dependency:** `hnswlib-rs` crate (pure Rust, Jan 2026, native Qi8)
**Primary embedding model:** nomic-embed-text (768 dimensions)
**Default search backend:** HNSW (brute-force available via feature flag)

---

## Table of Contents

1. [Architecture Overview](#1-architecture-overview)
2. [Storage Architecture](#2-storage-architecture)
3. [Feature Flags](#3-feature-flags)
4. [Cargo.toml Changes](#4-cargotoml-changes)
5. [New Module: src/hnsw.rs](#5-new-module-srchnsw-rs)
6. [New Module: src/quantize.rs](#6-new-module-srcquantize-rs)
7. [Modified Module: src/search.rs](#7-modified-module-srcsearch-rs)
8. [Modified Module: src/db.rs](#8-modified-module-srcdb-rs)
9. [Modified Module: src/lib.rs](#9-modified-module-srclibrs)
10. [Modified Module: src/embedder.rs](#10-modified-module-srcembedder-rs)
11. [Modified Module: src/knowledge.rs](#11-modified-module-srcknowledge-rs)
12. [Modified Module: src/documents.rs](#12-modified-module-srcdocuments-rs)
13. [Modified Module: src/conversation.rs](#13-modified-module-srcconversation-rs)
14. [Bug Fixes](#14-bug-fixes)
15. [Testing Strategy](#15-testing-strategy)
16. [Migration Path](#16-migration-path)
17. [Performance Targets](#17-performance-targets)
18. [Future Considerations (Do NOT Implement)](#18-future-considerations-do-not-implement)

---

## 1. Architecture Overview

### Before (v0.1.0)

```
┌─────────────────────────────────┐
│         memory.db (SQLite)       │
│  ┌───────────┐  ┌─────────────┐ │
│  │  Content   │  │  FTS5 Index │ │
│  │  + Vectors │  │  (BM25)     │ │
│  │  (f32 blob)│  │             │ │
│  └───────────┘  └─────────────┘ │
│         ↓ brute-force scan       │
│    cosine similarity on ALL rows │
└─────────────────────────────────┘
```

### After (v0.2.0)

```
┌──────────────────────┐  ┌──────────────────────┐  ┌──────────────────────┐
│   memory.db (SQLite)  │  │   memory.hnsw (graph) │  │ memory.vectors (qi8)  │
│  ┌──────┐ ┌────────┐ │  │  hnswlib-rs binary    │  │ InMemoryQi8VectorStore│
│  │Content│ │FTS5 Idx│ │  │  graph topology       │  │ quantized int8 vecs   │
│  │+ meta │ │(BM25)  │ │  │  key→NodeId mapping   │  │ scale + zero_point    │
│  └──────┘ └────────┘ │  └──────────────────────┘  └──────────────────────┘
│                       │             ↓                         ↓
│  ID lookups, FTS      │     O(log n) ANN search      vectors on demand
│  content retrieval    │     concurrent read/write
└──────────────────────┘
```

### Data Flow: Insert

```
add_fact("Earth is round", namespace, metadata)
  │
  ├─→ SQLite: INSERT content, namespace, metadata → fact_id (i64)
  ├─→ SQLite: INSERT FTS5 bridge row
  ├─→ Embedder: embed("Earth is round") → Vec<f32> [768 dims]
  ├─→ Quantizer: quantize(Vec<f32>) → Qi8 { data: Vec<i8>, scale: f32, zero_point: i8 }
  ├─→ SQLite: UPDATE embedding blob (f32, for reranking/reembedding)
  ├─→ HNSW: insert(store, fact_id.to_string(), qi8_ref) → NodeId
  └─→ Done
```

### Data Flow: Search

```
search("planet shape", config)
  │
  ├─→ FTS5 query → Vec<FtsHit> { id, bm25_score }
  │
  ├─→ Embedder: embed("planet shape") → Vec<f32> query_vec
  ├─→ Quantizer: quantize(query_vec) → Qi8
  ├─→ HNSW: search(store, qi8_query, ef_search=50, top_k * 3)
  │   └─→ Vec<Hit> { key: String(fact_id), distance }
  │   └─→ Convert distance to similarity score
  │
  ├─→ Optional rerank: load f32 vectors from SQLite for top HNSW candidates
  │   └─→ Compute exact cosine similarity on f32 vectors
  │
  ├─→ RRF fusion (unchanged algorithm)
  │   └─→ Merge FTS hits + vector hits by ID
  │   └─→ score = Σ 1/(k + rank) with recency boost
  │
  └─→ Load content from SQLite for top N → Vec<SearchResult>
```

### Key Architectural Decisions

1. **SQLite retains f32 embeddings.** The original float32 vectors stay in SQLite as blobs. This enables: (a) exact reranking after HNSW approximate search, (b) reembedding when models change via `reembed_all()`, (c) graceful fallback to brute-force if HNSW index is corrupted/missing.

2. **HNSW operates on quantized int8 vectors.** The HNSW graph and its vector store use `Qi8` (int8 with per-vector scale and zero_point). This gives 4x memory reduction in the index while HNSW's approximate nature absorbs the small recall loss from quantization.

3. **IDs bridge all three files.** The HNSW key is the stringified SQLite rowid (e.g., `"fact:42"`, `"chunk:17"`, `"msg:99"`). This allows direct lookup from HNSW results back to SQLite content without an additional mapping table.

4. **The three files are coupled.** If any sidecar file is missing or corrupted, the system must be able to rebuild it from SQLite (which is the source of truth for content + f32 vectors).

---

## 2. Storage Architecture

### Directory Convention

When the user opens a memory store with path `"/path/to/memory"`, the system creates/expects:

```
/path/to/memory/
├── memory.db          # SQLite: content, metadata, FTS5, f32 embeddings
├── memory.hnsw        # hnswlib-rs: graph topology + key mapping
└── memory.vectors     # hnswlib-rs: InMemoryQi8VectorStore binary
```

### Path Resolution

```rust
/// Given a base path, resolve all storage file paths
pub struct StoragePaths {
    pub base_dir: PathBuf,
    pub sqlite_path: PathBuf,    // base_dir/memory.db
    pub hnsw_path: PathBuf,      // base_dir/memory.hnsw
    pub vectors_path: PathBuf,   // base_dir/memory.vectors
}

impl StoragePaths {
    pub fn new(base_dir: impl AsRef<Path>) -> Self {
        let base_dir = base_dir.as_ref().to_path_buf();
        Self {
            sqlite_path: base_dir.join("memory.db"),
            hnsw_path: base_dir.join("memory.hnsw"),
            vectors_path: base_dir.join("memory.vectors"),
            base_dir,
        }
    }
}
```

### API Change for Opening

**Before (v0.1.0):**
```rust
let store = MemoryStore::open("memory.db", config).await?;
```

**After (v0.2.0):**
```rust
// New primary API — takes a directory
let store = MemoryStore::open("/path/to/memory", config).await?;

// Also accepts a direct StoragePaths for custom layouts
let paths = StoragePaths::new("/custom/path");
let store = MemoryStore::open_with_paths(paths, config).await?;
```

The `open()` function:
1. Creates the directory if it doesn't exist (`std::fs::create_dir_all`)
2. Opens/creates SQLite at `memory.db`
3. Runs migrations on SQLite
4. If HNSW feature is enabled:
   a. If `memory.hnsw` and `memory.vectors` exist → load them
   b. If they don't exist AND SQLite has embeddings → rebuild index (call `rebuild_hnsw_index()`)
   c. If they don't exist AND SQLite is empty → create empty HNSW + vector store
5. Returns `MemoryStore`

### Integrity and Recovery

```rust
impl MemoryStore {
    /// Rebuild HNSW index from SQLite f32 embeddings.
    /// Call this if sidecar files are missing, corrupted, or after reembed_all().
    pub async fn rebuild_hnsw_index(&self) -> Result<()>;

    /// Verify that HNSW index and SQLite are in sync.
    /// Returns IDs present in one but not the other.
    pub async fn verify_index_integrity(&self) -> Result<IntegrityReport>;

    /// Persist HNSW graph and vector store to disk.
    /// Called automatically on drop, but can be called explicitly.
    pub async fn flush_hnsw(&self) -> Result<()>;
}

pub struct IntegrityReport {
    pub in_sqlite_not_hnsw: Vec<String>,
    pub in_hnsw_not_sqlite: Vec<String>,
    pub is_consistent: bool,
}
```

---

## 3. Feature Flags

### Cargo.toml Features

```toml
[features]
default = ["hnsw"]

# Search backends
hnsw = ["dep:hnswlib-rs"]
brute-force = []  # Enables brute-force scan as search backend

# Embedding providers (future, not in 0.2.0)
# ollama = []
# openai = []
# fastembed = ["dep:fastembed"]
```

### Feature Flag Behavior

| Feature Combination | Vector Search Behavior |
|---|---|
| `default` (= `hnsw`) | HNSW with Qi8 quantization. Sidecar files required. |
| `hnsw, brute-force` | HNSW primary, brute-force available via `SearchConfig::force_brute_force` |
| `brute-force` only | v0.1.0 behavior. No sidecar files. SQLite-only. |
| neither | Compile error (at least one backend required) |

### Compile-Time Enforcement

```rust
#[cfg(not(any(feature = "hnsw", feature = "brute-force")))]
compile_error!("At least one search backend feature must be enabled: 'hnsw' or 'brute-force'");
```

### Conditional Compilation Pattern

Throughout the codebase, use this pattern:

```rust
#[cfg(feature = "hnsw")]
use crate::hnsw::HnswIndex;

#[cfg(feature = "hnsw")]
fn vector_search_hnsw(&self, query: &[f32], limit: usize) -> Result<Vec<VectorHit>> {
    // ...
}

#[cfg(feature = "brute-force")]
fn vector_search_brute_force(&self, query: &[f32], limit: usize) -> Result<Vec<VectorHit>> {
    // existing scan logic
}
```

---

## 4. Cargo.toml Changes

```toml
[package]
name = "semantic-memory"
version = "0.2.0"
edition = "2021"
description = "Hybrid semantic search with SQLite, FTS5, and HNSW — built for AI agents"
# ... other metadata unchanged

[features]
default = ["hnsw"]
hnsw = ["dep:hnswlib-rs"]
brute-force = []

[dependencies]
# Existing (unchanged)
rusqlite = { version = "0.31", features = ["bundled", "modern_sqlite"] }
tokio = { version = "1", features = ["rt", "sync", "macros"] }
reqwest = { version = "0.12", features = ["json"], default-features = false }
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "1"  # or "2" if already upgraded
bytemuck = { version = "1", features = ["derive"] }
uuid = { version = "1", features = ["v4"] }
chrono = { version = "0.4", features = ["serde"] }
tracing = "0.1"

# New
hnswlib-rs = { version = "0.1", optional = true }
# NOTE: Check exact version on crates.io at implementation time.
# The crate appeared on crates.io in Jan 2026. Pin to whatever the latest
# stable version is when implementing. The API surface we use:
#   Hnsw, HnswConfig, InMemoryQi8VectorStore, CosineQi8, Qi8Ref, Hit

[dev-dependencies]
tokio = { version = "1", features = ["rt-multi-thread", "macros"] }
tempfile = "3"
```

**IMPORTANT:** At implementation time, verify `hnswlib-rs` is published to crates.io and confirm the exact types exported. If the crate name on crates.io differs from the GitHub name (e.g., `hnswlib_rs` vs `hnswlib-rs`), adjust accordingly. The README examples show:
```rust
use hnswlib_rs::{Hnsw, HnswConfig, InMemoryQi8VectorStore, CosineQi8, Qi8Ref, Result};
```

If `hnswlib-rs` is NOT on crates.io at implementation time, use a git dependency:
```toml
hnswlib-rs = { git = "https://github.com/jean-pierreBoth/hnswlib-rs", optional = true }
```

If neither works (crate too new, API mismatch, etc.), fall back to `hnsw_rs` crate (the older one by same author) and implement SQ8 quantization manually. See Section 6 for the standalone quantization module that works with either backend.

---

## 5. New Module: src/hnsw.rs

This module wraps `hnswlib-rs` and provides the integration layer between SQLite IDs and the HNSW index.

### Types

```rust
use std::path::Path;
use std::sync::Arc;
use parking_lot::RwLock;  // or tokio::sync::RwLock if preferred
use crate::quantize::Quantizer;
use crate::error::MemoryError;

/// Configuration for the HNSW index
#[derive(Debug, Clone)]
pub struct HnswConfig {
    /// Max connections per node per layer. Higher = better recall, more memory.
    /// Recommended: 16 for <100k items, 32 for 100k-1M, 64 for >1M.
    pub m: usize,

    /// Width of search during index construction. Higher = better index quality, slower build.
    /// Recommended: 200 for most cases.
    pub ef_construction: usize,

    /// Width of search during queries. Higher = better recall, slower search.
    /// Recommended: 50-100 for most cases. Must be >= top_k.
    pub ef_search: usize,

    /// Embedding dimensionality. Must match the embedder output.
    /// Default: 768 (nomic-embed-text)
    pub dimensions: usize,

    /// Maximum number of elements the index can hold.
    /// The index will need to be rebuilt if this is exceeded.
    /// Default: 100_000
    pub max_elements: usize,
}

impl Default for HnswConfig {
    fn default() -> Self {
        Self {
            m: 16,
            ef_construction: 200,
            ef_search: 50,
            dimensions: 768,
            max_elements: 100_000,
        }
    }
}

/// Wrapper around hnswlib-rs providing semantic-memory's HNSW operations.
/// Thread-safe: can be shared across async tasks.
pub struct HnswIndex {
    // The exact inner types depend on hnswlib-rs API.
    // Expected shape:
    //   graph: Hnsw<String, CosineQi8>
    //   vectors: InMemoryQi8VectorStore
    //   quantizer: Quantizer
    //   config: HnswConfig
    //
    // Wrapped in Arc<RwLock<>> so MemoryStore can clone cheaply
    // and multiple async tasks can search concurrently.
    //
    // NOTE: hnswlib-rs claims to support concurrent search + mutation natively.
    // If that's true (verify!), the inner Hnsw may not need RwLock at all —
    // just Arc. Test this during implementation.
    inner: Arc<HnswIndexInner>,
}

struct HnswIndexInner {
    graph: /* Hnsw<String, CosineQi8> */,
    vectors: /* InMemoryQi8VectorStore */,
    quantizer: Quantizer,
    config: HnswConfig,
}
```

### Key Methods

```rust
impl HnswIndex {
    /// Create a new empty HNSW index.
    pub fn new(config: HnswConfig) -> Result<Self>;

    /// Load an existing HNSW index from disk.
    pub fn load(hnsw_path: &Path, vectors_path: &Path, config: HnswConfig) -> Result<Self>;

    /// Save the HNSW index to disk.
    pub fn save(&self, hnsw_path: &Path, vectors_path: &Path) -> Result<()>;

    /// Insert a vector with a string key.
    /// Key format: "{domain}:{id}" e.g. "fact:42", "chunk:17", "msg:99"
    /// The f32 vector is quantized to Qi8 before insertion.
    pub fn insert(&self, key: String, vector: &[f32]) -> Result<()>;

    /// Remove a vector by key. Tombstones the node in the graph.
    pub fn delete(&self, key: &str) -> Result<()>;

    /// Update a vector (delete + reinsert with repaired connections).
    pub fn update(&self, key: String, vector: &[f32]) -> Result<()>;

    /// Search for nearest neighbors. Returns keys and distances.
    /// The query vector is f32 and gets quantized internally.
    pub fn search(&self, query: &[f32], top_k: usize) -> Result<Vec<HnswHit>>;

    /// Number of vectors currently in the index.
    pub fn len(&self) -> usize;

    /// Whether the index is empty.
    pub fn is_empty(&self) -> bool;

    /// Rebuild the quantizer calibration from a set of vectors.
    /// Call this before bulk insert if you have representative data.
    pub fn calibrate_quantizer(&mut self, sample_vectors: &[Vec<f32>]) -> Result<()>;
}

/// A single hit from HNSW search
#[derive(Debug, Clone)]
pub struct HnswHit {
    /// The key that was inserted (e.g. "fact:42")
    pub key: String,
    /// Distance from query (lower = more similar for cosine distance)
    pub distance: f32,
}

impl HnswHit {
    /// Convert distance to similarity score in [0, 1] range.
    /// For cosine distance: similarity = 1.0 - distance
    /// (hnswlib-rs CosineQi8 returns cosine distance, not similarity)
    pub fn similarity(&self) -> f32 {
        (1.0 - self.distance).max(0.0)
    }

    /// Parse the domain and numeric ID from the key.
    /// "fact:42" → ("fact", 42)
    pub fn parse_key(&self) -> Result<(&str, i64)> {
        let (domain, id_str) = self.key.split_once(':')
            .ok_or_else(|| MemoryError::InvalidKey(self.key.clone()))?;
        let id = id_str.parse::<i64>()
            .map_err(|_| MemoryError::InvalidKey(self.key.clone()))?;
        Ok((domain, id))
    }
}
```

### Key Format Convention

All HNSW keys follow the format `"{domain}:{sqlite_rowid}"`:

| Domain | Table | Example Key |
|---|---|---|
| `fact` | facts | `"fact:42"` |
| `chunk` | document_chunks | `"chunk:17"` |
| `msg` | messages | `"msg:99"` |

This convention enables:
- Direct SQLite lookup from HNSW results: `SELECT * FROM facts WHERE id = 42`
- Domain filtering during search: only process hits where domain matches
- Disambiguation when all three domains share one HNSW index

### Single Index vs Per-Domain Indexes

**Decision: Single shared HNSW index for all domains.**

Rationale:
- Simplifies persistence (one pair of sidecar files, not three)
- Cross-domain search is a feature (agent searches "everything I know about X")
- Key prefix makes domain filtering trivial post-search
- Per-domain indexes would triple the memory-mapped file overhead

If a specific search only wants facts, the caller passes domain filter in `SearchConfig` and results are filtered after HNSW returns candidates. The over-fetching is minimal since HNSW search is O(log n) regardless.

---

## 6. New Module: src/quantize.rs

Scalar quantization (SQ8) converts f32 vectors to i8 with per-vector scale and zero_point. This module is independent of the HNSW backend and can be used with brute-force too.

### Theory

For each vector independently:
1. Find `min` and `max` across all 768 dimensions
2. Compute `scale = (max - min) / 254.0` (map to range [-127, 127])
3. Compute `zero_point = round(-128.0 - min / scale)` clamped to i8
4. For each dimension: `quantized[i] = round(original[i] / scale + zero_point)` clamped to i8
5. Store `(Vec<i8>, scale, zero_point)` — this is what hnswlib-rs calls `Qi8Ref`

To reconstruct (approximately): `original[i] ≈ (quantized[i] - zero_point) * scale`

### Implementation

```rust
/// Scalar quantization parameters for a single vector
#[derive(Debug, Clone)]
pub struct QuantizedVector {
    pub data: Vec<i8>,
    pub scale: f32,
    pub zero_point: i8,
}

/// Quantizer that converts f32 vectors to int8
#[derive(Debug, Clone)]
pub struct Quantizer {
    dimensions: usize,
}

impl Quantizer {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }

    /// Quantize a single f32 vector to int8 with per-vector calibration.
    /// This is asymmetric quantization — each vector gets its own scale/zero_point.
    pub fn quantize(&self, vector: &[f32]) -> Result<QuantizedVector> {
        assert_eq!(vector.len(), self.dimensions, "dimension mismatch");

        let min = vector.iter().copied().fold(f32::INFINITY, f32::min);
        let max = vector.iter().copied().fold(f32::NEG_INFINITY, f32::max);

        // Handle edge case: constant vector (all dimensions same value)
        if (max - min).abs() < f32::EPSILON {
            return Ok(QuantizedVector {
                data: vec![0i8; self.dimensions],
                scale: 1.0,
                zero_point: 0,
            });
        }

        let scale = (max - min) / 254.0;  // 254 = 127 - (-127)
        let zero_point_f = (-128.0 - min / scale).round();
        let zero_point = zero_point_f.clamp(-128.0, 127.0) as i8;

        let data: Vec<i8> = vector.iter()
            .map(|&v| {
                let q = (v / scale + zero_point as f32).round();
                q.clamp(-128.0, 127.0) as i8
            })
            .collect();

        Ok(QuantizedVector { data, scale, zero_point })
    }

    /// Dequantize back to f32 (approximate reconstruction).
    /// Used for debugging and verification, not in hot path.
    pub fn dequantize(&self, qv: &QuantizedVector) -> Vec<f32> {
        qv.data.iter()
            .map(|&q| (q as f32 - qv.zero_point as f32) * qv.scale)
            .collect()
    }

    /// Quantize and immediately produce a Qi8Ref for hnswlib-rs insertion.
    /// This avoids an intermediate allocation.
    #[cfg(feature = "hnsw")]
    pub fn to_qi8(&self, vector: &[f32]) -> Result<QuantizedVector> {
        self.quantize(vector)
    }
}
```

### Interaction with hnswlib-rs

When inserting into the HNSW index:

```rust
let qv = quantizer.quantize(&embedding_f32)?;
let qi8_ref = Qi8Ref {
    data: &qv.data,
    scale: qv.scale,
    zero_point: qv.zero_point,
};
hnsw.insert(&store, key, qi8_ref)?;
```

When searching:

```rust
let query_qv = quantizer.quantize(&query_f32)?;
let query_qi8 = Qi8Ref {
    data: &query_qv.data,
    scale: query_qv.scale,
    zero_point: query_qv.zero_point,
};
let hits = hnsw.search(&store, query_qi8, top_k, None)?;
```

### Quantization Error Budget

For 768-dim nomic-embed-text vectors (values typically in [-1.0, 1.0]):
- Scale ≈ 2.0 / 254 ≈ 0.0079 per step
- Max quantization error per dimension: ±0.004
- Cosine similarity error on normalized vectors: typically <0.5%
- This is well within HNSW's approximation budget

No calibration dataset is needed for per-vector asymmetric quantization. Each vector self-calibrates from its own min/max. This is the simplest correct approach and matches what hnswlib-rs's Qi8 format expects.

---

## 7. Modified Module: src/search.rs

### SearchConfig Changes

```rust
#[derive(Debug, Clone)]
pub struct SearchConfig {
    // Existing fields (unchanged)
    pub query: String,
    pub top_k: usize,
    pub domains: Vec<SearchDomain>,  // renamed from storage_types or similar
    pub namespace: Option<String>,
    pub rrf_k: f32,
    pub recency_boost: bool,
    pub recency_half_life_days: f64,

    // New fields for v0.2.0
    /// Number of candidates to fetch from HNSW before reranking.
    /// Higher = better recall, more SQLite lookups.
    /// Default: top_k * 3
    pub hnsw_candidates: Option<usize>,

    /// Whether to rerank HNSW results using exact f32 cosine similarity.
    /// Loads f32 vectors from SQLite for top HNSW candidates.
    /// Improves precision at cost of SQLite I/O.
    /// Default: false (quantized distance is usually good enough)
    pub rerank_with_f32: bool,

    /// Force brute-force scan even when HNSW is available.
    /// Requires the `brute-force` feature to be enabled.
    /// Useful for testing, debugging, or when HNSW index is being rebuilt.
    /// Default: false
    #[cfg(feature = "brute-force")]
    pub force_brute_force: bool,

    /// HNSW ef_search override for this specific query.
    /// If None, uses the index's configured ef_search.
    /// Higher values improve recall at the cost of latency.
    pub ef_search_override: Option<usize>,
}

impl Default for SearchConfig {
    fn default() -> Self {
        Self {
            query: String::new(),
            top_k: 10,
            domains: vec![SearchDomain::Facts, SearchDomain::Documents, SearchDomain::Conversations],
            namespace: None,
            rrf_k: 60.0,
            recency_boost: true,
            recency_half_life_days: 7.0,
            hnsw_candidates: None,
            rerank_with_f32: false,
            #[cfg(feature = "brute-force")]
            force_brute_force: false,
            ef_search_override: None,
        }
    }
}

#[derive(Debug, Clone, PartialEq)]
pub enum SearchDomain {
    Facts,
    Documents,
    Conversations,
}
```

### Modified Search Pipeline

The `search()` method in search.rs must be restructured:

```rust
pub async fn search(&self, config: &SearchConfig) -> Result<Vec<SearchResult>> {
    // 1. FTS5 search (unchanged)
    let fts_hits = self.fts_search(&config.query, &config.domains, &config.namespace).await?;

    // 2. Vector search — dispatch based on backend
    let embed_result = self.embedder.embed(&config.query).await?;
    let query_vec = embed_result;

    let vector_hits = self.vector_search(&query_vec, config).await?;

    // 3. RRF fusion (unchanged algorithm)
    let fused = self.rrf_fusion(fts_hits, vector_hits, config)?;

    // 4. Load content from SQLite for top results (unchanged)
    let results = self.load_results(fused, config.top_k).await?;

    Ok(results)
}

async fn vector_search(
    &self,
    query_vec: &[f32],
    config: &SearchConfig,
) -> Result<Vec<VectorHit>> {
    // Determine which backend to use
    #[cfg(feature = "brute-force")]
    if config.force_brute_force {
        return self.vector_search_brute_force(query_vec, config).await;
    }

    #[cfg(feature = "hnsw")]
    {
        let candidates = config.hnsw_candidates.unwrap_or(config.top_k * 3);
        let hnsw_hits = self.hnsw_index.search(query_vec, candidates)?;

        // Filter by domain if not searching all domains
        let filtered: Vec<HnswHit> = hnsw_hits.into_iter()
            .filter(|hit| {
                if let Ok((domain, _id)) = hit.parse_key() {
                    match domain {
                        "fact" => config.domains.contains(&SearchDomain::Facts),
                        "chunk" => config.domains.contains(&SearchDomain::Documents),
                        "msg" => config.domains.contains(&SearchDomain::Conversations),
                        _ => false,
                    }
                } else {
                    false
                }
            })
            .collect();

        // Optional f32 reranking
        if config.rerank_with_f32 {
            return self.rerank_with_exact_cosine(query_vec, &filtered).await;
        }

        // Convert to VectorHit format
        Ok(filtered.iter().map(|hit| {
            let (_domain, id) = hit.parse_key().unwrap(); // already filtered
            VectorHit {
                id,
                score: hit.similarity(),
            }
        }).collect())
    }

    #[cfg(not(feature = "hnsw"))]
    {
        self.vector_search_brute_force(query_vec, config).await
    }
}
```

### Reranking Implementation

```rust
/// Load f32 vectors from SQLite for HNSW candidates and compute exact cosine similarity.
async fn rerank_with_exact_cosine(
    &self,
    query_vec: &[f32],
    hnsw_hits: &[HnswHit],
) -> Result<Vec<VectorHit>> {
    let mut results = Vec::with_capacity(hnsw_hits.len());

    for hit in hnsw_hits {
        let (domain, id) = hit.parse_key()?;
        // Load the f32 embedding from SQLite
        let f32_vec = self.load_embedding(domain, id).await?;
        if let Some(vec) = f32_vec {
            let similarity = cosine_similarity(query_vec, &vec);
            results.push(VectorHit { id, score: similarity });
        }
    }

    // Re-sort by exact similarity
    results.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap_or(std::cmp::Ordering::Equal));
    Ok(results)
}
```

### Existing Brute-Force Code

The existing `scan_vector_rows` / brute-force logic moves behind `#[cfg(feature = "brute-force")]`. No changes to its implementation — it stays as-is for backward compatibility. The only change is the conditional compilation gate.

---

## 8. Modified Module: src/db.rs

### Schema Changes

**No schema changes to existing tables.** The SQLite schema for facts, document_chunks, messages, and FTS5 tables remains identical. The f32 embedding blobs stay in SQLite.

### New Table: hnsw_metadata

Add one metadata table to track HNSW index state:

```sql
CREATE TABLE IF NOT EXISTS hnsw_metadata (
    key TEXT PRIMARY KEY,
    value TEXT NOT NULL
);

-- Populated on HNSW save:
-- 'hnsw_version' → '0.2.0'
-- 'element_count' → '4523'
-- 'dimensions' → '768'
-- 'last_rebuilt' → '2026-02-24T21:00:00Z'
-- 'embedder_model' → 'nomic-embed-text'
```

This table lets the system detect when an HNSW rebuild is needed (e.g., embedder model changed, dimensions changed).

### Migration

Add as migration v3 (or whatever the next version is in the existing migration sequence):

```rust
fn migrate_v3(conn: &Connection) -> Result<()> {
    conn.execute_batch("
        CREATE TABLE IF NOT EXISTS hnsw_metadata (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        );
    ")?;
    Ok(())
}
```

### Connection Management

The `Arc<Mutex<Connection>>` pattern for SQLite remains unchanged. The HNSW index has its own concurrency model (see section 5). They don't share locks.

---

## 9. Modified Module: src/lib.rs

### MemoryStore Struct Changes

```rust
pub struct MemoryStore {
    // Existing
    conn: Arc<Mutex<Connection>>,
    embedder: Arc<dyn Embedder>,
    tokenizer: Arc<dyn Tokenizer>,
    config: MemoryConfig,

    // New in v0.2.0
    paths: StoragePaths,

    #[cfg(feature = "hnsw")]
    hnsw_index: HnswIndex,

    #[cfg(feature = "hnsw")]
    quantizer: Quantizer,
}
```

### MemoryConfig Changes

```rust
#[derive(Debug, Clone)]
pub struct MemoryConfig {
    // Existing fields (keep all)
    pub embedding: EmbeddingConfig,
    // ... other existing fields ...

    // New in v0.2.0
    #[cfg(feature = "hnsw")]
    pub hnsw: HnswConfig,
}

impl Default for MemoryConfig {
    fn default() -> Self {
        Self {
            embedding: EmbeddingConfig::default(),
            // ... other defaults ...
            #[cfg(feature = "hnsw")]
            hnsw: HnswConfig::default(),
        }
    }
}
```

### Constructor Changes

```rust
impl MemoryStore {
    /// Open or create a memory store at the given directory path.
    pub async fn open(
        base_dir: impl AsRef<Path>,
        config: MemoryConfig,
    ) -> Result<Self> {
        let paths = StoragePaths::new(base_dir);
        Self::open_with_paths(paths, config).await
    }

    /// Open with a custom embedder (for framework integration).
    /// This is the preferred path for llm-pipeline and agent-graph.
    pub async fn open_with_embedder(
        base_dir: impl AsRef<Path>,
        embedder: Arc<dyn Embedder>,
        tokenizer: Arc<dyn Tokenizer>,
        config: MemoryConfig,
    ) -> Result<Self> {
        let paths = StoragePaths::new(base_dir);
        Self::open_with_paths_and_embedder(paths, embedder, tokenizer, config).await
    }

    async fn open_with_paths(
        paths: StoragePaths,
        config: MemoryConfig,
    ) -> Result<Self> {
        // Create directory
        std::fs::create_dir_all(&paths.base_dir)?;

        // Open SQLite
        let conn = Connection::open(&paths.sqlite_path)?;
        // ... WAL mode, migrations, etc. (existing logic) ...

        // Create default embedder from config
        let embedder = Arc::new(OllamaEmbedder::new(&config.embedding)?);
        let tokenizer = /* existing tokenizer creation */;

        Self::initialize(paths, conn, embedder, tokenizer, config).await
    }

    async fn initialize(
        paths: StoragePaths,
        conn: Connection,
        embedder: Arc<dyn Embedder>,
        tokenizer: Arc<dyn Tokenizer>,
        config: MemoryConfig,
    ) -> Result<Self> {
        let conn = Arc::new(Mutex::new(conn));

        #[cfg(feature = "hnsw")]
        let (hnsw_index, quantizer) = {
            let quantizer = Quantizer::new(config.hnsw.dimensions);

            let hnsw_index = if paths.hnsw_path.exists() && paths.vectors_path.exists() {
                // Load existing index
                tracing::info!("Loading HNSW index from {:?}", paths.hnsw_path);
                HnswIndex::load(&paths.hnsw_path, &paths.vectors_path, config.hnsw.clone())?
            } else {
                // Check if SQLite has embeddings that need indexing
                let has_embeddings = {
                    let c = conn.lock();
                    // Check if any rows have non-null embeddings
                    let count: i64 = c.query_row(
                        "SELECT COUNT(*) FROM facts WHERE embedding IS NOT NULL",
                        [],
                        |r| r.get(0),
                    )?;
                    count > 0
                };

                if has_embeddings {
                    tracing::info!("SQLite has embeddings but no HNSW index. Rebuilding...");
                    let index = HnswIndex::new(config.hnsw.clone())?;
                    // Rebuild will happen after store is constructed
                    // Set a flag or do it here
                    index
                } else {
                    tracing::info!("Creating new empty HNSW index");
                    HnswIndex::new(config.hnsw.clone())?
                }
            };

            (hnsw_index, quantizer)
        };

        let store = Self {
            conn,
            embedder,
            tokenizer,
            config,
            paths,
            #[cfg(feature = "hnsw")]
            hnsw_index,
            #[cfg(feature = "hnsw")]
            quantizer,
        };

        // If we detected orphaned embeddings, rebuild now
        #[cfg(feature = "hnsw")]
        if !paths.hnsw_path.exists() {
            let has_embeddings = /* check again */;
            if has_embeddings {
                store.rebuild_hnsw_index().await?;
            }
        }

        Ok(store)
    }
}
```

### Drop Implementation (HNSW Persistence)

```rust
impl Drop for MemoryStore {
    fn drop(&mut self) {
        #[cfg(feature = "hnsw")]
        {
            if let Err(e) = self.hnsw_index.save(&self.paths.hnsw_path, &self.paths.vectors_path) {
                tracing::error!("Failed to save HNSW index on drop: {}", e);
            }
        }
    }
}
```

**IMPORTANT:** `Drop` cannot be async. The `save()` method on `HnswIndex` must be synchronous (it's just file I/O with `std::io::Write`). Callers who want guaranteed persistence should call `flush_hnsw().await` explicitly before dropping.

### Clone Semantics

`MemoryStore` should remain cheaply cloneable (Arc internals). The HNSW index is wrapped in `Arc` so clones share the same index. This is critical for use in `agent-graph` where multiple nodes may share a memory store.

---

## 10. Modified Module: src/embedder.rs

### Embedder Trait — No Changes

The existing trait is correct:

```rust
#[async_trait]
pub trait Embedder: Send + Sync {
    async fn embed(&self, text: &str) -> Result<Vec<f32>>;
    async fn embed_batch(&self, texts: Vec<String>) -> Result<Vec<Vec<f32>>>;
    fn dimensions(&self) -> usize;
}
```

### New: dimensions() Method

If `dimensions()` doesn't already exist on the trait, add it. The HNSW index needs to know dimensions at construction time, and it should come from the embedder, not from config duplication.

```rust
// In MemoryConfig construction or validation:
#[cfg(feature = "hnsw")]
{
    // Validate that embedder dimensions match HNSW config
    let embed_dims = embedder.dimensions();
    if config.hnsw.dimensions != embed_dims {
        tracing::warn!(
            "HNSW dimensions ({}) don't match embedder dimensions ({}). Using embedder's.",
            config.hnsw.dimensions, embed_dims
        );
        config.hnsw.dimensions = embed_dims;
    }
}
```

### MockEmbedder Updates

The existing `MockEmbedder` with deterministic hash-seeded xorshift is great. Ensure it implements `dimensions()`:

```rust
impl MockEmbedder {
    pub fn new(dimensions: usize) -> Self {
        Self { dimensions }
    }
}

impl Embedder for MockEmbedder {
    // ... existing embed/embed_batch ...
    fn dimensions(&self) -> usize {
        self.dimensions
    }
}
```

For HNSW tests, `MockEmbedder` vectors must be non-trivial (not all zeros, not all identical) so that cosine distances are meaningful. The existing hash-seeded approach handles this.

---

## 11. Modified Module: src/knowledge.rs

### Insert Path Changes

Every function that currently inserts an embedding blob into SQLite must ALSO insert into HNSW:

```rust
pub async fn add_fact(
    &self,
    content: &str,
    namespace: &str,
    metadata: Option<serde_json::Value>,
) -> Result<i64> {
    // 1. Embed (unchanged)
    let embedding = self.embedder.embed(content).await?;

    // 2. SQLite insert (unchanged — stores content, metadata, f32 embedding blob)
    let fact_id = self.with_conn(|conn| {
        // existing INSERT logic
        // ...
        Ok(fact_id)
    }).await?;

    // 3. FTS5 bridge insert (unchanged)
    self.insert_fts_bridge(fact_id, content).await?;

    // 4. NEW: HNSW insert
    #[cfg(feature = "hnsw")]
    {
        let key = format!("fact:{}", fact_id);
        self.hnsw_index.insert(key, &embedding)?;
    }

    Ok(fact_id)
}
```

### Delete Path Changes

Every function that deletes content must ALSO remove from HNSW:

```rust
pub async fn delete_fact(&self, fact_id: i64) -> Result<()> {
    // 1. Delete from SQLite + FTS5 (existing logic, unchanged)
    self.delete_fact_with_fts(fact_id).await?;

    // 2. NEW: Delete from HNSW
    #[cfg(feature = "hnsw")]
    {
        let key = format!("fact:{}", fact_id);
        self.hnsw_index.delete(&key)?;
    }

    Ok(())
}
```

### Update Path Changes

If facts can be updated (content changed, re-embedded):

```rust
pub async fn update_fact(&self, fact_id: i64, new_content: &str) -> Result<()> {
    let embedding = self.embedder.embed(new_content).await?;

    // SQLite update (existing)
    // FTS5 update (existing)

    // NEW: HNSW update
    #[cfg(feature = "hnsw")]
    {
        let key = format!("fact:{}", fact_id);
        self.hnsw_index.update(key, &embedding)?;
    }

    Ok(())
}
```

### Bulk Operations: reembed_all()

After `reembed_all()` completes, the HNSW index must be rebuilt:

```rust
pub async fn reembed_all(&self) -> Result<usize> {
    let count = /* existing reembed logic */;

    // After all embeddings are updated in SQLite, rebuild HNSW
    #[cfg(feature = "hnsw")]
    {
        tracing::info!("Reembedding complete. Rebuilding HNSW index...");
        self.rebuild_hnsw_index().await?;
    }

    Ok(count)
}
```

---

## 12. Modified Module: src/documents.rs

Same pattern as knowledge.rs. For `ingest_document`:

```rust
pub async fn ingest_document(
    &self,
    content: &str,
    source: &str,
    metadata: Option<serde_json::Value>,
) -> Result<Vec<i64>> {
    let chunks = self.chunker.chunk(content);
    let mut chunk_ids = Vec::new();

    for chunk_text in chunks {
        let embedding = self.embedder.embed(&chunk_text).await?;
        let chunk_id = /* SQLite insert */;
        /* FTS5 bridge insert */;

        #[cfg(feature = "hnsw")]
        {
            let key = format!("chunk:{}", chunk_id);
            self.hnsw_index.insert(key, &embedding)?;
        }

        chunk_ids.push(chunk_id);
    }

    Ok(chunk_ids)
}
```

For document deletion, iterate chunks and delete each from HNSW.

---

## 13. Modified Module: src/conversation.rs

Same pattern. `add_message_embedded`:

```rust
pub async fn add_message_embedded(
    &self,
    session_id: &str,
    role: &str,
    content: &str,
) -> Result<i64> {
    let embedding = self.embedder.embed(content).await?;
    let msg_id = /* SQLite insert */;

    #[cfg(feature = "hnsw")]
    {
        let key = format!("msg:{}", msg_id);
        self.hnsw_index.insert(key, &embedding)?;
    }

    Ok(msg_id)
}
```

**NOTE:** Not all messages need embeddings. Short messages ("ok", "thanks") don't benefit from semantic search. The existing logic for deciding which messages to embed should remain unchanged. Only embed messages that pass the threshold, and only insert those into HNSW.

---

## 14. Bug Fixes

These bugs were identified in the v0.1.0 review and should be fixed in v0.2.0:

### Bug 1: delete_namespace Lacks Transactional Atomicity (MEDIUM)

**File:** src/knowledge.rs (or wherever `delete_namespace` lives)

**Problem:** Currently loops through `delete_fact_with_fts()` calls, each in its own transaction. If the process crashes mid-loop, some facts are deleted and others aren't.

**Fix:** Wrap the entire loop in a single transaction.

```rust
pub async fn delete_namespace(&self, namespace: &str) -> Result<usize> {
    // Get all fact IDs in namespace
    let fact_ids: Vec<i64> = self.with_conn(|conn| {
        let mut stmt = conn.prepare("SELECT id FROM facts WHERE namespace = ?1")?;
        let ids = stmt.query_map([namespace], |row| row.get(0))?
            .collect::<std::result::Result<Vec<i64>, _>>()?;
        Ok(ids)
    }).await?;

    let count = fact_ids.len();

    // Delete all in a single transaction
    self.with_conn(|conn| {
        let tx = conn.unchecked_transaction()?;
        for &fact_id in &fact_ids {
            // Delete FTS bridge row
            tx.execute("DELETE FROM facts_fts_bridge WHERE fact_id = ?1", [fact_id])?;
            // Delete FTS content
            tx.execute(
                "INSERT INTO facts_fts(facts_fts, rowid, content) VALUES('delete', ?1, \
                 (SELECT content FROM facts WHERE id = ?1))",
                [fact_id],
            )?;
            // Delete fact
            tx.execute("DELETE FROM facts WHERE id = ?1", [fact_id])?;
        }
        tx.commit()?;
        Ok(())
    }).await?;

    // Delete from HNSW outside the SQLite transaction
    #[cfg(feature = "hnsw")]
    {
        for &fact_id in &fact_ids {
            let key = format!("fact:{}", fact_id);
            self.hnsw_index.delete(&key)?;
        }
    }

    Ok(count)
}
```

### Bug 2: raw_execute is pub #[doc(hidden)] (MINOR)

**File:** src/db.rs or src/lib.rs

**Problem:** SQL injection surface exposed as public API.

**Fix:** Gate behind `#[cfg(test)]` or a `testing` feature flag:

```rust
#[cfg(any(test, feature = "testing"))]
pub fn raw_execute(&self, sql: &str) -> Result<()> {
    // ...
}
```

### Bug 3: decode_buf in scan_vector_rows is Vestigial (MINOR)

**File:** src/search.rs

**Problem:** `decode_buf` is allocated and cleared each iteration but never written to. `bytemuck::try_cast_slice` is used directly instead.

**Fix:** Remove the `decode_buf` variable entirely.

```rust
// BEFORE (broken):
let mut decode_buf = Vec::new();
for row in rows {
    decode_buf.clear();  // never used
    let blob: Vec<u8> = row.get(1)?;
    let embedding: &[f32] = bytemuck::try_cast_slice(&blob)?;
    // ...
}

// AFTER (clean):
for row in rows {
    let blob: Vec<u8> = row.get(1)?;
    let embedding: &[f32] = bytemuck::try_cast_slice(&blob)?;
    // ...
}
```

### Bug 4: FTS Query Sanitization Edge Cases (MINOR)

**File:** src/search.rs

**Problem:** Current sanitization strips FTS5 operators but misses edge cases like leading NOT, bare AND/OR tokens, and unmatched quotes.

**Fix:** Improve the sanitizer:

```rust
fn sanitize_fts_query(query: &str) -> String {
    // Strip FTS5 special characters
    let cleaned: String = query.chars()
        .filter(|c| !matches!(c, '"' | '*' | '+' | '-' | '(' | ')' | '^' | '~'))
        .collect();

    // Split into tokens and filter problematic ones
    let tokens: Vec<&str> = cleaned.split_whitespace()
        .filter(|t| {
            let upper = t.to_uppercase();
            // Remove bare boolean operators
            !matches!(upper.as_str(), "AND" | "OR" | "NOT" | "NEAR")
        })
        .collect();

    if tokens.is_empty() {
        return String::new();
    }

    tokens.join(" ")
}
```

---

## 15. Testing Strategy

### Test Organization

```
tests/
├── hnsw_integration.rs      # HNSW-specific tests (gated on feature)
├── brute_force_parity.rs    # Verify HNSW and brute-force return similar results
├── quantization.rs           # SQ8 quantization accuracy tests
├── search_regression.rs      # Existing search tests, must still pass
├── storage_lifecycle.rs      # Open, close, reopen, rebuild tests
└── concurrent_access.rs      # Multi-task read/write on shared MemoryStore
```

### Critical Test Cases

**1. Quantization round-trip accuracy:**
```rust
#[test]
fn test_sq8_round_trip_accuracy() {
    let q = Quantizer::new(768);
    let original: Vec<f32> = (0..768).map(|i| (i as f32 / 768.0) * 2.0 - 1.0).collect();
    let quantized = q.quantize(&original).unwrap();
    let reconstructed = q.dequantize(&quantized);

    let max_error = original.iter().zip(reconstructed.iter())
        .map(|(a, b)| (a - b).abs())
        .fold(0.0f32, f32::max);

    assert!(max_error < 0.01, "Max quantization error: {}", max_error);
}
```

**2. Quantization with real-world vector distribution:**
```rust
#[test]
fn test_sq8_cosine_similarity_preservation() {
    let q = Quantizer::new(768);
    // Generate two similar vectors and two dissimilar vectors
    let v1 = random_normalized_vector(768);
    let v2 = perturb(&v1, 0.05);  // small perturbation
    let v3 = random_normalized_vector(768);  // unrelated

    let exact_sim_12 = cosine_similarity(&v1, &v2);
    let exact_sim_13 = cosine_similarity(&v1, &v3);

    // Quantize and compute approximate similarity
    let q1 = q.quantize(&v1).unwrap();
    let q2 = q.quantize(&v2).unwrap();
    let q3 = q.quantize(&v3).unwrap();
    let approx_sim_12 = cosine_similarity(&q.dequantize(&q1), &q.dequantize(&q2));
    let approx_sim_13 = cosine_similarity(&q.dequantize(&q1), &q.dequantize(&q3));

    // Ranking should be preserved
    assert!(approx_sim_12 > approx_sim_13);
    // Absolute error should be small
    assert!((exact_sim_12 - approx_sim_12).abs() < 0.02);
}
```

**3. HNSW insert → search round-trip:**
```rust
#[tokio::test]
async fn test_hnsw_insert_search() {
    let store = create_test_store_with_hnsw().await;

    store.add_fact("The Earth orbits the Sun", "science", None).await.unwrap();
    store.add_fact("Water boils at 100 degrees Celsius", "science", None).await.unwrap();
    store.add_fact("My favorite color is blue", "personal", None).await.unwrap();

    let results = store.search(&SearchConfig {
        query: "planetary orbital mechanics".to_string(),
        top_k: 2,
        ..Default::default()
    }).await.unwrap();

    assert!(!results.is_empty());
    assert!(results[0].content.contains("Earth"));
}
```

**4. Persistence: close and reopen:**
```rust
#[tokio::test]
async fn test_hnsw_persistence() {
    let dir = tempfile::tempdir().unwrap();

    // Create and populate
    {
        let store = MemoryStore::open(dir.path(), MemoryConfig::default()).await.unwrap();
        store.add_fact("Test fact", "ns", None).await.unwrap();
        store.flush_hnsw().await.unwrap();
    }  // Drop, triggers save

    // Reopen and verify
    {
        let store = MemoryStore::open(dir.path(), MemoryConfig::default()).await.unwrap();
        let results = store.search(&SearchConfig {
            query: "Test".to_string(),
            top_k: 1,
            ..Default::default()
        }).await.unwrap();
        assert_eq!(results.len(), 1);
    }
}
```

**5. HNSW rebuild from SQLite:**
```rust
#[tokio::test]
async fn test_hnsw_rebuild_from_sqlite() {
    let dir = tempfile::tempdir().unwrap();

    // Create and populate
    {
        let store = MemoryStore::open(dir.path(), MemoryConfig::default()).await.unwrap();
        for i in 0..100 {
            store.add_fact(&format!("Fact number {}", i), "ns", None).await.unwrap();
        }
        store.flush_hnsw().await.unwrap();
    }

    // Delete sidecar files
    std::fs::remove_file(dir.path().join("memory.hnsw")).unwrap();
    std::fs::remove_file(dir.path().join("memory.vectors")).unwrap();

    // Reopen — should rebuild automatically
    let store = MemoryStore::open(dir.path(), MemoryConfig::default()).await.unwrap();

    let results = store.search(&SearchConfig {
        query: "Fact number 50".to_string(),
        top_k: 1,
        ..Default::default()
    }).await.unwrap();
    assert!(!results.is_empty());
}
```

**6. Brute-force / HNSW parity (requires both features):**
```rust
#[cfg(all(feature = "hnsw", feature = "brute-force"))]
#[tokio::test]
async fn test_brute_force_hnsw_parity() {
    let store = create_populated_test_store(50).await;  // 50 facts

    let query = "test query";

    let hnsw_results = store.search(&SearchConfig {
        query: query.to_string(),
        top_k: 5,
        force_brute_force: false,
        ..Default::default()
    }).await.unwrap();

    let brute_results = store.search(&SearchConfig {
        query: query.to_string(),
        top_k: 5,
        force_brute_force: true,
        ..Default::default()
    }).await.unwrap();

    // Top result should be the same (or at least overlap significantly)
    // Exact parity not expected due to quantization, but top-1 should match
    assert_eq!(hnsw_results[0].id, brute_results[0].id,
        "Top result should match between HNSW and brute-force");

    // At least 3 of top 5 should overlap
    let hnsw_ids: HashSet<i64> = hnsw_results.iter().map(|r| r.id).collect();
    let brute_ids: HashSet<i64> = brute_results.iter().map(|r| r.id).collect();
    let overlap = hnsw_ids.intersection(&brute_ids).count();
    assert!(overlap >= 3, "Expected at least 3/5 overlap, got {}", overlap);
}
```

**7. Concurrent insert + search:**
```rust
#[tokio::test]
async fn test_concurrent_insert_and_search() {
    let store = Arc::new(create_test_store_with_hnsw().await);

    // Spawn inserters
    let insert_store = store.clone();
    let inserter = tokio::spawn(async move {
        for i in 0..50 {
            insert_store.add_fact(
                &format!("Concurrent fact {}", i), "ns", None
            ).await.unwrap();
        }
    });

    // Spawn searchers concurrently
    let search_store = store.clone();
    let searcher = tokio::spawn(async move {
        for _ in 0..20 {
            let _ = search_store.search(&SearchConfig {
                query: "concurrent".to_string(),
                top_k: 5,
                ..Default::default()
            }).await;
            tokio::time::sleep(Duration::from_millis(10)).await;
        }
    });

    // Both should complete without panics or deadlocks
    let (r1, r2) = tokio::join!(inserter, searcher);
    r1.unwrap();
    r2.unwrap();
}
```

**8. Edge cases:**
```rust
#[test]
fn test_quantize_constant_vector() {
    // All dimensions same value — edge case
    let q = Quantizer::new(4);
    let v = vec![0.5, 0.5, 0.5, 0.5];
    let result = q.quantize(&v).unwrap();
    // Should not panic or produce NaN
    assert_eq!(result.data.len(), 4);
}

#[test]
fn test_quantize_extreme_values() {
    let q = Quantizer::new(4);
    let v = vec![-1000.0, 0.0, 0.001, 1000.0];
    let result = q.quantize(&v).unwrap();
    assert_eq!(result.data.len(), 4);
}

#[tokio::test]
async fn test_search_empty_store() {
    let store = create_empty_test_store().await;
    let results = store.search(&SearchConfig {
        query: "anything".to_string(),
        top_k: 5,
        ..Default::default()
    }).await.unwrap();
    assert!(results.is_empty());
}

#[tokio::test]
async fn test_delete_then_search() {
    let store = create_test_store_with_hnsw().await;
    let id = store.add_fact("Delete me", "ns", None).await.unwrap();
    store.delete_fact(id).await.unwrap();

    let results = store.search(&SearchConfig {
        query: "Delete me".to_string(),
        top_k: 5,
        ..Default::default()
    }).await.unwrap();

    // Should not appear in results
    assert!(!results.iter().any(|r| r.id == id));
}
```

---

## 16. Migration Path

### No Data Migration Needed

Since the library has never been used in production, there is no migration path from v0.1.0 databases. The v0.2.0 schema includes the new `hnsw_metadata` table and the open path expects a directory instead of a file.

### If Someone DID Have a v0.1.0 Database

Document this in the README as a manual migration:

```rust
// Move your old memory.db into a new directory
// mkdir /path/to/memory
// mv memory.db /path/to/memory/memory.db
// Open with v0.2.0 — HNSW index will be built automatically from existing embeddings
let store = MemoryStore::open("/path/to/memory", config).await?;
```

The auto-rebuild logic in `open()` (Section 9) handles this case: SQLite has embeddings, sidecar files don't exist, so it rebuilds.

---

## 17. Performance Targets

### Benchmarks to Validate

| Operation | Target (768d, 10k items) | Target (768d, 100k items) |
|---|---|---|
| HNSW insert | < 1ms | < 2ms |
| HNSW search (top 10) | < 1ms | < 5ms |
| SQ8 quantize (single vec) | < 10μs | < 10μs |
| Full search pipeline (embed + HNSW + FTS + RRF) | < 100ms | < 150ms |
| Brute-force scan (for comparison) | ~15ms | ~150ms |
| Index save to disk | < 100ms | < 500ms |
| Index load from disk | < 100ms | < 500ms |
| Rebuild from SQLite | < 5s | < 60s |

The embedding API call (Ollama) typically takes 20-50ms and dominates latency. The search infrastructure should never be the bottleneck.

### Memory Targets

| Data Size | f32 in SQLite | Qi8 in Memory (HNSW) | HNSW Graph Overhead |
|---|---|---|---|
| 10k × 768d | 29 MB | 7.5 MB | ~5 MB (M=16) |
| 100k × 768d | 290 MB | 75 MB | ~50 MB (M=16) |

Total memory for 100k items with HNSW: ~125 MB. Well within homelab constraints.

---

## 18. Future Considerations (Do NOT Implement)

These are noted for architectural awareness but are explicitly **out of scope** for v0.2.0:

1. **Multi-provider embeddings** (OpenAI, Cohere, fastembed) — Trivial to add via feature flags + Embedder trait implementations. Save for v0.3.0 or v0.4.0.

2. **Product Quantization (PQ)** — Only needed at millions of vectors. SQ8 + HNSW handles 100k+ comfortably.

3. **Parallel HNSW insertion** — hnswlib-rs may support `parallel_insert`. Worth exploring for `rebuild_hnsw_index()` performance, but not required for correctness.

4. **WAL mode read concurrency** — Currently the `Arc<Mutex<Connection>>` serializes SQLite. WAL mode could allow concurrent readers with a single writer. This matters more after agent-graph integration surfaces real contention.

5. **Matryoshka dimensionality reduction** — Some embedding models support reduced dimensions (768 → 256) with minimal quality loss. Could be combined with SQ8 for extreme compression. Not needed at current scale.

6. **MCP server wrapper** — Exposing semantic-memory as an MCP server. Natural next step after agent-graph integration, but separate crate.

---

## Appendix A: hnswlib-rs API Quick Reference

Based on crates.io documentation (January 2026):

```rust
use hnswlib_rs::{
    Hnsw,                    // The graph. Generic over K (key type) and M (metric).
    HnswConfig,              // Builder for graph parameters.
    InMemoryQi8VectorStore,  // Quantized int8 vector storage.
    CosineQi8,               // Cosine distance metric for Qi8 vectors.
    Qi8Ref,                  // Borrowed quantized vector: { data: &[i8], scale: f32, zero_point: i8 }
    Hit,                     // Search result: { key: K, distance: f32 }
    Result,                  // hnswlib-rs error type
};

// Construction
let cfg = HnswConfig::new(dim, max_nodes).m(16).ef_construction(200).ef_search(50);
let hnsw: Hnsw<String, CosineQi8> = Hnsw::new(CosineQi8::new(), cfg);
let store = InMemoryQi8VectorStore::new(dim, max_nodes);

// Insert
let qi8 = Qi8Ref { data: &quantized_bytes, scale: 0.02, zero_point: 0 };
hnsw.insert(&store, "fact:42".to_string(), qi8)?;

// Search
let hits: Vec<Hit<String>> = hnsw.search(&store, query_qi8, top_k, None)?;
// hits[0].key == "fact:42", hits[0].distance == 0.123

// Delete
hnsw.delete("fact:42")?;

// Update (insert-or-update, resurrects deleted keys)
hnsw.set(&store, "fact:42".to_string(), new_qi8)?;

// Persistence (graph and vectors saved separately)
hnsw.save_to(&mut File::create("memory.hnsw")?)?;
store.save_to(&mut File::create("memory.vectors")?, hnsw.len())?;

// Loading
let hnsw = Hnsw::load_from(CosineQi8::new(), &mut File::open("memory.hnsw")?)?;
let (store, count) = InMemoryQi8VectorStore::load_from(&mut File::open("memory.vectors")?)?;
```

**CRITICAL IMPLEMENTATION NOTES:**

1. Verify these exact type names and method signatures against the actual crate before writing code. The crate is new (Jan 2026) and API may have evolved.

2. If `Hnsw::delete` doesn't exist (only tombstoning via `set`), adjust the `delete_fact`/`delete_namespace` implementations accordingly.

3. If `CosineQi8` doesn't exist, try `Cosine` as the metric with `InMemoryQi8VectorStore` — the store type may determine the distance computation, not the metric type.

4. The `None` parameter in `search()` may be a filter function `Option<&dyn Fn(&K) -> bool>`. If so, this is useful for domain filtering directly in the search instead of post-filtering.

---

## Appendix B: File Checklist

All files that need to be created or modified:

### New Files
- [ ] `src/hnsw.rs` — HNSW index wrapper
- [ ] `src/quantize.rs` — SQ8 scalar quantization
- [ ] `src/storage.rs` — `StoragePaths` struct (or inline in lib.rs)
- [ ] `tests/hnsw_integration.rs`
- [ ] `tests/quantization.rs`
- [ ] `tests/brute_force_parity.rs`
- [ ] `tests/storage_lifecycle.rs`
- [ ] `tests/concurrent_access.rs`

### Modified Files
- [ ] `Cargo.toml` — version, features, dependencies
- [ ] `src/lib.rs` — MemoryStore struct, constructors, MemoryConfig, Drop
- [ ] `src/search.rs` — SearchConfig, vector_search dispatch, reranking
- [ ] `src/db.rs` — migration v3 (hnsw_metadata table)
- [ ] `src/knowledge.rs` — HNSW insert/delete in all write paths
- [ ] `src/documents.rs` — HNSW insert/delete in all write paths
- [ ] `src/conversation.rs` — HNSW insert/delete in message write paths
- [ ] `src/embedder.rs` — dimensions() method if missing
- [ ] `README.md` — updated usage examples, feature flag docs

### Unchanged Files
- [ ] `src/chunker.rs` — no changes needed
- [ ] `src/error.rs` — may need new error variants (InvalidKey, HnswError)

---

## Appendix C: Error Types to Add

```rust
#[derive(Debug, thiserror::Error)]
pub enum MemoryError {
    // Existing variants unchanged...

    #[error("HNSW index error: {0}")]
    HnswError(String),

    #[error("Invalid HNSW key format: {0}")]
    InvalidKey(String),

    #[error("Quantization error: {0}")]
    QuantizationError(String),

    #[error("Storage path error: {0}")]
    StorageError(String),

    #[error("Index integrity check failed: {in_sqlite_not_hnsw} items in SQLite but not HNSW, {in_hnsw_not_sqlite} items in HNSW but not SQLite")]
    IntegrityError {
        in_sqlite_not_hnsw: usize,
        in_hnsw_not_sqlite: usize,
    },
}
```
