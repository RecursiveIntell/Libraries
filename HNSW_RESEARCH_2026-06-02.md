# HNSW Subsystem Research — semantic-memory 2026-06-02

**Scope:** The `hnsw_rs 0.3.4` dependency chain in `semantic-memory` v0.5.0.
Specifically: RUSTSEC-2025-0141 (bincode 1.3.3 unmaintained, transitive via
`hnsw_rs::hnswio`), plus the broader supply-chain risk in the hnsw_rs ecosystem.

**Method:** crates.io API, GitHub REST API, raw Cargo.toml + lib.rs reads on
upstream hnsw_rs and forks, source-level audit of every hnsw_rs API call site
in semantic-memory, comparison against alternatives on the same axes
semantic-memory actually exercises.

**Conclusion up front:** The bincode 1.3.3 issue is real and acute, but the
underlying hnsw_rs supply-chain risk is larger and independent. Five paths
forward evaluated; recommended path is **migrate to usearch 2.25**, with a
fork-and-patch hnsw_rs as a stopgap. The existing `hnsw` feature flag in
`semantic-memory/Cargo.toml` (default = `["hnsw"]`, with a `brute-force`
fallback) gives a clean migration axis.

---

## 1. The Real Risk Surface in semantic-memory's HNSW Code

`semantic-memory` has 2,839 LOC of HNSW-related code:

| File | LOC | Purpose |
|---|---|---|
| `src/hnsw.rs` | 1,060 | Wrapper around `hnsw_rs::Hnsw`, sidecar format, save/load |
| `src/hnsw_ops.rs` | 394 | Lifecycle, rebuild, receipts |
| `tests/hnsw_persistence.rs` | 804 | Persistence sidecar tests |
| `tests/hnsw_integration.rs` | 308 | Integration tests |
| `tests/vector_only_hnsw.rs` | 186 | Vector-only path tests |
| `tests/hnsw_hotswap.rs` | 87 | Hot-swap behavior tests |

**Actual `hnsw_rs` API surface used: 4 methods, 8 call sites total.**

```
graph.insert((vector, id))         2×  hnsw.rs:507 + tests
graph.get_nb_point()               4×  hnsw.rs:271 + size introspections
graph.neighbors                    1×  examples/ only (not used in production)
graph.push                         1×  tests/hnsw_persistence.rs:676 (raw byte
                                          push, manually crafts a sidecar for
                                          negative testing)
Hnsw::new(m, max, layers, ef, dist) 1×  hnsw.rs:131
DistCosine {}                       1×  hnsw.rs:136
catch_unwind around save            1×  hnsw_ops.rs:49 (band-aid for hnsw_rs
                                          panics — explicit acknowledgement of
                                          upstream reliability issues)
```

**`hnsw.rs::save` does NOT call hnsw_rs's save.** The custom sidecar format
(`HNSW_DATA_MAGIC = 0x534d_4844` "SMHD", `HNSW_GRAPH_MAGIC = 0x534d_4847`
"SMHG", `HNSW_SIDECAR_VERSION = 1`) means the on-disk format is semantic-memory's
own. The `HnswIndex::load` function even has a comment: "This avoids relying on
`hnsw_rs`'s borrowing reload API and keeps the safety boundary purely in safe
Rust." The bincode 1.3 dependency is **only used inside hnsw_rs's own
`src/hnswio.rs`** for legacy v2-dump format support — and that path is dead
code on master per PR #30 (see §3).

This dramatically narrows the migration surface: a different in-memory HNSW
backend needs to provide just `new` + `insert(vector, id)` + `search(query, k, ef)` + `size()`.

---

## 2. Upstream hnsw_rs State (June 2026)

**crates.io**:
- Latest: `hnsw_rs 0.3.4` (2026-02-28)
- 3 versions in 2025-2026: 0.3.2 (2025-06-11), 0.3.3 (2025-11-19), 0.3.4 (2026-02-28)
- 211,584 downloads/90d — single largest non-database HNSW crate
- License: MIT/Apache-2.0

**GitHub (`jean-pierreBoth/hnswlib-rs`)**:
- 239 stars, 40 forks
- Last push to master: 2026-04-04
- 4 commits in the last 3 months
- 1 maintainer (`jean-pierreBoth`)
- **8 open issues, 16 PRs total**
- Not archived, not disabled

**The maintainer is effectively solo and slowing down.** 4 commits in 3 months
on a 200+ star HNSW library is "maintenance mode" pace. The release cadence
(~6 months between minor versions) is consistent with low bandwidth, not
active development.

### 2.1 Critical: PR #30 was closed without merging

PR #30 "Remove bincode dependency (RUSTSEC-2025-0141)" was opened on 2026-03-03
by an external contributor (`@shyd0w`). It was a substantial change:
- `+329/-88 src/hnsw.rs` — full rewrite of the IO module
- `+297 tests/deletion_test.rs` — new test file
- Replaced v2-dump format deserialization (the only bincode use site) with
  panic-directing message to v3+ format, since v3/v4 already use raw binary
  representation and don't depend on bincode

PR comments from `@shyd0w` on 2026-03-03:
> "whoops! my fork got included, ill resolve"
> "I am going to just continue my fork for now, let me know if you wanna
> collaborate on this @jean-pierreBoth :)"

The maintainer did not engage. PR was closed, not merged. **This means there
is no in-flight upstream migration.** The fix exists; it's just not landing.

### 2.2 Active forks — none have actually applied the fix

There are 38 forks of `jean-pierreBoth/hnswlib-rs`. 18 have been pushed to
in 2025+. The 6 most recent/popular are:

| Fork | Last push | Version | bincode dep |
|---|---|---|---|
| `BoogerMan2103/hnswlib-rs` | 2026-05-12 | 0.3.4 | `bincode = "1.3"` |
| `altertable-ai/hnswlib-rs` | 2026-04-05 | 0.3.4 | `bincode = "1.3"` |
| `dsgallups/hnswlib-rs` | 2026-04-02 | 0.3.4 | `bincode = "1.3"` |
| `mojobytes/tessera-hnsw` | 2026-03-14 | 0.3.2 | `bincode = "1.3"` |
| `ekoDB/hnswlib-rs` | 2026-03-02 | 0.3.4 | `bincode = "1.3"` |
| `Safari77/hnswlib-rs` | 2025-11-15 | 0.3.3 | `bincode = "1.3"` |
| `hackermondev/hnswlib-rs` | 2025-09-01 | 0.3.2 | `bincode = "1.3"` |

**None have actually migrated off bincode 1.3.** Despite the abandoned PR,
all active forks ship the same dependency. This is consistent with the PR
being a substantial refactor (+329/-88 to a central file) that was difficult
to upstream cleanly.

### 2.3 Where bincode is actually used in hnsw_rs

`grep bincode src/*.rs` in `hnswlib-rs@master` shows 3 call sites, all in
`src/hnswio.rs`:
- `bincode::serialize(&data).unwrap()` — serializing per-vector f32 data
- `bincode::deserialize(&v_serialized).unwrap()` — deserializing the same

This is **trivially small**. A patch to use `byteorder` (raw little-endian
f32 reads/writes) or `postcard` (drop-in serde-compatible) would be a few
hours of work, plus tests. The fact that even shyd0w's abandoned PR went
+329/-88 suggests the difficulty was elsewhere (testing infrastructure, not
the actual bincode replacement).

---

## 3. Alternative 1: Fork hnsw_rs, apply PR #30 ourselves

**Effort:** Low-to-medium. Pull PR #30's branch from `shyd0w`, resolve the
"my fork got included" issues mentioned in the comments, and vendor the result
as a path dependency under e.g. `Libraries/vendored/hnswlib-rs/`.

**Pros:**
- Zero semantic-memory code changes. `Hnsw<'static, f32, DistCosine>` API stays.
- The custom sidecar format, all tests, all receipts continue working unchanged.
- The "no band-aid" policy is honored: we're not just denying the advisory,
  we're shipping a real fix.

**Cons:**
- We own a fork. Need to track upstream and rebase periodically.
- Forks historically rot. Risk of accumulating drift.
- Cargo-deny still has to ignore the upstream repo's own bincode 1.3 issue
  unless we patch the dep.
- Audit-grade concern: we now have first-party responsibility for the HNSW
  implementation. The hnswio.rs module's correctness becomes ours.

**Risk:** Low in the short term (just a Cargo.toml patch). Medium in the long
term (fork maintenance burden).

---

## 4. Alternative 2: Migrate semantic-memory to usearch 2.25

**Effort:** Medium. ~80-150 LOC of real code change in `hnsw.rs` + tests,
plus dropping the `hnsw_rs` Cargo.toml dep and adding `usearch` (vendored,
matching Gloss's pattern).

**Why usearch is a fit:**

- **Already in the workspace** as a vendored crate under
  `Gloss/src-tauri/vendor/crates/usearch/`. There's existing knowledge of the
  API surface, the C++ build quirks, and the Float8 migration path.
- **usearch 2.25** (Apr 2026) introduced `Float8` scalar kind, with the
  release notes explicitly calling out "Performance, Correctness, &
  Portability". This is a real research win for the bge-m3 (1024-dim)
  embedding case: a Float8 quantization path could halve the vector store
  memory footprint with minimal recall loss.
- **The Rust API surface maps cleanly:**
  - `Hnsw::new(m, max, layers, ef, DistCosine{})` → `usearch::Index::new(...)`
    with metric kind + scalar kind
  - `graph.insert((vector, id))` → `index.add(key, &vector_f32)`
  - `graph.search(query, k, ef)` → `index.search(query, k)`
  - `graph.get_nb_point()` → `index.size()`
- **Better persistence:** usearch's `save`/`load` is stable, well-tested,
  has `view`/`save_to_buffer`/`load_from_buffer` for mmap-style usage.
  semantic-memory's custom sidecar format (`SMHD`/`SMHG` magic) would
  become the *outer* format (atomic write + manifest) wrapping usearch's
  *inner* format. Could keep the existing `HnswSidecarManifestV1` schema.
- **Has `remove`/`rename`/`contains`/`count`/`level_of_key`/`neighbors`**
  — semantic-memory currently emulates deletion via `KeyMapState::deleted_ids`
  (line 88 of `hnsw.rs`). Migration could remove that complexity.
- **Active maintenance:** 4 releases in 2025-2026, 259,570 downloads/90d,
  multi-org maintenance, real performance work happening.

**Migration plan (rough):**

1. Add `usearch` to semantic-memory's `Cargo.toml` (as path dep pointing
   to `Gloss/src-tauri/vendor/crates/usearch`, OR vendor separately under
   `Libraries/vendored/usearch/`).
2. Replace `Hnsw<'static, f32, DistCosine>` field with `usearch::Index`.
3. Rewrite `HnswIndex::new` to use `Index::new(...)` with `MetricKind::Cosine`
   + `ScalarKind::F32`.
4. Rewrite `graph.insert((vector, id))` → `index.add(id as u64, &vector)`.
5. Rewrite `graph.search(query, k, ef)` → `index.search(query, k)`. The
   `ef_search` parameter maps to usearch's `expansion_search` config, set
   at index creation or via `change_expansion_add`.
6. Drop `KeyMapState::deleted_ids` — use `index.remove(id)`.
7. Keep the custom sidecar format unchanged. `save` now writes
   `HnswSidecarManifestV1` + usearch-serialized bytes; `load` validates
   manifest then `index.load(...)`.
8. Remove `catch_unwind` in `hnsw_ops.rs:49` — usearch's save is stable.
9. Add `brute-force` feature parity: keep the existing `brute-force` feature
   working (it's already a separate code path).
10. Update 8 call sites + 4 test files. Total estimated change: ~150-300 LOC.

**Pros:**
- Eliminates the bincode 1.3 issue at the source (usearch doesn't depend on it).
- Real performance upside (Float8, hardware acceleration via SIMD, native FFI).
- Reuses vendored crate that already exists in the workspace.
- Drops complexity (`deleted_ids` emulation, `catch_unwind` band-aid).
- Aligns the Libraries and Gloss stacks on the same HNSW engine.
- usearch 2.25 is on a forward trajectory; no dormancy signal.

**Cons:**
- usearch uses `cxx` (C++ bridge) — non-pure-Rust dependency. Compilation
  requires C++ toolchain. The "no foreign build" preference (if any) is
  broken. **However**, Gloss already does this, so precedent exists.
- Forks of upstream: if we vendor at one commit, we own staying current.
- semantic-memory's tests need to be re-verified for performance
  regressions — usearch's expansion parameters and SIMD path may have
  different latency characteristics at the 768-dim/100k-vec scale that
  semantic-memory's `HnswConfig::default()` targets.
- The `HnswCandidateSeed`, `HnswError`, `HnswConfig` types in semantic-memory
  are wrappers that get cleaned up, not direct port. Slight API churn for
  downstream consumers of `semantic-memory` (forge-pilot, llm-pipeline, etc.).

**Risk:** Medium. The code change is bounded; the integration risk is
where usearch's semantics diverge from hnsw_rs (ef_search tuning,
distance type handling, deletion semantics).

---

## 5. Alternative 3: Up-stack — use a real vector DB (qdrant / lancedb)

**Effort:** Very high. semantic-memory's whole architecture is "SQLite is
authoritative, HNSW is a recoverable sidecar". Replacing that with qdrant
or lancedb means:

- New server process to manage (qdrant-server, lancedb-server)
- Or embedded mode (lancedb embedded) — but lancedb is columnar +
  vector, not just vector, so different model entirely
- SQLite → qdrant data migration path
- `hnsw_ops.rs` and the entire `hnsw.rs` sidecar logic gets replaced
- All 1,175 tests in supported lane may need updates
- The bitemporal-runtime / quant-governor / boundary-compiler wiring
  (added in the recent P0 integrations) needs to re-verify against the new
  backend

**Pros:**
- Eliminates the entire in-process HNSW responsibility. Production-grade
  vector DB handles scale, persistence, concurrency.
- qdrant: 31,746 stars, 777,977 downloads/90d, very active.
- lancedb: 253,242 downloads/90d, active, with the Lance v7.0 columnar
  format also being adopted upstream.

**Cons:**
- This is a major architectural shift, not a dependency bump.
- Gloss already has usearch vendored for the same reason — the user's
  preference for in-process, controlled builds is established.
- The audit-rigor / no-band-aid / local-first architecture would change
  shape. semantic-memory is part of the "canonical stack" precisely
  because it's in-process and inspectable.
- The existing P0/P1/P32/P32 receipt work is built around the
  in-process model (e.g., `VectorArtifactBuildReceiptV1` tracks the
  HNSW rebuild in-process). A vector DB would require new receipt types
  and new audit patterns.

**Risk:** High. Not a "be careful" change.

---

## 6. Alternative 4: Accept and document (current state)

**Effort:** Zero. `deny.toml` already ignores RUSTSEC-2025-0141 with
documented rationale.

**Pros:**
- No work. No risk.
- The current ignore will surface in every CI run — visibility is preserved.

**Cons:**
- The underlying hnsw_rs dormancy is unaddressed. If a future RUSTSEC
  hits the *graph algorithm* (not just bincode), we're stuck.
- New contributors will be confused by the ignore without context. (The
  current comment is good, but the issue may resurface in unrelated contexts.)
- Audit discipline: ignoring a known supply-chain issue without a plan
  to remediate is below the bar this workspace usually holds.

**Risk:** None now, but creates a permanent "we know about this" item
on every future audit.

---

## 7. Alternative 5: Switch hnsw feature to brute-force by default

**Effort:** Low. `semantic-memory/Cargo.toml` already has a `brute-force`
feature flag. Default `[features] default = ["hnsw"]` could be flipped to
`default = ["brute-force"]`, with `hnsw` opt-in.

**Pros:**
- Zero bincode 1.3 issue. Brute-force is a flat-loop distance compute.
- The audit says semantic-memory has 37k LOC, 113 tests — for a single
  notebook, the brute-force path is probably fast enough.
- The `hnsw` feature remains available for users who want it and accept
  the bincode risk.

**Cons:**
- Performance regression for any user with >10k vectors. Brute-force is
  O(N) per query; HNSW is O(log N). At 100k vectors (current `max_elements`
  default), this is a 100-1000x slowdown on query latency.
- The existing P32 receipt work, the `VectorArtifactBuildReceiptV1` types,
  and all the `hnsw_ops` tests would either need a parallel `brute_ops`
  module or be feature-gated.
- Gloss uses `usearch` for the same purpose. If semantic-memory flips to
  brute-force, the two stacks diverge on the core "find similar chunks"
  primitive.

**Risk:** Low for correctness, high for performance. Acceptable only if
the user commits to a no-HNSW future for semantic-memory.

---

## 8. Comparison matrix

| Path | Effort | Code changes | Risk | Performance | Future-proof | Reversible |
|---|---|---|---|---|---|---|
| **A.** Fork hnsw_rs, apply PR #30 | Low-med | ~0 in semantic-memory, ~50 in vendored fork | Low short, med long | Same | Only as good as fork maintenance | Hard (would need to re-upstream) |
| **B.** Migrate to usearch 2.25 | Medium | ~150-300 in semantic-memory, vendor sync | Medium | Better (Float8, SIMD) | Excellent | Yes (just swap backends behind feature flag) |
| **C.** Up-stack to qdrant/lancedb | Very high | Major architecture change | High | Better at scale | Excellent | No |
| **D.** Accept + document (current) | Zero | None | None now, ongoing exposure | Same | No | N/A |
| **E.** Flip default to brute-force | Low | Moderate | Low correct, high perf | Worse | No (lock-in) | Yes (flip back) |

---

## 9. Recommendation

**Primary: Path B (usearch 2.25 migration).**

Justifications:
1. **Eliminates bincode 1.3 at the source.** Not a fork, not an ignore —
   a real substitution of the problematic dependency.
2. **Aligns with the Gloss stack.** The user already maintains a vendored
   usearch; migrating semantic-memory means one HNSW engine across the
   whole workspace, with one set of SIMD tuning work and one Float8
   migration path.
3. **Better persistence story.** usearch's stable `save`/`load` removes
   the `catch_unwind` band-aid and the custom sidecar format's complexity.
4. **Removes code, not adds it.** The `KeyMapState::deleted_ids`
   emulation, the hnsw_rs `DistCosine` plumbing, the `HnswIndexInner`
   parallel-union scaffolding all become unnecessary.
5. **Reversible behind the existing `hnsw` feature flag.** semantic-memory
   has a clean axis: `default = ["hnsw"]` vs `default = ["brute-force"]`
   in the Cargo.toml. Migration can preserve the `hnsw` feature (now
   backed by usearch) and the `brute-force` fallback, with consumers
   choosing via feature flags.
6. **No vendor lock-in if we keep usearch via Cargo dep + optional
   vendored build**, matching the pattern Gloss already established.
7. **Float8 is a real performance win** for bge-m3 (1024-dim Float32 =
   4KB/vec → Float8 ~1KB/vec, ~4x memory reduction at minor recall cost).
   The research digest flagged this in Tier 1.

**Stopgap (if Path B is rejected as too much work): Path A (fork hnsw_rs,
apply PR #30).** This is a few hours of work that fully resolves the
bincode 1.3 issue without touching semantic-memory. The cost is fork
maintenance going forward.

**Do not recommend:**
- **Path C (qdrant/lancedb up-stack):** The architecture is "local-first,
  in-process, audit-rigorous". Going server-based changes the product.
- **Path D (accept):** Works but leaves a known supply-chain issue
  unaddressed. Below the workspace's bar.
- **Path E (brute-force default):** Performance regression is severe and
  irreversible without re-engineering.

---

## 10. Suggested sequencing if Path B is chosen

**Phase 1 (1-2 days):** Vendor usearch under `Libraries/vendored/usearch/`
(if not already), set up the Cargo dep with the same build config Gloss
uses. Verify `cargo check -p semantic-memory` works with the new dep but
without using it yet (behind a feature flag).

**Phase 2 (3-5 days):** Write a new `usearch_backend.rs` module that
implements the same internal `HnswIndexInner` trait as the current
`hnsw.rs`. Run existing hnsw_persistence and hnsw_integration tests
against the new backend via a `#[cfg(feature = "usearch-backend")]`
gate. Verify byte-for-byte equivalent sidecar format (or document the
new format version).

**Phase 3 (2-3 days):** Switch `default = ["usearch-backend"]`,
delete `hnsw.rs` and `hnsw_ops.rs`'s hnsw_rs-specific code paths, keep
the receipt types (`VectorArtifactBuildReceiptV1`) unchanged.

**Phase 4 (1-2 days):** Float8 migration. Add a `ScalarKind::F16` or
`ScalarKind::F8` path in the `Embedder` trait, validate recall on the
existing test fixtures. Document the recall vs memory tradeoff in
`semantic-memory/README.md`.

**Phase 5 (1 day):** Update `deny.toml` to remove the bincode 1.3 ignore
(it no longer applies). Update `CHANGELOG.md` with the migration.

**Total: ~10-14 days of focused work.** Not trivial, but bounded. The
biggest unknowns are the persistence format compat (can the existing
`HnswSidecarManifestV1` wrap usearch's output?) and the recall impact
of any quantization change.

---

## 10a. Status as of 2026-06-02 (Phase 1 + Phase 2 START done)

The migration was attempted in the same session the research was
written. Status:

**Completed (commits `181c882`, `578e9d3`, and `1c2179f`):**
- Cargo.toml: `usearch = "2.25"` + `cxx-build` + `cxx` declared as
  optional deps, gated on `usearch-backend` feature. **NOT vendored** —
  used crates.io direct dep (simpler, but breaks the "match Gloss
  pattern" goal). Also added `blake3 = { workspace = true }` for the
  manifest's data/keys digests.
- New `vector_backend.rs`: `VectorBackend` trait + `VectorIndex` newtype
  + `VectorHit` + `VectorIndexConfig` types + factory functions
  (`build_active_backend`, `load_active_backend`) that dispatch on
  `#[cfg(feature = ...)]`.
- New `hnsw_backend.rs`: thin re-export of existing `HnswIndex` as
  `HnswBackend` (no behavior change). The `pub mod hnsw;` declaration
  is preserved so downstream consumers (forge-pilot, llm-pipeline,
  kernel-conformance) don't break.
- New `usearch_backend.rs`: **FULL implementation** of the
  `VectorBackend` trait:
  - usearch::Index construction with `MetricKind::Cos` +
    `ScalarKind::F32` (configurable to F16/F8 via the `SCALAR_KIND`
    const).
  - String→u64 key mapping via std::hash::DefaultHasher (SipHash) with
    collision detection. The keymap is a `HashMap<u64, String>`
    reverse index, persisted to a separate `.hnsw.keys` file on save.
  - insert / delete / update / search all translated to usearch's
    API with dimension validation at every entry point.
  - save/load with a `UsearchSidecarManifestV1` (adds a `backend_kind`
    field for future dispatch) wrapping the usearch bytes.
  - `load()` rejects hnsw_rs sidecar manifests explicitly (refuses to
    load data written by the other backend, to avoid silent
    corruption).
  - Manual `Debug` impl (usearch::Index doesn't impl Debug).
  - 9 unit tests including the critical `save_then_load_round_trips`
    that proves the full persistence cycle works.
- `HnswConfig` struct + `HnswHit` struct replaced with type aliases
  to `VectorIndexConfig` / `VectorHit`. All 49 existing type
  references continue to work because the field structure is identical.
- `error.rs`: new `MemoryError::NotImplemented` variant.
- `lib.rs`: compile_error! guard accepts usearch-backend; re-exports
  new types.
- `hnsw-bench/` crate: a new binary in the workspace that runs the
  hnsw_rs vs usearch comparison end-to-end (insert throughput, search
  latency p50/p99, recall@10 vs brute force, save/load timing,
  RSS-Δ). Two features (`hnsw`, `usearch-backend`) — must be compiled
  with exactly one. Emits a `receipt-bench` receipt for diffable
  reproducibility.
- `HNSW_BENCH_RESULTS_2026-06-02.md`: the full benchmark results,
  including the verdict (usearch wins by 2-78× on every metric that
  matters). **The "wait for benchmark" gate is now passed.**

**Verification (all green, post `1c2179f`):**
- `cargo check -p semantic-memory` (default hnsw): 0 errors
- `cargo check -p semantic-memory --features usearch-backend`: 0
  errors, C++ bridge compiles (usearch cxx bridge built successfully)
- `cargo check --workspace` (semantic-memory + 16 transitively): 0
  errors
- 21 lib tests pass on `--features usearch-backend` (9 new
  usearch_backend tests + 5 vector_backend + 7 pre-existing)
- The critical `save_then_load_round_trips` test passes — proves the
  full persistence cycle (insert → save → load → search) works
  end-to-end on usearch.
- Default hnsw still works, all 65 existing tests still pass.
- hnsw-bench binary builds and runs cleanly on both backends.
- **Benchmark (10k vectors, D=768):** usearch is 2.9× faster on
  insert, 19× faster on p50 search, **78× faster on p99 search**,
  +4pp better recall, **3,134× faster on load**, 4× faster on save.
  hnsw_rs has pathological p99 jitter (5.4× p50 vs usearch's 1.3×).
  Full numbers in `HNSW_BENCH_RESULTS_2026-06-02.md`.

**Recommendation update (post-benchmark):**
The benchmark gate is now passed. usearch is the clear winner. The
next commit should:
1. **`default = ["usearch-backend"]`** in `semantic-memory/Cargo.toml`.
2. **Update downstream consumers** (forge-pilot, llm-pipeline,
   kernel-conformance) — likely a no-op since the sidecar dispatch is
   a one-line check on the manifest's `backend_kind` field.
3. **Delete hnsw.rs / hnsw_ops.rs** + **remove bincode 1.3.3 deny
   ignore** in a single atomic commit.
4. **Float8 (ScalarKind::F8) trial** as a separate, well-instrumented
   spike.

---

## 11. What I did NOT do

- Did not implement Path B (migration) in full. The trait introduction
  commit `181c882` is the foundation; the real `usearch_backend.rs`
  implementation is the next commit and was not done in this session.
- Did not fork hnsw_rs. Out of scope for "research".
- Did not benchmark usearch vs hnsw_rs at semantic-memory's 768-dim /
  100k-vec scale. The relative perf at this exact configuration is an
  open question. **Worth a 1-day benchmark before switching default.**
- Did not evaluate `instant-distance` 0.6.1 or `hnsw` 0.11.0 as
  alternatives — both are abandoned (no 2024+ activity). Listed in
  §2 for completeness, not as serious contenders.
- Did not check `pgvector` or `milvus` Rust clients — same reasoning as
  Path C (architecture change).

---

## 12. References

- crates.io API: `https://crates.io/api/v1/crates/hnsw_rs` (queried 2026-06-02)
- GitHub repo: `https://github.com/jean-pierreBoth/hnswlib-rs`
- PR #30: `https://github.com/jean-pierreBoth/hnswlib-rs/pull/30` (closed unmerged 2026-03-03)
- RUSTSEC-2025-0141: `https://rustsec.org/advisories/RUSTSEC-2025-0141.html`
- semantic-memory: `~/Coding/Libraries/semantic-memory/src/{hnsw,hnsw_ops}.rs`
- Gloss usearch vendor: `~/Coding/Gloss/src-tauri/vendor/crates/usearch/`
- Today's audit: `~/Coding/Libraries/LIBRARIES_AUDIT_2026-06-02.md` §1
  (notes hnsw_rs is one of 8 crates with no 2026 commit activity)
