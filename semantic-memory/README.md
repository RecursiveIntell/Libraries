# semantic-memory

[![crates.io](https://img.shields.io/crates/v/semantic-memory?style=flat-square&color=6c5ce7)](https://crates.io/crates/semantic-memory)
[![docs.rs](https://img.shields.io/docsrs/semantic-memory?style=flat-square&color=74b9ff)](https://docs.rs/semantic-memory)
[![license](https://img.shields.io/badge/license-Apache--2.0-00b894?style=flat-square)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-f76707?style=flat-square)](https://www.rust-lang.org/)

**Local-first semantic memory engine — hybrid search, knowledge graphs, trust ledgers, and bitemporal provenance for AI agents.**

`semantic-memory` is the core Rust library that powers the [semantic-memory-mcp](https://crates.io/crates/semantic-memory-mcp) MCP server and the [mnemes](https://crates.io/crates/mnemes) multi-device control plane. It can also be used directly as a standalone embedded memory engine in any Rust application.

---

## Table of Contents

- [Quick Start](#quick-start)
- [Architecture](#architecture)
- [Installation & Features](#installation--features)
- [Core API](#core-api)
  - [Store & Facts](#store--facts)
  - [Search & Retrieval](#search--retrieval)
  - [Knowledge Graph](#knowledge-graph)
  - [Trust & Provenance](#trust--provenance)
  - [Lifecycle & Maintenance](#lifecycle--maintenance)
- [Configuration](#configuration)
- [Stack & Dependencies](#stack--dependencies)
- [Module Map](#module-map)

---

## Quick Start

```rust
use semantic_memory::{MemoryConfig, MemoryStore};

#[tokio::main]
async fn main() -> Result<(), semantic_memory::MemoryError> {
    // Open a store (creates memory.db + indexes in the config directory)
    let store = MemoryStore::open(MemoryConfig::default())?;

    // Store a fact — automatically embedded and indexed
    store.add_fact("general", "Rust was first released in 2015", None, None).await?;

    // Hybrid search: BM25 + vector + RRF fusion
    let results = store.search("when was Rust released", None, None, None).await?;
    for r in &results {
        println!("[{:.4}] {}", r.score, r.content);
    }

    // Search with explanation — exact scoring breakdown
    let explained = store.search_explained("when was Rust released", None, None, None).await?;

    Ok(())
}
```

### Without a GPU (CPU-only)

```rust
let config = MemoryConfig {
    base_dir: std::path::PathBuf::from("./my_memory"),
    embedding: EmbeddingConfig {
        dimensions: 768,
        model: "nomic-embed-text".into(),
        ..Default::default()
    },
    ..Default::default()
};
let store = MemoryStore::open(config)?;
```

---

## Architecture

```
┌─────────────────────────────────────────────────────────┐
│                   MemoryStore                            │
│                                                          │
│  ┌──────────────┐  ┌──────────────┐  ┌───────────────┐  │
│  │   SQLite     │  │  HNSW /      │  │  Knowledge    │  │
│  │   + FTS5     │  │  Brute-force │  │  Graph        │  │
│  │   (WAL mode) │  │  Vector      │  │  Typed edges  │  │
│  └──────┬───────┘  └──────┬───────┘  └───────┬───────┘  │
│         │                 │                   │          │
│         └─────────────────┼───────────────────┘          │
│                           │                              │
│  ┌────────────────────────▼───────────────────────────┐ │
│  │  Retrieval Pipeline                                 │ │
│  │  BM25(FTS5) + Vector(HNSW) → RRF Fusion → Rerank   │ │
│  │  + Adaptive RL Routing + Sparse (V36 dormant)       │ │
│  └────────────────────────┬───────────────────────────┘ │
│                           │                              │
│  ┌────────────────────────▼───────────────────────────┐ │
│  │  Trust & Provenance Layer                           │ │
│  │  Claims · Evidence · Bitemporal · Governed Access   │ │
│  └────────────────────────────────────────────────────┘ │
└─────────────────────────────────────────────────────────┘
```

### Design Principles

| Principle | Implementation |
|-----------|---------------|
| **SQLite is authoritative** | All durable records and embeddings live in SQLite. HNSW is an acceleration sidecar, journaled in SQLite and replayed on open. |
| **WAL concurrency** | One writer connection + pooled WAL reader connections. Reads never block writes. |
| **Strict integrity** | Malformed data (invalid roles, JSON, enums, embedding blobs) is surfaced through `verify_integrity()` — never silently converted to defaults. |
| **Append-only evolution** | Facts evolve through supersession, not deletion. The `supersedes` graph edge preserves audit trails. |
| **Bitemporal versioning** | Every fact carries `valid_time` (when true) and `transaction_time` (when recorded). |

---

## Installation & Features

```toml
[dependencies]
semantic-memory = "0.6"
```

### Feature Flags

| Feature | Default | Description |
|---------|---------|-------------|
| `hnsw` | **yes** | HNSW approximate nearest neighbor search (usearch backend) |
| `brute-force` | no | Exact brute-force vector search (no index) |
| `testing` | no | Test-only utilities and mock embedders |
| `turbo-quant-codec` | no | Fibonacci quantization: 8-16× compression with exact f32 rerank |
| `poly-kv-codec` | no | PolyKV compressed-domain retrieval backend |

At least one of `hnsw` or `brute-force` must be enabled. The default build uses HNSW with the usearch backend.

```bash
# Default: HNSW
cargo add semantic-memory

# Brute-force only (no HNSW dependency)
cargo add semantic-memory --no-default-features --features brute-force

# With compression
cargo add semantic-memory --features turbo-quant-codec
```

---

## Core API

### Store & Facts

```rust
use semantic_memory::{MemoryConfig, MemoryStore, MemoryKind, Sensitivity};

let store = MemoryStore::open(MemoryConfig::default())?;

// Add a fact with full metadata
store.add_fact_with_options(
    "research",                                    // namespace
    "HNSW achieves O(log N) query complexity",    // content
    Some("arxiv:1603.09320"),                      // source
    Some(MemoryKind::DurableFact),                 // memory kind
    Some(Sensitivity::Internal),                   // sensitivity
    None,                                          // idempotency key
).await?;

// Get a fact by ID
let fact = store.get_fact(&fact_id).await?;

// Update in-place (re-embeds, updates FTS)
store.update_fact(&fact_id, "Updated content").await?;

// Supersede — creates replacement, links old→new, auto-filters old from search
store.supersede_fact(&old_id, "Corrected content", Some("correction"), None).await?;

// List facts in a namespace (paginated)
let facts = store.list_facts("research", 50, 0).await?;

// List all namespaces
let namespaces = store.list_namespaces().await?;

// Ingest a document (auto-chunked, each chunk embedded + indexed)
store.ingest_document("My Research Notes", "research", &long_text).await?;
```

#### Memory Kinds

| Kind | Persistence | Description |
|------|------------|-------------|
| `DurableFact` | Permanent | Default — general knowledge |
| `Preference` | Durable | User/agent preferences |
| `ProjectState` | Durable | Project-specific state |
| `InstructionPolicy` | Durable | Doctrine/rule encoding |
| `Correction` | Durable | Error corrections |
| `Observation` | Durable | Recorded observations |
| `EpisodeSummary` | Durable | Summarized past sessions |
| `SkillProcedure` | Durable | Procedural knowledge |
| `EphemeralInference` | **Transient** | Requires evidence refs to promote |

#### Sensitivity Classes

| Class | Autocapture | Search Visibility |
|-------|------------|-------------------|
| `Public` | ✓ | Unrestricted |
| `Internal` (default) | ✓ | Namespace-scoped |
| `Confidential` | **Blocked** | Governed access |
| `Restricted` | **Blocked** | Governed access |

### Search & Retrieval

```rust
// Standard hybrid search (BM25 + vector + RRF)
let results = store.search(
    "quantum computing error correction",  // query
    Some(5),                                // top_k
    Some(vec!["research".into()]),          // namespace filter
    None,                                   // source type filter
).await?;

// Search with full scoring breakdown
let explained = store.search_explained("query", Some(5), None, None).await?;
// Returns per-result BM25 score, vector score, RRF rank, and final score

// Witnessed search — cache-bypassed, durable receipt, required for autonomous agents
let witnessed = store.search_witnessed(
    "query", 5, None, None, RetrievalMode::Hybrid, ReplayMode::NoReplay, None
).await?;

// Bitemporal search — "what did we know as of date X?"
let historical = store.search_as_of(
    "query", "2024-01-15T00:00:00Z", Some("research"), 5
).await?;

// Search conversations
let messages = store.search_conversations("deployment discussion", 5).await?;

// Adaptive search with RL routing
let adaptive = store.search_with_routing("query", 5, None, None, false).await?;

// Proof-debt gated search
let gated = store.search_proof_debt("query", 5, None, Some(500_000)).await?;
```

#### Retrieval Pipeline

```
Query → Profile (RL routing) → Parallel: [BM25(FTS5) | Vector(HNSW) | Graph]
                                  → RRF Fusion → Rerank → Results
```

The pipeline fuses BM25 lexical scores with HNSW vector similarity using Reciprocal Rank Fusion. When `search_explained()` is called, the exact per-stage breakdown is returned. V36 sparse retrieval is inherited but dormant by default (`sparse_weight = 0` in `SearchConfig`).

### Knowledge Graph

```rust
// Access the graph view
let graph = store.graph_view();

// Add typed edges
graph.add_semantic_edge(&fact_a, &fact_b, 0.85).await?;     // Similarity
graph.add_temporal_edge(&fact_a, &fact_b, 3600).await?;     // A preceded B by 1hr
graph.add_causal_edge(&fact_a, &fact_b, 0.9, &[&ev]).await?; // A caused B
graph.add_entity_edge(&fact_a, &fact_b, "cites").await?;     // Named relation

// Traverse
let path = graph.shortest_path(&start_id, &end_id, Some(5)).await?;
let neighbors = graph.get_neighbors(&fact_id).await?;

// Community detection (Leiden-inspired)
let communities = graph.detect_communities(1.0, None).await?;

// Topological analysis (Betti numbers)
let topology = graph.analyze_topology().await?;

// Factor graph belief propagation
let beliefs = graph.run_factor_graph(&nodes, config).await?;

// Contradiction detection (content-based — no pre-asserted edges needed)
let conflicts = graph.detect_contradictions("query", 10).await?;
```

#### Edge Types

| Type | Key Field | Semantics |
|------|-----------|-----------|
| **Semantic** | `cosine_similarity` (0.0–1.0) | Semantic relationship strength |
| **Temporal** | `delta_secs` (u64) | Time delta between facts |
| **Causal** | `confidence` (0.0–1.0) + evidence IDs | Causal relationship |
| **Entity** | `relation` (string) | Named relationship |

Edges are **never deleted** — they are invalidated via `invalidate_edge(edge_id, reason)`, preserving audit trails.

### Trust & Provenance

```rust
// Create a claim from a stored fact
let claim = store.create_claim(&fact_id, Some("paragraph 3")).await?;

// Add evidence supporting a claim
store.add_evidence(&claim_id, "Source text confirming the claim", SourceType::Document).await?;

// Judge support state
store.judge_support(&claim_id, SupportJudgment::Supported, Some("Verified against source")).await?;

// Verify a claim by risk class
let disposition = store.verify_claim(
    "Claim text", RiskClass::High, &[&evidence_id], true // refutation attempted
).await?;

// Set provenance confidence on any item
store.set_provenance(&item_id, 0.95, 5).await?; // confidence, support_count

// Governed access decisions
let assertion_auth = store.decide_assertion_authority(
    &fact_id, "agent:hermes", "user:josh", &["human:josh"], &scope
).await?;
```

#### Claim State Machine

```
draft → supported → contested → retracted
```

#### Verification by Risk Class

| Risk Class | Requirements | Disposition |
|------------|-------------|-------------|
| **Low** | Cheap integrity checks | Auto-promote |
| **Medium** | + metadata validation | Promote with caveats |
| **High** | Falsification attempt required | Promote only if survives refutation |
| **Critical** | Replay + falsification required | Quarantine if either fails |

### Lifecycle & Maintenance

```rust
// Run lifecycle analysis — syndrome detection, subtraction candidates
store.run_lifecycle(&item_ids).await?;

// Integrity check
store.verify_integrity().await?;

// Reconcile: rebuild FTS indexes from source data
store.reconcile(ReconcileAction::RebuildFts).await?;

// Vacuum: compact SQLite after large deletes
store.vacuum().await?;

// Check if embeddings are stale (model changed since last embed)
if store.embeddings_are_dirty().await? {
    store.reembed_all().await?; // ~138ms/fact on CPU
}

// Stats
let stats = store.stats().await?;
// Returns: fact count, chunk count, document count, DB size, embedding model, dimensions
```

#### Claim Ledger Compaction

```rust
// Preview compaction (dry run)
store.compact_claim_ledger(true, None, None, None, None).await?;

// Execute compaction with custom thresholds
store.compact_claim_ledger(
    false,       // dry_run
    Some(10000), // max_entries
    Some(16_777_216), // max_bytes (16 MiB)
    Some(256),   // retain_tail_entries
    Some(3),     // max_backups
).await?;
```

Compaction produces a hash-chained, digest-verified snapshot + retained tail. The manifest rename is the atomic publication boundary.

---

## Configuration

```rust
use semantic_memory::{MemoryConfig, EmbeddingConfig, SearchConfig};

let config = MemoryConfig {
    // Where memory.db and sidecars live
    base_dir: std::path::PathBuf::from("./memory_data"),

    embedding: EmbeddingConfig {
        model: "nomic-embed-text".into(),
        dimensions: 768,
        batch_size: 32,          // chunks per embed call
        timeout_secs: 30,        // embedder HTTP timeout
        ..Default::default()
    },

    search: SearchConfig {
        top_k: 10,               // default result count
        bm25_weight: 1.0,        // BM25 contribution to RRF
        vector_weight: 1.0,      // vector contribution to RRF
        sparse_weight: 0.0,      // sparse retrieval (V36 dormant)
        rerank_enabled: false,   // exact f32 cosine rerank
        ..Default::default()
    },

    // Embedder provider
    embedder: EmbedderKind::Candle, // or EmbedderKind::Ollama(url)

    ..Default::default()
};

let store = MemoryStore::open(config)?;
```

---

## Stack & Dependencies

`semantic-memory` is the **foundation crate** — nothing above it is required.

```
┌──────────────────────────────────────────────┐
│                  mnemes                       │  ← Multi-device control plane
│         (device identity, routing,            │     (optional — not required)
│          replication, pooled memory)          │
├──────────────────────────────────────────────┤
│             semantic-memory-mcp               │  ← MCP server for AI agents
│       (stdio JSON-RPC, tool profiles,         │     (optional — not required)
│        HTTP sidecar, agent integrations)      │
├──────────────────────────────────────────────┤
│            semantic-memory ← YOU ARE HERE     │  ← Core engine
│    (SQLite store, HNSW vectors, FTS5 search,  │     Standalone — use directly
│     knowledge graph, trust ledger, lifecycle) │     in any Rust application
└──────────────────────────────────────────────┘
```

Use `semantic-memory` directly when you want:
- An embedded memory engine in a Rust application
- Full control over storage, search, and graph APIs
- No MCP or HTTP transport layer

Use `semantic-memory-mcp` when you want:
- AI agents (Hermes, Claude, Codex) to access memory via MCP tools
- Tool profile filtering for safety
- Witnessed retrieval and governed access

Use `mnemes` when you want:
- Multi-device memory sharing with device identity
- Sparse shard routing across device stores
- Operation envelopes with idempotency and replication

---

## Module Map

```
src/
├── lib.rs                  # Crate root, re-exports, quick-start docs
├── config.rs               # MemoryConfig, EmbeddingConfig, SearchConfig
├── db.rs                   # SQLite connection, migrations, pragmas
├── error.rs                # MemoryError type hierarchy
├── types.rs                # Shared types: IDs, enums, metadata
│
├── storage.rs              # Core CRUD: facts, documents, conversations
├── search.rs               # Hybrid search: BM25 + vector + RRF
├── routing.rs              # Adaptive RL routing, query profiling
├── rl_routing.rs           # RL policy persistence and training
│
├── embedder.rs             # Embedder trait, Ollama/Mock providers
├── chunker.rs              # Recursive text chunking with overlap
├── tokenizer.rs            # Token counting trait + estimator
│
├── hnsw.rs                 # HNSW index management (feature-gated)
├── hnsw_backend.rs         # HNSW backend abstraction
├── hnsw_ops.rs             # HNSW build/compact/rebuild operations
├── usearch_backend.rs      # usearch vector backend
├── vector_backend.rs       # Vector backend trait
├── vector_codec.rs         # Vector compression/decompression
├── brute_force.rs          # Exact brute-force search (feature-gated)
│
├── graph.rs                # Knowledge graph: nodes, edges, traversal
├── graph_edges.rs          # Typed edge storage and queries
├── community.rs            # Leiden-inspired community detection
├── topology.rs             # Betti numbers, structural voids
├── factor_graph.rs         # Belief propagation over typed edges
├── decoder.rs              # Contradiction decoding, belief refinement
├── discord.rs              # Second-order graph search
│
├── provenance.rs           # Provenance tracking and confidence
├── temporal.rs             # Bitemporal versioning (valid_time + txn_time)
├── authority.rs            # Governed access: assertion/action decisions
├── authority_contracts.rs  # Authority contract types
├── origin_authority.rs     # Origin authority verification
│
├── subtraction.rs          # Lifecycle: subtraction candidates
├── subgraph_pruning.rs     # Access-frequency-based pruning
├── compression_governor.rs # Compression lifecycle governor
├── quantize.rs             # Vector quantization
├── quantize_governed.rs    # Governed quantization pipeline
├── pipeline.rs             # Search pipeline orchestration
│
├── projection_import.rs    # Bulk import with provenance
├── projection_batch.rs     # Batch projection types
├── projection_storage.rs   # Projection storage queries
├── projection_lane.rs      # Compatibility migration lanes
│
├── journal.rs              # Write-ahead journal for replication
├── benchmark.rs            # Retrieval benchmarking harness
├── hostile_benchmark.rs    # Hostile/edge-case benchmark suite
├── eval_contradiction.rs   # Contradiction evaluation
├── evidence_gap.rs         # Evidence gap detection
├── hubness.rs              # Hubness analysis for embeddings
├── matryoshka.rs           # Matryoshka embedding support
├── late_interaction.rs     # Late interaction (ColBERT-style)
├── state_epistemics.rs     # Epistemic state modeling
├── shadow_policy.rs        # Shadow policy evaluation
├── procedural_memory.rs    # Procedural memory storage
└── poly_kv_bridge.rs       # PolyKV codec bridge (feature-gated)
```

---

## License

Apache-2.0. See [LICENSE](LICENSE).

---

<p align="center">
  <em>Built with Rust · SQLite · HNSW · FTS5 · Ollama</em>
</p>
