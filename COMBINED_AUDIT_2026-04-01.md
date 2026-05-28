# Combined Audit: Libraries (8.4) + Recall (8.0) — April 1, 2026

**Auditor:** Claude Opus 4.6, full-source code-grounded inspection  
**Libraries scope:** 30 workspace members + 10 Primitives, ~124.5K lines Rust, 415 files, 1,312 tests  
**Recall scope:** 5 crates + 2 vendored deps + React/TS frontend, ~28.8K lines total (11.7K Recall Rust, 477 lines TypeScript), 268 tests  
**Prior baselines:** V5 Synthesis (Libraries 8.1, Recall 7.5, March 30 2026)

---

## I. Libraries Workspace — 8.4 / 10

*(Full findings in prior message; summary here for cross-reference.)*

### Key Closures Since March 30

- **LIB-001 (P0) governance fail-open** — Properly closed. `GovernanceMode::Strict` returns `Err` on missing claims/blocked gates. 16 tests. Feature compiled by default.
- Connection pool RAII guards with poison recovery and transaction rollback.
- HNSW sidecar lifecycle with rebuild-from-SQLite, stale detection, pending-ops journal.
- CEA persistence through caller-provided transaction handles.

### Open Findings (7)

| # | Title | Severity | Fix Effort |
|---|-------|----------|------------|
| L-1 | `unreachable!()` in 3 production paths | HIGH | 10 min |
| L-2 | No HTTP timeout on TUI reqwest clients | HIGH | 5 min |
| L-3 | `main_support/mod.rs` 1,591-line monolith | MEDIUM | 2-3 hrs |
| L-4 | Unbounded `PilotHistory` growth | MEDIUM | 15 min |
| L-5 | `DefaultHasher` for deterministic digest | LOW | 10 min |
| L-6 | Missing NaN validation on HNSW insert | LOW | 5 min |
| L-7 | Profile composition thin test coverage | LOW | 2-3 hrs |

---

## II. Recall Workspace — 8.0 / 10

### What Moved Since March 30

The Recall codebase has had substantial hardening since the March 30 snapshot. Every P1 finding from the V5 synthesis is either closed or materially improved.

**CRC-002 (write tools visible without handler)** — **Closed.** `build_tool_prompt()` filters write tools when `approval_handler.is_none()`. The model no longer sees tools it can't use.

**CRC-012 (write scope trusted from LLM)** — **Closed.** `execute_tool_call()` validates write tool scope against session scope before execution. Cross-scope writes are blocked by governance when `strict_scope` is true. Governance receipt records the decision either way.

**CRC-006 (memory intent word boundaries)** — **Closed.** `detect_memory_intent` uses word-boundary-aware matching. Test `preference_requires_word_boundary` verifies the fix.

**RC-001 (atomic writes)** — **Closed.** `auto_persist_turns()` implements write-to-temp → `BufWriter.flush()` → `sync_all()` → rename. Log rotation at 512KB with timestamped backups. Turn retention bounded at `MAX_TURNS`.

**Approval flow** — **Fully implemented.** The `TauriApprovalHandler` bridges session approval to frontend popup via oneshot channels. 120-second timeout with auto-deny. Frontend `ApprovalDialog.tsx` shows countdown, tool arguments, and side effect class. Clean implementation.

**HTTP timeouts** — Every `reqwest::Client` in Recall uses `Client::builder()` with both `connect_timeout` and `timeout`. The Ollama provider has 5s connect / 120s request. The embedder clients have 5s connect / 30s request. The config validator has 3s connect / 5s request. This is better discipline than the libraries workspace.

**CRC-010 (entity extraction star topology)** — **Closed.** The ingest pipeline now uses `blake3::hash(content)` as a content-derived entity ID when no entities are extracted, instead of a constant `"recall-document"` string that created a meaningless star topology.

### Recall Findings

#### R-1: Workspace Lints Declared but Never Inherited

**Severity:** HIGH  
**Location:** `Cargo.toml` (workspace) + all 5 member `Cargo.toml` files

**Problem:** The workspace root declares `[workspace.lints.rust] unsafe_code = "deny"` and `[workspace.lints.clippy] todo = "deny"`, but none of the 5 member crates include `[lints] workspace = true`. Rust's lint inheritance requires explicit opt-in per crate. The lints are completely unenforced.

This matters because `recall-embedder/src/chain.rs` contains `unsafe impl Send for SafeTextEmbedding` and `unsafe impl Sync for SafeTextEmbedding`. These are correctly documented and justified (ONNX runtime internals wrapped in a serializing Mutex), but if `unsafe_code = "deny"` were actually enforced, they'd need an explicit `#[allow(unsafe_code)]` annotation. The safety reasoning is sound — the real issue is that the safety net doesn't exist.

**Impact:** Any contributor could add `unsafe` blocks anywhere without compiler pushback. The workspace gives the impression of enforced safety without delivering it.

**Fix:** Add `[lints]\nworkspace = true` to all 5 member `Cargo.toml` files. Add `#[allow(unsafe_code)]` to the `SafeTextEmbedding` impl block.

#### R-2: No Tauri Command ↔ TypeScript Type Integration Test

**Severity:** MEDIUM  
**Location:** `recall-app/tests/app_tests.rs` vs `recall-app/ui/src/types.ts`

**Problem:** The 19 app tests cover config roundtrip, blocking semantics, RuntimeTruth construction, and error conversion. The TypeScript types in `types.ts` are manually aligned to the Rust types — `ReceiptView` mirrors `QueryReceipt`, `GovernanceReceipt` mirrors the Rust struct, etc. But there's no automated check that the two stay in sync. If someone adds `scope_status` to `QueryReceipt` (as was done for RC-017) but forgets `types.ts`, the frontend silently drops the field.

The types are currently aligned (I verified by comparing the Rust struct fields to the TypeScript interfaces), but this is maintained by human discipline, not by tooling.

**Impact:** Drift between Rust and TypeScript types causes silent data loss in the frontend — fields appear in the backend response but never render in the UI. This is specifically dangerous for governance receipts and approval records, where invisible data means invisible audit trail.

**Fix:** Add a test that serializes each command response type to JSON, then validates that every top-level key exists in a snapshot matching the TypeScript interface. This can be a `serde_json::to_value` + field-name assertion — it doesn't need a full TypeScript parser.

#### R-3: `session.rs` at 2,108 Lines

**Severity:** MEDIUM  
**Location:** `recall-session/src/session.rs`

**Problem:** The session orchestrator contains the `RecallSession` struct, the `query()` method (which handles both simple and multi-step paths), tool dispatch, memory policy execution, prompt budget calculation, conversation persistence, auto-persist with atomic writes, conversation history loading, and turn management. At 2,108 lines it's approaching the `main_support/mod.rs` monolith problem from the libraries.

The functions are logically related — they all touch the session — but the file contains at least 4 distinct responsibility clusters: prompt construction, tool dispatch + approval, persistence + history, and the core query orchestration.

**Impact:** Same as L-3 in the libraries: changes to persistence risk breaking prompt construction, diffs are hard to review, and contributor onboarding requires reading 2,100 lines before touching anything.

**Fix:** Extract into `prompt.rs` (prompt building + budget), `dispatch.rs` (tool execution + approval), `persist.rs` (auto-persist + conversation persistence + log rotation), keeping `session.rs` as the coordinator.

#### R-4: Scope Governance Exception Doesn't Check Temporal Validity

**Severity:** LOW  
**Location:** `recall-session/src/scope_governance.rs:56-74` — `exception_covers()`

**Problem:** `ProfileExceptionBundleV1` has `starts_at` and `expires_at` fields, but `exception_covers()` never checks them. An exception that expired in 2025 will still grant cross-domain access in 2026.

**Impact:** Expired governance exceptions continue to grant access. For a personal knowledge system this is low severity — the user created the exception — but it means the governance receipt claims an exception was valid when it may not be.

**Fix:** Add temporal validation:
```rust
let now = chrono::Utc::now().to_rfc3339();
if exception.expires_at < now || exception.starts_at > now {
    return false;
}
```

#### R-5: `flush_conversation_log` Missing `sync_all()`

**Severity:** LOW  
**Location:** `recall-session/src/session.rs` — `flush_conversation_log()` method (around line 1934)

**Problem:** `auto_persist_turns()` correctly does `BufWriter → flush → sync_all → rename` for crash safety. But the explicit `flush_conversation_log()` method does `fs::write → rename` without `sync_all()`. On Linux with ext4 in default mode, the write may be buffered in the page cache when the rename happens. If the system loses power between rename and cache writeback, the file is zero-length.

**Impact:** Rare data loss on power failure when the explicit flush path is used (as opposed to auto-persist, which is safe). Low severity because `auto_persist_turns()` is the hot path, and `flush_conversation_log()` is only called by the `flush_chat` Tauri command.

**Fix:** Replace `std::fs::write(&tmp, &json)` with the same `File::create → BufWriter → write_all → flush → sync_all` pattern used in `auto_persist_turns()`.

---

## III. Cross-System Comparative Analysis

### Where Recall Is Ahead of Libraries

**HTTP client discipline.** Every `reqwest::Client` in Recall has explicit `connect_timeout` and `timeout`. The libraries have two `Client::new()` calls in `main_support` with no timeout at all. Recall gets this right because the providers were written with constrained hardware in mind from the start.

**Error ergonomics.** Recall's `UserFacingError` → `AppErrorView` chain produces structured error payloads with error codes, human-readable titles, recovery actions, and technical details. The frontend can render error cards with actionable steps. The libraries' `PilotError` is well-structured for internal use but doesn't have a user-facing layer.

**Test density.** 268 tests across 11.7K lines of Recall-specific Rust (2.29 per 100 lines) vs 1,312 tests across 124.5K lines of library Rust (1.06 per 100 lines). Recall's test coverage relative to code volume is more than double. The `crown_jewel_tests.rs` (29 tests), `convergence_tests.rs` (26 tests), and `p0_stabilization_tests.rs` (26 tests) are particularly well-organized.

**Approval flow completeness.** The full circuit — `ApprovalHandler` trait → `TauriApprovalHandler` with oneshot channels → Tauri event emission → `ApprovalDialog.tsx` with countdown timer → `respond_to_approval` command → oneshot resolution — is a complete, tested implementation with a 120-second timeout and clean failure modes. The libraries don't have an equivalent user-facing approval surface because `forge-pilot` is a CLI/TUI tool, not a GUI app.

### Where Libraries Are Ahead of Recall

**Lint enforcement.** Libraries have `[lints] workspace = true` in every member crate. Recall declares lints but doesn't inherit them. This is the single most embarrassing gap in Recall.

**Verification ledger.** The libraries' `VerificationCase → CheckPlan → Attempt → ControlReceipt → LedgerEntry → replay_case` pipeline has no equivalent in Recall. Recall produces governance receipts and query receipts, but these are append-only metadata — there's no deterministic replay function that can reconstruct the session state from receipts.

**Governance enforcement depth.** Libraries' `GovernanceMode::Strict` makes governance failures propagate as `Result::Err`, which forces callers to handle them. Recall's governance produces receipts but doesn't block query execution — even when `strict_scope` is true, reads return data with a "blocked" receipt rather than actually blocking. Writes are blocked, reads aren't. This is a conscious design choice (the user owns the data), but it means governance receipts for reads are informational rather than authoritative.

**Schema stability discipline.** Libraries use `serde(default, skip_serializing_if)` pervasively across wire types, which makes schema evolution backward-compatible. Recall's types have these annotations on optional fields but the pattern is less consistent.

### Shared Patterns and Risks

Both codebases have the same monolith problem — `main_support/mod.rs` (1,591 lines) and `session.rs` (2,108 lines). Both files are coherent (everything touches the main orchestration concern) but too large for practical review.

Both use the canonical `ExportEnvelopeV3 → forge-memory-bridge → import_projection_batch` pipeline for getting data into semantic-memory. Recall's ingest pipeline correctly builds validated V3 envelopes through `build_envelope()`, which means Recall's knowledge base is projection-compatible with the libraries' verification pipeline. This is the key integration point — if you ever want `forge-pilot` to verify claims that originated from Recall journal entries, the projection schema already matches.

Both have `lock_or_recover` / `unwrap_or_else(|e| e.into_inner())` patterns for mutex poisoning. The approach is consistent: recover with a warning rather than crash. This is the right trade-off for a single-user local-first system.

---

## IV. Recommended Priority (Combined)

### Immediate (< 30 minutes total)

| # | System | Finding | Effort |
|---|--------|---------|--------|
| 1 | Recall | **Add `[lints] workspace = true` to all 5 crates** | 5 min |
| 2 | Libraries | Replace `unreachable!()` with error returns | 10 min |
| 3 | Libraries | Add HTTP timeouts to TUI clients | 5 min |
| 4 | Libraries | Add NaN validation to HNSW insert | 5 min |
| 5 | Libraries | Cap `PilotHistory` vectors | 15 min |

### Short-term (1-2 hours)

| # | System | Finding | Effort |
|---|--------|---------|--------|
| 6 | Recall | Add `sync_all` to `flush_conversation_log` | 10 min |
| 7 | Recall | Add temporal check to `exception_covers` | 10 min |
| 8 | Recall | Add Tauri command ↔ TS type integration test | 1 hr |
| 9 | Libraries | Replace `DefaultHasher` with `blake3` | 10 min |

### Medium-term (half-day)

| # | System | Finding | Effort |
|---|--------|---------|--------|
| 10 | Recall | Split `session.rs` into 4 modules | 2-3 hrs |
| 11 | Libraries | Split `main_support/mod.rs` into 5 modules | 2-3 hrs |
| 12 | Libraries | Property tests for profile composition | 2-3 hrs |

---

## V. Final Assessment

The two codebases are at different maturity stages but converging. The libraries workspace (8.4) is a mature, deeply-layered verification infrastructure with strong type-level guarantees and a verification ledger that has no open-source equivalent. Recall (8.0) is a well-built application layer that correctly uses the libraries' projection pipeline and adds user-facing features (approval flow, governance receipts, structured errors) that the libraries don't need.

The integration seam between them — Recall's `IngestPipeline` building `ExportEnvelopeV3` records that flow through `forge-memory-bridge` into `semantic-memory` — is clean and correct. Recall doesn't bypass the canonical import path or invent its own storage format. When `forge-pilot` eventually verifies Recall-originated claims, the projection schema will match.

The biggest practical risk across both systems is the pair of monolith files (`main_support/mod.rs` + `session.rs`, 3,699 lines combined). These are where the next regression will hide and where contributor friction will be highest.

The most embarrassing gap is Recall's unenforced workspace lints — a 5-minute fix that's been declared but not wired. The most architecturally significant gap is the `unreachable!()` calls in the libraries, specifically because `PlanKind::LlmGenerated` is on the near-term roadmap and will hit that crash path.

Neither codebase has critical bugs. Both are shipping-quality for a solo developer targeting a specific use case (local-first AI verification). The remaining issues are operational reliability and maintainability improvements, not correctness problems.
