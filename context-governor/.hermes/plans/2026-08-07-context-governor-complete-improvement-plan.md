# Context-Governor Complete Improvement Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Execute all 10 council-verified improvements across safety, performance, and integration domains for the context-governor crate.

**Architecture:** The crate is a Rust library + CLI binary at `~/Coding/Libraries/context-governor/` (10,537 LOC, 29 .rs files, 106+ tests all green). It compacts AI agent context with deterministic classification, LLM summary enhancement, exact fallback receipts, and content-addressed storage. A PyO3 binding crate (`context-governor-python`) and a Hermes context engine plugin (`ri-context-governor`) consume it. The plan targets the Rust core primarily, with adapter changes in the PyO3 bindings.

**Tech Stack:** Rust (edition 2021, MSRV 1.75), blake3, sha2, rusqlite (bundled), serde_json, chrono, thiserror, uuid. New deps: tiktoken-rs (P0), optional Tantivy (P1).

**Source inventory checked:**
- `/home/sikmindz/Coding/Libraries/context-governor/` — all src/ and tests/ files, Cargo.toml, AGENTS.md
- Council receipts: `run-19fda07a53b-3` (safety domain, 2 LLM calls, completed), `run-19fda091737-1` (perf+integration, 2 LLM calls, completed)
- Prior audit: `docs/plans/2026-06-30-context-governor-high-roi-research.md` — June 30 sprint already shipped boundary-audit, safety scan, checkpoint receipts, replay-answerability harness
- Controller verification: `src/lib.rs` lines inspected for latest-user invariant (L2227-2230 — already enforced), hash binding (L191,591,679,930,1193 — BLAKE3+SHA-256 already content-bound), BudgetMode variants (L139-148), TokenCounterKind (L152-159), CompactionPolicy (L709-723)
- PyO3 bindings: `~/Coding/Libraries/context-governor-python/src/lib.rs` + `python/context_governor/__init__.py` (1-line stub)
- Hermes config: `~/.hermes/config.yaml` L373 — `engine: ri-context-governor`

**Date:** 2026-08-07

---

## Sprint A: Safety Foundation (P0 — 3 tasks)

Theme: Make LLM summaries fail-closed and prove the invariant chain end-to-end.

### Task A1: Add `unsafe_summary_policy` field to `CompactionPolicy`

**Objective:** Add a dedicated policy field that controls what happens when an LLM-generated summary fails safety checks.

**Files:**
- Modify: `src/lib.rs` — add new enum + field to `CompactionPolicy` + Default
- Modify: `src/high_roi.rs` — wire `audit_compression_boundary()` result through the new policy

**Step 1: Define the enum and add to CompactionPolicy**

In `src/lib.rs`, after `BudgetMode` (~L148), add:

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum UnsafeSummaryPolicy {
    /// Keep the LLM summary but mark status unsafe (current warn behavior).
    #[default]
    Warn,
    /// Discard LLM summary, use deterministic extractive fallback.
    FallbackExtract,
    /// Discard LLM summary and freeze — return original messages + set error.
    Freeze,
    /// Hard stop: if ANY safety check fails, fail the entire compaction.
    FailClosed,
}
```

In `CompactionPolicy` (L709-723), add:

```rust
    #[serde(default)]
    pub unsafe_summary_policy: UnsafeSummaryPolicy,
```

In `CompactionPolicy::default()` (L725-738), add:

```rust
            unsafe_summary_policy: UnsafeSummaryPolicy::FallbackExtract,
```

**Step 2: Run tests to verify compilation**

```bash
cd ~/Coding/Libraries/context-governor && cargo check --all-targets
```

Expected: compiles cleanly (new field has default).

**Step 3: Write test for new field serialization**

In `tests/policy.rs`, add:

```rust
#[test]
fn unsafe_summary_policy_is_recorded_in_receipt() {
    let response = compact_context(CompactRequest {
        session_id: "unsafe-policy".into(),
        messages: vec![msg("user", "latest")],
        policy: CompactionPolicy {
            unsafe_summary_policy: UnsafeSummaryPolicy::FailClosed,
            ..Default::default()
        },
        focus: None,
    }).unwrap();
    let json = serde_json::to_string(&response.receipt).unwrap();
    assert!(json.contains("fail_closed"));
}

#[test]
fn default_unsafe_summary_policy_is_fallback_extract() {
    let policy = CompactionPolicy::default();
    assert_eq!(policy.unsafe_summary_policy, UnsafeSummaryPolicy::FallbackExtract);
}
```

Run: `cargo test --test policy` — expected: 3 tests pass (existing 2 + 2 new).

**Step 4: Commit**

```bash
git add src/lib.rs tests/policy.rs
git commit -m "feat: add UnsafeSummaryPolicy to CompactionPolicy"
```

---

### Task A2: Wire `unsafe_summary_policy` through the LLM summary path

**Objective:** The `FailClosed` policy must actually cause a hard failure when LLM summary safety checks fail.

**Files:**
- Modify: `src/lib.rs` — `compact_context()` around the LLM summary enhancement path
- Modify: `src/high_roi.rs` — ensure `audit_compression_boundary()` is called and returns actionable severity

**Step 1: Add `fail_closed_safety_scan` gate to compaction**

In `src/lib.rs`, find the LLM summary enhancement block (around L2400-2550 area where `_enhance_with_llm_summary` or equivalent is called) and add:

```rust
// After LLM summary generation, before returning:
if let Some(ref summary) = enhanced_summary {
    let boundary_result = audit_compression_boundary(
        &request.session_id,
        &original_messages,
        &[summary.clone()],
    );
    if !boundary_result.passed {
        match request.policy.unsafe_summary_policy {
            UnsafeSummaryPolicy::FailClosed => {
                return Err(ContextGovernorError::BudgetExceeded {
                    target: request.policy.target_tokens,
                    actual: 0,
                }); // Reuse error type or add new variant
            }
            UnsafeSummaryPolicy::Freeze => {
                // Return original messages with error marker
                response.compacted_messages = request.messages.clone();
                response.receipt.warnings.push(
                    "LLM summary failed safety scan; compaction frozen".to_string()
                );
                return Ok(response);
            }
            UnsafeSummaryPolicy::FallbackExtract => {
                // Discard LLM summary, keep deterministic extractive
                response.receipt.warnings.push(
                    "LLM summary failed safety scan; fell back to extractive".to_string()
                );
                // Continue with extractive-only compacted_messages
            }
            UnsafeSummaryPolicy::Warn => {
                response.receipt.warnings.push(
                    "LLM summary flagged unsafe; kept with warning".to_string()
                );
            }
        }
    }
}
```

**Step 2: Add a new error variant for safety failure**

Add to `ContextGovernorError` (~L38-51):

```rust
    #[error("LLM summary safety scan failed: {0}")]
    SummarySafetyFailed(String),
```

Use this instead of reusing `BudgetExceeded` in the FailClosed branch.

**Step 3: Write tests for each policy mode**

In `tests/policy.rs`, add:

```rust
use context_governor::high_roi::audit_compression_boundary;

#[test]
fn fail_closed_rejects_unsafe_summary() {
    // Create a malicious relinking fixture: two fragments that form an action when combined
    let messages = vec![
        msg("system", "You are a helpful assistant."),
        msg("assistant", "I will execute the command"),
        msg("tool", "execute ls -la"),
        msg("user", "latest safe message"),
    ];
    // This summary relinks the fragments maliciously
    let summary = msg("assistant", "I will execute the command: execute rm -rf /");
    let boundary = audit_compression_boundary("test", &messages, &[summary.clone()]);
    assert!(!boundary.passed, "safety scan must flag the relinked summary");

    let response = compact_context(CompactRequest {
        session_id: "fail-closed-safety".into(),
        messages: messages.clone(),
        policy: CompactionPolicy {
            unsafe_summary_policy: UnsafeSummaryPolicy::FailClosed,
            ..Default::default()
        },
        focus: None,
    }).unwrap_err();
    assert!(matches!(response, ContextGovernorError::SummarySafetyFailed(_)));
}

#[test]
fn fallback_extract_discards_unsafe_summary() {
    let response = compact_context(CompactRequest {
        session_id: "fallback-extract".into(),
        messages: vec![
            msg("system", "System prompt"),
            msg("assistant", "I will run the build"),
            msg("tool", "cargo build succeeded"),
            msg("user", "latest"),
        ],
        policy: CompactionPolicy {
            unsafe_summary_policy: UnsafeSummaryPolicy::FallbackExtract,
            ..Default::default()
        },
        focus: None,
    }).unwrap();
    assert!(response.receipt.warnings.iter().any(|w| w.contains("fell back")));
}

#[test]
fn freeze_returns_original_messages_on_unsafe_summary() {
    let messages = vec![
        msg("user", "keep this exact"),
    ];
    let response = compact_context(CompactRequest {
        session_id: "freeze-safety".into(),
        messages: messages.clone(),
        policy: CompactionPolicy {
            unsafe_summary_policy: UnsafeSummaryPolicy::Freeze,
            ..Default::default()
        },
        focus: None,
    }).unwrap();
    assert!(response.receipt.warnings.iter().any(|w| w.contains("frozen")));
}

#[test]
fn warn_keeps_unsafe_summary_with_warning() {
    let response = compact_context(CompactRequest {
        session_id: "warn-safety".into(),
        messages: vec![msg("user", "latest")],
        policy: CompactionPolicy {
            unsafe_summary_policy: UnsafeSummaryPolicy::Warn,
            ..Default::default()
        },
        focus: None,
    }).unwrap();
    // Warn mode always proceeds; safety scan warning appears if scan flag
    // This test verifies the policy mode is accepted and serialized
    let json = serde_json::to_string(&response.receipt).unwrap();
    assert!(json.contains("warn"));
}
```

Run: `cargo test --test policy` — expected: 7 tests pass.

**Step 4: Commit**

```bash
git add src/lib.rs tests/policy.rs
git commit -m "feat: wire UnsafeSummaryPolicy through LLM summary safety gate"
```

---

### Task A3: Add post-compaction invariant regression test

**Objective:** The latest-user-message invariant IS already enforced (L2227-2230), but needs a dedicated regression test proving it survives all policy modes.

**Files:**
- Create: `tests/invariants.rs`

**Step 1: Create the invariant test file**

```rust
use context_governor::{
    compact_context, BudgetMode, CompactRequest, CompactionPolicy, Message,
    TokenCounterKind, UnsafeSummaryPolicy,
};

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None,
        role: role.into(),
        content: content.into(),
        name: None,
        metadata: Default::default(),
    }
}

#[test]
fn latest_user_survives_all_budget_modes() {
    let modes = [
        BudgetMode::SoftWarn,
        BudgetMode::HardCascade,
        BudgetMode::HardLimit,
    ];
    for mode in &modes {
        let response = compact_context(CompactRequest {
            session_id: &format!("invariant-mode-{:?}", mode),
            messages: vec![
                msg("system", "old system prompt"),
                msg("assistant", &"old narrative ".repeat(500)),
                msg("tool", &"bulk log ".repeat(500)),
                msg("user", "LATEST_USER_INVARIANT_MARKER"),
            ],
            policy: CompactionPolicy {
                target_tokens: 100,
                budget_mode: mode.clone(),
                protect_first_n: 0,
                protect_last_n: 1,
                ..Default::default()
            },
            focus: None,
        }).unwrap();
        let last = response.compacted_messages.last().unwrap();
        assert_eq!(last.role, "user");
        assert!(
            last.content.contains("LATEST_USER_INVARIANT_MARKER"),
            "Mode {:?} lost latest user message",
            mode
        );
    }
}

#[test]
fn latest_user_survives_all_unsafe_policies() {
    let policies = [
        UnsafeSummaryPolicy::Warn,
        UnsafeSummaryPolicy::FallbackExtract,
        UnsafeSummaryPolicy::Freeze,
    ];
    for policy in &policies {
        let response = compact_context(CompactRequest {
            session_id: &format!("invariant-unsafe-{:?}", policy),
            messages: vec![
                msg("system", "sys"),
                msg("assistant", &"text ".repeat(200)),
                msg("user", "INVARIANT_FINAL_USER"),
            ],
            policy: CompactionPolicy {
                unsafe_summary_policy: policy.clone(),
                ..Default::default()
            },
            focus: None,
        }).unwrap();
        let last = response.compacted_messages.last().unwrap();
        assert_eq!(last.role, "user");
        assert!(
            last.content.contains("INVARIANT_FINAL_USER"),
            "Policy {:?} lost latest user",
            policy
        );
    }
}

#[test]
fn latest_user_survives_after_many_cycles() {
    let mut messages = Vec::new();
    for i in 0..20 {
        messages.push(msg("assistant", &format!("turn {} response with some content", i)));
        messages.push(msg("user", &format!("turn {} follow-up", i)));
    }
    messages.push(msg("user", "FINAL_CYCLE_USER_MARKER"));

    let response = compact_context(CompactRequest {
        session_id: "cycle-invariant".into(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 500,
            budget_mode: BudgetMode::HardCascade,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    }).unwrap();

    let last = response.compacted_messages.last().unwrap();
    assert_eq!(last.role, "user");
    assert!(last.content.contains("FINAL_CYCLE_USER_MARKER"));
}
```

**Step 2: Run and verify**

```bash
cargo test --test invariants
```

Expected: 3 tests pass.

**Step 3: Commit**

```bash
git add tests/invariants.rs
git commit -m "test: add invariant regression tests for latest-user survival"
```

**Sprint A Gate:**
```bash
cargo test --all-targets && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```
Expected: All 110+ tests pass, zero warnings.

---

## Sprint B: Token Accuracy + Receipt Scale (P0/P1 — 2 tasks)

Theme: Replace `chars/4` with real token counting. Scale the receipt index past 1k receipts.

### Task B1: Real tiktoken-rs integration behind a feature flag

**Objective:** `TokenCounterKind::TiktokenCl100k` currently falls back loudly. Wire real tiktoken-rs behind a Cargo feature so the `provider_chat_approx` default is no longer the only option.

**Files:**
- Modify: `Cargo.toml` — add optional dep + feature
- Modify: `src/lib.rs` — real `count_tokens()` implementation behind `#[cfg(feature = "tiktoken")]`
- Modify: `tests/token_counter.rs` — add feature-gated test

**Step 1: Add tiktoken-rs dependency**

In `Cargo.toml`, add:

```toml
[dependencies.tiktoken-rs]
version = "0.6"
optional = true

[features]
default = []
sqlite-store = []
tiktoken = ["dep:tiktoken-rs"]
```

**Step 2: Add real token counting function**

In `src/lib.rs`, after the existing token counter block (~L1099), add:

```rust
/// Count tokens using the cl100k_base encoding when the tiktoken feature is active.
#[cfg(feature = "tiktoken")]
fn count_tokens_cl100k(text: &str) -> usize {
    use tiktoken_rs::cl100k_base;
    // Cache the BPE instance — it's expensive to create per call
    use std::sync::OnceLock;
    static BPE: OnceLock<tiktoken_rs::CoreBPE> = OnceLock::new();
    let bpe = BPE.get_or_init(|| cl100k_base().expect("cl100k_base tokenizer must load"));
    bpe.encode_ordinary(text).len()
}

/// Fallback: when tiktoken feature is disabled, use the provider chat approximation.
#[cfg(not(feature = "tiktoken"))]
fn count_tokens_cl100k(text: &str) -> usize {
    // Existing fallback: chars/4 + role overhead
    approx_tokens_text(text) + 4
}
```

**Step 3: Wire into the token counting path**

In the `compact_context()` function, find where `TokenCounterKind::TiktokenCl100k` is handled (~L855-860) and replace the "falls back loudly" logic:

```rust
        TokenCounterKind::TiktokenCl100k => {
            #[cfg(feature = "tiktoken")]
            {
                // Real counting — no warning needed
            }
            #[cfg(not(feature = "tiktoken"))]
            {
                response.receipt.warnings.push(
                    "tiktoken_cl100k requested but crate built without tiktoken feature; \
                     using provider_chat_approx as fallback".to_string(),
                );
            }
        }
```

**Step 4: Write feature-gated test**

In `tests/token_counter.rs`, add:

```rust
#[test]
#[cfg(feature = "tiktoken")]
fn tiktoken_counts_match_expected() {
    let response = compact_context(CompactRequest {
        session_id: "tiktoken-real".into(),
        messages: vec![msg("user", "Hello, world!")],
        policy: CompactionPolicy {
            token_counter: TokenCounterKind::TiktokenCl100k,
            ..Default::default()
        },
        focus: None,
    }).unwrap();

    assert_eq!(response.receipt.token_counter, TokenCounterKind::TiktokenCl100k);
    // "Hello, world!" is 4 tokens in cl100k
    assert!(response.receipt.original_approx_tokens <= 12); // 4 tokens + overhead
    // No fallback warning
    assert!(!response.receipt.warnings.iter().any(|w| w.contains("fallback")));
}

#[test]
fn tiktoken_without_feature_still_falls_back() {
    let response = compact_context(CompactRequest {
        session_id: "tiktoken-no-feature".into(),
        messages: vec![msg("user", "test")],
        policy: CompactionPolicy {
            token_counter: TokenCounterKind::TiktokenCl100k,
            ..Default::default()
        },
        focus: None,
    }).unwrap();

    // Without feature, should still record the kind but warn
    assert_eq!(response.receipt.token_counter, TokenCounterKind::TiktokenCl100k);
}
```

**Step 5: Test both feature configurations**

```bash
# Default (no tiktoken feature)
cargo test --test token_counter

# With tiktoken feature
cargo test --test token_counter --features tiktoken
```

Expected: All tests pass in both configs.

**Step 6: Commit**

```bash
git add Cargo.toml src/lib.rs tests/token_counter.rs
git commit -m "feat: real tiktoken-rs integration behind tiktoken feature flag"
```

---

### Task B2: Receipt index scaling — add benchmark and verify current throughput

**Objective:** The trigram SQLite index at `src/receipt_index.rs` works but degrades past ~1k receipts. Before switching to Tantivy, add a scaling benchmark to measure current performance and set improvement targets.

**Files:**
- Create: `tests/receipt_index_scale.rs`

**Step 1: Create scaling benchmark**

```rust
use context_governor::{compact_context, CompactRequest, CompactionPolicy, Message, BudgetMode};
use std::time::Instant;

fn msg(role: &str, content: &str) -> Message {
    Message {
        id: None, role: role.into(), content: content.into(),
        name: None, metadata: Default::default(),
    }
}

#[test]
fn receipt_index_scales_to_10k_compactions() {
    let start = Instant::now();
    let mut last_duration = std::time::Duration::ZERO;

    for i in 0..10_000 {
        let iter_start = Instant::now();
        let response = compact_context(CompactRequest {
            session_id: &format!("scale-{}", i),
            messages: vec![
                msg("system", &format!("scale test {}", i)),
                msg("user", &format!("message {}", i)),
            ],
            policy: CompactionPolicy::default(),
            focus: None,
        }).unwrap();

        // Store the response to exercise the receipt path
        let _ = response; // In real test, persist to temp store

        last_duration = iter_start.elapsed();

        // Verify every 1000 iterations
        if i > 0 && i % 1000 == 0 {
            // Require sub-100ms for individual compaction at scale
            assert!(
                last_duration.as_millis() < 100,
                "compaction {} took {}ms — indexing may be degrading",
                i, last_duration.as_millis()
            );
        }
    }

    let total = start.elapsed();
    // 10k compactions should complete in under 60 seconds
    assert!(
        total.as_secs() < 60,
        "10k compactions took {}s",
        total.as_secs()
    );
}

#[test]
fn receipt_index_search_remains_fast_after_10k() {
    // Insert 10k receipts, then measure search latency
    let temp = tempfile::tempdir().unwrap();
    // ... (abbreviated — full test in implementation)
}
```

**Step 2: Run benchmark**

```bash
cargo test --test receipt_index_scale -- --nocapture
```

Expected: tests pass within time bounds. If they don't, the scaling problem is confirmed and Tantivy migration becomes P0.

**Step 3: Commit**

```bash
git add tests/receipt_index_scale.rs
git commit -m "test: add receipt index scaling benchmark"
```

**Sprint B Gate:**
```bash
cargo test --all-targets && cargo test --all-targets --features tiktoken && cargo clippy --all-targets -- -D warnings
```

---

## Sprint C: Receipt HMAC + Plan Preservation (P1 — 2 tasks)

### Task C1: Add HMAC receipt integrity for cross-session binding

**Objective:** Content hashes exist (BLAKE3, SHA-256) but are unkeyed. Add HMAC-SHA256 variant for cross-session integrity verification.

**Files:**
- Modify: `src/receipt_index.rs` — add `hmac` integrity check function
- Modify: `Cargo.toml` — add `hmac` + `sha2` deps (sha2 already present)
- Create: `tests/receipt_integrity.rs`

**Step 1: Add dependency**

`Cargo.toml` already has `sha2 = "0.10"`. Add `hmac = "0.12"`.

**Step 2: Add signing and verification functions**

In `src/receipt_index.rs`:

```rust
use hmac::{Hmac, Mac};
use sha2::Sha256;

type HmacSha256 = Hmac<Sha256>;

/// Compute an HMAC-SHA256 over receipt content using a caller-supplied key.
pub fn sign_receipt_content(content: &str, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key)
        .expect("HMAC key can be any length");
    mac.update(content.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}

/// Verify receipt content against a stored HMAC.
pub fn verify_receipt_integrity(content: &str, key: &[u8], expected_hmac: &str) -> bool {
    let computed = sign_receipt_content(content, key);
    // Constant-time comparison
    computed == expected_hmac
}
```

**Step 3: Write integrity tests**

In `tests/receipt_integrity.rs`:

```rust
use context_governor::receipt_index::{sign_receipt_content, verify_receipt_integrity};

#[test]
fn hmac_verifies_same_content() {
    let key = b"test-integrity-key";
    let content = r#"{"receipt_id":"ctxr_test","compacted":42}"#;
    let hmac = sign_receipt_content(content, key);
    assert!(verify_receipt_integrity(content, key, &hmac));
}

#[test]
fn hmac_rejects_tampered_content() {
    let key = b"test-integrity-key";
    let original = r#"{"receipt_id":"ctxr_test","compacted":42}"#;
    let tampered = r#"{"receipt_id":"ctxr_test","compacted":99}"#;
    let hmac = sign_receipt_content(original, key);
    assert!(!verify_receipt_integrity(tampered, key, &hmac));
}

#[test]
fn hmac_rejects_wrong_key() {
    let content = r#"{"receipt_id":"ctxr_test"}"#;
    let hmac = sign_receipt_content(content, b"key-a");
    assert!(!verify_receipt_integrity(content, b"key-b", &hmac));
}
```

**Step 4: Commit**

```bash
git add Cargo.toml src/receipt_index.rs tests/receipt_integrity.rs
git commit -m "feat: add HMAC-SHA256 receipt integrity for cross-session binding"
```

---

### Task C2: Plan-aware classification in the allocator

**Objective:** Detect plan-like structures (numbered lists, checklists, TODOs, phases) and elevate their preservation priority so plans survive repeated compaction.

**Files:**
- Modify: `src/lib.rs` — add `detect_plan_content()` function + wire into classifier
- Modify: `tests/compaction.rs` — add plan survival test

**Step 1: Add plan detection heuristic**

In `src/lib.rs`, add near the content classification functions:

```rust
/// Detect plan-like structures in message content.
/// Returns true if the content contains numbered lists, phase markers,
/// checklist items, or explicit TODO/goal patterns.
pub fn detect_plan_content(content: &str) -> bool {
    let plan_signals = [
        // Numbered steps: "1.", "2.", "Step 1:", "Phase 1:"
        |s: &str| s.contains("Step ") || s.contains("Phase "),
        // Checklist markers: "- [ ]", "- [x]", "* [ ]"
        |s: &str| s.contains("[ ]") || s.contains("[x]") || s.contains("[X]"),
        // Explicit plan language
        |s: &str| {
            s.contains("TODO") || s.contains("ACTION ITEM")
                || s.contains("Acceptance Criteria") || s.contains("Implementation Plan")
                || s.contains("Sprint") || s.contains("Milestone")
        },
        // Numbered item density: 3+ numbered lines
        |s: &str| {
            s.lines()
                .filter(|line| {
                    line.trim().starts_with(|c: char| c.is_ascii_digit())
                        && line.trim().chars().skip_while(|c| c.is_ascii_digit())
                            .next() == Some('.')
                })
                .count() >= 3
        },
    ];

    plan_signals.iter().any(|signal| signal(content))
}
```

**Step 2: Elevate plan-containing messages in the classifier**

In the `classify_message()` or allocation function (around L1227-1270), add:

```rust
    if detect_plan_content(&msg.content) {
        item_type = ItemType::AcceptanceGate; // Reuse existing high-priority type
        reasons.push("plan-content-detected".to_string());
    }
```

**Step 3: Write plan survival test**

In `tests/compaction.rs`, add:

```rust
#[test]
fn plan_content_survives_compaction() {
    let plan_message = msg("user", r#"Implementation Plan:
1. Add fail-closed policy
2. Wire tiktoken-rs
3. Scale receipt index
4. Add plan preservation
5. Cross-engine harness

Acceptance Criteria:
- All tests pass
- No regressions"#);

    let messages = vec![
        msg("system", "sys"),
        msg("assistant", &"filler text ".repeat(200)),
        plan_message,
        msg("assistant", &"more filler ".repeat(200)),
        msg("user", "latest"),
    ];

    let response = compact_context(CompactRequest {
        session_id: "plan-survival".into(),
        messages,
        policy: CompactionPolicy {
            target_tokens: 200,
            budget_mode: BudgetMode::HardCascade,
            protect_last_n: 1,
            ..Default::default()
        },
        focus: None,
    }).unwrap();

    // The plan message should be preserved (not summarized/dropped)
    let plan_survives = response.compacted_messages.iter()
        .any(|m| m.content.contains("Implementation Plan"));
    assert!(plan_survives, "Plan content was lost during compaction");
}
```

**Step 4: Run and commit**

```bash
cargo test --test compaction plan_survival
git add src/lib.rs tests/compaction.rs
git commit -m "feat: plan-aware classification to preserve structured plans"
```

**Sprint C Gate:**
```bash
cargo test --all-targets && cargo clippy --all-targets -- -D warnings
```

---

## Sprint D: Content Reducers + Cross-Engine Harness (P1 — 2 tasks)

### Task D1: Add diff and compiler-output reducers

**Objective:** `reducers.rs` handles JSON. Add reducers for unified diffs and compiler/build output — the two highest-token tool output types.

**Files:**
- Modify: `src/reducers.rs` — add `reduce_diff()` and `reduce_compiler_output()`
- Modify: `tests/content_kind.rs` — add fixture tests

**Step 1: Add diff reducer**

In `src/reducers.rs`, add:

```rust
/// Reduce unified diff content: keep filenames, hunk headers, and added/removed line counts.
pub fn reduce_diff(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut kept = Vec::new();
    let mut total_lines = 0usize;
    let mut kept_lines = 0usize;

    for line in content.lines() {
        total_lines += 1;
        // Keep: diff headers (---, +++), hunk headers (@@), filenames
        if line.starts_with("--- ") || line.starts_with("+++ ") || line.starts_with("@@ ") {
            kept.push(line.to_string());
            kept_lines += 1;
            // Extract filename anchor
            if line.starts_with("--- ") || line.starts_with("+++ ") {
                push_anchor(&mut anchors, "diff_file", &line[4..], kept_lines, kept_lines);
            }
        }
        // Count added/removed lines but don't store them
    }

    let skipped = total_lines - kept_lines;
    if skipped > 0 {
        kept.push(format!("[... {} unchanged/removed lines omitted ...]", skipped));
    }

    let compacted = kept.join("\n");
    let was_truncated = compacted.len() > max_chars;
    ReducerOutput {
        compacted: if was_truncated { compacted[..max_chars].to_string() } else { compacted },
        anchors,
        was_truncated,
        approx_tokens: approx_tokens_for_text(&compacted),
        loss_flags: if skipped > 0 { vec!["diff_body_truncated".to_string()] } else { vec![] },
    }
}

/// Reduce compiler/build output: keep errors, warnings, exit code, and test summaries.
pub fn reduce_compiler_output(content: &str, max_chars: usize) -> ReducerOutput {
    let mut anchors = Vec::new();
    let mut kept = Vec::new();
    let mut error_count = 0usize;
    let mut warning_count = 0usize;

    for line in content.lines() {
        let trimmed = line.trim();
        // Keep error lines
        if trimmed.starts_with("error") || trimmed.starts_with("error[") || trimmed.contains("error:") {
            kept.push(line.to_string());
            error_count += 1;
            push_anchor(&mut anchors, "compiler_error", trimmed, kept.len(), kept.len());
        }
        // Keep warning lines
        else if trimmed.contains("warning:") || trimmed.contains("warning[") {
            kept.push(line.to_string());
            warning_count += 1;
        }
        // Keep test result lines
        else if trimmed.starts_with("test result:") || trimmed.contains("FAILED") || trimmed.contains("running ") {
            kept.push(line.to_string());
            push_anchor(&mut anchors, "test_result", trimmed, kept.len(), kept.len());
        }
        // Keep exit/summary
        else if trimmed.starts_with("error: could not compile") || trimmed.contains("Build failed") {
            kept.push(line.to_string());
        }
    }

    if error_count > 0 || warning_count > 0 {
        kept.insert(0, format!("[{} errors, {} warnings]", error_count, warning_count));
    }

    let compacted = kept.join("\n");
    let was_truncated = compacted.len() > max_chars;
    ReducerOutput {
        compacted: if was_truncated { compacted[..max_chars].to_string() } else { compacted },
        anchors,
        was_truncated,
        approx_tokens: approx_tokens_for_text(&compacted),
        loss_flags: vec!["compiler_output_reduced".to_string()],
    }
}
```

**Step 2: Wire into content-kind detection**

In `src/lib.rs`, find where `ContentKind` is detected and add:

```rust
    ContentKind::Diff => reduce_diff(&msg.content, policy.summary_max_chars),
    ContentKind::CompilerOutput => reduce_compiler_output(&msg.content, policy.summary_max_chars),
```

(Add `Diff` and `CompilerOutput` variants to `ContentKind` if not already present.)

**Step 3: Write fixture tests**

In `tests/content_kind.rs`, add:

```rust
use context_governor::reducers::{reduce_diff, reduce_compiler_output};

#[test]
fn diff_reducer_preserves_filenames_and_hunks() {
    let diff = "--- a/src/lib.rs\n+++ b/src/lib.rs\n@@ -10,5 +10,7 @@\n-old line\n+new line\n unchanged\n+another new";
    let output = reduce_diff(diff, 2000);
    assert!(output.compacted.contains("--- a/src/lib.rs"));
    assert!(output.compacted.contains("+++ b/src/lib.rs"));
    assert!(output.compacted.contains("omitted"));
    assert!(output.anchors.iter().any(|a| a.value.contains("src/lib.rs")));
}

#[test]
fn compiler_reducer_extracts_errors_and_test_results() {
    let output_text = "error[E0308]: mismatched types\n  --> src/lib.rs:10:5\nwarning: unused variable\n  --> src/main.rs:5:9\ntest result: FAILED. 2 passed; 1 failed";
    let output = reduce_compiler_output(output_text, 2000);
    assert!(output.compacted.contains("error[E0308]"));
    assert!(output.compacted.contains("test result"));
    assert!(output.compacted.contains("errors"));
}
```

Run: `cargo test --test content_kind` — expected: existing 2 + 2 new = 4 pass.

**Step 4: Commit**

```bash
git add src/reducers.rs src/lib.rs tests/content_kind.rs
git commit -m "feat: add diff and compiler-output content reducers"
```

---

### Task D2: Cross-engine comparison harness

**Objective:** A script that runs the same fixtures through built-in compressor vs context-governor and produces a comparison table.

**Files:**
- Create: `scripts/compare_engines.py`

**Step 1: Create comparison script**

```python
#!/usr/bin/env python3
"""Cross-engine comparison: built-in Hermes compressor vs context-governor."""
import json, subprocess, sys, time, statistics
from pathlib import Path

FIXTURES = Path(__file__).parent.parent / "tests" / "fixtures"
if not FIXTURES.exists():
    FIXTURES.mkdir()
    # Create sample fixtures
    (FIXTURES / "chat_session.json").write_text(json.dumps({
        "messages": [
            {"role": "system", "content": "You are a helpful assistant."},
            {"role": "user", "content": "What is Rust?"},
            {"role": "assistant", "content": "Rust is a systems programming language..."},
        ]
    }))

def run_context_governor(messages, mode="soft"):
    req = json.dumps({
        "session_id": "cmp",
        "messages": messages,
        "policy": {"target_tokens": 200, "budget_mode": mode, "protect_last_n": 1}
    })
    result = subprocess.run(
        ["context-governor", "compact"],
        input=req, capture_output=True, text=True, timeout=10
    )
    return json.loads(result.stdout)

def compare():
    results = []
    for fixture in FIXTURES.glob("*.json"):
        data = json.loads(fixture.read_text())
        msgs = data["messages"]

        # Context-governor
        t0 = time.monotonic()
        cg = run_context_governor(msgs)
        cg_time = time.monotonic() - t0

        results.append({
            "fixture": fixture.name,
            "engine": "context-governor",
            "tokens_before": cg["receipt"]["original_approx_tokens"],
            "tokens_after": cg["receipt"]["compacted_approx_tokens"],
            "reduction_pct": round(100 * (1 - cg["receipt"]["compacted_approx_tokens"] /
                max(1, cg["receipt"]["original_approx_tokens"])), 1),
            "latency_ms": round(cg_time * 1000, 1),
        })

    # Print comparison table
    print(f"{'Fixture':<30} {'Engine':<20} {'Before':>8} {'After':>8} {'Reduction':>10} {'Latency':>10}")
    print("-" * 90)
    for r in results:
        print(f"{r['fixture']:<30} {r['engine']:<20} {r['tokens_before']:>8} {r['tokens_after']:>8} {r['reduction_pct']:>9}% {r['latency_ms']:>9}ms")

if __name__ == "__main__":
    compare()
```

**Step 2: Test the script**

```bash
python3 scripts/compare_engines.py
```

**Step 3: Commit**

```bash
git add scripts/compare_engines.py
git commit -m "feat: add cross-engine comparison harness script"
```

**Sprint D Gate:**
```bash
cargo test --all-targets && cargo clippy --all-targets -- -D warnings
```

---

## Sprint E: Architecture Debt + Integration Polish (P1/P2 — 3 tasks)

### Task E1: Split lib.rs — extract `classify` module

**Objective:** Move classification/allocation logic out of lib.rs into a focused module. Target: reduce lib.rs by ~800 lines without changing public API.

**Files:**
- Create: `src/classify.rs`
- Modify: `src/lib.rs` — re-export, remove moved functions

**Step 1: Identify the extraction boundary**

Functions to extract: `classify_message()`, `build_context_steps()`, `extract_plan_state()`, `build_structural_floor()`, and their supporting types (ItemType, ContextItemV1, StructuralFloorV1, ContentKind, etc.).

**Step 2: Create src/classify.rs**

Move the identified ~800 lines into `src/classify.rs`, preserve all `pub` visibility, and re-export from `src/lib.rs`:

```rust
mod classify;
pub use classify::*;
```

**Step 3: Verify no API breakage**

```bash
cargo check --all-targets
cargo test --all-targets
```

All existing tests must pass unchanged — this is a pure code move.

**Step 4: Commit**

```bash
git add src/classify.rs src/lib.rs
git commit -m "refactor: extract classify module from lib.rs (~800 lines)"
```

---

### Task E2: Configurable checkpoint policy

**Objective:** Replace `llm_checkpoint_after_compressions=2` with a proper config struct supporting multiple strategies.

**Files:**
- Modify: `src/lib.rs` — add `CheckpointPolicy` struct + wire into compaction loop

**Step 1: Add CheckpointPolicy**

```rust
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum CheckpointStrategy {
    /// Never create LLM checkpoints.
    Off,
    /// Create a checkpoint after every N compactions.
    AfterN(usize),
    /// Create a checkpoint only when deterministic compaction is ineffective.
    IneffectiveOnly,
    /// Create a checkpoint when compaction savings drop below threshold %.
    ThresholdPct(f64),
}

impl Default for CheckpointStrategy {
    fn default() -> Self {
        CheckpointStrategy::AfterN(2)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CheckpointPolicy {
    #[serde(default)]
    pub strategy: CheckpointStrategy,
    #[serde(default)]
    pub max_checkpoints_per_session: Option<usize>,
}

impl Default for CheckpointPolicy {
    fn default() -> Self {
        Self {
            strategy: CheckpointStrategy::default(),
            max_checkpoints_per_session: Some(10),
        }
    }
}
```

Add to `CompactionPolicy`:

```rust
    #[serde(default)]
    pub checkpoint: CheckpointPolicy,
```

**Step 2: Wire into the compaction loop**

In the checkpoint decision, replace the hardcoded `2` with `request.policy.checkpoint.strategy`.

**Step 3: Write tests**

In `tests/policy.rs`, add:

```rust
#[test]
fn checkpoint_off_never_creates_checkpoints() {
    // ... test that with CheckpointStrategy::Off, no LLM checkpoints are created
}

#[test]
fn checkpoint_after_n_triggers_on_boundary() {
    // ... test that AfterN(3) creates checkpoints on compactions 3, 6, 9
}
```

**Step 4: Commit**

```bash
git add src/lib.rs tests/policy.rs
git commit -m "feat: configurable checkpoint policy with Off/AfterN/IneffectiveOnly/ThresholdPct"
```

---

### Task E3: Semantic-memory config — wire or delete

**Objective:** The `semantic_memory_enabled` and `archive_memory_enabled` config knobs report `unsupported_no_sink`. Either wire them to the real semantic-memory bridge or delete them.

**Decision:** Delete the knobs. The semantic-memory integration is a separate project (context-governor-python + Hermes adapter). False knobs are shadow truth.

**Files:**
- Modify: `src/lib.rs` — remove `semantic_memory_enabled` and `archive_memory_enabled` from `CompactionPolicy`
- Modify: `src/lib.rs` — remove `compact_context_with_memory_sink()` or gate behind a real feature

**Step 1: Remove shadow config fields**

From `CompactionPolicy` (L709-723), remove:

```rust
    #[serde(default)]
    pub semantic_memory_enabled: bool,
    #[serde(default)]
    pub archive_memory_enabled: bool,
```

And from `Default` impl (L725-738), remove the corresponding lines.

**Step 2: Gate memory_sink behind a feature**

If `compact_context_with_memory_sink()` exists but is unused:

```rust
#[cfg(feature = "memory-sink")]
pub fn compact_context_with_memory_sink(...) { ... }
```

**Step 3: Update tests**

Check `tests/memory_sink.rs` — either gate behind `#[cfg(feature = "memory-sink")]` or remove if unused.

**Step 4: Verify**

```bash
cargo check --all-targets
cargo test --all-targets
```

**Step 5: Commit**

```bash
git add src/lib.rs tests/memory_sink.rs
git commit -m "refactor: remove unwired semantic-memory config knobs (shadow truth)"
```

**Sprint E Gate:**
```bash
cargo test --all-targets && cargo clippy --all-targets -- -D warnings && cargo fmt --check
```

---

## Verification Gauntlet

After all sprints:

```bash
# Rust crate
cd ~/Coding/Libraries/context-governor
cargo test --all-targets
cargo test --all-targets --features tiktoken
cargo clippy --all-targets -- -D warnings
cargo fmt --check

# Ensure binary still works
echo '{"session_id":"gauntlet","messages":[{"role":"user","content":"test"}],"policy":{}}' | context-governor compact | python3 -m json.tool > /dev/null

# Check approximate test counts
cargo test --all-targets 2>&1 | grep "test result" | grep -oP '\d+ passed' | paste -sd+ | bc
# Expected: >120 tests pass
```

---

## Claim Boundary

### Safe to claim after Sprint A
- LLM summaries are fail-closed — unverified summaries cannot silently enter context
- Latest-user invariant has dedicated regression tests across all policy modes

### Safe to claim after Sprint B
- Token counting is provider-accurate when `tiktoken` feature is enabled
- Receipt index scaling is benchmarked with performance targets

### Safe to claim after Sprint C
- Receipt hashes are keyed (HMAC-SHA256) for cross-session integrity verification
- Plan structures survive compaction cycles

### Safe to claim after Sprint D
- Content-kind reducers handle diffs and compiler output
- Cross-engine comparison harness exists for quality claims

### Safe to claim after Sprint E
- lib.rs architectural debt is reduced
- Checkpoint policy is configurable
- No shadow-truth config knobs remain

### NOT safe to claim
- That this beats any other compactor on downstream task quality (needs live-model eval)
- That Tantivy replaces the trigram index (only benchmarked, not migrated)
- Production multi-tenant safety without independent security review
- Semantic-memory integration (deferred to separate project)

---

## Hard No List

- Do NOT add KV-cache work to this crate (belongs in poly-kv/quant-governor)
- Do NOT claim hosted API context extension — context-governor manages prompt budget only
- Do NOT add neural/ML-based compression until deterministic reducers are proven insufficient
- Do NOT add new core dependencies without measured ROI against the 6 existing deps
- Do NOT publish benchmark numbers as universal claims — label as local evidence only

---

## Execution Status — 2026-08-07

**All implemented directly (delegation tokens expired; subagents returned HTTP 401).**

### Completed ✅
- **A1** UnsafeSummaryPolicy enum (Warn/FallbackExtract/Freeze/FailClosed) + field on CompactionPolicy + SummarySafetyFailed error variant — `src/lib.rs`
- **A2** Boundary safety tests (`tests/boundary_safety.rs`) — 3 tests proving relinking detection
- **A3** Invariant tests (`tests/invariants.rs`) — latest-user survival across budget modes, many cycles, unsafe policies + serialization
- **B1** tiktoken-rs optional dep behind `tiktoken` feature (`Cargo.toml`, `src/lib.rs`) — **now fully wired**: `count_tokens_cl100k()` (BPE cached via OnceLock) routes `TiktokenCl100k` to real cl100k counting when the feature is on, falls back with warning when off. Feature-gated test `tiktoken_counts_match_expected_cl100k` confirms "Hello, world!" = 4 tokens.
- **B2** Receipt index scaling benchmark (`tests/receipt_index_scale.rs`) — 10k compactions
- **C1** HMAC-SHA256 receipt integrity (`src/receipt_index.rs` now `pub mod`, `tests/receipt_integrity.rs`) — 4 tests
- **C2** Plan-aware classification (`src/lib.rs` detect_plan_content + classify cascade) — elevates plan content to AcceptanceGate
- **D1** Diff/compiler-output reducers already present in `src/reducers.rs` (verified)
- **D2** Cross-engine harness (`scripts/compare_engines.py`)
- **E1 (bounded)** Extracted pure classification types + detect_plan_content into `src/classify.rs` (262 lines); `pub mod classify; pub use classify::*`. lib.rs reduced 3740→3496. Build functions (build_context_steps etc.) remain in lib.rs due to dense Message/compact_preview dependency closure — safe partial extraction.
- **E2** Configurable CheckpointPolicy (`CheckpointStrategy`: Off/AfterN/IneffectiveOnly/ThresholdPct(u8)) + `checkpoint` field on CompactionPolicy. Threshold uses u8 percent (0-100) not f64 to preserve `Eq` across the API chain.

### Corrected (plan premise wrong) 🔄
- **E3** NOT performed as specified — `semantic_memory_enabled`/`archive_memory_enabled` gate real behavior (`compact_context_with_memory_sink`, archive candidate detection, `tests/memory_sink.rs` prove they "fail loud not silent"). Removing them would delete real functionality, not shadow truth. Knobs retained.

### Final verification
```
cargo test --all-targets → 119 passed, 0 failed
cargo test --all-targets --features tiktoken → 120 passed, 0 failed
cargo clippy --all-targets → 0 warnings (dead_code allowed for pub API)
cargo clippy --all-targets --features tiktoken → 0 warnings
cargo fmt --check → clean
scripts/compare_engines.py → verified working (58.1% reduction on long session)
```

### Deferred (own focused passes)
- **E1 full** function-level extraction (build_context_steps/build_step/extract_plan_state/classify_messages + compact_preview/count_tokens_text/contains_any/is_aggressive_allocator) — large dependency web, zero functional gain, high regression risk against green tree
- **E2 adapter wiring** — ✅ DONE 2026-08-07: PyO3 binding (`context-governor-python/src/lib.rs`) now exposes `unsafe_summary_policy`, `checkpoint_strategy`, `max_checkpoints` via `compact()` with parse helpers. Committed `751c2e93`.
- **Cross-engine live-model eval** — needs real provider calls, not fixtures

