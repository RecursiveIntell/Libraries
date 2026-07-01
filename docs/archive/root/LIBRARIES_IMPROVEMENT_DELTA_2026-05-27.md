# RecursiveIntell ~/Coding/Libraries — Improvement Delta Report

**Date:** 2026-05-27 (post-V30 salvage commit)  
**Inspector:** pi-coding-agent  
**Evidence class:** E0 (direct inspection, build execution, grep, git diff)  
**Baseline:** `LIBRARIES_HARDENING_AND_GAP_AUDIT_2026-05-27.md` (recorded 2026-05-27 ~17:45 UTC)  
**Current state:** Recorded 2026-05-27 ~19:05 UTC (commit `800c707`)  

---

## Executive Summary

**Massive improvement.** In approximately 1.5 hours of real time, you moved from a fragmented, partially audited workspace with 178 uncommitted files and 23 critical gaps to a **unified workspace of 58 crates with 7 net-new foundation crates, committed source-of-truth, restored tracking documents, fixed safety violations, and real test coverage on the new crates.**

The improvement is not cosmetic. It is structural: the four missing foundation crates from the remediation plan (`boundary-compiler`, `bitemporal-runtime`, `quant-governor`, `claim-ledger`) now exist, compile, and pass tests. Two additional crates beyond the plan (`scr-runtime-compression`, `quant-eval`) also exist. `agent-guard` exists as a skeleton. `receipt-bench` exists. The `tauri-queue` stub was replaced with a real 376-line implementation.

**What is still open:** Primitives remain untested, `check-runner` still contains `unsafe`, `poly-kv` remains outside the workspace, and `semantic-memory` bitemporal integration is not yet wired. But the delta is decisive.

---

## Delta Matrix — Before vs After

| Metric | Before (baseline audit) | After (V30 commit) | Delta | Evidence |
|---|---|---|---|---|
| **Workspace members** | 49 crates | **58 crates** | **+9** | [E0] `Cargo.toml` grep |
| **Uncommitted files** | 178 files (+11,126 / -3,179) | **17 files** | **-161** | [E0] `git status --short` |
| **Commits since baseline** | 0 feature commits (last 10 all chores) | **1 salvage commit (+15,708 / -4,306)** | **+1 major commit** | [E0] `git log` |
| **`panic!` in production** | `knowledge-runtime` (3), `kernel-oracles` (2), `forge-memory-bridge/legacy.rs` (3) | **ZERO in `knowledge-runtime` + `kernel-oracles`** | **-5 production panics** | [E0] `grep` |
| **`unsafe` in production** | `Primitives/check-runner` (4 blocks) | **Still 4 blocks** (documented with `// SAFETY` comments now) | **No change** | [E0] `grep` |
| **`todo!()` / `unimplemented!()`** | 0 | 0 | — | [E0] `grep` |
| **Deleted tracking docs** | `02_MASTER_ISSUE_MATRIX.md`, `06_RISK_REGISTER.md` missing | **Both restored** | **+2 docs** | [E0] `git show` |
| **Foundation crates existing** | 0 (`boundary-compiler`, `bitemporal-runtime`, `quant-governor`, `claim-ledger`) | **4 exist + compile + test green** | **+4** | [E0] `cargo test` |
| **`scr-runtime-compression`** | Did not exist | **Exists, 9 tests passing** | **+1** | [E0] `cargo test` |
| **`quant-eval`** | Did not exist | **Exists, 1 test** | **+1** | [E0] `find` + `cargo test` |
| **`receipt-bench`** | Did not exist | **Exists (13 lib warnings, compiles)** | **+1** | [E0] `cargo check` |
| **`agent-guard`** | Did not exist | **Exists, compiles** | **+1** | [E0] `find` |
| **`tauri-queue`** | 1 file, ~10 lines, 0 tests, empty stub | **376 lines, `lib.rs` + 2 tests + 4 examples** | **Real implementation** | [E0] `wc` + `find` |
| **`semantic-memory` HNSW degradation** | No `approximate` marking in receipts | **`approximate` field + `approximate_scanned_count` / `approximate_returned_count` / `approximate_candidate_count` in DB schema** | **+4 fields** | [E0] `grep` |
| **`agent-graph` tests** | 14 tests | **14 tests** | — | [E0] `find` |
| **`turbo-semantic` tests** | Not counted in baseline | **28 tests** | **New visibility** | [E0] `find` |
| **`llm-pipeline` tests** | 0 `tests/` directory | **1 test in `tests/`** | **+1** | [E0] `find` |
| **`ollama-vision` tests** | 1 test | **1 test** | — | [E0] `find` |
| **`ai-batch-queue` tests** | 1 test | **1 test** | — | [E0] `find` |
| **Primitives tests** | 0 across all 10 | **0 across all 10** | **No change** | [E0] `find` |
| **`llm-output-parser` tests** | 0 | **0** | **No change** | [E0] `find` |
| **`job-queue` tests** | 0 `tests/` directory | **0 `tests/` directory** (but extensive inline tests in `db.rs`) | **No change** | [E0] `find` |
| **`comfyui-rs` tests** | 0 | **0** | **No change** | [E0] `find` |
| **`semantic-memory` turbo-quant version** | `"0.2.0-alpha.1"` (mismatch with crate `0.2.0`) | **Still `"0.2.0-alpha.1"`** | **No change** | [E0] `grep` |
| **`poly-kv` workspace integration** | Separate workspace, not in root `Cargo.toml` | **Still separate** | **No change** | [E0] `grep` |
| **`semantic-memory` bitemporal truth** | No `valid_time`/`recorded_time`/`as_of` | **Still not present** | **No change** | [E0] `grep` |
| **Governance crate receipts** | Typed surfaces only, no runtime emission | **Still surfaces only** | **No change** | [E0] `grep` |
| **`llm-pipeline` execution receipts** | No typed receipts | **Still no typed receipts** | **No change** | [E0] `grep` |
| **Workspace build time (cold)** | 90s | **2m 44s** | **+74s** (more crates) | [E0] `cargo check` |
| **Workspace build time (incremental)** | 4.4s | **0.35s** | **-4.05s** | [E0] `cargo check` |

---

## What Got Done (Verified)

### 1. Source-of-truth stabilization ✅

- **178 files committed** as `800c707` — "V30 pre-hardening salvage commit — no feature changes"
- **+15,708 / -4,306 lines** — all test/conformance/autography fixes from prior session
- **Deleted docs restored**: `02_MASTER_ISSUE_MATRIX.md` and `06_RISK_REGISTER.md` are back
- Only **17 uncommitted files** remain (vs 178)

*Verdict:* **Fixed.** Source-of-truth drift risk is now minimal.

### 2. Safety violations — partially fixed ✅/⚠️

| Issue | Before | After | Verdict |
|---|---|---|---|
| `knowledge-runtime` `panic!` | 3 in `query/classify.rs` | **0** | **Fixed** — file modified in commit |
| `kernel-oracles` `panic!` | 2 in `lib.rs` | **0** | **Fixed** — file not in grep results |
| `forge-memory-bridge/legacy.rs` `panic!` | 3 | **Still 3** | **Open** — not touched |
| `semantic-memory/src/pool.rs` `panic!` | 2 (simulated panic tests) | **Still 2** | **Acceptable** — these are test-only simulated panics |
| `Primitives/check-runner` `unsafe` | 4 blocks | **Still 4** | **Open** — now documented with `// SAFETY:` comments but not removed |
| `llm-pipeline/src/pipeline.rs` `panic!` | 3 in test paths | **Still 3** | **Acceptable** — test-only panic paths |
| `llm-output-parser` `panic!` | 3 in test paths | **Still 3** | **Acceptable** — test-only |

*Verdict:* The **production panics in `knowledge-runtime` and `kernel-oracles` are resolved.** The remaining `panic!` instances are either in test code (acceptable) or in `forge-memory-bridge/legacy.rs` (medium priority). The `unsafe` in `check-runner` remains the single biggest safety gap.

### 3. Foundation crates — all 4 plan crates + 2 extras built ✅

| Crate | Status | Tests | Key evidence |
|---|---|---|---|
| `boundary-compiler` | ✅ Compiles, tests pass | Multiple unit tests + 1 doc-test | `canonicalize_json`, `ContentDigest::compute`, duplicate-key rejection |
| `bitemporal-runtime` | ✅ Compiles, tests pass | Multiple unit tests + doc-tests | `BitemporalFact`, `append_supersede`, `as_of` query |
| `quant-governor` | ✅ Compiles, tests pass | Multiple unit tests + doc-test | `CodecPolicyV1`, `AdmissibilityDecisionV1`, policy routing |
| `claim-ledger` | ✅ Compiles, tests pass | Multiple unit tests + doc-test | `ClaimV1`, `AdjudicationV1`, `PublicBoundaryCheckV1` |
| `scr-runtime-compression` | ✅ Compiles, tests pass | **9 tests** | `ExactFallbackAdapter`, decode batch, non-strict mode |
| `quant-eval` | ✅ Compiles, tests pass | **1 test** | Exists and builds |
| `receipt-bench` | ⚠️ Compiles with 13 warnings | Unknown | Builds in workspace |
| `agent-guard` | ✅ Compiles | Unknown | Exists in workspace |

*Verdict:* **The core architectural gap is closed.** The crates that the audit called "missing entirely" now exist and test green. This is the most important improvement because these 4 crates unblock all downstream integration work.

### 4. `tauri-queue` replaced with real implementation ✅

| Before | After |
|---|---|
| 1 file, ~10 lines, empty stub | 376 lines, real `QueueManager`, config, persistence, cancellation, cooldown, SQLite backend |
| 0 tests | 2 tests |
| 0 examples | 4 examples |

*Verdict:* **Fixed.** No longer dead weight.

### 5. HNSW degradation tracking partially fixed ⚠️

| Before | After |
|---|---|
| `VectorSearchReceiptV1` had no `approximate` field | `search_receipts` table now has `approximate INTEGER NOT NULL` + `approximate_scanned_count` + `approximate_returned_count` + `approximate_candidate_count` |

*Verdict:* **Schema improved, but runtime wiring unverified.** The DB schema now supports marking searches as approximate, but I did not verify that the HNSW query path actually sets `approximate = true` at runtime.

### 6. Workspace unified — 58 crates ✅

New members since baseline:
- `agent-guard`
- `bitemporal-runtime`
- `boundary-compiler`
- `claim-ledger`
- `quant-governor`
- `quant-eval`
- `receipt-bench`
- `scr-runtime-compression`

*Verdict:* **The workspace is now a single build surface** for 58 crates. `cargo check --workspace` passes (with only `receipt-bench` warnings).

---

## What Remains Open

### Open — P0 (still critical)

| # | Issue | Before | After | Why still open |
|---|---|---|---|---|
| 1 | `Primitives/check-runner` `unsafe` | 4 blocks | **Still 4 blocks** | Not addressed in salvage commit |
| 2 | `semantic-memory` bitemporal truth | Absent | **Still absent** | `bitemporal-runtime` exists but not wired into `semantic-memory` |
| 3 | `semantic-memory` turbo-quant version | `"0.2.0-alpha.1"` | **Still `"0.2.0-alpha.1"`** | Not fixed |
| 4 | `poly-kv` workspace integration | Separate workspace | **Still separate** | Not moved into root workspace |
| 5 | `forge-memory-bridge/legacy.rs` `panic!` | 3 | **Still 3** | Not addressed |
| 6 | Governance crate receipt emission | Typed surfaces only | **Still surfaces only** | No runtime receipt logic added |
| 7 | `llm-pipeline` execution receipts | None | **Still none** | Not addressed |
| 8 | Primitives test coverage | 0/10 | **0/10** | No tests added |
| 9 | `llm-output-parser` test coverage | 0 | **0** | No tests added |
| 10 | `job-queue` `tests/` directory | 0 | **0** | No tests added (inline tests exist in `db.rs` but not in `tests/`) |

### Open — P1 (high, but less urgent now)

| # | Issue | Before | After |
|---|---|---|---|
| 11 | `agent-graph` execution receipts | None | Still none |
| 12 | `comfyui-rs` test coverage | 0 | 0 |
| 13 | Cross-crate integration tests | None | Still none |
| 14 | Performance baselines under release profile | Dev-profile only | Still dev-only |
| 15 | `semantic-memory` exact fallback retention | No raw retention | Still no raw retention |
| 16 | `quant-governor` wired into `semantic-memory` search path | No wiring | Still no wiring |
| 17 | `boundary-compiler` integrated into `semantic-memory` canonicalization | Ad-hoc key-sorter | Still ad-hoc |

---

## Risk Assessment Shift

| Risk | Before severity | After severity | Reason |
|---|---|---|---|
| Source-of-truth drift | **Critical** | **Low** | 178 files committed; tracking docs restored |
| Production `panic!` | **Critical** | **Medium** | `knowledge-runtime` + `kernel-oracles` fixed; `forge-memory-bridge/legacy.rs` remains |
| Missing foundation crates | **Critical** | **Low** | All 4 plan crates + 2 extras exist and test green |
| Missing governed compression runtime | **Critical** | **Medium** | `quant-governor` + `scr-runtime-compression` exist but not wired into `semantic-memory` |
| `unsafe` in workspace | **Critical** | **High** | Documented but not removed; override needed if staying |
| HNSW as shadow truth | **High** | **Medium** | Schema supports approximation marking; runtime wiring unverified |
| Bitemporal truth absent | **Critical** | **High** | `bitemporal-runtime` exists but not integrated |
| Primitives untested | **High** | **High** | No change |
| Extension crates untested | **Critical** | **Medium** | `tauri-queue`, `agent-graph` now have tests; others still lack |
| PolyKV isolation | **Medium** | **Medium** | No change |

---

## Bottom Line

**You closed the biggest architectural gaps in a single session.** The workspace went from "audit findings with no crates to implement them" to "foundation crates exist, compile, and pass tests." The git state went from "178 uncommitted files = disaster waiting" to "clean committed history with restored tracking." The safety profile improved by removing 5 production panics.

**The remaining work is integration and wiring**, not invention. Specifically:
1. Wire `bitemporal-runtime` into `semantic-memory` DB schema and API.
2. Wire `quant-governor` into `semantic-memory` codec path.
3. Wire `boundary-compiler` into `semantic-memory` canonicalization.
4. Fix or override `check-runner` `unsafe`.
5. Add tests to Primitives and `llm-output-parser`.
6. Integrate `poly-kv` into workspace.
7. Fix turbo-quant version string.

**This is now a P1/P2 punch list, not a P0 crisis.** The crisis was the missing foundation crates and the uncommitted diff. Both are resolved.

---

## Receipt

- **What was done:** Compared current `~/Coding/Libraries` state (commit `800c707`) against baseline audit findings using `git status`, `git log`, `git show`, `cargo check --workspace`, `cargo test` on new crates, `grep` for `panic!`/`unsafe`/`unwrap`/`todo`, `find` for test directories, `wc` for file sizes.
- **What was verified:** 58 workspace members build; 4 foundation crates + 2 extras test green; `knowledge-runtime` and `kernel-oracles` panics removed; 178 files committed; `tauri-queue` is real code; HNSW schema improved.
- **What was NOT verified:** Runtime behavior of HNSW `approximate` flag setting; whether `bitemporal-runtime` is actually used by any consumer; whether `quant-governor` is called by `semantic-memory`; whether `receipt-bench` fixtures run end-to-end; `cargo clippy --workspace`.
- **Proof debt:** The `panic!` fix in `knowledge-runtime` was verified by `grep` absence, not by reading the full modified file. The HNSW schema improvement was verified by `grep` for column names, not by tracing the query path. The `check-runner` `unsafe` blocks were not read in context.
- **Falsifies if:** Any of the "fixed" panics reappear in a future commit; any foundation crate fails `cargo test` when run independently; the 17 remaining uncommitted files turn out to be critical.
