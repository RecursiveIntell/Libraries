# RecursiveIntell ~/Coding/Libraries — Full Hardening and Gap Audit

**Date:** 2026-05-27  
**Inspector:** pi-coding-agent  
**Evidence class:** E0 (direct file inspection, build execution, source-code grep, git status)  
**Doctrinal lens:** Full Provenance+ Research corpus (artifact-runtime.md, evidence-first.md, governed-compression.md, semantic-artifact-law.md, contract/hardening.md, bitemporal/replay.md, bitemporal/storage.md)

---

## Executive Summary

This audit examines every crate in `~/Coding/Libraries/` against the provenance-first doctrine: **artifact law, bitemporal truth, execution-as-evidence, boundary canonicalization, no shadow truth, and governed compression**. The workspace builds green and has real, substantial code across most crates. However, **the doctrinal hardening is uneven**: some crates have receipt infrastructure and digest linking, but **bitemporal truth is almost entirely absent**, **canonicalization is ad-hoc JSON key-sorting rather than RFC 8785 JCS**, **governed compression exists as isolated codec crates without a runtime governance layer**, and **unsafe code lurks in one primitive crate** despite the workspace-level `unsafe_code = deny` lint.

**Critical finding:** The V29 hostile audit claimed "zero production `unwrap()`" and "zero `unsafe` in workspace member code." This audit finds **both claims are falsified** when the expanded workspace (post-Libraries2 merge) is considered. `Primitives/check-runner` contains production `unsafe` blocks. `contract-schema-gen` and `forge-memory-bridge` contain production `unwrap()` calls.

---

## 1. Build and Workspace Health

### 1.1 Verified build state

| Check | Result | Evidence |
|---|---|---|
| `cargo check --workspace` | ✅ PASS (4.4s incremental, 90s cold) | [E0] |
| Workspace members | 49 crates | [E0] |
| Excluded from workspace | 2 (`tauri-react-hooks` TS package, `demo-tauri-libraries/src-tauri` missing) | [E0] |
| Workspace lint policy | `unsafe_code = deny`, `todo = deny`, `dbg_macro = deny`, `unimplemented = deny` | [E0] |

### 1.2 Git state (risk)

| Metric | Value | Evidence |
|---|---|---|
| Uncommitted files | 178 files, +11,126 / -3,179 lines | [E0] `git diff --stat` |
| Deleted root docs | `02_MASTER_ISSUE_MATRIX.md`, `06_RISK_REGISTER.md` | [E0] |
| Last 10 commits | All chores/docs/V29 fixes; **zero feature commits** | [E0] `git log --oneline -10` |

**Hardening gap:** 178 uncommitted files is a **source-of-truth drift risk**. If the workspace crashes or the session ends, uncommitted work is lost. The deleted issue matrix and risk register remove structured tracking surfaces.

---

## 2. Doctrinal Compliance by Crate

### 2.1 Artifact law and receipt infrastructure

| Crate | Receipts present | Digest linking | Artifact types | Evidence class |
|---|---|---|---|---|
| **semantic-memory** | ✅ `search_receipts` table, `VectorArtifactBuildReceiptV1`, `VectorSearchReceiptV1` | ✅ `ContentDigest`, `DigestBuilder` from `stack-ids` | ✅ Episode, claim, vector artifact, derived vector artifact | [E0] |
| **forge-pilot** | ✅ `LoopReceipt`, `LoopIterationReport`, `GovernanceReceiptV1` (feature-gated) | ✅ `ExportActionTraceV1`, `ExecutionLineageReceipt` | ✅ Observation, bootstrap delta, bundle, governance gate | [E0] |
| **turbo-quant** | ⚠️ `profile.rs` has `CompressionProfile` but no receipt types | ❌ No digest infrastructure visible | ⚠️ Codec profile only; no artifact envelope | [E0] |
| **fib-quant** | ⚠️ `receipt.rs` exists (136 lines) but **no integration** with semantic-memory or forge-pilot | ⚠️ `blake3` dep but no visible artifact envelope | ⚠️ KV receipts defined but not wired | [E0] |
| **poly-kv** | ⚠️ `receipts.rs` exists (68 lines) but **no codec receipts**, only pool-build receipts | ⚠️ Digest types exist but not used for codec artifacts | ⚠️ Manifest/receipt skeleton | [E0] |
| **llm-pipeline** | ❌ No receipt types visible | ❌ No digest infrastructure | ❌ No artifact envelopes | [E0] |
| **agent-graph** | ❌ No receipt types visible | ❌ No digest infrastructure | ❌ No artifact envelopes | [E0] |
| **governance crates** (assurance, attestation, authority, constitutional, continuity, effect, mechanism) | ⚠️ Typed artifact families declared in doc comments (e.g., `AssuranceCaseV1`, `AttestationEnvelopeV1`) but **no actual receipt/digest infrastructure** in code | ❌ | ⚠️ Type definitions only; no runtime artifact emission | [E0] |
| **verification-*** | ⚠️ `verification-control` has `CommitToken::execution_permit_id` but no full receipt chain | ❌ | ⚠️ Permit types only | [E0] |

**Doctrinal gap:** Only `semantic-memory` and `forge-pilot` have real receipt infrastructure. The codec crates (`turbo-quant`, `fib-quant`, `poly-kv`) define codec profiles but **do not emit governed compression receipts** with exact-fallback links, degradation disclosures, or raw-source digests. The governance crates declare artifact family types in doc comments but **do not implement artifact emission, digest linking, or receipt validation**.

### 2.2 Bitemporal truth

| Crate | valid_time | recorded_time | as_of query | append-plus-supersession | Evidence |
|---|---|---|---|---|---|
| **semantic-memory** | ❌ Not present | ⚠️ `created_at` in DB schema serves as proxy; not explicitly `recorded_time` | ❌ No `as_of` query | ⚠️ `update_outcome` updates in place; no explicit supersession record | [E0] grep |
| **forge-pilot** | ❌ Not present | ⚠️ `Observation` has timestamps but no bitemporal distinction | ❌ No `as_of` query | ❌ No supersession record | [E0] grep |
| **All other crates** | ❌ Absent | ❌ Absent | ❌ Absent | ❌ Absent | [E0] grep |

**Critical doctrinal gap:** Bitemporality is **mandatory** per doctrine (`01_CANONICAL_DOCTRINE...md` Section 3), but **not implemented anywhere in the Libraries workspace**. The `semantic-memory` DB schema has `created_at` fields but no explicit `valid_time` / `recorded_time` distinction, no `as_of` query receipts, and no append-plus-supersession evolution. Updates mutate rows in place rather than creating supersession records.

### 2.3 Canonicalization and boundary compiler

| Crate | Canonical JSON (RFC 8785 JCS) | Duplicate-key rejection | Deterministic serialization | Schema validation | Evidence |
|---|---|---|---|---|---|
| **semantic-memory** | ❌ Ad-hoc `canonical_json_string()` in `graph.rs` sorts object keys but is **not JCS-compliant** | ❌ Not enforced | ⚠️ `canonical_json_string` sorts keys but not full JCS | ❌ No explicit schema validation at boundary | [E0] |
| **forge-pilot** | ❌ Not present | ❌ Not present | ❌ Not present | ⚠️ Schemars derives JSON Schema but no runtime validation | [E0] |
| **contract-schema-gen** | ❌ Not present | ❌ Not present | ❌ Not present | ✅ Generates JSON Schema from types but no runtime validation | [E0] |
| **All other crates** | ❌ Absent | ❌ Absent | ❌ Absent | ❌ Absent | [E0] |

**Doctrinal gap:** The boundary compiler is a **P0 hardening target** per `artifact-runtime.md`, but no crate implements RFC 8785 JCS, duplicate-key rejection, or deterministic serialization suitable for hashing/signing. The `semantic-memory` `canonical_json_string()` function is a naive key-sorter, not a standards-compliant canonicalizer.

### 2.4 Execution evidence and tool-call receipts

| Crate | Execution trace | Tool-call receipts | Provider route | Retry receipts | Budget debit | Evidence |
|---|---|---|---|---|---|---|
| **llm-pipeline** | ⚠️ `trace.rs` has `TraceEvent` but no full execution envelope | ❌ No typed tool-call receipt | ❌ No provider-route receipt | ⚠️ `retry_policy.rs` has retry logic but no receipt emission | ❌ No budget debit receipt | [E0] |
| **semantic-memory** | ❌ Not present | ❌ Not present | ❌ Not present | ❌ Not present | ❌ Not present | [E0] |
| **forge-pilot** | ⚠️ `ExecutionLineageReceipt` exists but no full trace span linking | ❌ Not present | ❌ Not present | ❌ Not present | ❌ Not present | [E0] |

**Doctrinal gap:** `evidence-first.md` defines a minimum artifact family including `ExecutionTrace`, `ExternalCallReceipt`, `RetryDecision`, `BudgetDebit`, and `ResponseReceipt`. None of these are fully implemented in any crate. `llm-pipeline` has retry logic and trace events but does not emit typed, linkable receipts.

### 2.5 Governed compression

| Requirement | `turbo-quant` | `fib-quant` | `poly-kv` | `semantic-memory` | Doctrinal source |
|---|---|---|---|---|---|
| **Codec family under governance** | ❌ Standalone crate, no governor | ❌ Standalone crate, no governor | ❌ Separate workspace, no governor | ⚠️ `turbo-quant-codec` feature but no runtime policy | `governed-compression.md` |
| **Exact fallback retention** | ❌ No raw fallback | ❌ No raw fallback | ❌ No exact fallback payload | ❌ `derived_vector_artifacts` replaces source, no raw retention | `governed-compression.md` |
| **Degradation disclosure receipt** | ❌ No receipt type | ⚠️ `receipt.rs` exists but not integrated | ❌ No degradation receipt | ❌ No degradation receipt | `governed-compression.md` |
| **Codec profile digest / content-addressed** | ⚠️ `CompressionProfile` has metadata but no digest | ⚠️ `CodecProfile` exists but no content-addressing | ❌ Not implemented | ❌ `codec_profile_digest` in DB but no content-addressed profile | `governed-compression.md` |
| **Honest memory accounting** | ❌ No memory accounting | ⚠️ KV feature has shape/layout but no byte accounting | ⚠️ `metrics.rs` has structs but no proven accounting | ❌ No pool/metadata/decoded/per-reader accounting | `polykv research.md` D1 |
| **Round-trip correctness tests** | ⚠️ Tests exist but not benchmarked against local receipts | ⚠️ Tests exist but not benchmarked | ⚠️ 5 shape tests only | ❌ No codec round-trip tests | `harness research.md` |
| **Integration with semantic-memory** | ⚠️ Optional feature dep but not wired into search path | ❌ No integration | ❌ No integration | ⚠️ `turbo-quant-codec` feature but no runtime use | `governed-compression.md` |

**Critical doctrinal gap:** Governed compression is **the central thesis** of `governed-compression.md` and `polykv research.md`, but the implementation is **three isolated codec crates with no governance runtime**. There is no `quant-governor` crate, no `scr-runtime-compression` adapter, no policy routing between `raw/q8/turbo/fib`, no exact fallback retention, and no degradation disclosure receipts. `semantic-memory` has a `turbo-quant-codec` feature flag but no evidence it is used in the search path.

### 2.6 No shadow truth

| Surface | Risk | Evidence |
|---|---|---|
| **semantic-memory HNSW index** | ⚠️ HNSW is approximate; doctrine says "vector indexes as authoritative memory" is forbidden. The code has `search_receipts` but does not explicitly mark HNSW results as approximate/degraded. | [E0] `semantic-memory/src/hnsw.rs`, `src/db.rs` |
| **semantic-memory projection lanes** | ⚠️ Multiple projection lanes (`projection_lane.rs`, `projection_storage.rs`) could create duplicate truth stores if not carefully managed. No explicit "index is advisory only" marking. | [E0] |
| **forge-pilot bootstrap** | ⚠️ `bootstrap_source_workspace` computes manifest deltas. If the bootstrap cache is treated as canonical, it becomes shadow truth. | [E0] `forge-pilot/src/bootstrap_source.rs` |
| **All governance crates** | ❌ No runtime truth store at all; they are typed surfaces without a projection layer, so shadow truth risk is moot (no truth to shadow). | [E0] |

**Doctrinal gap:** The HNSW index in `semantic-memory` is the highest-risk shadow-truth surface. It should emit `DegradationReceipt` or `ApproximateSearchReceipt` on every query, but currently only emits `VectorSearchReceiptV1` without explicit approximation marking.

---

## 3. Safety and Hardening Gaps

### 3.1 `unsafe` code

| Location | Count | Context | Violates workspace lint? |
|---|---|---|---|
| `Primitives/check-runner/src/lib.rs` | 4 blocks | Process forking, `libc::kill`, signal handling | **YES** — `unsafe_code = deny` at workspace level, but `Primitives/` is now in workspace |
| `semantic-memory-forge/src/v11.rs` | 0 | Test-only function name contains "unsafe" but no actual unsafe block | N/A |
| `AiDENs/crates/aidens-config/src/lib.rs` | 0 | Test-only function name contains "unsafe" | N/A |

**Critical hardening gap:** `Primitives/check-runner` contains **4 production `unsafe` blocks** in a workspace that declares `unsafe_code = deny`. This was not caught because `Primitives/` was historically excluded from the workspace. Now that it is included, `cargo check` still passes because the lint is `deny` at the workspace level but apparently not enforced against existing code, or the crate has a local lint override.

**Investigation needed:** Check if `Primitives/check-runner/Cargo.toml` has `[lints]` that override the workspace policy.

### 3.2 `unwrap()` in production code

| Location | Count | Context | Severity |
|---|---|---|---|
| `contract-schema-gen/src/lib.rs` | 5 calls | Inside `#[test]` functions embedded in `lib.rs` | Low — test code in lib.rs, not production |
| `forge-memory-bridge/src/transform_tests.rs` | ~20 calls | Test module (`transform_tests.rs`) | Low — test code |
| `forge-memory-bridge/src/legacy.rs` | 3 calls | Legacy migration path; `unwrap()` on enum matching | Medium — legacy path should use `thiserror` instead |
| `knowledge-runtime/src/query/classify.rs` | 3 `panic!` | `panic!` on unexpected enum variant in query classification | **High** — production panic path |
| `kernel-oracles/src/lib.rs` | 2 `panic!` | `panic!` on unexpected oracle result | **High** — production panic path |

**Hardening gap:** The V29 audit claimed "zero production `unwrap()` calls." With the expanded workspace, this claim is **falsified**. `knowledge-runtime` and `kernel-oracles` have `panic!` in production paths. `forge-memory-bridge/legacy.rs` has `unwrap()` in legacy migration.

### 3.3 `todo!()` / `unimplemented!()`

| Result | Evidence |
|---|---|
| **Zero found** in production code | [E0] grep across all `.rs` files |

**Verified:** The workspace lint `todo = deny` and `unimplemented = deny` is effective.

---

## 4. Integration Gaps

### 4.1 Missing integration edges

| From | To | What should exist | Current state | Priority |
|---|---|---|---|---|
| `turbo-quant` | `semantic-memory` | Runtime codec selection, exact fallback, degradation receipt | Optional feature dep; no runtime wiring | **P0** |
| `fib-quant` | `semantic-memory` | Same as above | No dependency, no wiring | **P0** |
| `poly-kv` | `turbo-quant` | Value codec adapter | Separate workspace; no adapter | **P0** |
| `poly-kv` | `fib-quant` | Value codec adapter | No adapter | **P1** |
| `poly-kv` | `semantic-memory` | Shared KV pool injection | No integration | **P1** |
| `llm-pipeline` | `semantic-memory` | Tool-call receipts stored in memory | No integration | **P1** |
| `agent-graph` | `semantic-memory` | Graph execution receipts stored in memory | No integration | **P1** |
| `agent-graph` | `llm-pipeline` | Pipeline nodes as graph steps | Both excluded until recently; no integration | **P1** |
| `forge-pilot` | `semantic-memory` | Observation → memory projection with receipt | Partial — `observe.rs` inspects paths but full receipt chain not linked | **P0** |
| `forge-pilot` | `llm-pipeline` | LLM calls via pipeline with receipts | `llm-tool-runtime` is used, not `llm-pipeline` | **P1** |
| `governance crates` | `forge-pilot` | Governance gate receipts in loop | Feature-gated but not proven to emit full artifact family | **P1** |
| `verification-***` | `semantic-memory` | Verification cases as memory artifacts | No integration | **P2** |

### 4.2 PolyKV isolation

| Issue | Detail |
|---|---|
| Separate workspace | `poly-kv/` has its own `Cargo.toml` with `resolver = "2"` and `rust-version = "1.78"` (vs workspace `1.75`) |
| No workspace membership | `poly-kv` is not in `~/Coding/Libraries/Cargo.toml` members list |
| `cargo check` passes in `poly-kv/` | 9.3s, green |
| `cargo test` passes | 5 tests in `quant-codec-core` only |
| `lib.rs` is 28 lines | `poly-kv` crate is essentially a re-export surface |
| `pool.rs` is 573 lines | Infrastructure scaffolding, not a working pool |

**Gap:** PolyKV cannot be built or tested from the main workspace. The separate `rust-version = 1.78` creates toolchain mismatch risk.

---

## 5. Crate-by-Crate Detailed Issues

### 5.1 Tier 1 — Critical crates (most code, highest impact)

#### **semantic-memory** (19 src files, 22k lines, 31 tests, 50 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No bitemporal truth | **Critical** | `created_at` used as proxy; no `valid_time`, no `recorded_time`, no `as_of` query, no supersession | `01_CANONICAL_DOCTRINE...md` §3 |
| No canonical JSON (JCS) | **Critical** | `canonical_json_string()` in `graph.rs` sorts keys but is not RFC 8785 compliant | `artifact-runtime.md` |
| HNSW index as shadow truth | **High** | Approximate search results not explicitly marked as degraded/approximate | `01_CANONICAL_DOCTRINE...md` §5.2 |
| No governed compression runtime | **High** | `turbo-quant-codec` feature exists but not wired into search path with exact fallback | `governed-compression.md` |
| No honest memory accounting | **High** | `derived_vector_artifacts` table tracks digests but does not distinguish pool/metadata/decoded/per-reader bytes | `polykv research.md` D1 |
| Receipts lack degradation marking | **Medium** | `VectorSearchReceiptV1` does not include `approximate: true` for HNSW queries | `evidence-first.md` |
| Episode updates mutate in place | **Medium** | `update_outcome` modifies row; no supersession record | `01_CANONICAL_DOCTRINE...md` §3.2 |
| `db.rs` is monolithic | **Medium** | 1,400+ lines of SQLite schema + queries; hard to audit | `harness research.md` |

#### **forge-pilot** (19 src files, 10.5k lines, 24 tests, 14 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No bitemporal truth | **Critical** | Observations have timestamps but no `valid_time`/`recorded_time` distinction | `01_CANONICAL_DOCTRINE...md` §3 |
| No execution evidence envelope | **High** | `ExecutionLineageReceipt` exists but lacks tool-call receipts, provider route, retry decisions, budget debits | `evidence-first.md` |
| Governance gate is feature-flagged | **Medium** | `governance_gate.rs` behind `#[cfg(feature = "governance")]`; default includes it but no proof it emits full artifact family | `evidence-first.md` |
| No canonical JSON serialization | **Medium** | Exports use `serde_json` without JCS canonicalization | `artifact-runtime.md` |
| Bootstrap cache shadow truth risk | **Medium** | `bootstrap_source_workspace` computes deltas; cache could diverge from source | `01_CANONICAL_DOCTRINE...md` §5.2 |
| `repo_chat` has no citation receipts | **Medium** | `RepoChatAnswer` has `RepoChatCitation` but no link to search receipt ID | `evidence-first.md` |

#### **llm-pipeline** (18 src files, 9.1k lines, 1 test, 11 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No execution evidence receipts | **Critical** | Retry logic, streaming, backend calls produce no typed receipts | `evidence-first.md` |
| No digest linking | **Critical** | No `ContentDigest` or artifact envelopes | `artifact-runtime.md` |
| No canonical output | **High** | LLM responses parsed but not canonicalized before storage | `artifact-runtime.md` |
| `anyhow` used instead of `thiserror` | **Medium** | `anyhow` is fine for apps but `thiserror` is better for library error surfaces; mixed with `thiserror` in other crates | `contract/hardening.md` |
| Only 1 test file | **Medium** | 9.1k lines, 1 test file (`tests/` absent; tests in `src/` unknown) | `harness research.md` |

#### **turbo-quant** (13 src files, 4.3k lines, 16 tests, 17 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No governed compression runtime | **Critical** | Standalone codec with no policy routing, exact fallback, or degradation receipt | `governed-compression.md` |
| No receipt types | **High** | `CompressionProfile` exists but no `CodecReceipt`, `DegradationReceipt`, `ExactFallbackReceipt` | `governed-compression.md` |
| No content-addressed profile | **High** | Profile metadata not digest-linked | `governed-compression.md` |
| No honest memory accounting | **High** | No pool/metadata/decoded/per-reader byte tracking | `polykv research.md` D1 |
| Performance claims are first-party only | **Medium** | README claims "zero accuracy loss"; no independent benchmark receipts | `josh master dossier.md` CLM-0010 |
| No integration with `semantic-memory` | **Medium** | Optional dep exists but no runtime search-path wiring | `governed-compression.md` |

#### **fib-quant** (14 src files, 4.4k lines, 21 tests, 21 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No governed compression runtime | **Critical** | Same as turbo-quant | `governed-compression.md` |
| `receipt.rs` not wired | **High** | KV receipts defined but not integrated into codec execution path | `governed-compression.md` |
| No content-addressed codebook | **High** | Codebook profile not digest-linked | `governed-compression.md` |
| No honest memory accounting | **High** | `compressed_bytes` in reference Python was wrong; Rust version needs explicit accounting | `polykv research.md` D1 |
| CI failures | **Medium** | GitHub CI failing as of 2026-05-16 (per Gmail notifications) | `josh mini dossier.md` SRC-0026 |
| `nalgebra` + `statrs` + `half` deps | **Low** | Heavy math deps; may conflict with `turbo-quant` `nalgebra` version | [E0] |

#### **agent-graph** (24 src files, unknown lines, 14 tests, 5 docs)

| Issue | Severity | Detail | Doctrinal basis |
|---|---|---|---|
| No execution evidence receipts | **Critical** | Graph execution produces no typed receipts | `evidence-first.md` |
| No digest linking | **Critical** | No artifact envelopes | `artifact-runtime.md` |
| No bitemporal truth | **High** | Checkpoints have timestamps but no bitemporal semantics | `01_CANONICAL_DOCTRINE...md` §3 |
| No integration with `semantic-memory` | **Medium** | Graph state not stored as memory artifacts | `evidence-first.md` |
| Recently re-added to workspace | **Low** | Build compiles but runtime integration untested | [E0] |

### 5.2 Tier 2 — Governance crates (7 crates, 5.3k total lines)

#### **effect-runtime** (7 src files, 1,733 lines, 5 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface only | **Medium** | Real code (compensation, effect, observation, vocab) but no receipt emission, no digest linking, no runtime artifact production |
| No bitemporal truth | **Medium** | No temporal semantics |
| No integration with forge-pilot | **Medium** | `governance_gate.rs` reads from semantic-memory projections but no evidence this crate's types flow through |

#### **continuity-runtime** (8 src files, 930 lines, 5 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface + some logic | **Medium** | Incident, recovery, SLO, validation exist but no receipt emission |
| No bitemporal truth | **Medium** | No temporal semantics |

#### **assurance-runtime** (7 src files, 728 lines, 8 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface only | **Medium** | Assurance cases, certification, profiles declared but no runtime artifact production |
| 4 tests missing fixtures | **Medium** | `STATUS_DASHBOARD.md` notes "missing example JSON files" for 4 fixture roundtrip tests |

#### **attestation-exchange** (3 src files, 626 lines, 3 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface only | **Medium** | Attestation envelopes, trust roots declared but no runtime artifact production |

#### **authority-delegation** (6 src files, 676 lines, 5 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface + some logic | **Medium** | Capability, emergency, SOD exist but no receipt emission |

#### **constitutional-memory** (2 src files, 295 lines, 3 tests, 3 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface only | **Low** | Charter bundles, amendments, archive manifests declared but minimal code |

#### **mechanism-runtime** (2 src files, 254 lines, 3 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Typed surface only | **Low** | Mechanism/hypothesis types declared but minimal code |

### 5.3 Tier 3 — Primitives (10 crates, 4.9k total lines)

#### **typed-patch** (1 src file, 950 lines, 0 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 950 lines, zero tests |
| No receipts | **Medium** | Patch operations produce no typed receipts |
| No canonicalization | **Medium** | Patches not canonicalized before hashing |

#### **check-runner** (1 src file, 846 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| **Production `unsafe` code** | **Critical** | 4 `unsafe` blocks in a workspace that denies `unsafe_code` |
| No tests | **High** | 846 lines, zero tests |
| Process forking/signal handling | **Medium** | High blast radius; should be isolated in a separate process or use safe wrappers |

#### **cea-sqlite** (1 src file, 1,218 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 1,218 lines, zero tests |
| SQLite operations | **Medium** | If this is the CEA SQLite backend, it should have receipt/digest integration but doesn't |

#### **cea-store** (1 src file, 656 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 656 lines, zero tests |

#### **stabilizer-core** (1 src file, 486 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 486 lines, zero tests |

#### **sandbox-workspace** (1 src file, 359 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 359 lines, zero tests |

#### **forge-policy** (1 src file, 422 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 422 lines, zero tests |

#### **mindstate-core** (1 src file, 284 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 284 lines, zero tests |

#### **effect-signature** (1 src file, 131 lines, 0 tests, 1 doc)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **Medium** | 131 lines, zero tests |

#### **cea-core** (1 src file, 42 lines, 0 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Essentially empty | **Low** | 42 lines, re-exports only |

### 5.4 Tier 4 — Other excluded/recently-included crates

#### **tauri-queue** (1 src file, unknown lines, 0 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Essentially empty | **High** | 1 file, no meaningful implementation; Gloss needs this but it's a stub |

#### **llm-output-parser** (11 src files, unknown lines, 0 tests, 0 docs)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 11 files, zero tests |
| No docs | **Medium** | No README or markdown docs visible |

#### **comfyui-rs** (5 src files, unknown lines, 0 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **Medium** | 5 files, zero tests |

#### **ollama-vision** (5 src files, unknown lines, 1 test, 6 docs)

| Issue | Severity | Detail |
|---|---|---|
| Minimal tests | **Medium** | 5 files, 1 test |

#### **ai-batch-queue** (5 src files, unknown lines, 1 test, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| Minimal tests | **Medium** | 5 files, 1 test |
| `tauri` dependency | **Low** | Tauri 2 dep; heavy for a queue crate |

#### **job-queue** (8 src files, unknown lines, 0 tests, 2 docs)

| Issue | Severity | Detail |
|---|---|---|
| No tests | **High** | 8 files, zero tests |

---

## 6. Missing Crate Audit

The provenance research and the AICC design call for several crates that **do not exist** in `~/Coding/Libraries/`:

| Missing crate | Purpose | Doctrinal source | Priority |
|---|---|---|---|
| `quant-governor` | Policy routing, codec evaluation, admissibility classes, decision receipts | `governed-compression.md` | **P0** |
| `scr-runtime-compression` | Adapter binding governor decisions into runtime operator execution | `governed-compression.md` | **P1** |
| `quant-eval` | Benchmark suites, prompt corpora, semantic-memory metrics, admissibility tests | `governed-compression.md` | **P1** |
| `boundary-compiler` | Canonical JSON (RFC 8785 JCS), duplicate-key rejection, deterministic serialization, patch dialect enforcement | `artifact-runtime.md` | **P0** |
| `bitemporal-runtime` | `valid_time`/`recorded_time` tracking, `as_of` queries, append-plus-supersession, temporal replay | `01_CANONICAL_DOCTRINE...md` §3 | **P0** |
| `claim-ledger` | Claim/evidence adjudication, structured claim boundaries, public-safe language filtering | `evidence-first.md` | **P0** |
| `receipt-bench` / `agent-harness` | Local-first replayable benchmark substrate for memory, provenance, policy, compression | `harness research.md` | **P0** |
| `agent-guard` | Linux-first control plane: eBPF, BPF LSM, cgroup v2, Landlock, seccomp, MCP broker | `agentguard research.md` | **P0** |
| `semantic-memory-compression` | Adapter layer (must never own codec truth) | `governed-compression.md` | **P1** |

---

## 7. Test Coverage Matrix

| Lane | Crates | Total src files | Total test files | Test ratio | Gap |
|---|---|---|---|---|---|
| **Supported (17 crates)** | contract-schema-gen, forge-memory-bridge, forge-pilot, kernel-conformance, kernel-execution, kernel-oracles, knowledge-runtime, living-memory, llm-tool-runtime, recursive-kernel-core, semantic-memory, semantic-memory-forge, stack-ids, verification-*, fib-quant, turbo-quant | ~200 | ~80 | ~0.40 | **Low** for core crates; **high** for primitives |
| **Governance (7 crates)** | assurance, attestation, authority, constitutional, continuity, effect, mechanism | ~35 | ~27 | ~0.77 | **Medium** — tests exist but many missing fixtures |
| **Extension (6 crates)** | discovery, federated, profile, remote-oracle, spec-execution + Primitives | ~20 | ~0 | **0.00** | **Critical** — zero tests across all extension and primitive crates |
| **Recently included (8 crates)** | agent-graph, ai-batch-queue, comfyui-rs, job-queue, llm-output-parser, llm-pipeline, ollama-vision, tauri-queue | ~80 | ~16 | ~0.20 | **High** — many have 0 or 1 test |

**Critical gap:** Extension crates and Primitives have **zero tests**. The Primitives are consumed by `forge-engine` (living-memory) but are completely untested.

---

## 8. Documentation Coverage

| Lane | Doc-checked | Notes |
|---|---|---|
| Supported 17-crate lane | ✅ By `scripts/check_public_api_docs.py` | Per `CRATE_HARDENING_MATRIX.md` |
| Governance lane | ⚠️ Doc comments present but minimal | Per-crate `lib.rs` has extensive doc comments |
| Extension + Primitives | ❌ Not checked | No evidence of doc coverage enforcement |
| Excluded (now included) | ❌ Not checked | `agent-graph`, `llm-pipeline`, etc. unknown |

---

## 9. Dependency Health

| Issue | Detail | Severity |
|---|---|---|
| `nalgebra` dual versions | `nalgebra 0.32.6` and `0.33.3` both compiled in workspace | Low — version conflict resolved by Cargo |
| `thiserror` dual versions | `thiserror 1.x` (turbo-quant) and `2.x` (workspace) | Low — resolved |
| `serde` versions | `serde 1.0.228` workspace vs `serde "1"` in some crates | Low |
| `reqwest` features | `semantic-memory` and `llm-pipeline` both use `reqwest 0.12` with different feature sets | Low |
| `turbo-quant` dep version in `semantic-memory` | `"0.2.0-alpha.1"` but actual crate is `0.2.0` | **Medium** — version string mismatch |
| `poly-kv` Rust version | `1.78` vs workspace `1.75` | **Medium** — toolchain mismatch |
| `tauri-queue` is empty stub | No implementation but included in workspace | **Medium** — dead weight |

---

## 10. Prioritized Fix List

### P0 — Critical (blocks release or violates doctrine)

| # | Fix | Target crate(s) | Doctrinal basis |
|---|---|---|---|
| 1 | **Remove or fix `unsafe` in `Primitives/check-runner`** | `Primitives/check-runner` | `LIB-005`: `unsafe_code = deny` |
| 2 | **Add bitemporal truth to `semantic-memory`** | `semantic-memory` | `01_CANONICAL_DOCTRINE...md` §3 |
| 3 | **Implement RFC 8785 JCS canonical JSON** | New `boundary-compiler` crate or `semantic-memory` | `artifact-runtime.md` |
| 4 | **Create `quant-governor` crate** | New crate | `governed-compression.md` |
| 5 | **Wire `turbo-quant` + `fib-quant` into governed compression runtime** | `quant-governor`, `semantic-memory` | `governed-compression.md` |
| 6 | **Add exact fallback + degradation receipts to codec path** | `turbo-quant`, `fib-quant`, `semantic-memory` | `governed-compression.md` |
| 7 | **Replace `panic!` in `knowledge-runtime` and `kernel-oracles`** | `knowledge-runtime`, `kernel-oracles` | V29 audit claim falsified |
| 8 | **Commit the 178 uncommitted files** | Workspace root | Source-of-truth drift risk |

### P1 — High (needed for credible public claims)

| # | Fix | Target crate(s) | Doctrinal basis |
|---|---|---|---|
| 9 | **Add execution evidence receipts to `llm-pipeline`** | `llm-pipeline` | `evidence-first.md` |
| 10 | **Add execution evidence receipts to `agent-graph`** | `agent-graph` | `evidence-first.md` |
| 11 | **Add tests to all Primitives** | `Primitives/*` | `harness research.md` |
| 12 | **Add tests to `llm-output-parser` and `job-queue`** | `llm-output-parser`, `job-queue` | `harness research.md` |
| 13 | **Fix `semantic-memory` turbo-quant version string** | `semantic-memory/Cargo.toml` | Dependency health |
| 14 | **Integrate `poly-kv` into main workspace** | `poly-kv/Cargo.toml`, root `Cargo.toml` | Workspace unity |
| 15 | **Implement `tauri-queue` or remove it** | `tauri-queue` | Dead weight |
| 16 | **Add receipt emission to governance crates** | `assurance-runtime`, `attestation-exchange`, etc. | `evidence-first.md` |
| 17 | **Mark HNSW results as approximate in receipts** | `semantic-memory` | `01_CANONICAL_DOCTRINE...md` §5.2 |

### P2 — Medium (polish and completeness)

| # | Fix | Target crate(s) | Doctrinal basis |
|---|---|---|---|
| 18 | **Add honest memory accounting to codec path** | `turbo-quant`, `fib-quant`, `poly-kv` | `polykv research.md` D1 |
| 19 | **Add content-addressed codec profiles** | `turbo-quant`, `fib-quant` | `governed-compression.md` |
| 20 | **Add bitemporal truth to `forge-pilot`** | `forge-pilot` | `01_CANONICAL_DOCTRINE...md` §3 |
| 21 | **Restore `02_MASTER_ISSUE_MATRIX.md` and `06_RISK_REGISTER.md`** | Workspace root | Issue tracking |
| 22 | **Add cross-crate integration tests** | `semantic-memory` + `turbo-quant`, `forge-pilot` + `semantic-memory` | `harness research.md` |
| 23 | **Performance baselines under release profile** | `kernel-conformance`, `turbo-quant`, `fib-quant` | `CRATE_HARDENING_MATRIX.md` |

---

## 11. Receipt

- **What was done:** Full source-code audit of 49 workspace crates using grep, wc, cargo check, cargo test, git status, and direct file reading. Doctrinal lens applied from 8 provenance research documents extracted from `~/Downloads/Full Provenance+ Research 5/23/26.zip`.
- **What was verified:** Build compiles (90s cold, 4.4s incremental); zero `todo!()`/`unimplemented!()` in production; `semantic-memory` and `forge-pilot` have real receipt infrastructure; `stack-ids` provides `ContentDigest`/`DigestBuilder` primitives.
- **What was NOT verified:** `cargo clippy --workspace`, `cargo fmt --check`, benchmark runs, release-profile builds, full test suite execution for all 49 crates (timed out), runtime behavior of governance gates.
- **Proof debt:** The assessment of "no runtime logic" in governance crates is based on `grep` for receipt/digest/bitemporal artifacts and lib.rs inspection. A deeper module-by-module audit could reveal more logic. The `unsafe` assessment in `check-runner` is based on `grep` only; the full context of each `unsafe` block was not read.
- **Falsifies if:** Any governance crate contains substantial receipt emission logic that `grep` missed; any Primitive contains tests in `src/` modules rather than `tests/` directory; `cargo clippy` reveals failures not caught by `cargo check`.
