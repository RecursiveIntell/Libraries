# Semantic Memory Governed Write and Retrieval Coherence Implementation Plan

> **For Hermes:** Execute with strict TDD, preserving the lean autonomous boundary and canonical MemoryAuthority ownership.

**Goal:** Make governed HTTP appends fully hybrid-retrievable, make ordinary cross-process search caches authority-epoch coherent, and make witnessed receipts expose real authority state without widening autonomous mutation rights.

**Architecture:** Keep `lean`/`standard` MCP profiles read-only. All writes continue through `MemoryAuthority`; the writer computes canonical embeddings before the atomic authority transaction and journals any derived-index work. Cache entries become validity-bound to the durable authority epoch. Witnessed retrieval remains cache-bypassing and exact-preferred, but reports the actual snapshot/epoch instead of hardcoded unavailable values.

**Tech Stack:** Rust, SQLite WAL/FTS, semantic-memory MemoryStore/MemoryAuthority, semantic-memory-mcp, rmcp, Cargo tests.

---

## Evidence-backed current state

- Active Hermes MCP profile: `lean` in `/home/sikmindz/.hermes/config.yaml`.
- Lean/standard intentionally expose only witnessed search and assertion/action authority decisions.
- Governed HTTP `/add` returns `authority_receipt_v1` and persists canonical facts through `MemoryAuthority`.
- `MemoryAuthority` currently inserts authority facts with `embedding = NULL`; verified governed facts are BM25-visible but have null vector rank/cosine and can be buried in hybrid top-k.
- Ordinary search cache is process-local; a mutation in writer process B cannot clear reader process A’s cache.
- Witnessed search already uses `ReturnReceipt + PreferExact`; do not add sidecar mutation or autonomous writes to that path.
- Witnessed MCP responses currently hardcode authority snapshot and retrieval epoch as unavailable/null despite durable authority state.

## Hard constraints

- Do not change Hermes MCP from `lean`.
- Do not expose mutation/admin tools in lean or standard.
- Do not write directly to SQLite outside canonical store/authority APIs.
- Do not reconcile or mutate ANN sidecars from witnessed search.
- Do not weaken source/provenance hydration filters.
- Do not let callers supply authoritative embeddings without writer-side computation and validation.
- Preserve append/supersession lineage, origin labels, idempotency, atomic receipts, Current-state filtering, and existing public API compatibility where practical.

## Task 1 — RED: two-process witnessed and ordinary-cache regression tests

**Files:**
- Modify tests under `/home/sikmindz/Coding/Libraries/semantic-memory/tests/` or crate-local tests in `src/lib.rs`.
- Modify `/home/sikmindz/Coding/Libraries/semantic-memory-mcp/tests/integration.rs`.

1. Add a two-store test sharing one temporary memory directory.
2. Prime reader A’s ordinary search cache.
3. Append a sourced, uniquely named canary through writer B’s `MemoryAuthority`.
4. Assert reader A witnessed search (`ReturnReceipt + PreferExact`) sees the canary without reopening.
5. Assert repeated ordinary search in reader A sees the canary after the planned epoch-aware cache fix.
6. Add a lean MCP regression proving the canary is returned while mutation tools remain unavailable.
7. Run focused tests and record the expected RED failures for ordinary cache coherence and hybrid/vector evidence.

## Task 2 — GREEN: governed embedding/index completion

**Files:**
- Modify `/home/sikmindz/Coding/Libraries/semantic-memory/src/authority.rs`.
- Reuse canonical embedding/quantization/index-journal helpers from `knowledge.rs`, `lib.rs`, and `db.rs`; refactor shared helpers only where necessary.
- Extend affected tests.

1. Compute embeddings inside the governed writer workflow using the configured store embedder before opening the authority transaction.
2. Carry validated embedding material into append and supersede mutations without making embedding bytes part of semantic content identity/idempotency digests.
3. Persist f32 and existing quantized representations consistently with ordinary facts.
4. Enqueue supported derived-index operations transactionally so sidecars can reconcile through existing writer/admin machinery.
5. On embedding failure, return an error before canonical visibility; do not silently commit lexical-only facts.
6. Preserve receipt, epoch, lineage, origin, supersession, and fault-rollback guarantees.
7. Prove governed append and supersede return non-null vector evidence under exact hybrid search.

## Task 3 — GREEN: authority-epoch-aware ordinary search cache

**Files:**
- Modify `/home/sikmindz/Coding/Libraries/semantic-memory/src/lib.rs` and the smallest supporting cache type/API files.
- Extend core two-store tests.

1. Bind every cache entry to the durable authority/retrieval epoch observed when created.
2. Read the current durable epoch before accepting a cache hit.
3. Treat epoch mismatch as a cache miss and replace the stale entry after live search.
4. Do not use vector count or ANN generation as the invalidation authority.
5. Keep witnessed `ReturnReceipt` cache bypass unchanged.

## Task 4 — GREEN: witnessed authority state receipt completion

**Files:**
- Add/read-only public authority snapshot API in `/home/sikmindz/Coding/Libraries/semantic-memory/src/authority.rs` or an existing authority-contract module.
- Modify `/home/sikmindz/Coding/Libraries/semantic-memory-mcp/src/server.rs`.
- Extend MCP tests.

1. Expose the current typed authority snapshot ID and retrieval epoch through a read-only store/authority API.
2. Populate `current_snapshot_id`, `retrieval_epoch`, and authority stage outcome in `sm_search_witnessed`.
3. Remove hardcoded `unavailable` degradation when the API succeeds.
4. Fail contained or report a typed degradation if authority-state reading genuinely fails; do not invent values.

## Task 5 — Verification and review

Run in `/home/sikmindz/Coding/Libraries/semantic-memory`:

```bash
cargo fmt --all -- --check
cargo test --all-targets
cargo test --all-targets --features brute-force
cargo test --all-targets --all-features
cargo clippy --all-targets --all-features -- -D warnings
```

Run in `/home/sikmindz/Coding/Libraries/semantic-memory-mcp`:

```bash
cargo fmt --all -- --check
cargo test
cargo test --features full
cargo clippy --all-targets --all-features -- -D warnings
```

Live acceptance:

1. Build fresh release binary without overwriting active binary until tests pass.
2. Start isolated temporary HTTP writer and lean MCP reader on a temporary database.
3. Append a sourced canary through HTTP `/add`.
4. Verify normal witnessed top-k returns it with non-null vector evidence, actual authority snapshot ID, retrieval epoch, and durable retrieval receipt.
5. Prime ordinary search before a second append and verify the second fact appears without restarting the reader.
6. Confirm lean tools/list still exposes exactly the three governed read-only tools.
7. Run independent code review on the final diff.

## Claim boundary

After all gates pass, it is safe to claim local source/test proof that governed writes are hybrid-indexed, ordinary cross-process caches are epoch coherent, and witnessed responses carry real authority state. Do not claim external benchmark superiority, distributed consistency beyond the tested shared SQLite deployment, or production security certification.
