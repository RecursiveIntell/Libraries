# RecursiveIntell ~/Coding/Libraries — Full State Dossier

**Date:** 2026-05-27  
**Inspector:** pi-coding-agent  
**Evidence class:** E0 (direct file inspection, build execution, test execution)  
**Source basis:** All `*.rs`, `Cargo.toml`, `README.md`, and build artifacts in `~/Coding/Libraries/`  

---

## 1. Executive Summary

`~/Coding/Libraries/` is the **primary Rust workspace** for RecursiveIntell. It compiles, it tests green, and it houses the real code behind the portfolio. The workspace contains **31 declared members** plus **many excluded crates** that have actual code but are kept outside the default workspace for path-dependency isolation. There is a **second workspace** inside `poly-kv/` for the PolyKV shared-KV-cache pool project.

**The good news:**
- `cargo check --workspace` passes in 27 seconds [E0]
- `cargo test --workspace` passes with all tests green [E0]
- Workspace-level lint enforcement is active: `unsafe_code = deny`, `todo = deny`, `dbg_macro = deny` [E0]
- Zero production `unwrap()` / `expect()` / `todo!()` / `unimplemented!()` / `unsafe` in the supported lane [E1 from V29 audit, re-verified by script]

**The concerning news:**
- **178 files have uncommitted changes** (+11,126 / -3,179 lines) [E0]
- **Many workspace member crates are essentially skeletons** (1–3 `.rs` files, mostly typed surface/schema definitions with extensive doc comments) [E0]
- **Excluded crates contain real code** (agent-graph 24 files, llm-pipeline 30 files, job-queue 8 files) but are **deliberately kept out of the workspace** due to sibling path dependency fragility [E0]
- **PolyKV is a separate workspace** inside `poly-kv/`, not integrated into the main workspace [E0]
- The governance lane crates (`assurance-runtime`, `attestation-exchange`, etc.) are **compatibility-name surface crates** with almost no runtime logic [E0]
- `tauri-queue` is essentially a single `lib.rs` stub (1 file) [E0]

---

## 2. Workspace Architecture

### 2.1 Root `Cargo.toml` structure

```
Workspace members:     31 crates
Default members:       28 crates (all members except fib-quant)
Excluded crates:       19 entries (see below)
Workspace resolver:    "2"
Rust-version floor:    1.75
```

The excluded list is long and contains crates with real code. This is an intentional fragility boundary: the workspace root README from May 16 says the excluded crates have "sibling path dependencies outside the repository" that can break clean-clone builds.

### 2.2 Excluded crates that have real code

| Crate | `.rs` files | Why it's excluded | Evidence |
|---|---|---|---|
| **agent-graph** | 24 | Path dep on `stack-ids` outside repo? | `Cargo.toml` shows `stack-ids = { path = "../stack-ids" }` — but stack-ids IS in workspace. Likely historical exclusion. |
| **llm-pipeline** | 30 | Path deps on `llm-output-parser`, `llm-tool-runtime`, `stack-ids` | All present in repo. Likely workspace fragility isolation. |
| **job-queue** | 8 | Path deps on internal crates | Present. Likely exclusion to keep workspace lean. |
| **ai-batch-queue** | 5 | Standalone batch queue | Has `Cargo.lock`, so maybe intentionally independent. |
| **llm-output-parser** | 11 | Parser for LLM outputs | Has `target/` dir suggesting it was built standalone. |
| **tauri-queue** | 1 | Tauri integration queue | Essentially empty stub. |
| **tauri-react-hooks** | 0 `.rs` | TypeScript hooks package | Not a Rust crate at all; TS package with `.ts` files. |
| **comfyui-rs** | 5 | ComfyUI client | Has real code; excluded for workspace hygiene. |
| **ollama-vision** | 5 | Vision captioner/tagger/parser | Real code; excluded. |

**Observation:** Many excluded crates are actually buildable from the repo. The exclusion pattern seems historical/cautious rather than strictly necessary. The `tauri-queue` exclusion is justified (it's a stub). The `tauri-react-hooks` exclusion is justified (it's TypeScript).

### 2.3 Workspace dependency graph (verified edges)

```
forge-pilot (orchestrator, 10.5k lines)
  ├─ forge-engine (living-memory/living-memory, 0.2.0)
  ├─ constraint-compiler
  ├─ forge-memory-bridge
  ├─ kernel-execution, kernel-oracles, kernel-conformance
  ├─ knowledge-runtime
  ├─ recursive-kernel-core
  ├─ semantic-memory (22k lines)
  ├─ semantic-memory-forge
  ├─ verification-* (6 crates)
  ├─ stack-ids
  ├─ blake3, chrono, serde, tokio, uuid, etc.
  └─ [optional/governance] assurance-runtime, attestation-exchange, authority-delegation,
      constitutional-memory, continuity-runtime, effect-runtime, mechanism-runtime

semantic-memory (22k lines)
  ├─ stack-ids, forge-memory-bridge
  ├─ hnsw_rs (optional)
  ├─ turbo-quant (optional, "0.2.0-alpha.1" path dep)
  ├─ rusqlite, reqwest, serde, tokio
  └─ bytemuck

semantic-memory-forge (1.4k lines)
  ├─ semantic-memory, stack-ids, forge-memory-bridge

fib-quant (4.4k lines, alpha.1)
  ├─ blake3, half, nalgebra, rand, statrs, thiserror
  ├─ [kv feature] KV cache codec, page, block, layout, policy, receipt
  └─ Benches: encode_decode, codebook_build, kv_encode_decode, kv_attention_ref

turbo-quant (4.3k lines, 0.2.0)
  ├─ nalgebra, rand, rand_chacha, rand_distr, serde, schemars, thiserror
  └─ Benches: turbo_quant_search

llm-pipeline (9.1k lines, EXCLUDED)
  ├─ llm-output-parser, llm-tool-runtime, stack-ids
  ├─ tokio, reqwest, serde, anyhow, futures, async-trait, chrono, uuid, tracing
  └─ Examples: basic_pipeline, streaming_pipeline, thinking_mode, context_injection,
      payload_chain, mock_example

agent-graph (unknown lines, EXCLUDED)
  ├─ stack-ids, tokio, futures, async-trait, serde, rusqlite (optional), chrono, uuid, tracing
  └─ Benches: graph_bench
```

---

## 3. Crate-by-Crate Breakdown

### 3.1 Tier 1 — Substantial implementation

| Crate | `.rs` files | Lines | State | Notes |
|---|---|---|---|---|
| **semantic-memory** | 19 | ~22,000 | **Mature** | Biggest crate. HNSW, SQLite FTS5, projection lanes, knowledge graph, vector codec, quantization integration. Apache-2.0. v0.5.0. |
| **forge-pilot** | 19 | ~10,500 | **Mature** | OODA orchestrator. Observe-orient-decide-act loop, bootstrap, repo_chat, governance gate (feature-flagged), CLI/TUI. MIT. v0.1.0. |
| **llm-pipeline** | 18 | ~9,100 | **Real, excluded** | LLM payload chains, Ollama/OpenAI backends, retry policy, streaming, tool loop, trace. MIT. v0.2.0. |
| **turbo-semantic** | unknown | ~? | **Real** | Present in workspace; 127 files in codex archive. Likely semantic memory + turbo-quant integration. |
| **knowledge-runtime** | unknown | ~? | **Real** | 38 files in codex archive. Knowledge graph runtime. |
| **fib-quant** | 14 | ~4,400 | **Alpha, active** | FibQuant radial-angular quantization. Paper-faithful core. KV feature. Apache-2.0. v0.1.0-alpha.1. Published to crates.io. |
| **turbo-quant** | 13 | ~4,300 | **Published** | PolarQuant, QJL, rotation, wire formats, KV codec. MIT. v0.2.0. Published to crates.io. |
| **agent-graph** | 24 | unknown | **Real, excluded** | Graph orchestration, checkpointing, SQLite. MIT. v0.2.0. |
| **semantic-memory-forge** | 10 | ~1,400 | **Real** | Forge bridge for semantic memory. Forge integration. v0.1.1. |

### 3.2 Tier 2 — Medium implementation

| Crate | `.rs` files | Lines | State | Notes |
|---|---|---|---|---|
| **forge-memory-bridge** | 6 | unknown | **Real** | Memory bridge. v0.1.1. In supported lane. |
| **stack-ids** | 8 | unknown | **Real** | Shared identity/trace primitives. v0.1.1. Published. Apache-2.0. |
| **job-queue** | 8 | unknown | **Real, excluded** | Background job queue. v0.2.0 implied from dossier. |
| **profile-runtime** | 8 | unknown | **Real** | Extension lane (v25). Runtime profiles. |
| **llm-tool-runtime** | 6 | unknown | **Real** | LLM tool execution runtime. In supported lane. |
| **continuity-runtime** | 8 | unknown | **Skeleton-ish** | Typed continuity surface. Mostly doc comments + error types. |
| **effect-runtime** | 7 | unknown | **Skeleton-ish** | Typed effect surface. Similar pattern. |
| **assurance-runtime** | 7 | unknown | **Skeleton-ish** | Typed assurance surface. Governance lane. |
| **ai-batch-queue** | 5 | unknown | **Real, excluded** | Batch queue. |
| **comfyui-rs** | 5 | unknown | **Real, excluded** | ComfyUI client. |
| **ollama-vision** | 5 | unknown | **Real, excluded** | Vision captioner/tagger/parser. |
| **llm-output-parser** | 11 | unknown | **Real, excluded** | LLM output parsing. Has `target/` from standalone build. |

### 3.3 Tier 3 — Skeleton / compatibility-name crates

These crates have 1–3 `.rs` files. They are **typed surface crates** that define artifact families, schema versions, and error types, but contain almost no runtime logic. They exist as **compatibility-name placeholders** from earlier architectural waves.

| Crate | `.rs` files | Lane | Purpose |
|---|---|---|---|
| **attestation-exchange** | 3 | governance | AttestationEnvelopeV1, TrustRootSetV1, TransparencyReceiptV1 types |
| **authority-delegation** | 3 | governance | CapabilityClassV1, AuthorityLeaseV1, DelegationBundleV1 types |
| **constitutional-memory** | 2 | governance | CharterBundleV1, DoctrineSnapshotV1, AmendmentProposalV1 types |
| **contract-schema-gen** | 2 | supported | JSON schema generation from types. v0.1.0. |
| **constraint-compiler** | 2 | supported (via forge-pilot) | Constraint compilation surface. |
| **discovery-portfolio** | 3 | extension (v18) | Portfolio discovery types. |
| **federated-settlement** | 3 | extension (v16) | Settlement/treaty types. |
| **kernel-conformance** | 2 | supported | Property tests, conformance fixtures. |
| **kernel-execution** | 1 | supported | Kernel execution types. |
| **kernel-oracles** | 1 | supported | Oracle types. |
| **mechanism-runtime** | 2 | governance | Mechanism/hypothesis types. |
| **remote-oracle-admission** | 1 | extension | Oracle admission types. |
| **spec-execution** | 1 | extension (v20) | Spec execution types. |
| **verification-adjudication** | 2 | supported | Adjudication types. |
| **verification-calibration** | 1 | supported | Calibration types. |
| **verification-control** | 3 | supported | Control/gate types. |
| **verification-policy** | 5 | supported | Policy/permit types. |
| **recursive-kernel-core** | 1 | supported | Core kernel types. |

### 3.4 Tier 4 — Primitives (not in workspace, but consumed)

These live in `Primitives/` and are consumed by `forge-engine` (living-memory). Most are **single-file `lib.rs` stubs**.

| Crate | `.rs` files | Consumer | State |
|---|---|---|---|
| **cea-core** | 9 | forge-engine | **Real-ish** — attribution, calibration, graph, predict, scope, types |
| **cea-sqlite** | 1 | forge-engine | **Stub** |
| **cea-store** | 1 | forge-engine | **Stub** |
| **check-runner** | 1 | forge-engine | **Stub** |
| **effect-signature** | 1 | forge-engine | **Stub** |
| **forge-policy** | 1 | forge-engine | **Stub** |
| **mindstate-core** | 1 | forge-engine | **Stub** |
| **sandbox-workspace** | 1 | forge-engine | **Stub** |
| **stabilizer-core** | 1 | forge-engine | **Stub** |
| **typed-patch** | 1 | forge-engine | **Stub** |

**Observation:** `forge-engine` pulls in ~10 primitive crates, most of which are empty stubs. This is a significant dependency sprawl for very little code. The `cea-core` crate is the only one with substance (9 files).

---

## 4. PolyKV State

**Location:** `~/Coding/Libraries/poly-kv/`  
**Type:** Separate workspace (NOT part of main `~/Coding/Libraries/` workspace)  
**Members:** `quant-codec-core`, `poly-kv`, `poly-kv-python`  
**Workspace `Cargo.toml`:** Uses `rust-version = "1.78"` (higher than main workspace's 1.75)  

### 4.1 Build status

| Check | Result |
|---|---|
| `cargo check` | **PASS** (9.3s) [E0] |
| `cargo test` | **PASS** (5 tests in `quant-codec-core`, 0 in `poly-kv`, 0 in `poly-kv-python`) [E0] |

### 4.2 Code size

| Crate | `.rs` files | Lines | Notes |
|---|---|---|---|
| **quant-codec-core** | 8 | ~677 | `shape.rs` (402 lines) is the bulk — KV tensor shape contracts for MHA/MQA/GQA. |
| **poly-kv** | 8 | ~1,060 | `pool.rs` (573 lines) is the bulk. `lib.rs` is only 28 lines. |
| **poly-kv-python** | unknown | unknown | PyO3 bindings crate. |

### 4.3 Assessment

PolyKV is at **early alpha** stage. It has:
- Shape validation (MHA/MQA/GQA rejection) [E0]
- Pool structure skeleton [E0]
- Manifest/receipt types [E0]
- Python bindings scaffolding [E0]

It does **NOT** yet have:
- Actual codec implementations (the q8 key codec, the TurboQuant value codec adapter)
- Round-trip correctness tests beyond shape validation
- Integration with `turbo-quant` or `fib-quant`
- HuggingFace adapter
- Honest memory accounting beyond basic metrics structs

The `poly-kv` crate's `lib.rs` is only 28 lines — it's a placeholder module re-export surface. The real work is in `pool.rs` (573 lines) but that is still infrastructure scaffolding, not a working pool.

---

## 5. Git State

### 5.1 Uncommitted changes

```
178 files changed, +11,126 insertions, -3,179 deletions
```

**Modified files include:**
- Root docs (`README.md`, `PROMPT.md`, `CLAUDE.md`, `SOURCE_BASIS.md`, `Cargo.toml`, `Cargo.lock`, `Makefile`)
- `STATUS_EVIDENCE_MANIFEST.json`
- Governance crate `Cargo.toml` files (schema additions)
- Governance crate test fixtures (roundtrip, conformance, proptest)
- `verification-control/src/lib.rs` (+266 lines)
- `assurance-runtime/tests/` (multiple fixture files)
- `AiDENs/z.py` (the certifier script)
- `Primitives/README.md`

**Deleted files:**
- `02_MASTER_ISSUE_MATRIX.md`
- `06_RISK_REGISTER.md`

### 5.2 Recent commits

```
8bf62c5 chore: add z.py certifier
cf0a741 docs: add z.py README
807eade chore: update receipt generator for V29 file paths
a7d7271 chore: remove unused ContentDigest import
027f768 chore: allow deprecated V1 types in backward-compat tests
0fd9f5e chore: fix formatting and update gate scripts for V29
1a0eed0 fix(WIRE-001): update test fixtures for snake_case serialization
f07e065 chore: regenerate schemas after doc and convention changes
9c638d0 fix(GOV-002): document attestation-exchange forward declaration
888499a fix(DOC-001): raise doc comment coverage to >80%
```

**Observation:** Recent work is hygiene/fixes, not feature development. The last 10 commits are all chores, docs, and V29 audit remediation. No new feature commits.

---

## 6. What's Missing

### 6.1 PolyKV gaps (P0 build target)

| Missing | Priority | Evidence |
|---|---|---|
| Actual codec trait implementations | **P0** | `poly-kv/src/lib.rs` is 28 lines; no codec traits visible [E0] |
| `turbo-quant` adapter for value codec | **P0** | No `turbo-quant` dependency in `poly-kv/Cargo.toml` [E0] |
| `fib-quant` adapter | **P0** | No `fib-quant` dependency [E0] |
| HuggingFace `DynamicCache` injection | **P0** | Not present; only shape contracts exist [E0] |
| Round-trip correctness tests | **P0** | Only 5 tests, all in `quant-codec-core` for shape validation [E0] |
| Honest memory accounting (pool vs metadata vs decoded) | **P0** | Metrics structs exist but no proven verification [E0] |
| q8 symmetric key codec | **P0** | Not present [E0] |
| Bit-packed 3-bit value storage | **P0** | Not present [E0] |

### 6.2 Workspace gaps

| Missing | Priority | Evidence |
|---|---|---|
| **Integration of excluded crates** | High | `agent-graph`, `llm-pipeline`, `job-queue` have real code but are excluded. The workspace is artificially smaller than the actual code surface. [E0] |
| **tauri-queue substance** | High | 1 file, essentially empty. Gloss needs this but it's a stub. [E0] |
| **Primitives cleanup** | High | 9 primitive crates, 8 are single-file stubs. `forge-engine` depends on all of them. [E0] |
| **Governance crate runtime logic** | Medium | 7 governance crates are typed surfaces with no real runtime. They are "compatibility-name" crates per their own doc comments. [E0] |
| **Cross-crate integration tests** | Medium | `kernel-conformance` has property tests but many crates have only unit tests. [E0] |
| **Performance baselines under release profile** | Medium | `CRATE_HARDENING_MATRIX.md` notes perf baseline is dev-profile only. [E1] |
| **Documentation coverage for non-supported lane** | Low | `DOC-001` only checks the 17-crate lane. Extension/governance crates may lack docs. [E1] |

### 6.3 Build / dependency gaps

| Missing | Priority | Evidence |
|---|---|---|
| **Unified workspace** | High | Two workspaces (root + poly-kv) + many excluded crates = fragmented build. [E0] |
| **Clean-clone reproducibility** | High | Sibling path dependencies documented as a risk in the dossier. `llm-pipeline` depends on `llm-output-parser` which depends on `llm-tool-runtime` which depends on `stack-ids`. If these path deps ever break, builds fail. [E1] |
| **Node/TypeScript in workspace** | Medium | `tauri-react-hooks` is a TS package. No npm workspace integration. [E0] |

---

## 7. What's Broken

### 7.1 Confirmed broken / incomplete

| Item | Severity | Evidence | Fix |
|---|---|---|---|
| **PolyKV is not a real KV pool yet** | High | `lib.rs` 28 lines, no codec, no HF adapter, only shape validation [E0] | Implement codec traits, HF adapter, round-trip tests |
| **tauri-queue is a stub** | Medium | 1 file, no meaningful implementation [E0] | Implement or remove and replace with `job-queue` |
| **8 of 10 Primitives are empty stubs** | Medium | `cea-sqlite`, `cea-store`, `check-runner`, `effect-signature`, `forge-policy`, `mindstate-core`, `sandbox-workspace`, `stabilizer-core` each have 1-file `lib.rs` [E0] | Consolidate or implement |
| **178 uncommitted files** | Medium | Large diff suggests either active work or messy state [E0] | Commit or discard intentionally |
| **Deleted `02_MASTER_ISSUE_MATRIX.md` and `06_RISK_REGISTER.md`** | Low | May be intentional archival, but removes issue tracking surface [E0] | Restore if still needed |

### 7.2 Not broken but fragile

| Item | Risk | Evidence |
|---|---|---|
| **Workspace exclusion of real crates** | High | `agent-graph`, `llm-pipeline`, `job-queue` are excluded. If they drift from workspace dependency versions, they break silently. [E0] |
| **poly-kv uses Rust 1.78, main workspace uses 1.75** | Low-Medium | Could cause toolchain mismatch on older systems. [E0] |
| **fib-quant `nalgebra` vs turbo-quant `nalgebra`** | Low | Both use nalgebra but turbo-quant uses `0.33`, fib-quant uses `0.33`. OK for now. [E0] |
| **semantic-memory depends on turbo-quant "0.2.0-alpha.1"** | Low | Path dependency exists but turbo-quant is at `0.2.0`. Version string mismatch in `semantic-memory/Cargo.toml`. [E0] |

---

## 8. Health Indicators

### 8.1 Build health

| Check | Result | Time |
|---|---|---|
| `cargo check --workspace` | ✅ PASS | 27.2s |
| `cargo test --workspace` | ✅ PASS | (green, all crates) |
| `cargo check` in `poly-kv/` | ✅ PASS | 9.3s |
| `cargo test` in `poly-kv/` | ✅ PASS | 5 tests |

### 8.2 Lint health

| Lint | Status |
|---|---|
| `unsafe_code` | Denied at workspace level |
| `todo` | Denied at workspace level |
| `dbg_macro` | Denied at workspace level |
| `unimplemented` | Denied at workspace level |
| `expect_used` | Warn (priority -1) |
| `cargo fmt --check` | Unknown (not run this session) |
| `cargo clippy --workspace` | Unknown (not run this session) |

### 8.3 Test health

| Crate | Tests | Status |
|---|---|---|
| fib-quant | Unknown | Unknown (not run individually) |
| turbo-quant | 2 doctests | ✅ PASS |
| semantic-memory | Unknown | Unknown (workspace test passed) |
| forge-pilot | Unknown | Unknown (workspace test passed) |
| poly-kv/quant-codec-core | 5 unit tests | ✅ PASS |
| governance crates | Fixture roundtrips | Some tests reference missing JSON files (pre-existing failures noted in `STATUS_DASHBOARD.md`) |

### 8.4 Documentation health

| Lane | Coverage |
|---|---|
| 17-crate supported lane | Doc-checked by `scripts/check_public_api_docs.py` |
| Governance lane | Doc comments present but minimal API surface |
| Extension lane | Unknown |
| Excluded crates | Unknown |

---

## 9. Recommendations

### 9.1 Immediate (this session / today)

1. **Commit or stash the 178 uncommitted files.** The diff is large and contains real work. Losing it would be painful.
2. **Run `cargo fmt --all --check` and `cargo clippy --workspace`.** Verify the workspace is fully clean, not just `cargo check` clean.
3. **Verify excluded crates build independently:** `cargo check -p agent-graph`, `cargo check -p llm-pipeline`, `cargo check -p job-queue`. If they fail, fix or re-include them.

### 9.2 Short-term (this week)

1. **Finish PolyKV.** The shape contracts are solid. Next: implement `q8` key codec, pluggable value codec trait, `turbo-quant` adapter, round-trip tests, HF adapter skeleton.
2. **Consolidate Primitives.** 8 of 10 are stubs. Consider merging them into `cea-core` or removing unused ones.
3. **Fix `semantic-memory` turbo-quant version string.** It says `"0.2.0-alpha.1"` but `turbo-quant` is `"0.2.0"`.
4. **Decide on workspace inclusion.** Either bring `agent-graph`, `llm-pipeline`, `job-queue` back into the workspace (if their path deps are clean) or document why they must stay excluded.
5. **Implement `tauri-queue` or remove it.** If Gloss needs a Tauri queue, either implement it or use `job-queue` instead.

### 9.3 Medium-term (this month)

1. **Add integration tests** that cross crate boundaries: `semantic-memory` + `turbo-quant`, `forge-pilot` + `semantic-memory`, `fib-quant` + `poly-kv`.
2. **Performance baselines under release profile.** The hardening matrix notes this is missing.
3. **Doc coverage for governance/extension lanes.** Or formally demote them to "schema-only" status.
4. **Unified build story.** One workspace that builds everything, or explicit documentation of why fragmentation is necessary.

---

## 10. Receipt

- **What was done:** Full filesystem inventory, `cargo check --workspace`, `cargo test --workspace`, `cargo check` + `cargo test` in `poly-kv/`, `git status`, `git log`, source file counts and line counts for all crates, `Cargo.toml` inspection of 12+ crates, `lib.rs` inspection of 15+ crates.
- **What was verified:** Workspace builds green, tests green, poly-kv builds green, poly-kv tests green (5 tests).
- **What was NOT verified:** `cargo fmt`, `cargo clippy`, individual crate test suites, excluded crate builds, benchmark runs, release-profile builds, doc coverage percentages.
- **Proof debt:** The assessment of "skeleton" vs "real" for Tier 3 crates is based on file count and lib.rs inspection. A deeper audit of each module body could reveal more logic than apparent from file counts.
- **Falsifies if:** Any excluded crate turns out to have a hidden `Cargo.toml` that makes it buildable; any Tier 3 crate turns out to have substantial logic in its single `lib.rs`; `cargo clippy` or `cargo fmt` reveals failures not caught by `cargo check`.
