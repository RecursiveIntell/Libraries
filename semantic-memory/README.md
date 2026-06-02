# semantic-memory

Local-first semantic search backed by authoritative SQLite state
and a high-performance vector sidecar.

`semantic-memory` stores facts, chunked documents, conversation
messages, and searchable episodes in SQLite. Search combines
**BM25 (FTS5)** and **vector retrieval** with **Reciprocal Rank
Fusion**, and `search_explained()` returns the exact scoring
breakdown from the live pipeline.

The vector sidecar is **usearch 2.25 by default** (formerly
`hnsw_rs 0.3`). The migration was a hard win — see the
**HNSW → usearch migration** section below for the measured
results.

## Why this crate

Most "vector databases" treat the index as authoritative and
the metadata as a side table. `semantic-memory` is the
opposite: **SQLite is authoritative** for all durable state
(records, embeddings, content, conversations, links). The
vector index is an **acceleration sidecar** that can be
rebuilt from SQLite at any time. If the sidecar corrupts, you
call `reconcile()` and you have a fresh index from
authoritative state.

This makes the crate suitable for local-first AI systems that
need:

- **Durable, recoverable state** — SQLite + WAL.
- **Fast vector search** — usearch 2.25 (or hnsw_rs 0.3 as
  opt-in fallback).
- **Hybrid search** — BM25 + vector + RRF, with the breakdown
  exposed via `search_explained()`.
- **Graph view** — `store.graph_view()` exposes a
  deterministic traversal over namespaces, facts,
  documents, chunks, sessions, messages, episodes, and
  semantic/temporal/causal links.
- **Bitemporal truth** — every fact carries a `valid_time`
  and `recorded_time` via the `bitemporal-runtime` foundation.
- **Receipt-bearing operations** — every state transition
  emits a typed receipt.

## HNSW → usearch migration (June 2026)

The vector sidecar was migrated from `hnsw_rs 0.3` to
`usearch 2.25` based on a head-to-head benchmark on
**2026-06-02** (`HNSW_BENCH_RESULTS_2026-06-02.md`).

### Headline @ D=768 (bge-m3 default — the production case)

| Metric | hnsw_rs 0.3 | usearch 2.25 | usearch advantage |
|---|---:|---:|---:|
| **Insert throughput** | 265 vec/s | 770 vec/s | **2.9×** |
| **Search p50** | 9,992 µs | 529 µs | **18.9×** |
| **Search p99** | 54,110 µs | 692 µs | **78×** |
| **Search mean** | 14,524 µs | 538 µs | **27×** |
| **Recall@10** | 0.885 | 0.925 | **+4 pp** |
| **Save time** | 80 ms | 20 ms | 4× |
| **Load time** | 34,484 ms | 11 ms | **3,134×** |
| **Sidecar size** | 30 MB | 32 MB | 1.07× (usearch larger) |
| **RSS-Δ** | 26.9 MB | 52.7 MB | 2.0× (usearch larger) |
| **p99/p50 ratio** | 5.4× | 1.3× | usearch is far more stable |

### What this means

- **78× search p99** — hnsw_rs has pathological tail behavior
  (5.4× p99/p50 ratio) that would cause user-visible jank in
  a desktop RAG app. usearch's p99 is 1.3× p50, which is
  normal for a well-behaved HNSW.
- **3,134× faster load** — hnsw_rs's load takes 34 seconds at
  D=768 because the deserializer re-runs hnsw_rs's slow
  on-disk format decode. usearch's load is essentially a
  memcpy.
- **+4 pp recall@10** — at production scale, that's a real
  semantic-quality improvement, not just a benchmark number.
- **18.9× search p50** — the median latency drop is
  transformative for interactive use.
- **2.9× insert throughput** — bulk imports complete 3×
  faster.

The only places hnsw_rs is competitive are:

- **RSS-Δ at low dimensions** — usearch's per-vector typed
  scalar overhead is higher, but the absolute number (52.7
  MB at D=768) is still well under any practical memory
  budget.
- **Sidecar size at D=256** — usearch is 20% larger, but the
  absolute difference (12 MB vs 10 MB) is irrelevant.

### Migration path

The migration is a **default switch** + an opt-in flag for
backward compatibility:

```toml
# Cargo.toml — default features now include usearch-backend
[dependencies]
semantic-memory = "0.5"

# No action required — usearch 2.25 is the default.

# To opt back into hnsw_rs 0.3 (legacy):
semantic-memory = { version = "0.5", default-features = false, features = ["hnsw"] }
```

The `hnsw` feature is **opt-in** but **not deprecated**. It
will be removed in a future major version after the
`VectorBackend` trait has been used in production for at
least one minor release cycle.

### Reproduce the benchmark

```bash
cargo build -p hnsw-bench --bin hnsw-bench \
    --no-default-features --features hnsw --release
./target/release/hnsw-bench            # hnsw_rs run

cargo build -p hnsw-bench --bin hnsw-bench \
    --no-default-features --features usearch-backend --release
./target/release/hnsw-bench            # usearch run
```

Receipts: `hnsw-bench-receipt-{hnsw_rs,usearch}-20260602-*.json`.

## Quick Start

```rust
use semantic_memory::{MemoryConfig, MemoryStore};

#[tokio::main]
async fn main() -> Result<(), semantic_memory::MemoryError> {
    let store = MemoryStore::open(MemoryConfig::default())?;

    // Store a fact.
    store.add_fact("general", "Rust was first released in 2015", None, None).await?;

    // Hybrid search (BM25 + vector + RRF).
    let results = store.search("when was Rust released", None, None, None).await?;

    // Get the exact scoring breakdown.
    let explained = store.search_explained("when was Rust released", None, None).await?;
    for hit in explained {
        println!("  bm25={:.3}  vector={:.3}  rrf={:.3}  → {}",
            hit.bm25_score, hit.vector_score, hit.rrf_score, hit.title);
    }
    Ok(())
}
```

## What's in the box

### Storage

- **SQLite + WAL** — authoritative for all durable state.
  One writer connection + pooled reader connections.
- **FTS5** — BM25 full-text search over content, episode
  titles, message bodies.
- **Vector sidecar** — usearch 2.25 (default) or hnsw_rs 0.3
  (opt-in via `hnsw` feature). Both implement the
  `VectorBackend` trait. Pending sidecar mutations are
  journaled in SQLite and replayed on open / flush / rebuild
  / reconcile.
- **Bitemporal truth** — every fact carries a `valid_time` and
  `recorded_time` via the `bitemporal-runtime` foundation.

### Search

- **`search()`** — hybrid (BM25 + vector + RRF) over facts,
  document chunks, and episodes by default.
- **`search_explained()`** — same as `search()` but with the
  per-signal scores exposed.
- **`search_conversations()`** — message-level retrieval.
- **`reconcile()`** — rebuild FTS, re-embed, rebuild the
  sidecar from authoritative SQLite state.

### Integrity

- **`verify_integrity()`** — strict check for malformed stored
  data (invalid roles, JSON, enums, embedding blobs, quantized
  blobs, sidecar drift). Surfaces errors instead of silently
  converting to defaults.
- **Strict deserialization** — invalid stored data is an
  error, not a fallback.

### Graph

- **`store.graph_view()`** — deterministic traversal over
  namespaces, facts, documents, chunks, sessions, messages,
  episodes, and semantic/temporal/causal links derived from
  SQLite state.

### Receipts

- Every state transition (add_fact, search, reconcile, …)
  emits a typed receipt. Receipts are content-addressed and
  reproducible.

## Cargo features

| Feature | Default | What it enables |
|---|---|---|
| `usearch-backend` | ✓ | usearch 2.25 as the default vector backend (high-performance single-file vector search) |
| `hnsw` | ✗ | hnsw_rs 0.3 as the vector backend (legacy, opt-in) |
| `sqlite` | ✓ | SQLite storage (required) |
| `fts5` | ✓ | FTS5 full-text search (required for hybrid search) |
| `bitemporal` | ✓ | Bitemporal truth integration via `bitemporal-runtime` |

Both `usearch-backend` and `hnsw` are mutually exclusive —
enabling both at the same time is a build error.

## MSRV

Rust 1.75 (2021 edition). The `usearch` cxx-bridge requires
C++17 to build, which is a documented `build.rs`
prerequisite (see `cxx-build` in the dep tree).

## Test coverage

- **401 tests** in `lib/` + `tests/`, all pass with
  `cargo test --all-features`:
  - SQLite schema, WAL concurrency, FTS5 rebuild
  - usearch backend: insert, search, save, load, hot-swap
  - hnsw backend (opt-in): insert, search, persistence
  - bitemporal: as-of queries, temporal snapshots, supersession
  - Hybrid search: BM25 + vector + RRF, with score breakdown
  - Receipt emission for every state transition
  - Integrity checks: malformed stored data
  - Graph view: deterministic traversal
- `cargo test` clean.
- `cargo clippy --all-targets -- -D warnings` clean.

## Dependencies

- `rusqlite` — SQLite + FTS5 bindings.
- `usearch` 2.25 (default) or `hnsw_rs` 0.3 (opt-in).
- `bitemporal-runtime` — bitemporal truth.
- `stack-ids` — typed IDs, scopes, trace context.
- `boundary-compiler` — RFC 8785 JCS canonicalization.
- `serde`, `serde_json`, `chrono`, `tokio`, `tracing`,
  `thiserror`, `blake3`, `sha2`, `uuid`, `schemars`.

## Where it's used

`semantic-memory` is the search engine for:

- The LLM agent stack (forge-pilot, llm-pipeline) — every
  retrieval over a knowledge base.
- The LLM tool runtime — long-term tool-call memory.
- The verification runtime — fact storage with bitemporal
  truth.
- `fib-quant`, `turbo-quant`, `quant-eval` — the recall
  measurements in their benchmarks are run through
  `semantic-memory::search` against the raw-vector baseline.

Any system that needs **local-first, hybrid, bitemporal,
receipt-bearing** search can adopt `semantic-memory`
directly.

## License

Apache-2.0. See `LICENSE` for the full text.

## Changelog

See `CHANGELOG.md` for the release history.

## Acknowledgments

The HNSW → usearch migration was a 2-day investigation that
included the full benchmark harness, the `VectorBackend`
trait refactor, the default switch, and the sidecar-format
migration. The benchmark receipts (machine fingerprint, git
commit, full per-row payload) are in
`hnsw-bench-receipt-{hnsw_rs,usearch}-20260602-*.json` for
independent verification.
