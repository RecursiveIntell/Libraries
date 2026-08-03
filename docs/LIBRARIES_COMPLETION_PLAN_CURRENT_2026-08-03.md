# Libraries Completion Plan — Current Reconciliation and Execution

> **For Hermes:** Use the phase gates and task-level RED/GREEN/evidence/rollback fields below. Do not promote historical completion documents or Agent Graph advice over current source, manifests, worktree state, and reproducible command output.

**Goal:** Finish the Libraries compressed-retrieval and evidence substrate without replacing canonical f32 memory, creating shadow truth, or claiming runtime support before a CPU-only vertical slice is proven.

**Architecture:** `fib-quant`, `turbo-quant`, and `quant-codec-core` own codec mathematics and profile/shape contracts. The nested `poly-kv` workspace owns pool construction, compressed scoring, readers, persistence, and pool receipts. `semantic-memory` remains the canonical retrieval owner: it owns authoritative SQLite embeddings, derived-generation admission, candidate policy, exact reranking, fallback, and semantic search receipts. MCP, Python, GPU, HF, vLLM, FlashInfer, TensorRT, CUDA, and serving adapters remain downstream and blocked until the Rust path is independently certified.

**Tech Stack:** Rust 2021, Cargo workspaces, SQLite/WAL, `semantic-memory`, nested `poly-kv`, FibQuant, TurboQuant, `quant-governor`, `scr-runtime-compression`, typed serde receipts, and CPU-only benchmark fixtures.

---

## 0. Evidence cutoff and current verdict

**Evidence cutoff:** `2026-08-03T01:45:13-05:00` (final closeout refresh; see receipt for command-level timestamps)  
**Canonical root:** `/home/sikmindz/Coding/Libraries`  
**Root branch/HEAD:** `main` / `90bf644a2732658d7c07604ee4b2657520e78122`  
**Current root status:** dirty; see the ownership table and closeout receipt.  
**Current completion verdict:** **CONDITIONAL / NOT RELEASE-COMPLETE**.  
**Machine-readable closeout:** `docs/receipts/LIBRARIES_CLOSEOUT_2026-08-03.json` (`complete: false`).

The repository has a buildable focused semantic-memory feature path and passing focused tests. It does **not** yet prove that FibQuant/PolyKV compressed candidate generation is reachable through semantic-memory search, that persisted pools reload into a usable `SharedKvPool`, or that approximate candidates are exact-reranked from the live derived artifact. Those are the remaining high-value gates.

### Evidence already verified in the current tree

| Evidence | Command / source | State | Meaning |
|---|---|---|---|
| Main workspace inventory | `cargo metadata --manifest-path Cargo.toml --no-deps --format-version 1` | **verified**: 64 packages / 64 members | Current root package count; supersedes historical 676-package text |
| AiDENs inventory | `cargo metadata --manifest-path AiDENs/Cargo.toml --no-deps --format-version 1` | **verified**: separate 34-package workspace | Must be tested and planned separately; it is not a member of the main workspace |
| Semantic-memory feature resolution | `cargo check -p semantic-memory --features fib-quant-codec --lib` | **verified pass** | PolyKV adapter symbols resolve under the current manifest |
| Semantic-memory tests | `cargo test -p semantic-memory --all-targets --features fib-quant-codec`; then default all-targets tests | **verified pass**, exit 0 | Compile/test evidence only; not runtime reachability of a compressed semantic search path |
| AiDENs boundary suite | `cargo test --manifest-path AiDENs/Cargo.toml --all-targets` | **verified pass**, 28 passed / 0 failed | Separate workspace regression evidence |
| Workspace strict Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **verified failed**, exit 101 | Current blockers are outside the focused PolyKV method-resolution path |
| PolyKV API presence | `poly-kv/crates/poly-kv/src/pool.rs:657`, `:792` | **verified by source** | Canonical methods are `attention_topk_compressed` and `attention_topk_compressed_prepared` |
| FibQuant policy reachability | `semantic-memory/src/search.rs:1024-1026` | **verified by source** | `FibQuantCandidateOnly` still returns `NotImplemented`; feature forwarding is not search integration |
| PolyKV reload | `semantic-memory/src/poly_kv_backend.rs:75` | **verified by source** | `load()` discards loaded blocks and leaves `pool: None`; restart/reload is incomplete |
| ProveKV candidate path | `semantic-memory/src/search.rs:1051-1084` | **verified by source** | Current path loads metadata/payload but still performs brute-force f32 retrieval; it does not score through `SharedKvPool` |

### Post-plan validation refresh

The following gates were rerun after the source edits and plan baseline above. They supersede the earlier strict-Clippy failure receipt but do not change the P0 implementation verdict.

| Gate | Exact command | Current result | Evidence boundary |
|---|---|---|---|
| Formatting | `cargo fmt --all -- --check` | **pass**, exit 0 | Source formatting only |
| Main workspace compile | `cargo check --workspace --all-targets` | **pass**, exit 0 | Compile-time evidence; Cargo emitted only the known ignored non-root `quant-governor` profile warning |
| All feature compile | `cargo check --all-features` | **pass**, exit 0 | Compile-time feature resolution; not runtime support |
| Strict workspace Clippy | `cargo clippy --workspace --all-targets -- -D warnings` | **pass**, exit 0 | Workspace lint closure; no claim that P0 compressed retrieval is implemented |
| Semantic-memory compressed profile | `cargo test -p semantic-memory --all-targets --features fib-quant-codec` | **pass**, 121 passed / 3 ignored in unit target plus all integration/example targets passed | Feature/test evidence; FibQuant search dispatch remains `NotImplemented` by source |
| Semantic-memory compressed Clippy | `cargo clippy -p semantic-memory --all-targets --features fib-quant-codec -- -D warnings` | **pass**, exit 0 | Strict feature-profile lint evidence |
| PolyKV feature-off | `cd poly-kv && cargo test -p poly-kv --no-default-features --all-targets` | **pass** | Confirms the now-mandatory serde persistence contract is internally coherent; zero adapter tests is expected without adapter feature |
| PolyKV FibQuant | `cd poly-kv && cargo test -p poly-kv --no-default-features --features fibquant-adapter --all-targets` | **pass**, FibQuant adapter/pool/persistence suites passed | Nested-workspace evidence |
| PolyKV all features | `cd poly-kv && cargo test -p poly-kv --all-features --all-targets` | **pass**, including benches | Nested-workspace evidence; benchmark output is fixture-local |
| PolyKV strict lint | `cd poly-kv && cargo clippy --workspace --all-targets --all-features -- -D warnings` | **pass**, exit 0 | Nested workspace lint evidence |
| Root full tests | `cargo test --workspace --all-targets --no-fail-fast` | **pass**, exit 0 | Full root regression evidence; output was independently returned by Cargo, not promoted from a background self-report |
| AiDENs regression | `cargo test --manifest-path AiDENs/Cargo.toml --all-targets` | **pass**, exit 0 | Separate nested workspace evidence |
| MCP integration assets | `python3 integrations/tests/validate_integrations.py` from `semantic-memory-mcp` | **pass**, exit 0 | Read-only structural integration validation |
| MCP protocol profiles | Built `semantic-memory-mcp --features full`; sent `initialize` + `tools/list` over stdio for `lean`, `standard`, `agent`, and `full` with a temporary mock store | **pass**, all four exited 0; tool counts 4, 4, 12, and 67 respectively | Real process/protocol evidence; tool visibility only, not compressed retrieval proof |
| Root documentation | `cargo doc --workspace --no-deps` | **blocked**, exit 101 | Pre-existing duplicate `_native` lib output filename between `agent-graph-python` and `llm-pipeline-python`; outside this plan's compressed-retrieval scope |
| Nested PolyKV documentation | `cd poly-kv && cargo doc --workspace --no-deps --all-features` | **pass**, exit 0 | Nested documentation evidence |

### Historical material that is not current completion proof

- `docs/LIBRARIES_COMPLETION_PLAN_MASTER.md` and `docs/LIBRARIES_COMPLETION_EXECUTIVE_SUMMARY.md` are dated `2026-05-29` and report 676 packages / 52 crates. They remain architecture context only.
- `docs/STATUS_REPORT_2026-05-29.md` and `development/high-assurance-engineering/references/phase3-completion-receipt.md` contain source-reported or historical receipts with later-conflicting claims. They must not be used as current green evidence.
- Agent Graph review output is advisory because the inspected graph had no repository source-witness or tool node. It cannot establish Cargo feature resolution or runtime reachability.

---

## 1. Source inventory checked

### Governing instructions

- `AGENTS.md` — root scope and phase discipline.
- `semantic-memory/AGENTS.md` — semantic-memory ownership, authoritative import/search state, and compatibility boundaries.
- `poly-kv/AGENTS.md` — codec/pool ownership, exact fallback, no silent lossy behavior, and receipt requirements.
- `semantic-memory-mcp/CLAUDE.md` — MCP feature/runtime distinction and protocol-level `tools/list` requirement.
- `development/writing-plans/SKILL.md` — current-state section, bite-sized executable tasks, and claim boundary.
- `development/high-assurance-engineering/SKILL.md` — dirty multi-workspace preflight, source identity, receipt closure, and current-main reruns.
- `development/compressed-codec-semantic-memory-integration/SKILL.md` — codec admission, logical integrity, wire separation, pool reachability, and CPU-only measurement order.
- `development/governed-work-closeout/SKILL.md` — machine-readable receipts and explicit incomplete states.
- `development/repository-reconciliation/SKILL.md` — nested repository/gitlink ownership and no bulk staging.

### Canonical source surfaces

- `Cargo.toml`, `Cargo.lock` — main workspace membership and dependency resolution.
- `AiDENs/Cargo.toml` — separate nested workspace membership.
- `fib-quant/` — FibQuant mathematics, codec/profile behavior, and owner tests.
- `turbo-quant/` — TurboQuant mathematics and current semantic-memory adapter reference.
- `poly-kv/Cargo.toml`, `poly-kv/crates/poly-kv/Cargo.toml` — nested workspace and package feature closure.
- `poly-kv/crates/quant-codec-core/` — shared codec/profile/shape contracts.
- `poly-kv/crates/poly-kv/src/pool.rs` — `SharedKvPool` compressed scoring and prepared-index API.
- `poly-kv/crates/poly-kv/src/store.rs` — content-addressed pool persistence and reload primitives.
- `semantic-memory/Cargo.toml` — feature forwarding and path dependency ownership.
- `semantic-memory/src/config.rs` — `DerivedVectorBackendPolicy` and exact-rerank policy.
- `semantic-memory/src/search.rs` — actual candidate dispatch and current fallback behavior.
- `semantic-memory/src/poly_kv_backend.rs` — existing backend prototype; currently not the canonical semantic search dispatch and has an incomplete reload path.
- `semantic-memory/src/poly_kv_bridge.rs` — intentionally fail-closed migration bridge; it must not be revived as a second semantic-memory wrapper.
- `semantic-memory/src/db.rs` — V21 TurboQuant artifact generations and V24 ProveKV pool generations.
- `semantic-memory/src/types.rs` — typed generation/build/candidate receipt structures.
- `semantic-memory/tests/` — existing generation, receipt, corruption, and search tests.
- `semantic-memory-mcp/Cargo.toml`, `semantic-memory-mcp/src/` — downstream feature/runtime surface; no MCP promotion before the Rust path is proven.
- `quant-governor/`, `scr-runtime-compression/`, `quant-eval/`, `receipt-bench/` — policy, adapter, measurement, and receipt substrates to be admitted in dependency order.

---

## 2. Workspace and ownership reconciliation

| Surface | Current observation | Disposition | Rollback / re-admission |
|---|---|---|---|
| Main Libraries root | `main`, HEAD `90bf644...`; modified `Cargo.lock`, `semantic-memory/Cargo.toml`, `semantic-memory/src/poly_kv_backend.rs`; modified gitlink `cea-bridge`; untracked `semantic-memory-mcp-transport/` | Preserve; no bulk staging | Revert only explicitly admitted paths; preserve nested candidates untouched |
| `cea-bridge` | Parent gitlink changed from `c72fa14...` to `71dc37...`; child checkout is clean at `71dc37...` | **Unresolved gitlink candidate**; not part of this plan | Re-admit only after owner/path/consumer proof; restore parent gitlink to recorded SHA if rejected |
| `semantic-memory-mcp-transport/` | Separate clean Git repo, branch `feat/streamable-http-single-owner-20260729-v3`, HEAD `e1560a...`, origin `RecursiveIntell/semantic-memory-mcp.git`; appears untracked to parent | **Quarantine**, no flattening into parent | Re-admit only through explicit parent/submodule/package ownership decision and protocol tests |
| `semantic-memory` manifest/source edits | Current working-tree feature forwarding and lint repairs | **Candidate source changes**; focused build/test evidence exists | Preserve raw f32 path; revert only these two paths if adapter admission fails |
| `Cargo.lock` | One generated `fib-quant` dependency line under `poly-kv` | **Generated derivative**; do not stage automatically | Regenerate from the admitted manifest closure; do not hand-edit as policy |
| `_salvage_from_libraries2/` and archives | Historical/salvage material with overlapping Cargo manifests | Evidence only; do not promote as source | Re-admission requires current-source comparison and owner decision |

**Hard boundary:** Do not use `git add -A`, broad reset, cleanup, flattening, or generated-artifact staging to manufacture closure.

---

## 3. Target state and non-goals

### Target state

1. A CPU-only, feature-gated compressed candidate path is reachable through the real semantic-memory search policy.
2. Canonical raw f32 embeddings remain authoritative and unchanged.
3. The codec crate owns mathematics and wire/profile identity; PolyKV owns pool/readers/persistence; semantic-memory owns artifact admission, candidate policy, exact rerank, fallback, and semantic receipts.
4. Every accepted generation proves complete source-row/item coverage, shape/profile/digest agreement, finite quality policy, exact-fallback availability, and reader/reload integrity.
5. Approximate candidate scores are labeled approximate and are exact-reranked when policy requires it. They are never presented as exact similarity.
6. Failure is visible: stale, corrupt, missing, unsupported, or incomplete artifacts fall back to authoritative f32 with a reason in the receipt.
7. CPU quality, resident memory, metadata, reader scratch, bandwidth, latency, and fallback behavior have local dated receipts on a named workload.

### Explicit non-goals

- No replacement or deletion of canonical f32 embeddings.
- No reimplementation of FibQuant/TurboQuant mathematics in PolyKV or semantic-memory.
- No revival of `poly_kv_bridge` as a second source of semantic retrieval truth.
- No PolyKV workspace flattening or parent-workspace merge unless a separate ownership/release decision admits it.
- No MCP/Python/GPU/HF/vLLM/FlashInfer/TensorRT/CUDA/serving claim before the CPU Rust contract passes.
- No public compression, recall, latency, throughput, or production-readiness claim from synthetic fixtures or historical reports.
- No unrelated workspace Clippy cleanup inside the compressed-retrieval lane.

---

## 4. Severity-ranked implementation order

### P0-A — Establish the real compressed semantic retrieval boundary

**Evidence:** `FibQuantCandidateOnly` is explicitly `NotImplemented` in `semantic-memory/src/search.rs:1024-1026`; the existing `PolyKvBackend` module is not called by the dispatch path. The `ProveKvPoolCandidateOnly` branch loads metadata and payload but calls the brute-force f32 path at `search.rs:1051-1060`.

**Decision:** Implement one canonical path, not two. The preferred surface is `SharedKvPool::attention_topk_compressed` / `_prepared` from PolyKV, called by semantic-memory only after a validated generation is loaded. If the artifact shape is not a truthful semantic-vector pool shape, reject it and keep the policy fallback explicit rather than relabeling KV-cache behavior as nearest-neighbor retrieval.

**Acceptance:** A real test config selects the policy; a materialized generation is loaded; compressed candidate IDs are produced by the PolyKV API; exact f32 rerank returns final hits; the receipt identifies codec/profile/generation, candidate count, exact rerank, fallback, and degradation state.

### P0-B — Close PolyKV shape, profile, and reload integrity

**Evidence:** `poly-kv/src/pool.rs` currently constructs the adapter with fixed `(head_dim, 4, 32, 42)` parameters, and `semantic-memory/src/poly_kv_backend.rs:75` discards loaded blocks with `TODO: rebuild pool from loaded blocks`.

**Decision:** The admitted generation must carry the exact pool manifest/profile/shape/digest authority. The reader must reconstruct from persisted manifest plus blocks and verify the exact fallback. A fixed adapter profile is not acceptable for cross-profile artifacts.

**Acceptance:** Persist → fresh process/store open → manifest load → block verification → `SharedKvPool` reconstruction → prepared scoring → same result/receipt digest. Truncation, block digest mutation, shape mismatch, profile substitution, missing fallback, and stale generation all fail closed.

### P0-C — Make generation publication atomic and count-complete

**Evidence:** V21 and V24 generation tables exist, but current ProveKV search validates only that a row/payload can be loaded; it does not validate realized payload structure against item map, source snapshot, profile, or receipt counts before using it.

**Decision:** Treat compressed artifacts as rebuildable projections. Build into a new generation, validate all source rows and realized items, persist manifest/items/receipts in one transaction, then supersede the old generation. On any mismatch, publish a failed/quarantined status and keep the previous valid generation or exact path.

**Acceptance:** Missing/duplicate/reordered/out-of-range items, mismatched source snapshot, stale profile, count disagreement, invalid digest, and partial publication are rejected without changing the active valid generation.

### P1-A — Close workspace strict-lint debt as a separate lane

Current reproducible blockers from `cargo clippy --workspace --all-targets -- -D warnings`:

- `knowledge-runtime/src/query/classify.rs:9` — unused `thiserror::Error` import.
- `llm-pipeline/src/llm_call_tests.rs:578` — test `expect_err`; `:660` — unused `RetryStrategy` import.
- `llm-pipeline/src/backend/ollama.rs:467,552` — `expect()` in test/provider fixture paths.
- `llm-pipeline/src/backend/openai.rs:440,457,466,501,511,514-517,527-535,568` — `expect()`/`expect_err()` in test/provider fixture paths.
- `llm-pipeline/src/tool_loop.rs:986,1102-1105` — `expect()` in test/tool receipt fixture paths.
- `semantic-memory-mcp/src/server.rs:68` — manual clamp.

**Decision:** Do not mix these changes into P0 compressed retrieval. Repair them in owner-specific commits, rerun strict workspace Clippy, and retain the current failed receipt until the full command passes.

### P1-B — CPU-only measurement and claim closure

No benchmark receipt is admitted until the whole path is functional. Measure raw f32, logical serde/audit envelope, owner-controlled framed wire, exact fallback, manifest/receipt/profile bytes, pool blocks, reader scratch, resident total, candidate quality, exact-rerank quality, and latency. Label the fixture synthetic and keep representative-model validation separate.

### P2 — Downstream adapters

Only after P0 and P1 gates pass: semantic-memory-mcp feature/runtime checks, Python bindings, read-only inspection, and later accelerator/serving adapters. A feature-gated stub is not support.

---

## 5. Executable task plan

Each task must be completed in the named owner/workspace. A task is not closed by a self-report; the controller reruns its command and records the receipt.

### Phase 0 — Admission and baseline

#### Task 0.1: Freeze candidate paths and source identity

**Owner/workspace:** controller; `/home/sikmindz/Coding/Libraries` plus nested `poly-kv` and `AiDENs`.  
**Files:** no source edits; receipt under `docs/receipts/`.  
**Entry:** record branch/HEAD, scoped status, exact target file hashes, Cargo metadata for both workspaces, and nested Git roots.  
**Output gate:** source identity and dirty-boundary table match the current filesystem.  
**Abort:** if a target file is another repository, generated output, or unresolved gitlink, quarantine it rather than editing.

#### Task 0.2: Capture focused baselines

**Commands:**

```bash
cargo check -p semantic-memory --features fib-quant-codec --lib
cargo test -p semantic-memory --all-targets --features fib-quant-codec
cargo test -p semantic-memory --all-targets
cargo test --manifest-path AiDENs/Cargo.toml --all-targets
cargo clippy --workspace --all-targets -- -D warnings
```

**Expected current state:** first four commands pass; strict workspace Clippy fails on the P1-A list until that separate lane is repaired.  
**Rollback:** no source rollback; preserve the failed baseline receipt.

### Phase 1 — PolyKV owner contract

#### Task 1.1: Add an explicit admitted profile identity to compressed scoring

**Files:** `poly-kv/crates/poly-kv/src/pool.rs`, `poly-kv/crates/poly-kv/src/adapters/`, `poly-kv/crates/quant-codec-core/`; tests in `poly-kv/crates/poly-kv/tests/`.  
**RED:** construct two same-shape profiles with different seeds/codebooks and prove the wrong profile is rejected before scoring.  
**GREEN:** pass an admitted profile/manifest identity into the adapter/scorer; remove fixed hidden profile assumptions from the admitted path.  
**Commands:**

```bash
cd /home/sikmindz/Coding/Libraries/poly-kv
cargo test -p poly-kv --no-default-features --features fibquant-adapter --all-targets
cargo test -p poly-kv --all-features --all-targets
```

**Acceptance:** profile digest, shape, codebook/rotation identity, codec ID, and lossy/quality policy are receipt-bound.

#### Task 1.2: Prove full-block cardinality and role/layout handling

**RED:** feed a block containing multiple logical vectors to a one-vector-only adapter and require typed rejection or checked cardinality derivation; test key/value role mismatch.  
**GREEN:** derive a checked logical shape from the admitted manifest or reject the unsupported shape.  
**Acceptance:** no silent `len == head_dim` assumption; no key/value role coercion; all arithmetic is checked.

#### Task 1.3: Close persistence/reload before semantic integration

**Files:** `poly-kv/crates/poly-kv/src/store.rs`, `pool.rs`, `reader.rs`, persistence tests.  
**RED:** persist, drop, reopen, load, and score; mutate/truncate manifest/block/fallback and require failure.  
**GREEN:** reconstruct a `SharedKvPool` from verified persisted artifacts without duplicating pool bytes per reader.  
**Acceptance:** deterministic manifest/receipt identity and identical selected token indices across fresh process reload.

### Phase 2 — Semantic-memory generation and dispatch

#### Task 2.1: Define the semantic-vector artifact admission contract

**Files:** `semantic-memory/src/types.rs`, `semantic-memory/src/db.rs`, `semantic-memory/src/error.rs`, tests in `semantic-memory/tests/`.  
**RED:** mutate generation source snapshot, item count, item map, profile, encoding, fallback declaration, or artifact digest and require fail-closed rejection.  
**GREEN:** add a typed validator that recomputes realized counts/digests and checks the authoritative f32 snapshot.  
**Migration:** use the existing V24 generation family if its semantics fit; evolve it versionedly rather than creating a duplicate generation authority. If a new field is required, add an idempotent migration and retain old-reader rejection/compatibility behavior explicitly.

#### Task 2.2: Build one complete generation from authoritative embeddings

**Files:** `semantic-memory/src/db.rs`, `semantic-memory/src/lib.rs`, generation tests.  
**RED:** empty source, dimension mismatch, partial embedder result, duplicate item key, non-finite value, and interrupted publication each produce a failed generation with no active partial generation.  
**GREEN:** read all admitted source tables, construct the exact PolyKV input shape, retain exact fallback, write item map and receipt, then atomically activate the generation.  
**Acceptance:** source row count == item-map count == realized candidate count; raw f32 rows remain unchanged.

#### Task 2.3: Replace the current FibQuant `NotImplemented` arm only after 2.1/2.2

**Files:** `semantic-memory/src/config.rs`, `semantic-memory/src/search.rs`, `semantic-memory/src/types.rs`, search tests.  
**RED:** with no feature, no generation, stale generation, or corrupt generation, require an explicit fallback/error reason; never silently call a different backend.  
**GREEN:** feature-enabled dispatch calls `SharedKvPool::attention_topk_compressed` or `_prepared`, maps token/item IDs through the validated item map, and exact-reranks against f32 when required.  
**Acceptance:** receipt has `candidate_backend`, `codec_family`, profile/generation/manifest digests, approximate flag, candidate count, exact-rerank flag, fallback/degradations, and raw-row count.

#### Task 2.4: Keep `poly_kv_backend` and bridge boundaries honest

**Files:** `semantic-memory/src/poly_kv_backend.rs`, `semantic-memory/src/poly_kv_bridge.rs`, `semantic-memory/src/lib.rs`.  
**Decision gate:** either make the backend a tested canonical implementation used by the dispatch, or mark it migration-only/deprecated and remove its misleading active surface in a separate compatibility change. Do not keep two implementations with divergent persistence semantics.  
**RED:** restart test against the current `load()` path must fail or expose the missing rebuild; bridge tests must continue rejecting semantic-embedding-to-KV relabeling.  
**GREEN:** only after the canonical owner is selected and tested; otherwise preserve the fail-closed bridge and leave the backend quarantined.

### Phase 3 — Integrity, fallback, and receipts

#### Task 3.1: Add hostile logical corruption matrix

**Files:** `semantic-memory/tests/`, `poly-kv/crates/poly-kv/tests/`.  
**Cases:** missing/duplicate/reordered items; overlapping/out-of-range pages; shape/profile substitution; source/codebook/rotation digest substitution; count and fallback-reason mismatch; overflow; truncated/trailing bytes; unknown version/flags.  
**Acceptance:** all cases fail closed even when attacker-controlled local digests are recomputed.

#### Task 3.2: Add exact fallback and stale-generation tests

**Cases:** unavailable codec feature, quality budget breach, missing fallback, stale source snapshot, failed generation, invalidated generation, reader scratch ceiling, and exact-rerank disabled when policy requires it.  
**Acceptance:** authoritative f32 results remain available where policy allows; receipt names the reason; no degraded result is labeled exact or verified.

#### Task 3.3: Add durable receipt replay and digest tests

**Files:** `semantic-memory/src/types.rs`, receipt persistence code, `semantic-memory/tests/search_tests.rs` and generation tests.  
**Acceptance:** receipt serialization is deterministic by parsed-value comparison, replay identifies generation/profile/source digests, and privacy-sensitive query inputs remain opt-in.

### Phase 4 — CPU measurement and downstream admission

#### Task 4.1: Implement the whole-path CPU benchmark

**Owner:** `quant-eval` / `receipt-bench`, consuming owner APIs; no codec math duplication.  
**Measure:** raw f32, logical JSON/audit envelope, framed wire, exact fallback, manifest, profile/codebook/rotation, receipts, pool blocks, reader scratch, total resident, candidate quality, exact-rerank quality, and median/p95 latency.  
**Acceptance:** dated machine-readable receipt names workload, shape, host, feature profile, command, raw samples, all byte categories, fallback, and claim boundary. Synthetic numbers remain fixture-local.

#### Task 4.2: Admit semantic-memory-mcp only after Rust closure

**Commands after core closure:**

```bash
cd /home/sikmindz/Coding/Libraries/semantic-memory-mcp
cargo fmt --all -- --check
cargo check --all-features
cargo test --features full
python3 integrations/tests/validate_integrations.py
```

Then build the actual binary and inspect MCP `tools/list` for each runtime profile. Cargo feature `full` and runtime `--tool-profile full` must be reported separately.  
**Acceptance:** downstream runtime proves only the capabilities actually registered; no MCP claim is inferred from static Cargo features.

#### Task 4.3: Defer Python/GPU/serving adapters

**Entry gate:** Phase 3 receipts and Phase 4 CPU receipt pass; exact rollback and quarantine are documented.  
**Acceptance:** each adapter gets its own ABI/runtime/process-boundary test. A compile-only feature stub is explicitly recorded as unsupported.

---

## 6. Separate repair lane: workspace Clippy

This lane is intentionally independent from compressed retrieval.

1. `knowledge-runtime/src/query/classify.rs:9`: remove or correctly gate the unused import; run package Clippy.
2. `llm-pipeline/src/llm_call_tests.rs:660`: remove unused `RetryStrategy`; use assertions that do not trigger `expect_used` where the test contract requires typed failure.
3. `llm-pipeline/src/backend/ollama.rs` and `openai.rs`: replace fixture/provider `expect` calls with typed assertions or narrowly scoped test-only lint policy; do not weaken production error handling.
4. `llm-pipeline/src/tool_loop.rs`: replace test receipt `expect` chains with explicit assertions preserving the intended failure message.
5. `semantic-memory-mcp/src/server.rs:68`: use `.clamp(20, MAX_SEARCH_FETCH_K)` after confirming the bounds are ordered.
6. Re-run `cargo clippy --workspace --all-targets -- -D warnings`; record all diagnostics, not a truncated tail.

**Lane gate:** no compressed-retrieval task is marked complete based on this lane; no workspace-wide green claim is made until this exact command passes.

---

## 7. Verification matrix

| Gate | Command | Required state |
|---|---|---|
| Formatting | `cargo fmt --all -- --check` from each applicable workspace | pass; explicit nested workspace included |
| Main focused compile | `cargo check -p semantic-memory --features fib-quant-codec --lib` | pass |
| Main focused tests | semantic-memory default and feature all-target tests | pass; zero-test feature-gated files reported honestly |
| PolyKV feature-off | `cargo test -p poly-kv --no-default-features --all-targets` | pass or typed unsupported result |
| PolyKV feature-on | `cargo test -p poly-kv --no-default-features --features fibquant-adapter --all-targets` | pass |
| PolyKV all features | `cargo test -p poly-kv --all-features --all-targets` | pass |
| PolyKV strict lint | `cargo clippy -p poly-kv --all-features --all-targets -- -D warnings` | pass |
| Semantic-memory strict lint | `cargo clippy -p semantic-memory --all-targets --features fib-quant-codec -- -D warnings` | pass |
| Main workspace strict lint | `cargo clippy --workspace --all-targets -- -D warnings` | pass; currently blocked by P1-A |
| Main workspace tests | `cargo test --workspace --all-targets --no-fail-fast` | fresh post-edit receipt required |
| AiDENs regression | `cargo test --manifest-path AiDENs/Cargo.toml --all-targets` | pass; separate workspace |
| MCP runtime | `tools/list` for each profile plus integration validator | pass only after core closure |
| Artifact integrity | hostile corruption/reload/replay suite | pass |
| Benchmark | CPU whole-path receipt | observed locally; no universal claim |

A command run before the final source edit is stale for the final gate.

---

## 8. Claim boundary

### Safe after the current focused evidence

- The current semantic-memory source compiles and its all-target tests pass under the `fib-quant-codec` feature profile.
- The current PolyKV compressed-scoring method exists in the nested source and resolves through the semantic-memory feature dependency.
- AiDENs' separate JSON-boundary test suite passes 28 tests.
- The root currently contains 64 Cargo packages and AiDENs contains a separate 34-package workspace.

### Not yet safe to claim

- FibQuant candidate generation is integrated into semantic-memory search.
- PolyKV persistence/reload is complete for semantic-memory use.
- Compressed artifacts are integrity-admitted across all source rows and fallback paths.
- Compressed retrieval improves recall, latency, throughput, or resident memory.
- Any universal compression ratio or production readiness.
- MCP/Python/GPU/HF/vLLM/FlashInfer/TensorRT/CUDA support.
- Workspace-wide Clippy is green.
- The root tree is clean or publish-ready.

---

## 9. Rollback and quarantine

- Preserve canonical f32 embeddings and the existing `Disabled`/exact retrieval policy as the always-available recovery path.
- If a generation fails validation, mark it failed/invalidated with a reason and do not supersede the last valid generation.
- If reader reload or exact-rerank parity fails, disable the compressed policy and retain the artifact for diagnosis; do not delete it or silently select another backend.
- Revert source only by explicit allowlist (`semantic-memory/Cargo.toml`, `semantic-memory/src/poly_kv_backend.rs`, and later admitted paths); never reset unrelated dirty paths.
- Revert a generated `Cargo.lock` delta only by regenerating from the admitted manifest closure, not by manual line editing.
- Keep `cea-bridge` and `semantic-memory-mcp-transport/` outside this plan until ownership and consumer admission are explicit.
- Any publication, commit, parent gitlink update, or remote push is a separate authority step and is not implied by this plan.

---

## 10. Closeout handoff contract

A phase receipt must contain: task ID; evidence cutoff; source owner/root; branch/HEAD; scope and non-goals; changed paths; exact commands and exit codes; artifact paths/digests; evidence state; passed/failed/skipped checks; unresolved delta; rollback/quarantine; next gate; and `complete: true` only when every required gate passes.

The final Libraries handoff must include:

1. current root and nested-workspace identities;
2. exact source/test paths changed by the admitted implementation;
3. all commands rerun after the last source edit;
4. current focused and full-workspace results;
5. artifact and receipt digests;
6. explicit separation of verified, source-observed, historical, inferred, and blocked claims;
7. reconciliation rows for every dirty Git root/gitlink/untracked candidate;
8. an auditor-rerunnable command list and rollback procedure.

**Plan status at this cutoff:** the plan artifact is complete; implementation remains conditional on the P0-A/P0-B/P0-C vertical slice and current workspace closure gates. No release or publication claim is licensed.
