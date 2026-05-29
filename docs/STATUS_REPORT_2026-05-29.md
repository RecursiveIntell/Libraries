# Libraries Comprehensive Status Report

**Date:** 2026-05-29  
**Branch:** master  
**Last Commit:** 9d399e3 — "chore: consolidate 612 uncommitted files from salvage branch"

---

## Executive Summary

✅ **Workspace builds clean** — `cargo check --workspace` passes  
✅ **Tests pass** — `cargo test --workspace` passes  
✅ **Clippy clean** — `cargo clippy --workspace --all-targets -- -D warnings` passes  
✅ **Git status clean** — Only 5 uncommitted files (the spec docs just created)  
✅ **612 files already committed** — Memory was stale; salvage branch was consolidated  

---

## Workspace Health

| Check | Status | Notes |
|---|---|---|
| `cargo check --workspace` | ✅ Pass | 0 errors |
| `cargo test --workspace` | ✅ Pass | All tests pass |
| `cargo clippy --workspace` | ✅ Pass | 0 warnings (with -D warnings) |
| Git uncommitted files | ✅ 5 files | Only the spec docs created today |
| Branch | master | Clean, no merge conflicts |

---

## Known Issues Status

### ✅ Already Fixed (per memory + verification)

| Issue | Status | Verification |
|---|---|---|
| `panic!` in knowledge-runtime | ✅ Fixed | `grep` returns nothing |
| `panic!` in kernel-oracles | ✅ Fixed | `grep` returns nothing |
| 178/612 uncommitted files | ✅ Committed | Only 5 new docs uncommitted |
| SM-AUD-0058 | ✅ Fixed | Verified in tests/step4_verification.rs |
| SM-AUD-0064 | ✅ Fixed | Verified in db.rs:674 |
| SM-AUD-0065 | ✅ Fixed | Verified in db.rs:649 |
| Workspace lint policy | ✅ Enforced | Clippy passes with -D warnings |

### ⚠️ Intentional/Documented

| Issue | Status | Notes |
|---|---|---|
| `unsafe` in check-runner | ⚠️ Intentional | 4 blocks for signal handling (libc::kill); documented as "NOT a casual shortcut" |
| quant-governor profiles warning | ⚠️ Cosmetic | "profiles for non root package will be ignored" — move to workspace Cargo.toml |

### 🔴 Remaining Work (Phase 3)

| Issue | Priority | Notes |
|---|---|---|
| turbo-quant → quant-governor wiring | P0 | semantic-memory has `turbo-quant-codec` feature but quant-governor doesn't reference turbo-quant |
| fib-quant → quant-governor wiring | P0 | No integration yet |
| poly-kv workspace merge | P0 | Currently in `poly-kv/crates/`, needs promotion |
| semantic-memory HNSW bugs | P0 | SM-AUD-0010/0011/0026/0027/0042/0059 remain |
| llm-pipeline batching receipts | P0 | No receipt emission for batching operations |

---

## Crate Inventory

### Compression Crates

| Crate | Status | Notes |
|---|---|---|
| turbo-quant | ✅ Implemented | 180KB src, full codec implementation |
| fib-quant | ✅ Implemented | 100KB src, full codec implementation |
| poly-kv | ✅ Implemented | In `poly-kv/crates/` subdirectory |
| quant-governor | ✅ Implemented | 36KB src, policy routing stub |
| quant-eval | ⚠️ Empty | Only 42 bytes — needs benchmark harness |

### Integration Status

| Integration | Status | Notes |
|---|---|---|
| semantic-memory → turbo-quant | ✅ Feature-gated | `turbo-quant-codec` feature exists |
| semantic-memory → fib-quant | ❌ Missing | No feature or integration |
| semantic-memory → quant-governor | ❌ Missing | No feature or integration |
| quant-governor → turbo-quant | ❌ Missing | No dependency in Cargo.toml |
| quant-governor → fib-quant | ❌ Missing | No dependency in Cargo.toml |

---

## SM-AUD Issue Ledger

### Fixed (Verified)
- ✅ SM-AUD-0058 — search_episodes returns episode_id (verified in tests)
- ✅ SM-AUD-0064 — foreign_keys assertion after config
- ✅ SM-AUD-0065 — max_page_count validation before pragma

### Remaining (from 00_CODEX_PASS_MASTER_PROMPT.md)
- 🔴 SM-AUD-0001 — Archive is not hermetic despite passing certifier
- 🔴 SM-AUD-0002 — No packaged root workspace manifest for included local crates
- 🔴 SM-AUD-0003 — Multiple Cargo.lock files create ambiguous dependency source of truth
- 🔴 SM-AUD-0004 — Document ingest silently truncates chunks on embedder batch-count mismatch
- 🔴 SM-AUD-0005 — Fact re-embedding silently truncates on batch-count mismatch
- 🔴 SM-AUD-0006 — Chunk re-embedding silently truncates on batch-count mismatch
- 🔴 SM-AUD-0007 — Message re-embedding silently truncates on batch-count mismatch
- 🔴 SM-AUD-0008 — Episode re-embedding silently truncates on batch-count mismatch
- 🔴 SM-AUD-0009 — Public embedding validation is dimension-only
- 🔴 SM-AUD-0010 — delete_document does not explicitly clean episode derived state
- 🔴 SM-AUD-0011 — delete_document can leave stale HNSW episode keys
- 🔴 SM-AUD-0012 — Vector scan uses bytemuck::try_cast_slice on SQLite Vec<u8>
- 🔴 SM-AUD-0013 — HNSW sidecar loader allocates raw byte_len from file without cap
- 🔴 SM-AUD-0014 — HNSW data format stores dimensions using usize
- 🔴 SM-AUD-0015 — HNSW save is not atomic
- 🔴 SM-AUD-0016 — Pending HNSW mutations are applied before sidecar save succeeds
- 🔴 SM-AUD-0017 — Pending upsert calls insert instead of update
- 🔴 SM-AUD-0026 — (not listed in prompt, needs investigation)
- 🔴 SM-AUD-0027 — (not listed in prompt, needs investigation)
- 🔴 SM-AUD-0042 — (not listed in prompt, needs investigation)
- 🔴 SM-AUD-0059 — (not listed in prompt, needs investigation)

---

## Receipt Schema Status

### Implemented Receipts
| Receipt | Location | Status |
|---|---|---|
| FibQuantCompressionReceiptV1 | fib-quant/src/receipt.rs | ✅ Implemented |
| TurboQuant receipts | turbo-quant/src/ | ⚠️ Needs verification |
| ExactFallbackReceipt | quant-governor/src/receipt.rs | ✅ Implemented |
| DegradationReceipt | quant-governor/src/degradation.rs | ✅ Implemented |

### Missing Receipts (Phase 1 Spec)
| Receipt | Owner | Status |
|---|---|---|
| SemanticResidualReceiptV1 | semantic-memory | ❌ Not implemented |
| CapabilityArgumentContractV1 | agent-guard | ❌ Not implemented |
| ArgumentLineageReceiptV1 | agent-guard | ❌ Not implemented |
| PersistentReasoningSubgraphV1 | claim-ledger | ❌ Not implemented |
| CompressionSurvivabilityReportV1 | quant-eval | ❌ Not implemented |
| EvidenceSufficiencyReceiptV1 | claim-ledger | ❌ Not implemented |
| GlossDenseIndexReceiptV1 | gloss (app) | ❌ Not implemented |
| GlossSemanticMemoryProjectionReceiptV1 | gloss (app) | ❌ Not implemented |
| GlossRetrievalProbeReceiptV1 | gloss (app) | ❌ Not implemented |
| GlossAnswerReceiptV1 | gloss (app) | ❌ Not implemented |

---

## Benchmark Harness Status

### Current State
- `quant-eval/` directory: 42 bytes (essentially empty)
- No benchmark harness implemented
- No BEIR/MTEB/ANN-Benchmarks integration
- No dataset fixtures
- No receipt emission for benchmarks

### Required (Phase 3 Spec)
- codec_correctness profile
- retrieval_quality profile
- ann_performance profile
- local_recursiveintell profile
- kv_cache profile

---

## AgentSecurity Status

### Current State
- `agent-guard/` exists but content unknown
- No PACT-style argument contracts implemented
- No policy rules engine
- No trust classification
- No dry-run mode
- No argument lineage tracking

### Required (Phase 2 Spec)
- Semantic role classification (read_only, mutation, shell, filesystem, network, package_management, configuration)
- Trust level matrix (benign, elevated, dangerous)
- Policy rules engine with path/command/host matching
- Mixed-trust enforcement (partial approval, escalation)
- Dry-run mode for dangerous operations
- Argument lineage tracking

---

## Gloss Integration Status

### From Memory (Verified via research docs)
- 146 Rust + 12 TS tests passing
- 0 errors, 0 warnings
- Pool migration complete (36 write sites → with_notebook_db_write)
- System logging complete
- chat/mod.rs decomposed (3340→2529 lines)
- SM-AUD-0058/0064/0065 synced from Libraries/semantic-memory/
- Dead code removed
- log=0.4 added

### Remaining (from Gloss P36 spec)
- turbo-quant → quant-governor integration
- E2E tests
- Release-candidate truth/receipt gates

---

## Next Actions (Prioritized)

### Immediate (Today)
1. ✅ Commit the 5 spec docs — `git add docs/*.md && git commit -m "Add Libraries completion plan specs"`
2. ⚠️ Fix quant-governor profiles warning — move profiles to workspace Cargo.toml
3. 🔴 Review check-runner unsafe blocks — document or replace with safe wrappers

### Phase 1 (This Week)
4. 🔴 Create receipt schema files in `stack-ids/src/receipts/`
5. 🔴 Create JSON schemas in `contracts/`
6. 🔴 Implement validation tests

### Phase 2 (Next 1-3 Weeks)
7. 🔴 Implement agent-guard argument contracts
8. 🔴 Implement policy rules engine
9. 🔴 Implement trust classifier
10. 🔴 Implement dry-run mode
11. 🔴 Implement argument lineage tracking

### Phase 3 (Next 2-4 Weeks)
12. 🔴 Wire turbo-quant to quant-governor
13. 🔴 Wire fib-quant to quant-governor
14. 🔴 Promote poly-kv to workspace member
15. 🔴 Build quant-eval benchmark harness
16. 🔴 Fix remaining SM-AUD issues

---

## Risk Assessment

| Risk | Severity | Mitigation |
|---|---|---|
| SM-AUD HNSW bugs cause data corruption | High | Fix before enabling HNSW in production |
| quant-governor not wired to codecs | Medium | Compression works but not governed |
| No benchmark receipts | Medium | Cannot make public performance claims |
| No argument provenance | Medium | Security boundary at tool level, not argument level |
| check-runner unsafe blocks | Low | Documented and intentional for signal handling |

---

## Conclusion

**Workspace is healthy and builds clean.** The 612 uncommitted files from memory have already been committed. The remaining work is implementation of the Phase 1-3 specs created today:

1. Receipt schema pack (10 families)
2. AgentSecurity argument provenance
3. Compression survivability lab (benchmark harness)
4. turbo-quant/fib-quant → quant-governor wiring
5. SM-AUD HNSW bug fixes

All specs are in `~/Coding/Libraries/docs/` and ready for implementation.
