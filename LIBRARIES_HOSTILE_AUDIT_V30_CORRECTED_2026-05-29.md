# Libraries Hostile Audit — V30 Corrected (2026-05-29)

**Inspector:** deepseek-v4-pro (self-identifying)
**Evidence class:** E0 (direct build, test execution, source inspection, git log)
**Basis:** 57 workspace crates, full cargo check, per-crate test execution
**Correction note:** This audit replaces the stale 2026-05-27 audit and the erroneous first-pass 2026-05-29 audit. Several claims from that first pass were WRONG — see Section 0 for retractions.

---

## 0. RETRACTED CLAIMS (things the first-pass audit got wrong)

| False Claim | Reality |
|---|---|
| "Primitives have ZERO tests" | **79 tests across 10 crates, all passing** (cea-core 11, cea-sqlite 10, cea-store 5, check-runner 11, effect-signature 5, forge-policy 7, mindstate-core 7, sandbox-workspace 11, stabilizer-core 6, typed-patch 6) |
| "forge-memory-bridge legacy.rs has 3 production panic!" | All 3 are **inside #[test] functions** — no production panic in this file |
| "knowledge-runtime has production panic!" | Replaced with `unreachable!` (still not ideal, but intentional V30 hardening) |
| "kernel-oracles has production panic!" | Same — `unreachable!` replacement |
| "8 new crates are untested skeletons" | All have real implementations; quant-eval has 19 tests; receipt-bench has 5; others have doctests |
| "llm-pipeline has zero receipts" | Has `ToolReceipt`, `ToolLoopRunner`, `ToolReceiptSink` wired from llm-tool-runtime |
| "tauri-queue is a stub" | 376 lines, real re-export wrapper bridging job-queue events to Tauri frontend |
| "178 uncommitted files" | Only ~51 uncommitted (36 untracked AiDENs codex artifacts, 14 modified AiDENs files, 1 deleted, 2 bundles/) |

---

## 1. WHAT V30 ACTUALLY ACCOMPLISHED (git log: 77165c0 → 859134a)

Phases 1-3 were committed by the user. This was NOT caught by the first-pass audit.

### Phase 1 (Workspace Integrity) — DONE

| Task | Commit | Status |
|---|---|---|
| check-runner unsafe fix | 77165c0 | `killpg` scoped-allow with safety justification |
| Replace panic! with unreachable! | 77165c0 | knowledge-runtime (5 sites), kernel-oracles (2 sites) |
| Primitives test survey | 77165c0 | 79 tests verified across 10 crates |
| Workspace lint enforcement | 77165c0 | Confirmed |

### Phase 2 (8 New Crates) — DONE

| Crate | Files | Lines | Real Code? | Tests |
|---|---|---|---|---|
| claim-ledger | 6 .rs | ~600+ | Yes — types, ids, ledger, receipt, error | 1 doctest |
| boundary-compiler | 6 .rs | ~830+ | Yes — RFC 8785 JCS canonicalizer, duplicate-key scan, digest, profile, schema | 1 doctest |
| bitemporal-runtime | 4 .rs | ~570+ | Yes — BitemporalRecord<T>, queries, types | 0 tests |
| quant-governor | 6 .rs | ~940 | Yes — policy, decision, degradation, receipt, error | 1 doctest |
| agent-guard | 4 .rs | ~290+ | Yes — Linux control_plane, types, receipt, error | 1 doctest (ignored) |
| receipt-bench | 5 .rs | ~875 | Yes — suite, receipt, fingerprint, error | 5 tests |
| scr-runtime-compression | 5 .rs | ~? | Yes — CompressedSearchPath, ExactFallbackAdapter, codec_dispatch | 1 doctest + 1 ignored |
| quant-eval | 8 .rs | ~1,700 | Yes — compression, semantic, admissibility benchmarks | **19 tests** + 5 doctests |

### Phase 3 (Wiring + Fixes) — DONE

| Task | Commit | Status |
|---|---|---|
| turbo-quant/fib-quant → quant-governor wiring | e2a7971 | scr-runtime-compression codec_dispatch module, factory functions, adapter integration |
| SM-AUD-0064 (PRAGMA foreign_keys) | 32620a8 | Assert after connection config |
| SM-AUD-0065 (max_page_count validation) | 32620a8 | Range check before setting |
| PolyKV workspace merge | a2cee0a | poly-kv + quant-codec-core in workspace, rust-version 1.78→1.75 |
| SM-AUD-0058/0059 (episode_id fixes) | 28ef811 | search_episodes + get_episode use correct IDs |
| semantic-memory turbo-quant version | Earlier | Fixed to "0.2.0" |

### Phase 4-5 (Closeout) — COMMITTED

| Task | Commit | Status |
|---|---|---|
| P31A finish pack | 66ad0a5 | Hostile audit finish pack, boundary-compiler-core merged |
| Root docs sync | 859134a | All root docs to certified state |
| cargo fmt + test cleanup | 0e62100 | Phase 5 final cleanup |

---

## 2. REMAINING P0 GAPS (verified — not retracted)

### P0-1: semantic-memory has no bitemporal integration

- `bitemporal-runtime` crate exists in workspace with full type system and query primitives
- **Zero references** to it in any semantic-memory source file
- **Zero references** in semantic-memory Cargo.toml
- Episodes use `created_at` only — no valid_time/recorded_time
- Episode updates mutate in place — no append-plus-supersession
- No `as_of` query exists
- **Evidence:** E0 grep of all 27 semantic-memory source files

### P0-2: quant-governor is not wired into semantic-memory

- `quant-governor` has 940 lines of policy, decision, degradation types
- `scr-runtime-compression` has codec_dispatch that wires turbo-quant/fib-quant through the governor
- **But semantic-memory itself does not consume quant-governor**
- **Zero references** in semantic-memory Cargo.toml
- `semantic-memory/src/quantize.rs` does not route through quant-governor
- The governed compression path exists but is not integrated into the primary memory store

### P0-3: JCS canonicalization not wired into semantic-memory

- `boundary-compiler` has a real RFC 8785 JCS canonicalizer (348 lines, duplicate-key detection)
- `semantic-memory/src/graph.rs` still uses its own `canonical_json_string()` function (naive key-sorting)
- Line 198-207 shows the naive approach is still the active path
- The JCS crate exists but is unused by any consumer

### P0-4: check-runner unsafe still violates workspace lint

- 3 production `unsafe` blocks (lines 249, 824, 845/857) plus 1 in test (line 249)
- Workspace declares `unsafe_code = deny` — local `#[allow]` in the code
- The V30 fix documented the justification but did not resolve the structural violation
- **Fix needed:** Extract to `check-runner-sys` crate with explicit `#![allow(unsafe_code)]`, keep main crate clean

### P0-5: 8 new crates lack dedicated test files (only doctests or minimal)

| Crate | Unit tests | Dedicated test files | Status |
|---|---|---|---|
| bitemporal-runtime | 0 | None | **No tests** |
| receipt-bench | 5 (from suite.rs?) | None | **Minimal** |
| agent-guard | 1 (ignored doctest) | None | **No runnable tests** |
| claim-ledger | 1 doctest | None | **Minimal** |
| boundary-compiler | 1 doctest | None | **Minimal** |
| scr-runtime-compression | 1 doctest + 1 ignored | None | **Minimal** |
| quant-governor | 1 doctest | None | **Minimal** |
| quant-eval | 19 tests | tests/integration.rs | **Adequate** |

The V30 plan says "minimal-viable implementation" but bitemporal-runtime with 0 tests is below the bar.

---

## 3. REMAINING P1 GAPS

### P1-1: knowledge-runtime/kernel-oracles use unreachable! instead of thiserror

- The V30 audit plan (Phase 1, task 1.2) said "Replace `panic!` with structured error propagation using `thiserror`"
- What was actually done: replaced `panic!` with `unreachable!`
- `unreachable!` still panics at runtime — it just signals "this branch shouldn't be reachable"
- **Fix needed:** Convert to `thiserror` error propagation as originally planned

### P1-2: claim-ledger not wired into forge-pilot

- `claim-ledger` has types, boundary checks, adjudication
- **Zero references** in forge-pilot Cargo.toml
- No public-claim check in forge-pilot export path

### P1-3: agent-graph has no execution receipt infrastructure

- agent-graph has 0 references to `Receipt` or `receipt` in source (grep verified)
- No `GraphExecutionReceiptV1`, no `StepExecutionReceiptV1`, no `CheckpointReceiptV1`
- This was a P0 doctrinal gap that remains open

### P1-4: llm-pipeline has tool-loop receipts but no pipeline-level receipts

- Has `ToolReceipt` from llm-tool-runtime (tool-level)
- Has `ToolLoopRunner` with receipt sink
- **Missing:** `PipelineExecutionReceiptV1`, `ProviderCallReceiptV1`, `RetryDecisionReceiptV1`, `BudgetDebitV1`
- Per `evidence-first.md`, the pipeline should emit a complete receipt chain

### P1-5: Governance crates — typed surfaces with no runtime receipt emission

- All 7 governance crates have extensive type definitions and doc comments
- `attestation-exchange`: 298 lines with 12 receipt references (doc-comment only)
- Most have 40-46 lines in lib.rs (thin re-export surfaces)
- **None emit runtime receipts**
- Test infrastructure exists (8-21 tests per crate) but no receipt type structures

### P1-6: SM-AUD tracking is incomplete

- SM-AUD-0010, SM-AUD-0011, SM-AUD-0026, SM-AUD-0027, SM-AUD-0042: No grep matches found in source
- Only SM-AUD-0058, 0059, 0064, 0065 were fixed per commit messages
- Memory says "0058/0064/0065 fixed, others pending" — confirmed

---

## 4. P2 / POLISH GAPS

### P2-1: Uncommitted files
- 51 uncommitted: 36 untracked AiDENs codex artifacts (can be gitignored), 14 modified AiDENs files, 2 bundles/
- Root Libraries files are clean — only AiDENs subdirectory + bundles/ are dirty

### P2-2: No release-profile benchmarks
- Per CRATE_HARDENING_MATRIX.md, performance baselines are dev-profile only
- No `cargo bench --release` results in evidence/

### P2-3: Missing tracking docs
- `02_MASTER_ISSUE_MATRIX.md` and `06_RISK_REGISTER.md` still deleted
- Not restored per the V30 plan's Phase 0 task 0.2

### P2-4: Cargo fmt / clippy status
- One clippy warning observed (non-root package profiles)
- `cargo fmt --check` not verified this session

### P2-5: nalgebra dual versions
- `nalgebra 0.32.6` and `0.33.3` both compiled (Cargo resolves but wasteful)

---

## 5. CORRECTED PRIORITIZED EXECUTION PLAN

### Phase 0 — Immediate (today, 1-2 hrs)

| # | Task | What | Verification |
|---|---|---|---|
| 0.1 | .gitignore codex artifacts | Add `AiDENs/AiDENs-aidens-codex-context-*.json`, `AiDENs/AiDENs-aidens-codex-context-*.md` to .gitignore (in AiDENs/) | `git status --short` shows only intentional changes |
| 0.2 | Commit current state | `git add -A` + salvage commit for AiDENs P31A artifacts + bundles/ | Clean tree |
| 0.3 | Restore tracking docs | Restore `02_MASTER_ISSUE_MATRIX.md` + `06_RISK_REGISTER.md` from git history or recreate from this audit | Both files on disk |
| 0.4 | Convert unreachable! to thiserror | knowledge-runtime classify.rs (5 sites), kernel-oracles lib.rs (2 sites) | `grep unreachable!` returns zero in those files |

### Phase 1 — Critical Integration (this week, 6-10 hrs)

| # | Task | What | Verification |
|---|---|---|---|
| 1.1 | **Wire bitemporal-runtime into semantic-memory** | Add dep to Cargo.toml, add DB migration for valid_time/recorded_time/superseded_by columns, replace `update_outcome` with supersession, add `query_episodes_as_of()` API | `cargo test -p semantic-memory` passes; as_of query returns historically correct results |
| 1.2 | **Wire quant-governor into semantic-memory** | Add dep to Cargo.toml, route `quantize.rs` through governor via scr-runtime-compression adapter, add codec_governance_receipt_id to derived_vector_artifacts, store exact fallback | `cargo test -p semantic-memory` passes; compression path emits CodecGovernanceReceiptV1 |
| 1.3 | **Wire boundary-compiler JCS into semantic-memory** | Replace `canonical_json_string()` in graph.rs:226 with `boundary_compiler::canonicalize_value()`, add dep | `cargo test -p semantic-memory` passes; canonical JSON is RFC 8785 compliant |
| 1.4 | **Fix check-runner unsafe structural violation** | Extract unsafe blocks into `Primitives/check-runner-sys/` with explicit `#![allow(unsafe_code)]`, keep main crate deny-compliant | `grep unsafe Primitives/check-runner/src/` returns only SAFETY comments |

### Phase 2 — Receipt Infrastructure (1-2 weeks, 8-16 hrs)

| # | Task | What | Verification |
|---|---|---|---|
| 2.1 | Add execution receipts to llm-pipeline | `PipelineExecutionReceiptV1`, `ProviderCallReceiptV1`, `RetryDecisionReceiptV1`, `BudgetDebitV1` — emit on every pipeline run | Test pipeline run produces complete receipt chain |
| 2.2 | Add execution receipts to agent-graph | `GraphExecutionReceiptV1`, `StepExecutionReceiptV1`, `CheckpointReceiptV1` | Test graph execution produces receipts |
| 2.3 | Add runtime receipt types to governance crates | Each of 7 crates: add receipt module with typed receipt, roundtrip test | `cargo test --workspace` passes with new receipt tests |
| 2.4 | Wire claim-ledger into forge-pilot export | Add dep, run public boundary check before export | E3 claim blocked from public export |
| 2.5 | Fix remaining SM-AUD items | SM-AUD-0010/0011/0026/0027/0042 in semantic-memory | All SM-AUD items resolved or documented |

### Phase 3 — Test Coverage (1 week, 6-10 hrs)

| # | Task | What | Verification |
|---|---|---|---|
| 3.1 | Add tests to bitemporal-runtime | Minimum: 5 tests — supersession chain, as_of query, contradiction, interval overlap, roundtrip | `cargo test -p bitemporal-runtime` ≥ 5 tests pass |
| 3.2 | Add tests to claim-ledger | Minimum: 5 tests — claim creation, evidence linking, boundary check, adjudication, roundtrip | `cargo test -p claim-ledger` ≥ 5 tests pass |
| 3.3 | Add tests to boundary-compiler | Minimum: 5 tests — JCS roundtrip, duplicate-key rejection, known RFC test vectors, schema validation, digest stability | `cargo test -p boundary-compiler` ≥ 5 tests pass |
| 3.4 | Add tests to quant-governor | Minimum: 5 tests — policy routing, strict fallback required, degradation class, admissibility downgrade, byte accounting | `cargo test -p quant-governor` ≥ 5 tests pass |
| 3.5 | Add tests to receipt-bench | Already has 5; verify fixture coverage against B01-B12 requirements | B01, B03, B04, B05, B06 minimum |
| 3.6 | Cross-crate integration tests | semantic-memory + turbo-quant roundtrip, forge-pilot + semantic-memory observation, llm-pipeline + agent-graph | Integration tests pass |

### Phase 4 — Hardening & Verification (2-3 days, 4-6 hrs)

| # | Task | What | Verification |
|---|---|---|---|
| 4.1 | Full lint suite | `cargo fmt --check`, `cargo clippy --workspace -- -D warnings`, fix all warnings | All pass with zero warnings |
| 4.2 | Release-profile benchmarks | `cargo bench` (where available), capture to evidence/ | Baseline JSON committed |
| 4.3 | `cargo test --workspace` full run | Verify every crate passes (fix failures) | ✅ all green |
| 4.4 | Hostile audit re-run | Delegate to fresh subagent for hostile audit of V30 state | All P0 findings resolved or formally waived |
| 4.5 | Closeout receipt | Update CLOSE_V30_RECEIPT.md with final state | Receipt committed |

---

## 6. WHAT'S NOT MISSING (counter to first-pass audit)

- **Primitives tests**: 79 tests across 10 crates, all passing — my claim of "zero" was a test invocation error
- **forge-memory-bridge panics**: All 3 are inside `#[test]` functions — zero production panic
- **SM-AUD-0058/0059/0064/0065**: All committed and fixed
- **turbo-quant version string**: Fixed to "0.2.0"
- **poly-kv workspace merge**: Done — in workspace, rust-version aligned to 1.75
- **scr-runtime-compression codec dispatch**: Wired — `build_adapter()` factory, real codec wiring
- **llm-pipeline receipts**: Has tool-level receipts (ToolReceipt from llm-tool-runtime) — missing pipeline-level only
- **tauri-queue**: 376 lines of real re-export wrapper (not a stub)
- **quant-eval**: 19 integration tests + 5 doctests (not untested)
- **HNSW approximate marking**: Already present in `VectorSearchReceiptV1` (field `approximate: bool`, `candidate_backend`, `approximate_scanned_count`, `exact_rerank`) — my first-pass claim that it's absent was wrong

---

## 7. THE REAL CRITICAL PATH

The V30 plan created 8 foundation crates, but they are **not wired into the crates that need them**. The critical path is:

```
semantic-memory ← bitemporal-runtime   (P0-1: zero integration)
semantic-memory ← quant-governor       (P0-2: zero integration)
semantic-memory ← boundary-compiler    (P0-3: still uses naive sorter)
forge-pilot ← claim-ledger             (P1-2: zero integration)
llm-pipeline → own receipts            (P1-4: missing pipeline-level receipts)
agent-graph → own receipts             (P1-3: missing entirely)
```

**Priority order:** 1.1 (bitemporal) → 1.2 (governor) → 1.3 (JCS) → then receipt infrastructure.

---

## 8. RECEIPT

- **What was done:** Re-audited from git log to discover Phases 1-3 already committed. Verified every P0 claim with direct source inspection, test execution, and grep. Corrected 7 false claims from first-pass audit.
- **What's verified:** cargo check passes, 10 Primitives have 79 passing tests, new crates have real implementations, HNSW approximate marking exists, forked-panics are test-only, SM-AUD-0058/59/64/65 fixed, poly-kv merged, scr-runtime codec_dispatch wired.
- **What's NOT verified:** cargo fmt, cargo clippy, release-profile benchmarks, full workspace test timeout.
- **Proof debt:** SM-AUD-0010/0011/0026/0027/0042 status not confirmed — memory says "pending."
- **Falsifies if:** bitemporal-runtime IS wired into semantic-memory somewhere I missed; quant-governor IS wired somewhere I missed; JCS IS used somewhere I missed; SM-AUD items are all resolved.
- **Hostile-auditor handoff:** The 8 foundation crates exist and have real code. The gap is **integration**, not creation. The remaining work is ~30-40 hours: wiring 4 foundations into semantic-memory/forge-pilot, adding receipt infrastructure, and adding tests to the untested new crates.
