# Complete Context Engine Implementation Plan

> **For Hermes:** Execute this plan with strict TDD. Use controller verification after each phase because the tasks touch shared Rust files.

**Goal:** Upgrade `context-governor` from a fast receipt-bearing heuristic crate into a plugin-ready, auditable agent context engine.

**Architecture:** Keep the core crate deterministic and dependency-light by default. Add honest token-count metadata, configurable budget behavior, content-aware extractive compression, searchable local persistence, a generic memory sink trait, and shell-friendly CLI subcommands. Learned compression and host-specific integrations stay optional/adapter-facing.

**Tech Stack:** Rust 2021, serde/serde_json, blake3, chrono, uuid, thiserror, tempfile for tests. Optional tokenizer features can be added later; v0.1.x must remain clean without model/runtime dependencies.

---

## Evidence-backed current state

Repo path: `/home/sikmindz/Coding/Libraries/context-governor`
Date: 2026-06-27

Verified before this plan:
- `cargo fmt --check` passed.
- `cargo clippy --all-targets -- -D warnings` passed.
- `cargo test --all-targets` passed: 8 integration tests.
- `cargo check --all-targets` passed.
- `cargo publish --dry-run --allow-dirty` passed: 13 files, 89.6 KiB package.
- `cargo doc --no-deps --document-private-items` passed.
- `cargo deny check` passed advisories/bans/licenses/sources; parent workspace deny.toml emits unmatched-allowance warnings only.
- `cargo tree --duplicates` printed nothing.

Current gaps to close:
- `semantic_memory_enabled` is API shadow truth: it exists but archives nothing.
- `allocator` is metadata only; no strategy dispatch exists.
- Token counting is char/4 with no receipt disclosure beyond `approx_tokens` field names.
- Budget overflow only warns; no hard/fail-closed mode exists.
- Summary/loss report is shallow and does not extract structured operational anchors.
- No content-type compression for logs, JSON, diffs, cargo output, or search results.
- `FileContextStore` cannot search/expand across receipts.
- CLI only supports one-shot stdin compacting.
- README/package lacks license file, changelog, and integration docs.
- No replay/eval harness exists for task-quality comparisons.

Claim boundary:
- This plan does not claim KV-cache compression.
- This plan does not claim learned LLMLingua-equivalent compression.
- This plan makes prompt-level context governance more operationally useful and testable.

---

## Phase 0: API truth and budget safety

### Task 0.1: Add explicit token counter metadata

**Objective:** Receipts must disclose which token estimator was used.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/policy.rs`

**Steps:**
1. Add `TokenCounterKind { ApproxChars }`.
2. Add `token_counter: TokenCounterKind` to `CompactionPolicy` with serde default.
3. Add `token_counter: TokenCounterKind` to `ContextCompactionReceiptV1`.
4. Update token counting helpers to route through policy-aware functions.
5. Test that default policy emits `approx_chars` in receipt JSON.

**Gate:** `cargo test --test policy token_counter_kind_is_recorded_in_receipt`

### Task 0.2: Add soft/hard/fail-closed budget modes

**Objective:** Users must choose between current safety-first warnings, strict cascade, and refusal.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/policy.rs`

**Steps:**
1. Add `BudgetMode { SoftWarn, HardCascade, FailClosed }`.
2. Add `budget_mode` to `CompactionPolicy` with default `SoftWarn`.
3. Add `BudgetExceeded { target_tokens, minimum_required_tokens }` error.
4. In `FailClosed`, error if must-preserve/latest/protected exact messages alone exceed target.
5. In `HardCascade`, include as much summary as fits, but do not exceed target unless exact-preserve items alone exceed it. If exact-preserve alone exceeds target, return `BudgetExceeded`.
6. Preserve current behavior in `SoftWarn`.

**Gate:**
- `cargo test --test policy hard_cascade_keeps_output_under_budget_when_possible`
- `cargo test --test policy fail_closed_errors_when_exact_preserve_exceeds_budget`

---

## Phase 1: Structure the context model

### Task 1.1: Add content-kind classification

**Objective:** Each item records coarse content type for deterministic compression and search.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/content_kind.rs`

**Steps:**
1. Add `ContentKind { PlainText, Json, Diff, Rust, Markdown, CargoOutput, ShellLog, SearchResults, Unknown }`.
2. Add `content_kind` to `ContextItemV1`.
3. Implement `detect_content_kind(role, content)`.
4. Test JSON, git diff, cargo error output, Rust source, markdown, and plain text detection.

**Gate:** `cargo test --test content_kind`

### Task 1.2: Add structured anchors to summary loss report

**Objective:** The summary/loss receipt must expose task-critical anchors instead of only item IDs.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/structured_summary.rs`

**Steps:**
1. Add `StructuredContextSummaryV1` with fields:
   - `active_task`
   - `acceptance_gates`
   - `files`
   - `commands`
   - `errors`
   - `decisions`
   - `unresolved_questions`
   - `fallback_item_ids`
2. Add `structured_summary` to `SummaryLossReportV1`.
3. Extract anchors deterministically from messages and classified items.
4. Include structured anchors in the injected summary text.

**Gate:** `cargo test --test structured_summary`

---

## Phase 2: Memory and persistence

### Task 2.1: Replace semantic-memory placeholder with generic memory sink helpers

**Objective:** Make memory archival honest and adapter-friendly.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/memory_sink.rs`

**Steps:**
1. Rename/replace `semantic_memory_enabled` with `archive_memory_enabled` while preserving backwards compatibility via serde alias if possible.
2. Add `MemoryArchiveRecordV1` with receipt ID, item ID, content hash, content kind, sensitivity, archive reason, preview.
3. Add `MemorySink` trait.
4. Add `archive_response_to_memory(response, sink)` returning IDs and records attempted.
5. Populate receipt memory ID fields only through this helper, not fake IDs.

**Gate:** `cargo test --test memory_sink`

### Task 2.2: Add searchable file store APIs

**Objective:** Exact fallback must be discoverable across saved receipts.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/store.rs`

**Steps:**
1. Add `FileContextStore::expand(receipt_id, item_id, max_chars)`.
2. Add `FileContextStore::search(query, top_k, scope)` returning receipt ID + hit.
3. Add safe receipt ID sanitization test.
4. Preserve existing `save/load/list_receipts` behavior.

**Gate:** `cargo test --test store`

---

## Phase 3: CLI usability

### Task 3.1: Add CLI subcommands without heavy deps

**Objective:** Any host can use the crate via shell without linking Rust.

**Files:**
- Modify: `src/main.rs`
- Test: `tests/cli.rs`

**Subcommands:**
- `context-governor compact < request.json > response.json`
- `context-governor store --dir DIR < response.json`
- `context-governor expand --dir DIR --receipt RECEIPT --item ITEM [--max-chars N]`
- `context-governor search --dir DIR --query TEXT [--top-k N]`
- `context-governor diff < response.json`

**Steps:**
1. Keep no-args behavior as backwards-compatible compact.
2. Implement small manual arg parser.
3. Emit JSON for machine-readable subcommands.
4. Test compact/store/search/expand/diff through `env!("CARGO_BIN_EXE_context-governor")`.

**Gate:** `cargo test --test cli`

---

## Phase 4: Content-aware compression

### Task 4.1: Add deterministic content previews

**Objective:** Summaries should compress noisy artifacts according to type.

**Files:**
- Modify: `src/lib.rs`
- Test: `tests/content_compression.rs`

**Steps:**
1. Implement `content_aware_preview(kind, text, max_chars)`.
2. JSON: keep top-level keys and selected scalar examples.
3. Diff: keep file headers and hunk headers.
4. Cargo output: keep errors/warnings/test-result lines.
5. Shell logs: keep command-ish first line, error lines, and tail.
6. Plain text: existing whitespace compaction.
7. Use it in `build_summary`.

**Gate:** `cargo test --test content_compression`

---

## Phase 5: Evaluation and docs

### Task 5.1: Add replay/eval example

**Objective:** Provide a repeatable quality harness beyond synthetic throughput.

**Files:**
- Create: `examples/replay_eval.rs`
- Create: `docs/eval-harness.md`

**Steps:**
1. Build a small deterministic transcript suite.
2. Compare full/head-tail/context-governor modes for whether critical strings remain directly visible or recoverable through fallback search/expand.
3. Print CSV metrics.
4. Document limits: this is a recoverability harness, not LLM task-quality proof.

**Gate:** `cargo run --example replay_eval`

### Task 5.2: Add release polish

**Objective:** Bring package quality up to RecursiveIntell bar.

**Files:**
- Create: `LICENSE`
- Create: `CHANGELOG.md`
- Create: `docs/integrations/hermes.md`
- Create: `docs/architecture.md`
- Modify: `README.md`
- Modify: `Cargo.toml` include list if needed

**Steps:**
1. Add Apache-2.0 license text or pointer-compatible full text.
2. Add changelog entry for unreleased complete engine upgrade.
3. Add Hermes integration doc: where to call compact/store/search/expand, no restart claim unless verified.
4. Add architecture doc with text diagram and operational flow.
5. Update README with new API/CLI/claim boundary.

**Gate:** `cargo package --allow-dirty --list` contains docs/license.

---

## Phase 6: Final verification

Run:

```bash
cargo fmt --check
cargo clippy --all-targets -- -D warnings
cargo test --all-targets
cargo check --all-targets
cargo run --release --example perf
cargo run --example replay_eval
cargo doc --no-deps --document-private-items
cargo publish --dry-run --allow-dirty
cargo deny check
```

Final report must include:
- shipped feature list
- changed file list
- exact command receipts
- known non-claims/deferred items
- whether publish is now safe
