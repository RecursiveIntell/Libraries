[Watch](https://x.com/JSense/status/2033582183600791991?s=20)

# semantic-memory

Hybrid semantic search with SQLite, FTS5, and HNSW — built for AI agents.

## Overview

`semantic-memory` is a local-first semantic memory store backed by authoritative SQLite state with an optional HNSW sidecar for approximate nearest-neighbor acceleration. It stores facts, chunked documents, conversation messages, and searchable episodes, combining BM25 (FTS5) and vector retrieval via Reciprocal Rank Fusion.

## Features

- **Hybrid search** — BM25 full-text search (FTS5) fused with vector similarity via Reciprocal Rank Fusion
- **Five search modes** — `search()`, `search_explained()`, `search_conversations()`, `search_fts_only()`, `search_vector_only()`
- **HNSW sidecar** — optional approximate nearest-neighbor index; journaled in SQLite so sidecar failures never lose writes
- **Brute-force backend** — exact cosine similarity search when HNSW is not needed
- **Graph view** — deterministic traversal over namespaces, facts, documents, chunks, sessions, messages, episodes, and semantic/temporal/causal/entity links
- **Projection import pipeline** — canonical `ProjectionImportBatchV3` lane for ingesting verified data from `forge-memory-bridge`, with queryable claim versions, relation versions, episodes, entity aliases, and evidence references
- **Episode system** — episodes with causal edges (`episode_causes` table), outcomes (Confirmed/Refuted/Inconclusive/Pending), confidence scoring, and verification status
- **Explained search** — `search_explained()` returns exact scoring breakdowns (BM25 score, vector similarity, RRF combined) from the live pipeline
- **Integrity verification** — `verify_integrity()` surfaces invalid roles, JSON, enums, embedding blobs, quantized blobs, and sidecar drift
- **Reconciliation** — `reconcile()` rebuilds FTS or fully re-embeds derived state from SQLite
- **Concurrent reads** — WAL mode with pooled reader connections; writes serialize through a single writer connection
- **Quantized vectors** — configurable vector quantization for reduced storage

## Quick start

```rust
use semantic_memory::{MemoryConfig, MemoryStore};

#[tokio::main]
async fn main() -> Result<(), semantic_memory::MemoryError> {
    let store = MemoryStore::open(MemoryConfig::default())?;

    // Store a fact
    store.add_fact("general", "Rust was first released in 2015", None, None).await?;

    // Search
    let results = store.search("when was Rust released", None, None, None).await?;
    println!("{:?}", results);

    Ok(())
}
```

## API surface

### Facts

| Method | Description |
|--------|-------------|
| `add_fact()` | Store a fact with optional source and metadata |
| `update_fact()` | Update content and re-embed |
| `delete_fact()` / `delete_namespace()` | Delete by ID or namespace |
| `get_fact()` / `list_facts()` | Retrieve facts |
| `get_fact_embedding()` | Get a fact's embedding vector |

### Documents

| Method | Description |
|--------|-------------|
| `ingest_document()` | Chunk and store a document |
| `delete_document()` | Delete document and its chunks |
| `list_documents()` | List paginated |
| `count_chunks_for_document()` | Count chunks for a document |

### Conversations

| Method | Description |
|--------|-------------|
| `create_session()` | Create a conversation session |
| `add_message()` / `add_message_fts()` / `add_message_embedded()` | Add messages (FTS-only, embedded, or both) |
| `get_recent_messages()` | Fetch recent messages |
| `get_messages_within_budget()` | Fetch messages within a token budget |
| `session_token_count()` | Count tokens in a session |
| `search_conversations()` | Hybrid search over messages |

### Search

| Method | Description |
|--------|-------------|
| `search()` | Hybrid BM25 + vector search with Reciprocal Rank Fusion |
| `search_explained()` | Same pipeline, returns `ScoreBreakdown` per result |
| `search_conversations()` | Hybrid search restricted to messages |
| `search_fts_only()` | BM25 full-text search only (no embedding needed) |
| `search_vector_only()` | Vector similarity only |

`SearchSourceType` controls which data is searched: `Facts`, `Documents`, `Episodes`, `Messages`.

### Projection imports

| Method | Description |
|--------|-------------|
| `import_projection_batch()` | Canonical atomic import of `ProjectionImportBatchV3` batches |
| `query_projection_imports()` | Query import log |
| `query_projection_import_failures()` | Query failed imports |
| `query_claim_versions()` | Query imported claims with temporal intervals |
| `query_relation_versions()` | Query imported relations with validity windows |
| `query_episodes()` | Query imported episodes |
| `query_entity_aliases()` | Query canonical entity names and aliases |
| `query_evidence_refs()` | Query evidence bundle references |

### Graph

`store.graph_view()` returns a `GraphView` with:

- `neighbors(node_id, direction, max_depth)` — find neighboring nodes up to N hops
- `path(from, to, max_depth)` — BFS shortest path between two nodes

Node IDs follow the pattern `type:id` (e.g. `fact:abc`, `episode:xyz`, `namespace:general`). Edge types: `Semantic`, `Temporal`, `Causal`, `Entity`.

### Utilities

| Method | Description |
|--------|-------------|
| `embed()` / `embed_batch()` | Embed text |
| `chunk_text()` | Chunk text using configured strategy |
| `embedding_displacement()` | Compute vector distance between texts |
| `stats()` | Database statistics |
| `verify_integrity()` | Validate all stored data |
| `reconcile()` | Rebuild FTS or re-embed everything |
| `reembed_all()` | Re-embed all facts, chunks, messages, and episodes |
| `vacuum()` | Reclaim database space |
| `rebuild_hnsw_index()` / `flush_hnsw()` / `compact_hnsw()` | HNSW sidecar maintenance |

## Feature flags

| Flag | Default | Description |
|------|---------|-------------|
| `hnsw` | yes | HNSW approximate nearest-neighbor search via `hnsw_rs` |
| `brute-force` | no | Exact cosine similarity search (no external index) |
| `testing` | no | Expose test-only helpers |

At least one search backend (`hnsw` or `brute-force`) must be enabled.

## Architecture

- **SQLite is authoritative** for all durable records and embeddings
- **HNSW is an acceleration sidecar** — pending mutations are journaled in SQLite and replayed on open, flush, rebuild, or reconcile
- **Schema version**: V17 (V1–V9 core schema, V10 legacy projection import, V11–V17 canonical projection storage)
- **Episode identity**: episodes are canonically identified by `episode_id` (TEXT PK); `document_id` is a non-unique FK — one document can have many episodes
- **Projection plane** — this crate sits on the projection plane of the stack; `semantic-memory-forge` owns raw verification truth, `forge-memory-bridge` owns transformation, and this crate owns queryable projected truth

## Ecosystem

**Depends on:**
- `stack-ids` -- canonical cross-crate IDs, trace context, and digest primitives
- `forge-memory-bridge` -- projection import batch schemas and transformation

**Depended on by:**
- `knowledge-runtime`
- `forge-pilot`
- `forge-engine` (living-memory)
- `kernel-conformance`
- `contract-schema-gen`

## stack-ids integration

Uses `ScopeKey`, `TraceCtx`, `ContentDigest`, `EnvelopeId`, `ClaimId`,
`ClaimVersionId`, `EntityId`, `EpisodeId`, `RelationId`, and `ImportBatchId`
from `stack-ids` for scoped storage, projection imports, and content addressing.

## License

MIT
