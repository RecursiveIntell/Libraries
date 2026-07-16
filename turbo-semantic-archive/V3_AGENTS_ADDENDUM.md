# V3_AGENTS_ADDENDUM.md — Agent Directives for v0.3.0

## Agent Role: Rust Systems Engineer

You are implementing a correctness and performance upgrade to a semantic search library. The crate is pre-1.0 and has no external users, so breaking changes to internal APIs are acceptable. Breaking changes to the public `MemoryStore` API require explicit approval in V3_SPEC.md.

## Critical Constraints

### Compilation Order Matters

The phases in `CLAUDE_CODE_PROMPT.md` are topologically sorted. Phase 3 (RwLock) changes how every other phase accesses the HNSW index. If you implement Phase 5 before Phase 3, you'll write code that doesn't compile after Phase 3 lands. **Follow the phase order.**

### Feature Flag Awareness

Every HNSW code path must be gated behind `#[cfg(feature = "hnsw")]`. The `brute-force` feature must continue to work as a fallback. After each phase, verify both feature configurations compile:

```bash
cargo check --features hnsw
cargo check --features brute-force
```

### Test After Every Phase

Do not batch changes. After completing each phase:
```bash
cargo test --features "hnsw,testing"
cargo clippy --features "hnsw,testing" -- -D warnings
```

If tests fail, fix them before proceeding.

### The Single Mutex<Connection> — Don't Try to Fix It

The single `Mutex<Connection>` is a known bottleneck but is explicitly out of scope for v0.3.0. Do not refactor database access patterns, add connection pooling, or introduce a second connection. The `with_conn()` pattern stays as-is.

### HNSW Keymap Persistence — Deferred Writes

Do NOT write to SQLite on every `HnswIndex::insert()`. This would be a massive performance regression. Use the dirty flag + batch flush strategy described in V3_SPEC.md §2. The keymap is flushed:
- On `Drop` (alongside graph save)
- On explicit `flush_hnsw()` 
- On `rebuild_hnsw_index()` / `compact_hnsw()`

### Box::leak — Try Simple First

For Phase 8, try the simpler `_reloader_keepalive: Option<Box<HnswIo>>` approach first. Only use `ManuallyDrop` + custom `Drop` if the simpler approach causes issues. Test by:
1. Creating a MemoryStore
2. Adding data
3. Dropping the MemoryStore
4. Verifying no panics or memory corruption (run under `cargo test` with `RUST_BACKTRACE=1`)

### Quantization — Don't Over-Optimize

The primary goal of wiring quantization is **storage savings** (3.97× on SQLite blob size) and **future-proofing** for Qi8-native HNSW backends. HNSW search still uses f32 vectors in v0.3.0 because `hnsw_rs` v0.3 doesn't support i8 natively. Don't try to hack i8 vectors into the f32 graph.

## Code Quality Standards

- All `pub` items need doc comments
- All `unsafe` blocks need `// SAFETY:` comments explaining the invariant
- No `unwrap()` on fallible operations in non-test code except `Mutex::lock()` (standard Rust convention: poisoned mutex = unrecoverable)
- Use `tracing::warn!` for degraded states, `tracing::error!` for failures, `tracing::info!` for lifecycle events
- Prefer `.map_err()` with context over bare `?` when the error message would be ambiguous

## File Modification Checklist

Before submitting each phase, verify:

- [ ] `cargo check --features hnsw` passes
- [ ] `cargo check --features brute-force` passes  
- [ ] `cargo test --features "hnsw,testing"` passes
- [ ] `cargo clippy --all-features -- -D warnings` passes
- [ ] No dead code warnings for new additions
- [ ] Doc comments on all new public items
