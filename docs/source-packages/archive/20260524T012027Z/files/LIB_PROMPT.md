# PROMPT.md — Libraries Fix Execution Guide

## Pre-flight
Read these files before any code changes:
1. `CLAUDE.md` (always)
2. `MASTER_ISSUE_MATRIX.md` (for current phase)

## Session strategy

### Session 1: Phase 0 — Fix Retrieval Architecture (15 min)

**Goal:** Make domain-scoped queries through KnowledgeRuntime use hybrid search.

1. Open `knowledge-runtime/src/runtime/core.rs`
2. Find the `scope_requires_pushdown` branch (around line 701)
3. Apply the exact fix from MASTER_ISSUE_MATRIX.md

**Before:**
```rust
if scope_requires_pushdown {
    projection_results
} else if projection_results.len() < leg.limit {
```

**After:**
```rust
if scope_requires_pushdown && projection_results.len() >= leg.limit {
    projection_results
} else if projection_results.len() < leg.limit {
```

4. Add test in `knowledge-runtime/tests/` or inline:
```rust
#[tokio::test]
async fn domain_scoped_query_falls_back_to_hybrid() {
    // Create runtime with domain scope, import a projection,
    // query with semantic content that won't substring-match.
    // Verify results come back (from hybrid path).
}
```

**Gate:**
```bash
cargo check -p knowledge-runtime
cargo test -p knowledge-runtime
```

---

### Session 2: Phase 1 — Crash Safety (30 min)

**Goal:** Remove unreachable!(), add timeouts, add NaN validation.

Apply all three fixes from MASTER_ISSUE_MATRIX.md Phase 1.

**Gate:**
```bash
cargo check --workspace
cargo test --workspace
# Verify:
grep -rn 'unreachable!' --include='*.rs' | grep -v test | grep -v '// '
# Should return zero non-comment matches
grep -n 'Client::new()' forge-pilot/src/main_support/mod.rs
# Should return zero matches
```

---

### Session 3: Phase 2 — Hardening (1 hr)

**Goal:** Cap history, replace hasher, add composition tests.

Apply all three fixes from MASTER_ISSUE_MATRIX.md Phase 2.

For LIB-LOW-003 (proptest), create `profile-runtime/tests/composition_proptest.rs`:
```rust
use proptest::prelude::*;
use profile_runtime::compose::*;
use profile_runtime::rules::*;

proptest! {
    #[test]
    fn union_fold_contains_all_inputs(
        values in prop::collection::vec("[a-z]{3,8}", 1..5)
    ) {
        let contributions = values.iter().map(|v| /* build contribution */).collect();
        let result = fold_contributions(FoldClassV1::Union, contributions);
        for v in &values {
            assert!(result.string_values.contains(v));
        }
    }
    
    // Similar tests for Intersection ⊆ every input,
    // MinOfMaxima ≤ every max, etc.
}
```

**Gate:**
```bash
cargo check --workspace
cargo test --workspace
```

---

### Session 4: Phase 3 — Decomposition (3 hr)

**Goal:** Split main_support/mod.rs into 5 modules.

Follow the module split plan in MASTER_ISSUE_MATRIX.md. Work incrementally:

1. Create `main_support/cli.rs` — move `CommandArgs` and argument parsing
2. Create `main_support/storage.rs` — move store opening and path detection
3. Create `main_support/explain.rs` — move all `explain_*` formatting functions
4. Create `main_support/provider.rs` — move chat functions and streaming
5. Create `main_support/tui.rs` — move interactive loop and status
6. Reduce `main_support/mod.rs` to `run()` + re-exports

After each file extraction: `cargo check -p forge-pilot`

**Gate:**
```bash
cargo check --workspace
cargo test --workspace
wc -l forge-pilot/src/main_support/*.rs
# No file should exceed 600 lines
```

---

## Context window management

Each session should begin by reading:
1. `CLAUDE.md` (always)
2. `MASTER_ISSUE_MATRIX.md` (for the specific phase)
3. The source files being modified

## Cross-workspace note

**LIB-CRIT-001** fixes the root cause of Recall's retrieval failure. Recall has a workaround (RCL-CRIT-001) that bypasses knowledge-runtime for the default observe path, but the proper fix is here in the Libraries workspace. Both fixes should be applied.
