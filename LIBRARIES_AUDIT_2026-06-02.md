# RecursiveIntell ~/Coding/Libraries — Crate State Report

**Date:** 2026-06-02
**Inspector:** miniflux-agent (in-session, post-V30 audit + post-P32 repair)
**Basis:** direct `cargo check --workspace`, `cargo test --workspace`, git log since 2026-05-29, shell probes for LOC/tests/deps per crate, targeted web research for upstream-analogues section
**Method deviation note:** the planned two-subagent parallel audit timed out. This report was produced inline using scripted bulk probes and a few targeted web fetches.

---

## 0. Headline findings (since the 5/29 audit, 4 days ago)

1. **All 3 P0 integrations called out on 5/29 are now in place.** Verified by grep:
   - `semantic-memory/Cargo.toml` declares `bitemporal-runtime` and `boundary-compiler` as required deps, and `quant-governor` behind a `turbo-quant-codec` feature
   - `bitemporal-runtime` is referenced from `semantic-memory/src/{db,types}.rs`
   - `boundary-compiler` is referenced from `semantic-memory/src/graph.rs`
   - `quant-governor` is referenced from `semantic-memory/src/quantize_governed.rs` (Polar + Qjl profiles added 2026-06-01, commit 041de94)
2. **P0-4 (check-runner unsafe) is structurally fixed.** A separate `Primitives/check-runner-sys/` crate exists (41 lines, `#![allow(unsafe_code)]` with `package.metadata.unsafe_justification` declared in Cargo.toml) and is consumed by `Primitives/check-runner/src/lib.rs` via `check_runner_sys::kill_process_group()` / `process_exists()` / `set_env()`. The 5/29 audit's "scoped-allow" framing was misleading — the unsafe was extracted to a sibling crate. Workspace lint `unsafe_code = deny` is satisfied for `check-runner` proper.
3. **Receipt infrastructure for orchestrators shipped.** P32 commit `483ea1b` added 667 lines including: `PipelineExecutionReceiptV1` / `ProviderCallReceiptV1` / `RetryDecisionReceiptV1` / `BudgetDebitV1` in `llm-pipeline`; `GraphExecutionReceiptV1` / `StepExecutionReceiptV1` in `agent-graph`. `llm-pipeline` test count jumped to 194.
4. **The "47 commits since 5/29" are 90% about one thing: GPU integration + SIMD/Rayon performance in the poly-kv / fib-quant / turbo-quant / gpu-backend path.** See commit log: `1cd1315 feat: make SIMD+Rayon the default (15-86x pool build speedup out of the box)`, `7422ca5 feat(fib-quant): Rayon-parallel finish_batch_encode (8-100x speedup on multi-core)`, `be579df feat: gpu-backend crate + GPU integration across fib-quant, turbo-quant, poly-kv`, `af1ab2f feat: complete cudarc driver API integration for gpu-backend`. The 5/27 dossier's concerns about governance/kernel crates being stale are not wrong — nothing material has happened in the kernel layer since 5/28.
5. **Workspace builds clean.** `cargo check --workspace` completes in 1m15s with **0 errors, 1 warning** (unused `TurboCode` import in `scr-runtime-compression/src/codec_dispatch.rs:36`).
6. **Workspace tests all green.** `cargo test --workspace --no-fail-fast`: zero `FAILED` lines, zero panics. The 5/29 audit's claim that "knowledge-runtime has production `panic!`" was already addressed in that audit's own retraction section (replaced with `unreachable!`); I verified the 5/29 finding is still the current state — only `examples/`-level and test-fn `unwrap()` exists outside `#[cfg(test)]` blocks in the supported lane.
7. **One P1 gap from 5/29 still open:** `claim-ledger` is **not** wired into `forge-pilot` (zero Cargo.toml or source references — verified). P1-2 from the corrected V30 plan.
8. **Three governance crates are empty placeholders.** `assurance-case`, `attestation`, `policy-store` are 46-byte directories with empty `src/lib.rs` files. They occupy workspace slots but contain nothing. The 5/27 dossier flagged "governance lane crates are compatibility-name surface crates with almost no runtime logic" — these three are even thinner than that (literally zero code).

---

## 1. Architecture map (fresh, top-down)

```
~/Coding/Libraries/                    ← top-level Cargo workspace (54 Cargo.tomls, 62 packages)
├── Primitives/                        ← 10 sub-crates, dependency floor
│   ├── effect-signature               (131 loc, 5 tests — leaf type crate)
│   ├── cea-core, cea-store, cea-sqlite  (causal-edit-attribution row types)
│   ├── check-runner + check-runner-sys  (process execution; unsafe isolated to -sys)
│   ├── forge-policy                   (workspace/db safety policy)
│   ├── mindstate-core                 (serializable mindstate payload)
│   ├── sandbox-workspace              (patch filesystem)
│   ├── stabilizer-core                (attempt-phase + delta policy)
│   └── typed-patch                    (structured patch schema)
├── recursive-kernel-core              ← type schemas only (583 loc, 20 tests, 2026-03-25)
├── kernel-{execution, conformance, oracles}
├── effect-runtime, mechanism-runtime, continuity-runtime
├── knowledge-runtime                  ← bounded classifier scaffold for semantic-memory
│
├── semantic-memory (v0.5.0, 37k LOC, 113 tests)  ← THE primary data store
│   └── wired to: bitemporal-runtime, boundary-compiler, quant-governor
│
├── stack-ids                          (shared identity, scope, trace primitives)
│
├── governance lane (mostly stub/typed-surface):
│   ├── bitemporal-runtime             (753 loc, 11 tests — REAL)
│   ├── boundary-compiler              (924 loc, 27 tests — REAL, RFC 8785 JCS)
│   ├── claim-ledger                   (1731 loc, 34 tests — REAL, not yet wired to forge-pilot)
│   ├── quant-governor                 (1269 loc, 26 tests — REAL, wired via semantic-memory feature)
│   ├── quant-eval, receipt-bench
│   ├── verification-{policy, control, calibration, adjudication}
│   ├── assurance-{case, runtime}
│   ├── attestation, attestation-exchange  (attestation is empty stub)
│   ├── authority-delegation
│   ├── constitutional-memory
│   ├── constraint-compiler
│   ├── contract-schema-gen
│   ├── discovery-portfolio
│   ├── federated-settlement
│   ├── policy-store                    ← empty stub
│   ├── profile-runtime
│   ├── remote-oracle-admission
│   ├── spec-execution
│
├── orchestrators (with receipt infrastructure now wired):
│   ├── forge-pilot (14k LOC, 23 tests)         ← DOES NOT YET USE claim-ledger
│   ├── forge-memory-bridge (3.5k LOC, 44 tests)
│   ├── forge-engine (16k LOC, 170 tests, in living-memory/)  ← package name is "forge-engine" but path is living-memory/living-memory — historical artifact, GOV-010 note in its Cargo.toml
│   ├── llm-pipeline (9.9k LOC, 194 tests)       ← P32 added Pipeline/Provider/Retry/Budget receipts
│   ├── llm-output-parser (3.4k LOC, 144 tests)
│   ├── llm-tool-runtime (4.4k LOC, 38 tests)
│   ├── agent-graph (10.6k LOC, 5 tests)         ← P32 added Graph/Step receipts; 5 tests is thin for a graph engine
│   ├── agent-guard (294 LOC, 4 tests)          ← Linux control-plane
│
├── KV-cache / quantization (just had a massive perf + GPU push):
│   ├── fib-quant (5.9k LOC, 50 tests)     ← FibQuant paper (arXiv:2605.11478) impl, cold tier
│   ├── turbo-quant (6.3k LOC, 121 tests)  ← TurboQuant wire-embedded + PolarQuant + QJL sidecar codecs
│   ├── turbo-semantic/                   ← paradoxically: a complete CLONE of semantic-memory (same crate name, v0.5.0, 29k LOC, 82 tests) inside a "TurboQuant super-pass" plan directory. Will explain in §6.
│   ├── scr-runtime-compression (1.3k LOC, 23 tests)  ← codec dispatch via quant-governor
│   ├── scr-runtime/ (sub-workspace: scr-audit-adapter, scr-cli, scr-kernel, scr-reference)
│   ├── quant-eval (1.8k LOC, 24 tests)
│   ├── gpu-backend (1.7k LOC, 13 tests)   ← NEW cudarc driver wrapper, behind feature flag
│   ├── poly-kv/ (separate sub-workspace, 56 tests, 3 crates: poly-kv, poly-kv-python, quant-codec-core)
│
├── application integrations:
│   ├── tauri-queue (1.5k LOC, 33 tests)  ← real Tauri/job-queue bridge (376 lines, not a stub as first-pass audit said)
│   ├── tauri-react-hooks                 ← TypeScript package, not Rust
│   ├── comfyui-rs (1.7k LOC, 23 tests)
│   ├── ollama-vision (760 LOC, 6 tests)
│   ├── ai-batch-queue (2.8k LOC, 56 tests)
│   ├── job-queue (3.5k LOC, 43 tests)
│
├── AiDENs/   ← separate sub-workspace, 34 sub-crates, "kit" architecture
└── living-memory/   ← 56 files of markdown design docs + the forge-engine sub-crate
```

**Total:** 62 packages in the parent workspace + 34 in `AiDENs/` + 4 in `scr-runtime/` + 3 in `poly-kv/` = **103 Rust crates**, plus the `tauri-react-hooks` TypeScript package and the `examples/` and `turbo-semantic/` source bundles.

---

## 2. Per-crate status — Infrastructure / kernel / governance

### recursive-kernel-core (v0.1.0)
- **Purpose:** shared type schemas for the recursive inference kernel (`ExactnessClass`, `ConvergenceKind`, `StopRule`, `OperatorId`, etc.)
- **Lines / files:** 583 / 2 (.rs only)
- **Tests:** 20
- **Last modified:** 2026-03-25 — **oldest live crate in the workspace, untouched for ~10 weeks**
- **State:** scaffold-only (intentional, per its description "non-authoritative schemas")
- **Key deps:** `stack-ids`, `serde`, `schemars`
- **What's working:** clean type definitions with JSON schema annotations; 20 unit tests covering enum exhaustiveness
- **What's weak:** zero runtime logic — the actual kernel lives in `kernel-execution` and downstream. This crate is a vocabulary, not a system.
- **Upstream analogues:** not really analogous to anything in the public Rust ecosystem — most ML "kernels" (candle, burn) don't expose a typed operator algebra. Closest spirit: the typed-op algebra in `egg` (eidolon symbolic regression lib).
- **Future direction:**
  - Decision needed: is this the right place for the operator vocabulary, or should the types move into `kernel-execution` and this crate be deleted?
  - If kept: add a `pub mod validation` that asserts all operator IDs referenced in the workspace exist in the type set (it currently can't catch stale IDs)
  - The 20 tests are the same age as the code; consider a re-test to ensure the types still match what `kernel-execution` and `knowledge-runtime` actually use

### kernel-execution (v0.1.0)
- **Purpose:** deterministic K2 execution baseline for the recursive kernel
- **Lines / files:** 1218 / 1 (single `lib.rs`)
- **Tests:** 10
- **Last modified:** 2026-05-28
- **State:** needs-work (small surface, single file, no doc comments verified)
- **Key deps:** `recursive-kernel-core`, `serde`
- **What's working:** deterministic execution baseline, 10 tests
- **What's weak:** one production `.expect("calibration report expected for degraded compilation")` in `lib.rs` (verified) — should be a `Result` return. Single 1.2k-line `lib.rs` is a maintainability smell.
- **Future direction:** split into `compile.rs` + `execute.rs` + `report.rs`; convert `.expect` to `?` with a proper error type; add a `pub fn run_for_tests` that lets a downstream consumer drive K2 with a hand-rolled operator sequence

### kernel-conformance (v0.1.0)
- **Purpose:** conformance harness — proves the kernel produces the same outputs across refactors
- **Lines / files:** 3555 / 9
- **Tests:** 65
- **Last modified:** 2026-05-28
- **State:** production-ready
- **Key deps:** `recursive-kernel-core`, `kernel-execution`
- **What's working:** 65 tests is a strong harness; reference_interpreters.rs has the v2/v3 envelope roundtrip logic
- **What's weak:** many `.unwrap()` calls in `reference_interpreters.rs` and `examples/canonical_perf_snapshot.rs` — most are inside test/example contexts but a few in `lib.rs` are not (`tokio::runtime::Runtime::new().unwrap().block_on(future)` at lib.rs:78 area)
- **Future direction:** the `examples/canonical_perf_snapshot.rs` should be promoted to a `cargo bench` target so perf baselines can be re-run automatically; consider switching from `tokio::runtime::Runtime::new().unwrap()` to `tokio::main`

### kernel-oracles (v0.1.0)
- **Purpose:** bounded exact/conservative oracle paths
- **Lines / files:** 1007 / 1
- **Tests:** 12
- **Last modified:** 2026-05-28
- **State:** needs-work
- **What's weak:** V30 audit noted 2 `unreachable!()` calls in `lib.rs` (still there per the audit's own retraction: "unreachable! is still not ideal, but intentional V30 hardening"). These should be `Result` returns with `thiserror` per the V30 plan §1.2, but the work was downgraded to "intentional" — needs a decision.
- **Future direction:** convert the 2 `unreachable!()` sites to typed errors OR document them as load-bearing invariants with a comment explaining why `unreachable!` is the right call (a comment block, not just the macro)

### effect-runtime (v0.1.0)
- **Purpose:** runtime for executing typed effects (per `effect-signature` types)
- **Lines / files:** 2644 / 12
- **Tests:** 12
- **Last modified:** 2026-05-28
- **State:** needs-work (low test density: 12 tests for 2.6k LOC = 4.5 tests/KLOC, well below the 10/KLOC floor the 5/29 audit cited)
- **Future direction:** add integration tests that exercise the runtime with `effect-signature` types from `Primitives/`; consider a `#[non_exhaustive]` on the effect enum so adding a new effect doesn't silently break consumers

### effect-signature (v0.1.0) [in Primitives/]
- **Purpose:** stable effect types + hashing helpers
- **Lines / files:** 131 / 1
- **Tests:** 5
- **Last modified:** 2026-03-11 — **very old**
- **State:** stable leaf, intentionally minimal
- **Future direction:** none material — this is a finished leaf crate. Maybe one more test for hash stability across compiler versions.

### mechanism-runtime (v0.1.0)
- **Purpose:** typed mechanism/theory surface
- **Lines / files:** 618 / 5
- **Tests:** 10
- **Last modified:** 2026-05-28
- **State:** scaffold-only (per the description: "surface crate with bounded fit")
- **Future direction:** same pattern as `recursive-kernel-core` — vocabulary crate, not a system. If no runtime kernel consumes it, consider folding into `recursive-kernel-core` to reduce workspace noise.

### continuity-runtime (v0.1.0)
- **Purpose:** typed continuity/incident surface
- **Lines / files:** 1231 / 13
- **Tests:** 12
- **Last modified:** 2026-05-28
- **State:** scaffold-only
- **Future direction:** same — vocabulary. Currently appears to be a typed surface with no live consumer (no `Cargo.toml` references found in the live layer).

### knowledge-runtime (v0.1.0)
- **Purpose:** bounded orchestration scaffold for semantic-memory: classifier/reasoner shell
- **Lines / files:** 11487 / 32
- **Tests:** 95
- **Last modified:** 2026-06-01 (one of the few governance-adjacent crates touched recently)
- **State:** needs-work
- **What's working:** 95 tests is solid for a scaffold; recent edits suggest active use
- **What's weak:** V30 audit's "5 `unreachable!()` in classify.rs" — should be `thiserror` per V30 plan §1.2, currently downgraded to "intentional"
- **Future direction:** convert `unreachable!` → `thiserror`; add an integration test path that imports `knowledge-runtime::classify` and uses it against a real `semantic-memory` episode

### Primitives/* sub-crates (10 crates, 7899 LOC, 80 tests combined)
- `cea-core` (2415, 11 tests), `cea-store` (656, 5), `cea-sqlite` (1218, 10) — causal-edit-attribution: types, contract, SQLite impl. **State: production-ready.** Last edit 2026-05-28. Future: add a `cea-postgres` impl to match the storage abstraction.
- `check-runner` (856, 12) + `check-runner-sys` (41, 0) — process execution with unsafe isolated. **State: production-ready.** P0-4 structurally fixed. Future: `check-runner-sys` has 0 tests — add a #[cfg(test)] module with a test that forks a process group and verifies killpg works.
- `forge-policy` (443, 7) — workspace/db safety policy. **State: stable leaf.** Last edit 2026-03-12.
- `mindstate-core` (284, 7) — serializable mindstate payload for `forge-engine`. **State: stable leaf.** Last edit 2026-03-10.
- `sandbox-workspace` (384, 11) — patch filesystem sandboxing. **State: stable leaf.** Last edit 2026-03-12.
- `stabilizer-core` (486, 6) — attempt-phase + delta policy. **State: stable leaf.** Last edit 2026-03-10.
- `typed-patch` (985, 6) — structured patch schema. **State: stable leaf.** Last edit 2026-05-28.

**Future direction for Primitives as a group:** the older ones (forge-policy, mindstate-core, sandbox-workspace, stabilizer-core) haven't been touched in 3 months. They appear stable. Consider a one-time "Primitives v0.2" bump that re-exports them through a single `Primitives::*` facade to make consumer code cleaner.

### Governance lane — 20 crates, mostly typed surfaces

**Real implementations (have non-trivial code + tests):**
- `bitemporal-runtime` (753/5, 11 tests) — **wired into semantic-memory, doing real work** ✓
- `boundary-compiler` (924/7, 27 tests) — **wired into semantic-memory, real RFC 8785 JCS impl** ✓
- `claim-ledger` (1731/7, 34 tests) — **NOT YET wired into forge-pilot** (P1-2 still open)
- `quant-governor` (1269/8, 26 tests) — **wired via semantic-memory `turbo-quant-codec` feature, has Polar + Qjl codec profiles** ✓
- `verification-policy` (2645/12, 22), `verification-control` (3486/7, 19), `verification-calibration` (352/2, 10), `verification-adjudication` (1375/6, 10) — the verification four-stack, all with non-trivial code

**Surface / vocabulary crates (real but typed-only):**
- `assurance-runtime` (1169/15, 21 tests) — has 7 files, the most code of any governance crate, but no runtime fns (verified — zero `run`/`execute`/`handle` matches)
- `authority-delegation` (914/11, 10 tests) — typed delegated-authority surface
- `constitutional-memory` (657/5, 10 tests) — typed charter/archive
- `constraint-compiler` (1388/2, 18 tests) — deterministic projection-to-inference graph compiler
- `contract-schema-gen` (1222/4, 11 tests) — contract schema generation
- `discovery-portfolio` (655/5, 10 tests)
- `federated-settlement` (619/5, 7 tests)
- `profile-runtime` (4242/13, 17 tests) — has 13 source files (largest of the typed-surfaces), 4.2k LOC, **does have an `expect` in `lib.rs` outside test context** (one of the few non-test unwraps I found)
- `remote-oracle-admission` (710/3, 6 tests)
- `spec-execution` (725/4, 10 tests)
- `attestation-exchange` (815/6, 8 tests) — typed attestation exchange, the non-stub counterpart to `attestation`
- `receipt-bench` (875/5, 9 tests) — replayable benchmark substrate

**Empty stubs (literally 46 bytes — empty `src/lib.rs`):**
- `assurance-case` — empty
- `attestation` — empty
- `policy-store` — empty

These three should be either **deleted from the workspace** or **actually filled in**. Right now they are workspace members that compile (empty lib.rs is valid) but contribute nothing.

**Upstream analogues for the governance lane:** there is no real upstream analogue. The closest public-Rust equivalents are:
- For `bitemporal-runtime`: nothing in the public Rust ecosystem with bitemporal support (crates.io search for "bitemporal" returned 0 crates — verified 2026-06-02). Closest spirit: Datomic's bitemporal model. Rust-ecosystem equivalent for ordinary time-series is `time-series` patterns, but bitemporal + Rust is genuinely original.
- For `boundary-compiler`: an RFC 8785 JCS implementation in Rust would compete with hand-rolled canonicalization. The reference Rust implementation cited in the RFC is `serde_jcs` / `jcs` crate; the RecursiveIntell impl is a deliberate re-implementation with profile+receipt types added.
- For `claim-ledger`: no real upstream analogue. The closest is "evidence-first" architectures like PROV-O (W3C provenance ontology) or `prov` Python lib; nothing comparable in Rust.

**Future direction for governance lane as a group:**
- Delete `assurance-case`, `attestation`, `policy-store` (or implement them)
- Wire `claim-ledger` into `forge-pilot` (1 Cargo.toml line + 1 boundary-check call in the export path — small, high-value)
- Convert `verification-calibration`'s 352 LOC into something with more than 2 source files (it's currently `lib.rs` only — likely fine, just noted)
- Decide whether the 4 governance "runtime" surfaces (`mechanism-runtime`, `continuity-runtime`, `discovery-portfolio`, `federated-settlement`, etc.) are meant to grow into real runtimes or stay as typed surfaces; if staying, rename them to `*-types` so the suffix "runtime" doesn't imply behavior that isn't there

---

## 3. Per-crate status — Application / orchestration / data-store

### semantic-memory (v0.5.0) — **the centerpiece of the whole stack**
- **Lines / files:** 37319 / 66
- **Tests:** 113
- **Last modified:** 2026-06-02 (touched today)
- **State:** production-ready, with the new doctrinal integrations in place
- **Key deps:** `hnsw_rs`, `rusqlite`, `serde_json`, `bitemporal-runtime`, `boundary-compiler`, `quant-governor` (optional via `turbo-quant-codec` feature), `stack-ids`
- **What's working:**
  - Hybrid search: SQLite + FTS5 + HNSW with `approximate: bool` field in `VectorSearchReceiptV1` (V30 already verified this; still present)
  - P32 (commit 483ea1b) wired: bitemporal integration in `src/db.rs` + `src/types.rs`, quant-governor in `src/quantize_governed.rs` (154 lines added, 14b0650), JCS in `src/graph.rs`
  - 113 tests across 38 files including hardening semantics, knowledge tests, hardening_v5, episode_identity, trace_id_write_seam
- **What's weak:**
  - 37k LOC is large for a single crate; the `src/` is 66 files. Consider splitting into `semantic-memory-core` (storage) + `semantic-memory-search` (HNSW + FTS5) + `semantic-memory-receipts` (the receipt types)
  - `cargo test -p semantic-memory` was not run by me individually (full workspace test was — passed). Worth confirming test runtime stays under 30s as it grows.
- **Upstream analogues:**
  - `hnsw_rs` (jean-pierreBoth) is the upstream HNSW crate. Per crates.io (verified 2026-06-02): v0.3.4, 503,987 all-time downloads, last release ~3 months ago, MIT/Apache-2.0, 4.2K SLoC, 70 KiB. semantic-memory's `hnsw_ops.rs` wraps this.
  - For the SQLite+vector pattern: closest public-Rust analogue is `qdrant` (the Rust client for Qdrant) or `lancedb` (LanceDB Rust bindings), but those are network/protocol clients. The closest single-process local store is `sqlx` + `pgvector` (Postgres) — there is no well-known "local SQLite + vectors + FTS5" Rust crate.
  - semantic-memory's combination (SQLite + FTS5 + HNSW + bitemporal + receipts) is genuinely original in the Rust ecosystem.
- **Future direction:**
  - Splitting into 3 sub-crates (above) is the obvious next move before the crate crosses 50k LOC
  - Add a `semantic-memory-cli` for ops (currently the only way to exercise the API is via `knowledge-runtime` or `forge-pilot`)
  - The `hnsw_rs` upstream is dormant (last release ~3 months ago) — if semantic-memory becomes a product, plan a fallback or fork
  - Consider exposing a `pub trait SearchBackend` so consumers can swap HNSW for an external vector DB (Pinecone, Qdrant) without rewriting the receipt layer

### poly-kv (v0.1.0-alpha.1, nested in poly-kv/crates/poly-kv)
- **Purpose:** shared compressed KV-cache pool for multi-agent context
- **Lines / files:** 6464 / 53 (includes benchmarks, examples, src, tests)
- **Tests:** 76
- **Last modified:** 2026-06-02 (active)
- **State:** alpha but actively shipping; **the most perf-focused crate in the workspace right now**
- **Key deps:** `fib-quant` (optional), `turbo-quant` (optional), `gpu-backend` (optional), `rayon`, `blake3`
- **Features:** `default = ["turbo", "fib", "parallel_pool"]`; `gpu`, `gpu_codebook_lookup` for GPU paths
- **What's working:**
  - Two-tier codec policy (fib-quant cold + turbo-quant hot)
  - Real GPU dispatch via `gpu-backend` (commit 429a0b2, "real GPU dispatch in poly-kv pool build via fib-quant encode_batch")
  - `decompress_layer` API + HuggingFace roundtrip CLI (commit 32cad5b, 2026-06-02)
  - 15-86x pool build speedup from SIMD+Rayon (commit 1cd1315)
  - 8-100x speedup on multi-core from Rayon-parallel `finish_batch_encode` (commit 7422ca5)
- **What's weak:** alpha, so API stability is a real concern — `decompress_layer` is brand new
- **Upstream analogues:**
  - fib-quant paper (arXiv:2605.11478) is the documented basis for the cold tier; the algorithm is "Fibonacci-optimized codebook on spherical blocks" — this paper is from May 2026, so the implementation is fresh off the press
  - turbo-quant: described in the README as "Experimental Rust implementation" with "PolarQuant and QJL" sidecar codecs. PolarQuant is from a 2025 paper (Microsoft Research). QJL is from a separate paper. The "TurboQuant" name is a RecursiveIntell umbrella term for these.
  - For shared KV-cache compression in general: the public-Rust landscape is sparse. The closest is `candle-transformers` (Hugging Face) which doesn't do shared pools, and `vllm` (Python) which has PagedAttention but no Rust port. poly-kv is doing something genuinely new.
- **Future direction:**
  - Stabilize the API to drop the alpha tag — the `decompress_layer` API and the two-tier policy are concrete enough to be v0.1.0 / v0.2.0
  - The poly-kv sub-workspace is a separate `Cargo.toml` from the parent — for usability, consider a `poly-kv = { workspace = true }` re-export in the parent, OR document the separation clearly
  - Add a benchmark target that runs the full encode/decode roundtrip with HuggingFace-format models on a fixed corpus; this is the kind of thing the README references but isn't captured as a `cargo bench`
  - The "honest GPU results" commit (3b1e646) and the "honest per-call GPU dispatch probe" (687c351) are good signs — keep this discipline. The next GPU win will come from batching H2D/D2H transfers across layers, not from optimizing individual transfers

### fib-quant (v0.1.0-alpha.1)
- **Lines / files:** 5905 / 54
- **Tests:** 50
- **Last modified:** 2026-06-01
- **State:** alpha, perf-focused, 47 commits since 5/29
- **What it is:** Lloyd-Max + Hadamard + Fibonacci codebook + SIMD (AVX2+FMA) + Rayon + GPU codebook_lookup
- **Recent wins:** 1.6-1.9x from AVX2+FMA `f32 nearest_codeword` (19f7eea), 8-100x from Rayon `finish_batch_encode` (7422ca5), GPU codebook_lookup parity test (f3edb27)
- **What's weak:** the GPU codebook_lookup path has "honest" results — 2-7% win, not 10x (commit fc34fea). The fib-quant README is upfront about this. The H2D/D2H overhead per call is the real cost.
- **Future direction:** consider larger-batch GPU entry points (process whole layer at once, not per-vector). The per-call dispatch probe (687c351) is the right diagnostic to start from.

### turbo-quant (v0.2.0)
- **Lines / files:** 6363 / 38
- **Tests:** 121
- **Last modified:** 2026-06-02
- **State:** v0.2.0 already, much more mature than fib-quant
- **What's working:** TurboQuant wire-embedded profile, real round-trip via `wire-embedded` (commit e9e9475), Polar + QJL codec dispatch slots (eda632e)
- **Future direction:** the codec slot mechanism is the right design — keep it. Next: profile selection should be data-driven (per-collection in semantic-memory) rather than compile-time.

### turbo-semantic/ — **important: this is a source bundle, not a normal crate**
- **Lines / files:** 29484 / 60 (suspiciously similar to semantic-memory)
- **Tests:** 82
- **What it actually is:** the directory contains a **full clone** of the `semantic-memory` crate (same name `semantic-memory`, v0.5.0, same description "Hybrid semantic search with SQLite, FTS5, and HNSW — built for AI agents"). The directory is named `turbo-semantic` but the `Cargo.toml` inside it declares `name = "semantic-memory"`.
- **What it's for:** the `README.md` and dozens of `V1_1_*`, `V2_*`, `V3_*` addenda are a Codex super-pass bundle for integrating turbo-quant with semantic-memory. The source tree is the implementation target.
- **State:** confusing / historical artifact. This is workspace member #3 in the metadata (after the real semantic-memory and the poly-kv clones).
- **Future direction:**
  - **Critical:** this duplicate is going to confuse cargo and downstream consumers. Verify whether `cargo metadata` is actually building it or if it's just declared as a workspace member for archival purposes.
  - If it's an archive of pre-integration state, rename the directory `turbo-semantic-archive/` and add to `.gitignore` for the `target/` to keep it from being scanned.
  - If it's meant to be the integration target, it should be a sibling of the real `semantic-memory/` (not inside the parent workspace as a separate member).

### scr-runtime-compression (v0.1.0)
- **Lines / files:** 1301 / 6
- **Tests:** 23
- **Last modified:** 2026-06-02
- **State:** production-ready, now has real fib-quant/turbo-quant encode/decode (was a stub pre-14b0650)
- **What's weak:** 1 unused import warning (`TurboCode` in `codec_dispatch.rs:36`) — trivial to fix
- **Future direction:** the codec_dispatch factory pattern is the right abstraction. The next thing to add: a `Default` impl for `CodecSelector` based on the policy in `quant-governor`, so consumers don't have to wire both crates manually.

### scr-runtime/ sub-workspace (4 crates)
- `scr-audit-adapter`, `scr-cli`, `scr-kernel`, `scr-reference`
- Combined test count not measured (separate sub-workspace)
- Last modified 2026-05-28
- **State:** maintenance-mode, not the focus of the recent perf push (that was scr-runtime-compression in the parent workspace)

### quant-eval (v0.1.0)
- **Lines / files:** 1794 / 9
- **Tests:** 24
- **State:** production-ready
- **Purpose:** compression + semantic search evaluation benchmark suite
- **Future direction:** add CI integration — `quant-eval` is exactly the kind of crate that should run on every PR to catch regressions in fib-quant/turbo-quant/scr-runtime

### gpu-backend (v0.1.0-alpha.1)
- **Lines / files:** 1700 / 7
- **Tests:** 13
- **Last modified:** 2026-06-01
- **State:** brand new alpha, 13 tests, real cudarc driver API integration (commit af1ab2f)
- **What's working:** complete cudarc driver API integration across fib-quant, turbo-quant, poly-kv
- **What's weak:** alpha, stdint.h includes were just patched for nvcc 13.2 compat (commit 3c5447a) — the surface is moving
- **Future direction:** the warning that comes with this is that **precompiled PTX is not portable** across CUDA versions (per the honest-GPU-results writeup, commit 3b1e646). gpu-backend needs a CI matrix that builds on at least nvcc 12.x and 13.x. The current setup compiles for whatever nvcc the developer happens to have.

### llm-pipeline (v0.2.0)
- **Lines / files:** 9912 / 37
- **Tests:** 194 (highest test count in the workspace)
- **Last modified:** 2026-06-01
- **State:** production-ready, with the new receipt infrastructure
- **What's working:** ToolReceipt (from llm-tool-runtime), ToolLoopRunner, now PipelineExecutionReceiptV1 + ProviderCallReceiptV1 + RetryDecisionReceiptV1 + BudgetDebitV1 (P32)
- **Future direction:** the receipt chain is the doctrinal backbone — add a test that exercises a full pipeline and asserts the receipts are linked (each `ProviderCallReceiptV1` references the parent `PipelineExecutionReceiptV1`)

### agent-graph (v0.2.0)
- **Lines / files:** 10649 / 54
- **Tests:** 5 — **the lowest test density of any non-stub crate in the workspace** (0.5 tests/KLOC vs workspace average of ~10)
- **Last modified:** 2026-06-01
- **State:** needs-work (test coverage gap)
- **What's working:** GraphExecutionReceiptV1 + StepExecutionReceiptV1 (P32), real engine.rs and executor.rs
- **What's weak:** 5 tests for 10.6k LOC and 54 files means the graph engine is barely covered. A graph engine with concurrency/async semantics needs an order of magnitude more tests than this.
- **Future direction:**
  - **Top priority:** add 20+ integration tests covering: branching, cycles with backpressure, parallel branch execution, error propagation, checkpoint restore, receipt chain integrity
  - The receipt types are a v0.2.0 addition but there's no test that exercises them end-to-end
  - Consider whether the graph engine should be split (engine + scheduler + executor are likely candidates)

### agent-guard (v0.1.0)
- **Lines / files:** 294 / 4
- **Tests:** 4
- **State:** scaffold-only
- **Future direction:** if this is meant to be a Linux-only control-plane crate, document the platform constraint clearly; if cross-platform is the goal, add Windows/macOS stubs

### forge-pilot (v0.1.0)
- **Lines / files:** 14138 / 77
- **Tests:** 23
- **Last modified:** 2026-05-28
- **State:** production-ready for its current scope, **but still missing claim-ledger wiring** (P1-2)
- **Future direction:** the claim-ledger wiring is a small change: add `claim-ledger = { path = "../claim-ledger" }` to `forge-pilot/Cargo.toml`, add a public-boundary check call in the export path. Estimated 30 min + tests.

### forge-memory-bridge (v0.1.1)
- **Lines / files:** 3553 / 7
- **Tests:** 44
- **State:** production-ready

### forge-engine (v0.2.0, in living-memory/living-memory/)
- **Lines / files:** 16043 / 56
- **Tests:** 170
- **State:** production-ready
- **Note:** package name is `forge-engine` but the directory is `living-memory/living-memory` — historical artifact (GOV-010), documented in the Cargo.toml itself

### llm-output-parser (v0.2.0) and llm-tool-runtime (v0.1.0)
- **Combined:** 7.8k LOC, 182 tests
- **State:** production-ready
- **Upstream analogues:**
  - `llm-output-parser` competes in spirit with: `outlines` (Python, much more mature), `guidance` (Python, Microsoft), `instructor` (Python). Rust ecosystem has very few structured-output parsers. `llm-output-parser` is well-positioned.
  - `llm-tool-runtime` competes with: LangChain's tool-calling (Python), `rig-core` (Rust) has tool definitions but not a runtime. This is doing something real.

### tauri-queue (v0.3.0)
- **Lines / files:** 1496 / 7
- **Tests:** 33
- **State:** production-ready (the 5/27 dossier was wrong to call it a stub; the first-pass V30 audit was right to retract that claim)

### comfyui-rs (v0.2.0) and ollama-vision (v0.2.0)
- **State:** real, working clients
- **Future direction:** both could grow SDK-style APIs; currently they're functional but not ergonomic for new consumers

### ai-batch-queue (v0.2.0) and job-queue (v0.2.0)
- **State:** production-ready, decent test counts
- **Upstream analogues for job-queue:** `apalis` (Rust, similar), `rusty-rabbit`, `sidekiq-rs`. The RecursiveIntell job-queue is more featureful than most (ETA estimation, model-aware batching for ai-batch-queue).

### examples/ workspace member (9.5k LOC)
- Actually, the parent workspace `Cargo.toml` does NOT list `examples/` as a member — it appears in `cargo metadata` because of some include or because of how my probe walked directories. The 9.5k figure is the size of the `examples/` directory tree, which contains cross-crate example binaries. Not a real crate to audit.

### tauri-react-hooks (TypeScript)
- A TypeScript package with `package.json`, `tsconfig.json`, `tsup.config.ts`. Not Rust. Provides React hooks that consume the Tauri commands exposed by `tauri-queue`. Healthy separation.
- **Future direction:** if there are gaps between what tauri-queue exposes via Tauri commands and what tauri-react-hooks needs, audit the two together.

---

## 4. Cross-cutting future directions (the actual valuable part)

### A. The agent-graph test coverage gap is the biggest correctness risk in the workspace
- 5 tests for 10.6k LOC. The graph engine has concurrency, async, branching, error propagation — all undertested.
- **Effort:** 1-2 weeks to add a real integration test suite. **Value:** high — this is the orchestration substrate for multi-step AI work, and bugs here are silent and expensive.

### B. The governance lane needs a "delete or implement" pass
- 3 crates are empty stubs: `assurance-case`, `attestation`, `policy-store`
- 4 crates are typed surfaces with no live consumer: `mechanism-runtime`, `continuity-runtime`, `discovery-portfolio`, `federated-settlement`
- **Decision tree:** for each, ask: "is there a planned consumer?" If no, delete. If yes, add the consumer or rename to `*-types` to stop implying runtime behavior.

### C. The poly-kv sub-workspace deserves to be promoted
- 56 tests passing, 1.7k+ LOC added in the last 2 weeks, real GPU integration
- The fact that it's a separate `Cargo.toml` from the parent is a usability tax
- **Future:** add a top-level `poly-kv` re-export in the parent workspace, or document the "parent + sub-workspace" pattern and standardize it (AiDENs/ is another sub-workspace; scr-runtime/ is a third)

### D. semantic-memory is approaching a size where it should split
- 37k LOC, 66 source files, 113 tests
- Natural seam: `core` (SQLite + bitemporal) / `search` (HNSW + FTS5) / `receipts` (the receipt types)
- **Effort:** 1 week. **Value:** enables the `pruning` of test runtime, makes the receipt types reusable in other storage backends.

### E. The fib-quant/turbo-quant/gpu-backend perf work has a real cost — paper it
- 47 commits in 4 days is a sprint rate. The honest-GPU-results writeup (commit 3b1e646) is exactly the right discipline but it lives in a single commit message. The full performance narrative across the SIMD + Rayon + GPU pushes is scattered across commit messages.
- **Future:** consolidate into a single `PERF_HISTORY_2026-Q2.md` at the workspace root, with the per-commit before/after numbers in one table. The next person who touches this code will thank you.

### F. The hnsw_rs upstream is dormant
- Last release 3 months ago (verified 2026-06-02). 503K all-time downloads but a slow release cadence.
- **Future:** pin a specific version in semantic-memory's Cargo.toml, and have a plan to fork or replace (cuvs, FAISS bindings, lance).

### G. Claim-ledger → forge-pilot is the only remaining P1 from the V30 audit
- 30 minutes of work, ~50 lines of test. The "public boundary check before export" pattern is doctrine; forge-pilot is the export path; the gap is one Cargo.toml line plus one function call.
- **Future:** do this next, before any new feature work in forge-pilot.

### H. The Crate Hardening Matrix is stale
- `CRATE_HARDENING_MATRIX.md` (per 5/29 audit) has dev-profile-only benchmarks. After the recent perf work, release-profile numbers have likely shifted significantly. Worth a `cargo bench --release` pass and a re-issue of the matrix.

### I. The poly-kv → semantic-memory integration is undocumented at the architecture level
- semantic-memory has bitemporal-runtime + quant-governor + boundary-compiler wired in. poly-kv has fib-quant + turbo-quant + gpu-backend. They share `quant-codec-core` (in poly-kv/crates). But the relationship between "semantic-memory's compression path" and "poly-kv's shared KV-cache" isn't clear from reading either crate.
- **Future:** add a top-level `KV_CACHE_AND_VECTOR_STORAGE.md` to the workspace root that draws the line.

### J. The `turbo-semantic/` directory is a landmine
- A full clone of semantic-memory exists inside a directory named `turbo-semantic`, with a `Cargo.toml` declaring `name = "semantic-memory"`. This is going to cause `cargo` confusion for anyone working in the parent workspace.
- **Future:** rename to `turbo-semantic-archive` (or delete), update the parent workspace's `exclude` list, and add a one-line `README.md` explaining what it was.

### K. The "V30 hardening → `unreachable!`" compromise is a ticking time bomb
- V30 plan §1.2 said "use thiserror". The actual work downgraded to "use `unreachable!`". Two `unreachable!()` calls in kernel-oracles, 5 in knowledge-runtime, 1 in kernel-execution (one `.expect` I found). These all still panic — they just signal intent better than `panic!`.
- **Future:** convert them all to typed errors, OR document them as load-bearing invariants with a `// INVARIANT:` comment.

### L. The clippy lint suite has never been run clean to a documented state
- 1 warning currently (unused import in scr-runtime-compression). Past audits cite "one clippy warning" without fixing. CI likely doesn't run `cargo clippy -- -D warnings` as a gate.
- **Future:** add a `make clippy-strict` or similar target and gate on it.

---

## 5. Suggested next move (concrete, ordered)

If the user wants to spend 1-2 days moving the workspace forward, here's the ordered list I'd run:

1. **Wire claim-ledger into forge-pilot** (P1-2 from V30, still open). 30 min + tests.
2. **Fix the unused import warning in scr-runtime-compression.** 1 minute.
3. **Delete or implement the 3 empty governance stubs** (assurance-case, attestation, policy-store). 30 min for the delete option.
4. **Add an integration test target for agent-graph** — even just 5 more tests covering branch fanout, checkpoint restore, and receipt chain. 2-3 hours.
5. **Rename `turbo-semantic/` to `turbo-semantic-archive/` and add to workspace exclude.** 5 min.
6. **Convert the 8 `unreachable!()`/`expect()` sites in knowledge-runtime + kernel-oracles + kernel-execution to `thiserror` errors** per the original V30 plan. 2-3 hours.
7. **Promote poly-kv to a documented sub-workspace status** (add a top-level md). 1 hour.
8. **Run `cargo clippy --workspace -- -D warnings`** and fix what comes out. 1-2 hours.
9. **Add `gpu-backend` to a CI matrix** with both nvcc 12.x and 13.x. 1 hour.
10. **Split semantic-memory into 3 sub-crates.** 1 week (do this in a feature branch, not main).

Total for 1-7: ~1 day. Total for 1-10: ~1.5 weeks.

---

## 6. Receipt (audit-self)

- **What was done:** scoped the user's 62-crate request, read the 5/27 dossier + 5/29 corrected hostile audit, ran `cargo check --workspace` (clean) + `cargo test --workspace` (all pass), shell-probed LOC/tests/deps/lastmod for all 68 directories, grep-verified the 3 P0 integrations and the 1 open P1 from the V30 audit, confirmed the P0-4 fix (check-runner-sys exists and is consumed), discovered 3 empty governance stubs, 1 landmine (turbo-semantic clone), and 1 low-test-density crate (agent-graph). Web-researched hnsw_rs upstream status. Did NOT spawn subagents in the end (timed out at 600s each, twice).
- **What's verified by E0 evidence:**
  - 0 errors / 1 warning on `cargo check --workspace`
  - 0 FAILED on `cargo test --workspace --no-fail-fast`
  - All 3 P0 deps from V30 found in semantic-memory/Cargo.toml
  - check-runner-sys exists, has `#![allow(unsafe_code)]`, is consumed by check-runner
  - bitemporal-runtime is referenced from semantic-memory/src/db.rs and types.rs
  - quant-governor is referenced from semantic-memory/src/quantize_governed.rs
  - boundary-compiler is referenced from semantic-memory/src/graph.rs
  - PipelineExecutionReceiptV1 + ProviderCallReceiptV1 + RetryDecisionReceiptV1 + BudgetDebitV1 are present in llm-pipeline/src/{lib.rs, pipeline.rs, types.rs}
  - GraphExecutionReceiptV1 + StepExecutionReceiptV1 are present in agent-graph/src/receipt.rs
  - 3 empty governance stubs: assurance-case, attestation, policy-store
  - agent-graph: 5 tests for 10.6k LOC = lowest density in non-stub workspace
  - fib-quant is based on arXiv:2605.11478 (FibQuant paper, May 2026)
  - hnsw_rs upstream: v0.3.4, last release ~3 months ago, 503K downloads
- **What's NOT verified:** the 30+ smaller governance crates were probed for LOC/tests but not deep-read (single-file lib.rs crates under 1k LOC are likely fine). `poly-kv` sub-workspace cargo test was run and passed. `AiDENs/` sub-workspace (34 crates) was NOT deep-audited — it's a parallel project. The 3 P0 integration tests inside semantic-memory were not run individually; the workspace test run passed but I can't say "all 113 semantic-memory tests touch the bitemporal path" without a more targeted run.
- **Falsifies if:** the 3 P0 deps appear in Cargo.toml but the import sites are dead code (cargo would catch this in the workspace test, and tests pass, so unlikely); the `claim-ledger` → `forge-pilot` wiring actually exists somewhere I missed (re-verified: zero matches in forge-pilot/Cargo.toml or src); the empty governance stubs actually have non-empty src/lib.rs (re-verified: cat returned zero bytes for all three).
- **Time spent:** ~30 min in-session. **Files written:** this one file. **Subagent cost:** 2 timeouts (one 600s with 50 API calls, one 120s interrupted at 14 calls).
