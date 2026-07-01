# fib-quant: Production Vector Database Gap Analysis

## Date: 2026-06-26
## Status: Research Report — No code changes made

---

## Executive Summary

fib-quant is a solid experimental codec with a well-designed encode/decode/scoring
pipeline, but it lacks nearly every feature required for production vector database
usage. Comparing against what semantic-memory requires from its vector backend
(`VectorBackend` trait), what turbo-quant's sidecar provides, and what real
vector DBs need, fib-quant is missing **9 critical feature areas**.

The core issue: fib-quant's `FibSidecarIndex` is an **in-memory, linear-scan,
inner-product-only, no-persistence, no-delete, no-filter index**. Every search
scores every stored code (`score_all` iterates all entries). For production use
with 100K–1M vectors, this is O(n) per query — it needs approximate search
structure (IVF or HNSW integration), persistence, and the operational features
listed below.

---

## 1. HNSW Integration (CRITICAL)

### What semantic-memory has

`semantic-memory/src/hnsw.rs` implements `HnswIndex` wrapping `hnsw_rs::Hnsw` with:
- `insert(key, vector)` / `delete(key)` / `update(key, vector)`
- `search(query, top_k)` → `Vec<VectorHit>` sorted by ascending distance
- `save(dir, basename)` / `load(dir, basename, config)` with manifest + digests
- Keymap persisted to SQLite (`hnsw_keymap` table), graph in sidecar files
- Compaction when `deleted_ratio > threshold`
- The `VectorBackend` trait (`vector_backend.rs`) abstracts the backend so
  hnsw_rs and usearch are interchangeable

### What fib-quant has

`FibSidecarIndex` (sidecar.rs) does linear scan over all entries:
```rust
fn score_all(&self, query: &[f32]) -> Result<Vec<(usize, f32)>> {
    for (idx, (_, code)) in self.entries.iter().enumerate() {
        let s = self.scorer.inner_product_estimate(query, code)?;
        scored.push((idx, s));
    }
}
```
No graph structure, no approximate search. O(n) per query.

### What's missing

- **Compressed-vector HNSW**: Store `FibCodeV1` at HNSW graph nodes. Use the
  Gram-table approximate score for graph traversal, then exact rerank with
  raw f32 vectors from SQLite.
- **HNSW-aware encoding API**: The codec needs to expose the compressed
  representation in a format HNSW can use for distance comparisons during
  construction (not just query scoring).

### Proposed API

```rust
// fib-quant/src/hnsw_bridge.rs (new module)

/// A compressed vector entry suitable for HNSW graph node storage.
/// Contains the FibCodeV1 for approximate scoring plus a reference
/// to the raw vector for exact rerank.
pub struct HnswCompressedEntry {
    pub code: FibCodeV1,
    /// Optional raw vector reference (e.g., SQLite rowid or key)
    /// for exact rerank after approximate candidate selection.
    pub raw_vector_ref: Option<String>,
}

/// Build an HNSW index from compressed fib-quant codes.
/// The HNSW graph uses Gram-table approximate inner product for
/// distance during construction and search. Exact rerank uses
/// raw vectors fetched from the caller's store.
pub struct FibHnswIndex {
    scorer: FibScorer,
    // Delegates to an underlying HNSW implementation
    // (hnsw_rs or a custom graph)
    // ...
}

impl FibHnswIndex {
    pub fn new(scorer: FibScorer, config: FibHnswConfig) -> Result<Self>;
    pub fn insert(&mut self, key: String, code: FibCodeV1) -> Result<()>;
    pub fn insert_batch(&mut self, entries: Vec<(String, FibCodeV1)>) -> Result<()>;
    pub fn delete(&mut self, key: &str) -> Result<()>;
    pub fn search(
        &self,
        query: &[f32],
        top_k: usize,
        oversample: usize,
    ) -> Result<Vec<ScoredCandidate<String>>>;
    pub fn save(&self, dir: &Path, basename: &str) -> Result<()>;
    pub fn load(dir: &Path, basename: &str, scorer: FibScorer) -> Result<Self>;
}
```

---

## 2. Memory-Mapped Index Persistence (CRITICAL)

### What semantic-memory has

`HnswIndex::save` writes:
- `*.hnsw.graph` — graph structure with magic header, dim, vector_count
- `*.hnsw.data` — vector data in packed binary format
- `*.hnsw.manifest.json` — schema_version, generation_id, digests, dimensions, vector_count
- Atomic writes via temp files + rename
- `load` replays vectors from the data sidecar

SQLite uses `PRAGMA mmap_size = 268435456` (256MB) for memory-mapped I/O.

### What fib-quant has

`FibSidecarIndex` has **no save/load/persistence at all**. Entries are `Vec<(Id, FibCodeV1)>`
in memory only. `FibCodeV1` has `to_compact_bytes()` / `from_compact_bytes()` for
individual codes, and `FibCodeWireV1` provides self-describing wire format, but
there is no index-level serialization.

### What's missing

- **Index serialization format**: A binary file containing all codes + IDs + metadata
- **mmap-friendly layout**: Fixed-width records so codes can be accessed by offset
- **Manifest with digests**: Profile digest, codebook digest, entry count
- **Atomic save**: Temp file + rename for crash safety

### Proposed API

```rust
// fib-quant/src/sidecar_persist.rs (new module)

/// On-disk sidecar index format.
/// Layout:
///   [0..8]   magic: "FIBSIDX\0"
///   [8]      version: 1
///   [9..13]  ambient_dim (u32)
///   [13..17] block_dim (u32)
///   [17..21] codebook_size (u32)
///   [21..29] rotation_seed (u64)
///   [29..61] profile_digest (32 bytes)
///   [61..69] entry_count (u64)
///   [69..77] index_size_bytes (u64)
///   [77..109] codebook_digest (32 bytes)
///   [109..141] rotation_digest (32 bytes)
///   then: entry records, each:
///     [0..8]  id_len (u64)
///     [8..8+id_len] id bytes
///     [8+id_len..] compact FibCodeV1 bytes (variable length)
///
/// For mmap-friendly access, a fixed-width variant could use
/// a separate offset index: [offset_table][code_data]
pub const SIDECAR_MAGIC: [u8; 8] = *b"FIBSIDX\0";
pub const SIDECAR_VERSION: u8 = 1;
pub const SIDECAR_HEADER_SIZE: usize = 141;

pub struct FibSidecarManifest {
    pub schema_version: String,
    pub generation_id: String,
    pub profile_digest: String,
    pub codebook_digest: String,
    pub rotation_digest: String,
    pub ambient_dim: u32,
    pub entry_count: u64,
    pub created_at: String,
}

impl<Id> FibSidecarIndex<Id>
where
    Id: Clone + Eq + std::fmt::Debug + Serialize + for<'de> Deserialize<'de>,
{
    /// Save the index to a directory with atomic writes.
    pub fn save(&self, dir: &Path, basename: &str) -> Result<()>;

    /// Load an index from a sidecar directory. Requires the scorer
    /// to be provided (codebook + Gram table must match the saved profile).
    pub fn load(
        dir: &Path,
        basename: &str,
        scorer: FibScorer,
    ) -> Result<Self>;

    /// Load an index in mmap mode — codes are accessed by offset
    /// without copying into heap memory. Requires fixed-width code format.
    pub fn load_mmap(
        dir: &Path,
        basename: &str,
        scorer: FibScorer,
    ) -> Result<FibMmapSidecarIndex<Id>>;
}

/// Memory-mapped read-only sidecar index.
/// Codes are accessed via mmap offsets. No insert/delete —
/// rebuild via FibSidecarIndex for mutations.
pub struct FibMmapSidecarIndex<Id> { /* ... */ }

impl<Id> FibMmapSidecarIndex<Id> {
    pub fn search(&self, query: &[f32], top_k: usize, oversample: usize)
        -> Result<Vec<ScoredCandidate<Id>>>;
}
```

---

## 3. Batch Ingest API for 100K+ Vectors (HIGH)

### What fib-quant has

`FibQuantizer::encode_batch(&self, vectors: &[&[f32]])` exists in codec.rs (line 529)
and uses Rayon parallelism for n >= 16. This is good for encoding.

`FibSidecarIndex::add_batch` exists but just loops `add()`:
```rust
pub fn add_batch(&mut self, entries: Vec<(Id, FibCodeV1)>) {
    self.entries.reserve(entries.len());
    for (id, code) in entries {
        self.add(id, code);
    }
}
```

### What's missing

- **Bulk encode + insert pipeline**: A single API that takes raw vectors, encodes
  them in parallel, and inserts into the index — avoiding the intermediate
  `Vec<FibCodeV1>` allocation
- **Progress callback** for long-running ingest (100K+ vectors)
- **Memory budget awareness**: Estimate peak memory during batch encode
- **Resumable ingest**: Save progress, resume after crash

### Proposed API

```rust
// fib-quant/src/batch_ingest.rs (new module)

pub struct BatchIngestConfig {
    pub batch_size: usize,
    pub parallel: bool,
    pub progress_callback: Option<Arc<dyn Fn(usize, usize) + Send + Sync>>,
}

pub struct BatchIngestResult {
    pub total_encoded: usize,
    pub total_bytes: usize,
    pub elapsed_micros: u128,
    pub errors: Vec<(usize, FibQuantError)>,
}

impl<Id> FibSidecarIndex<Id>
where
    Id: Clone + Eq + std::fmt::Debug,
{
    /// Encode and insert a large batch of raw vectors.
    /// Encodes in parallel chunks, inserts incrementally,
    /// reports progress via callback.
    pub fn ingest_raw_batch(
        &mut self,
        vectors: &[&[f32]],
        ids: &[Id],
        config: &BatchIngestConfig,
    ) -> Result<BatchIngestResult>;
}
```

---

## 4. Update/Delete in the Compressed Index (HIGH)

### What semantic-memory has

`HnswIndex::delete(key)` marks deleted, `update(key, vector)` deletes + reinserts,
`needs_compaction()` checks deleted ratio, `flush_keymap()` persists to SQLite.

### What fib-quant has

**Nothing.** `FibSidecarIndex` has `add` and `add_batch` only. No `delete`, no
`update`, no compaction, no tombstones. The `entries: Vec<(Id, FibCodeV1)>` has
no mechanism for removal.

### What's missing

- `delete(id)` — remove an entry by ID
- `update(id, new_code)` — replace an entry's code
- Tombstone marking for lazy deletion (avoid immediate compaction)
- Compaction when deleted ratio exceeds threshold
- ID-based lookup (currently linear scan by insertion order, no HashMap)

### Proposed API

```rust
impl<Id> FibSidecarIndex<Id>
where
    Id: Clone + Eq + std::fmt::Debug + Hash,
{
    /// Delete an entry by ID. Marks as deleted (lazy).
    pub fn delete(&mut self, id: &Id) -> Result<()>;

    /// Update an existing entry's code. Equivalent to delete + add
    /// but preserves the ID slot.
    pub fn update(&mut self, id: Id, code: FibCodeV1) -> Result<()>;

    /// Ratio of deleted entries to total.
    pub fn deleted_ratio(&self) -> f32;

    /// Whether compaction should run (deleted_ratio > threshold).
    pub fn needs_compaction(&self) -> bool;

    /// Compact: remove tombstones, rebuild internal storage.
    pub fn compact(&mut self) -> Result<()>;

    /// Number of live (non-deleted) entries.
    pub fn live_len(&self) -> usize;
}
```

---

## 5. Distance Metrics Beyond Inner Product (HIGH)

### What semantic-memory has

`HnswIndex` uses `DistCosine` from hnsw_rs. The `VectorBackend` trait's `search`
returns `VectorHit { distance: f32 }` where distance is `1 - cosine_similarity`.

`TurboQuantCodec` in `vector_codec.rs` exposes:
- `score_inner_product(artifact, query)` → f32
- `score_l2(artifact, query)` → f32 (squared L2 distance)
- `prepare_query(query)` for batch scoring

### What fib-quant has

`FibScorer` supports **inner product only**:
- `inner_product_estimate(query, code)` → f32
- `score_prepared(prepared, code)` → f32
- `score_batch(query, codes)` → Vec<ScoredItem>

No L2 distance, no cosine distance (cosine would require `1 - IP/||q||/||v||`).
The Gram table `G[i,j] = <cw_i, cw_j>` is inherently an inner product structure.

### What's missing

- **L2 distance estimate**: Would need a different precomputed table
  (`||cw_i - cw_j||^2 = ||cw_i||^2 + ||cw_j||^2 - 2<cw_i, cw_j>`) — derivable
  from the Gram table but needs an API
- **Cosine distance**: `1 - inner_product / (||q|| * ||v||)` — needs the norms
  which are already stored in FibCodeV1, so this is a thin wrapper
- **Metric-aware search**: The search API should accept a metric parameter

### Proposed API

```rust
// fib-quant/src/scoring.rs (extensions)

/// Distance metric for approximate scoring.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Inner product (higher = closer)
    InnerProduct,
    /// Cosine similarity (higher = closer, range [-1, 1])
    Cosine,
    /// Squared L2 distance (lower = closer)
    L2Squared,
}

impl FibScorer {
    /// Estimate the distance between a query and a stored code
    /// under the given metric.
    pub fn distance_estimate(
        &self,
        query: &[f32],
        code: &FibCodeV1,
        metric: DistanceMetric,
    ) -> Result<f32>;

    /// Prepare a query for batch scoring under the given metric.
    pub fn prepare_query_with_metric(
        &self,
        query: &[f32],
        metric: DistanceMetric,
    ) -> Result<FibPreparedQuery>;

    /// L2 squared distance estimate from the Gram table.
    /// ||q - v||^2 = ||q||^2 + ||v||^2 - 2<q, v>
    /// Uses the stored norm + Gram-table IP estimate.
    pub fn l2_squared_estimate(&self, query: &[f32], code: &FibCodeV1) -> Result<f32>;

    /// Cosine similarity estimate = IP / (||q|| * ||v||)
    pub fn cosine_estimate(&self, query: &[f32], code: &FibCodeV1) -> Result<f32>;
}

impl<Id> FibSidecarIndex<Id> {
    /// Search with an explicit distance metric.
    pub fn search_with_metric(
        &self,
        query: &[f32],
        top_k: usize,
        oversample: usize,
        metric: DistanceMetric,
    ) -> Result<Vec<ScoredCandidate<Id>>>;
}
```

---

## 6. Filter-Then-Search / Metadata Filtering (MEDIUM)

### What semantic-memory has

Search in `search.rs` supports namespace filtering, source type filtering
(`SearchSourceType`), temporal filtering, and metadata conditions. These filters
are applied **before** vector search (in SQL) or during RRF fusion.

### What fib-quant has

**Nothing.** No filtering capability. `FibSidecarIndex::search` scores all entries
regardless.

### What's missing

- **Pre-filter**: Only score entries matching a predicate (e.g., namespace, tags)
- **Filter callback**: A closure or trait object that decides per-entry inclusion
- **Boolean filter**: A bitset of allowed entry indices for pre-computed filters

### Proposed API

```rust
// fib-quant/src/sidecar.rs (extensions)

/// A filter predicate for sidecar search.
pub trait SearchFilter: Send + Sync {
    /// Return true if the entry at `idx` with `id` should be scored.
    fn includes(&self, idx: usize, id: &Id) -> bool;
}

/// A bitset filter for pre-computed allowed entry indices.
pub struct BitsetFilter {
    allowed: Vec<bool>,
}

impl BitsetFilter {
    pub fn new(len: usize) -> Self;
    pub fn allow(&mut self, idx: usize);
    pub fn deny(&mut self, idx: usize);
}

impl<Id> FibSidecarIndex<Id> {
    /// Search with a pre-filter. Only entries passing the filter
    /// are scored.
    pub fn search_filtered(
        &self,
        query: &[f32],
        top_k: usize,
        oversample: usize,
        filter: &dyn SearchFilter<Id>,
    ) -> Result<Vec<ScoredCandidate<Id>>>;
}
```

---

## 7. IVF / Coarse-to-Fine Search (MEDIUM)

### What's missing

fib-quant could serve as a coarse quantizer in an IVF-style architecture:
1. Cluster vectors into `nlist` coarse clusters (using FibQuant's codebook)
2. At query time, find the nearest `nprobe` clusters
3. Score only vectors in those clusters with the Gram table
4. Rerank with exact f32

This would give O(nprobe * nlist_avg) instead of O(n) search.

### Proposed API

```rust
// fib-quant/src/ivf.rs (new module)

/// IVF index using FibQuant as the coarse quantizer.
pub struct FibIvfIndex<Id>
where
    Id: Clone + Eq + std::fmt::Debug,
{
    scorer: FibScorer,
    /// Coarse cluster centroids (encoded as FibCodeV1)
    centroids: Vec<FibCodeV1>,
    /// Inverted lists: cluster_idx → Vec<(Id, FibCodeV1)>
    inverted_lists: Vec<Vec<(Id, FibCodeV1)>>,
    nlist: usize,
    nprobe: usize,
}

impl<Id> FibIvfIndex<Id> {
    pub fn new(scorer: FibScorer, nlist: usize, nprobe: usize) -> Result<Self>;

    /// Train the coarse quantizer on a sample of vectors.
    pub fn train(&mut self, sample: &[&[f32]]) -> Result<()>;

    /// Insert a vector into the IVF index.
    pub fn insert(&mut self, id: Id, code: FibCodeV1) -> Result<()>;

    /// Search: find nprobe nearest clusters, score only their members.
    pub fn search(
        &self,
        query: &[f32],
        top_k: usize,
        oversample: usize,
    ) -> Result<Vec<ScoredCandidate<Id>>>;
}
```

---

## 8. Recall Guarantees with Confidence Intervals (MEDIUM)

### What fib-quant has

`eval.rs` has `recall_at_k`, `ndcg_at_k`, and `run_benchmark` which compute
point estimates of recall and nDCG on a corpus. Good for evaluation but not
for runtime recall guarantees.

### What's missing

- **Per-query recall estimate**: After a search, estimate the probability that
  the true top-K is in the returned candidates (based on score distribution)
- **Confidence intervals**: Bootstrap or analytical bounds on recall
- **Adaptive oversampling**: Increase oversample when confidence is low

### Proposed API

```rust
// fib-quant/src/sidecar.rs (extensions)

pub struct SearchQualityEstimate {
    /// Estimated recall@k for this query (0.0-1.0)
    pub estimated_recall: f32,
    /// Confidence interval (lower, upper) at 95%
    pub recall_ci_95: (f32, f32),
    /// Whether the result meets a recall target
    pub meets_target: bool,
    /// Recommended oversample if recall is insufficient
    pub recommended_oversample: usize,
}

impl<Id> FibSidecarIndex<Id> {
    /// Search with quality estimation. Returns candidates plus
    /// a quality estimate for this specific query.
    pub fn search_with_quality(
        &self,
        query: &[f32],
        top_k: usize,
        oversample: usize,
        recall_target: f32,
    ) -> Result<(Vec<ScoredCandidate<Id>>, SearchQualityEstimate)>;
}
```

---

## 9. SQLite Integration / VectorCodec Trait Impl for semantic-memory (HIGH)

### What semantic-memory has

`vector_codec.rs` defines:
```rust
pub trait VectorCodec: Send + Sync {
    fn profile(&self) -> &VectorCodecProfileV1;
    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError>;
    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError>;
}
```

`TurboQuantCodec` implements this trait AND provides:
- `score_inner_product(artifact, query)` — approximate scoring from compressed bytes
- `prepare_query(query)` / `score_inner_product_prepared(artifact, prepared)` — batch scoring
- `score_l2(artifact, query)` — L2 distance

`VectorArtifactV1` carries `profile: VectorCodecProfileV1` + `encoded: Vec<u8>` with
digest validation. This is the persistence format stored in SQLite BLOB columns.

### What fib-quant has

`compat.rs` implements `quant_codec_core::VectorCodec` trait:
```rust
impl VectorCodec for FibQuantizer {
    type EncodedBlock = FibCodeV1;
    fn encode_block(&self, input: &[f32]) -> Result<FibCodeV1, QuantCodecError>;
    fn decode_block(&self, block: &FibCodeV1, out: &mut [f32]) -> Result<(), QuantCodecError>;
}
```

But this is the **wrong trait** for semantic-memory integration. semantic-memory
uses its own `VectorCodec` trait (different signature, uses `VectorArtifactV1` with
byte-level encoding, not structured `FibCodeV1`).

### What's missing

- **`semantic_memory::vector_codec::VectorCodec` impl for fib-quant**: A codec
  adapter that wraps `FibQuantizer` and produces `VectorArtifactV1` with
  `codec: "fib_quant"` profile
- **Approximate scoring from `VectorArtifactV1`**: Like `TurboQuantCodec::score_inner_product`
  but for fib-quant — decode the `FibCodeV1` from the artifact bytes, use the Gram
  table for scoring
- **Profile construction**: `VectorCodecProfileV1::fib_quant(dim, k, N, seed)` constructor

### Proposed API

```rust
// semantic-memory integration — could live in semantic-memory/src/vector_codec.rs
// behind a `fib-quant-codec` feature, mirroring the turbo-quant pattern

/// Optional FibQuant codec backend.
#[cfg(feature = "fib-quant-codec")]
#[derive(Debug, Clone)]
pub struct FibQuantCodec {
    profile: VectorCodecProfileV1,
    quantizer: fib_quant::FibQuantizer,
    scorer: fib_quant::FibScorer,
}

#[cfg(feature = "fib-quant-codec")]
impl FibQuantCodec {
    pub fn new(
        dim: usize,
        block_dim: usize,
        codebook_size: usize,
        seed: u64,
    ) -> Result<Self, MemoryError>;

    /// Estimate inner product from a FibQuant artifact.
    pub fn score_inner_product(
        &self,
        artifact: &VectorArtifactV1,
        query: &[f32],
    ) -> Result<f32, MemoryError>;

    /// Prepare a query for batch scoring.
    pub fn prepare_query(
        &self,
        query: &[f32],
    ) -> Result<fib_quant::FibPreparedQuery, MemoryError>;

    /// Batch score using a prepared query.
    pub fn score_inner_product_prepared(
        &self,
        artifact: &VectorArtifactV1,
        prepared: &fib_quant::FibPreparedQuery,
    ) -> Result<f32, MemoryError>;
}

#[cfg(feature = "fib-quant-codec")]
impl VectorCodec for FibQuantCodec {
    fn profile(&self) -> &VectorCodecProfileV1 { &self.profile }
    fn encode(&self, vector: &[f32]) -> Result<VectorArtifactV1, MemoryError>;
    fn decode(&self, artifact: &VectorArtifactV1) -> Result<Vec<f32>, MemoryError>;
}
```

And in `VectorCodecProfileV1`:
```rust
impl VectorCodecProfileV1 {
    #[cfg(feature = "fib-quant-codec")]
    pub fn fib_quant(
        dim: usize, block_dim: usize, codebook_size: usize, seed: u64,
    ) -> Result<Self, MemoryError> {
        Ok(Self {
            schema_version: PROFILE_SCHEMA_V1.into(),
            codec: "fib_quant".into(),
            dim: dim_u32(dim)?,
            bits: wire_index_bits(codebook_size) as u8,
            projections: None,
            seed: Some(seed),
            codec_version: "fib-quant:0.1.0-alpha.2".into(),
            scoring_semantics: "inner_product_estimate".into(),
            normalization: "caller_supplied".into(),
        })
    }
}
```

---

## Summary: Missing Features Ranked by Priority

| # | Feature | Priority | Effort | Impact |
|---|---------|----------|--------|--------|
| 1 | HNSW integration (compressed graph nodes) | CRITICAL | Large | O(log n) vs O(n) search |
| 2 | Index persistence + mmap | CRITICAL | Medium | No index = no production use |
| 9 | semantic-memory VectorCodec impl | HIGH | Small | Direct integration path |
| 4 | Update/delete/compaction | HIGH | Medium | Mutability required for DB |
| 5 | L2/cosine distance metrics | HIGH | Small | Most DBs need L2 |
| 3 | Batch ingest API (100K+) | HIGH | Medium | Scale requirement |
| 6 | Filter-then-search | MEDIUM | Small | Pre-filter before scoring |
| 7 | IVF coarse-to-fine | MEDIUM | Large | Alternative to HNSW |
| 8 | Recall confidence intervals | MEDIUM | Medium | Quality guarantees |

## Key Files Referenced

- `fib-quant/src/sidecar.rs` — Linear-scan sidecar index (no persistence, no delete)
- `fib-quant/src/scoring.rs` — Gram-table inner product scorer (IP only, no L2/cosine)
- `fib-quant/src/codec.rs` — FibQuantizer with encode_batch (good batch encode)
- `fib-quant/src/compat.rs` — quant-codec-core trait impl (wrong trait for semantic-memory)
- `fib-quant/src/wire.rs` — Self-describing wire format (good per-code, no index-level)
- `fib-quant/src/residual.rs` — Two-level quantization (no scoring integration)
- `fib-quant/src/eval.rs` — Benchmark harness (recall/nDCG, no runtime guarantees)
- `semantic-memory/src/vector_codec.rs` — VectorCodec trait + TurboQuantCodec adapter
- `semantic-memory/src/vector_backend.rs` — VectorBackend trait (insert/delete/search/save)
- `semantic-memory/src/hnsw.rs` — HNSW with sidecar persistence, keymap, compaction
- `semantic-memory/src/quantize_governed.rs` — Governance pipeline (codec-agnostic)
- `turbo-quant/src/index.rs` — TurboSidecarIndex (also linear, but has source_digest, byte accounting)
- `turbo-quant/src/kv.rs` — KV compressor with shadow mode, attention scores

## What fib-quant Already Does Well

- **Codec math**: Encode/decode, codebook, rotation, Lloyd-Max refinement — solid
- **Compact wire format**: `to_compact_bytes()` / `to_compact_v2_bytes()` with feature flags
- **Self-describing wire**: `FibCodeWireV1` carries profile metadata for standalone decode
- **Batch encode**: `encode_batch` with Rayon parallelism and GPU fallback
- **Approximate scoring**: Gram-table IP estimation without full decode
- **Prepared query**: `FibPreparedQuery` avoids redundant rotation/argmin per code
- **Residual quantization**: Two-level encoding for better fidelity
- **Benchmark harness**: `run_benchmark` with recall@K, nDCG@K, compression ratio
- **Digests**: Profile, codebook, rotation digests for integrity checking