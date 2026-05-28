# Libraries Master Issue Matrix — 2026-04-01

## Supersedes: HOSTILE_AUDIT_SYNTHESIS_V5 (2026-03-30, score 8.1)
## Current score: 8.4 / 10

## Closures since V5
- **LIB-001** (P0 governance fail-open): CLOSED — `GovernanceMode::Strict`, 16 tests, default feature
- **LIB-005** (workspace lints): CLOSED — enforced, all members inherit
- **CLIB-005** (governance never gates): CLOSED — `gate_execution_with_mode()` promotes Blocked to Err

---

## Phase 0: Fix Retrieval Architecture (15 min)

| ID | Title | Severity | Location | Fix | Acceptance |
|---|---|---|---|---|---|
| LIB-CRIT-001 | Hybrid search fallback suppressed for domain-scoped queries | CRIT | `knowledge-runtime/src/runtime/core.rs:695-711` | Change `scope_requires_pushdown` branch to fall back to hybrid search when `projection_results.len() < leg.limit` | Domain-scoped KnowledgeRuntime queries return hybrid results |

### Exact fix for LIB-CRIT-001

In `knowledge-runtime/src/runtime/core.rs`, replace the `scope_requires_pushdown` branch (around line 701):

```rust
// BEFORE:
if scope_requires_pushdown {
    projection_results
} else if projection_results.len() < leg.limit {

// AFTER:
// LIB-CRIT-001: Even with scope pushdown, fall back to hybrid
// search when projection results are insufficient. The projection
// path only has substring matching; hybrid search uses
// FTS5 + HNSW + RRF for semantic retrieval.
if scope_requires_pushdown && projection_results.len() >= leg.limit {
    projection_results
} else if projection_results.len() < leg.limit {
```

This single condition change makes the fallback reachable when `scope_requires_pushdown` is true but results are thin (which they almost always are, since substring matching returns far fewer results than semantic search).

**Add a test** in `knowledge-runtime/tests/` that creates a KnowledgeRuntime with a domain-scoped query and verifies results come from the hybrid path.

---

## Phase 1: Crash Safety (30 min)

| ID | Title | Severity | Location | Fix | Acceptance |
|---|---|---|---|---|---|
| LIB-HIGH-001 | `unreachable!()` in 3 production paths | HIGH | `act.rs:323`, `main_support/mod.rs:420`, `obs/trace.rs:388` | Replace with `Err(...)` or `return Err(...)` | `grep -rn 'unreachable!' --include='*.rs' \| grep -v test` returns zero |
| LIB-HIGH-002 | No HTTP timeout on TUI clients | HIGH | `main_support/mod.rs:505,546` | `Client::builder().timeout(Duration::from_secs(300)).build()` | `grep 'Client::new()' main_support` returns zero |
| LIB-LOW-002 | Missing NaN validation on HNSW insert | LOW | `semantic-memory/src/hnsw.rs:validate_dimensions()` | Add `!v.is_finite()` check | Test: `insert([NaN, 0.0, ...])` returns `Err` |

### Exact fixes

**LIB-HIGH-001** — `forge-pilot/src/act.rs:323`:
```rust
// BEFORE:
_ => unreachable!(),
// AFTER:
_ => return Err(PilotError::Other(format!("unsupported plan kind for oracle execution: {:?}", plan))),
```

`forge-pilot/src/main_support/mod.rs:420`:
```rust
// BEFORE:
None => unreachable!("provider presence checked"),
// AFTER:
None => return Err("provider not available after validation".into()),
```

`knowledge-runtime/src/obs/trace.rs:388`:
```rust
// BEFORE:
_ => unreachable!(),
// AFTER:
_ => "unknown".into(),
```

**LIB-HIGH-002** — Both call sites in `main_support/mod.rs`:
```rust
// BEFORE:
let client = reqwest::Client::new();
// AFTER:
let client = reqwest::Client::builder()
    .timeout(std::time::Duration::from_secs(300))
    .connect_timeout(std::time::Duration::from_secs(10))
    .build()
    .map_err(|e| format!("failed to build HTTP client: {e}"))?;
```

**LIB-LOW-002** — `semantic-memory/src/hnsw.rs`:
```rust
fn validate_dimensions(vector: &[f32], expected: usize) -> Result<(), MemoryError> {
    if vector.len() != expected {
        return Err(MemoryError::HnswError(format!(
            "expected {} dimensions, got {}", expected, vector.len()
        )));
    }
    // LIB-LOW-002: Reject NaN/infinity embeddings
    if vector.iter().any(|v| !v.is_finite()) {
        return Err(MemoryError::HnswError(
            "embedding contains NaN or infinity values".into()
        ));
    }
    Ok(())
}
```

---

## Phase 2: Hardening (1 hr)

| ID | Title | Severity | Location | Fix | Acceptance |
|---|---|---|---|---|---|
| LIB-MED-002 | Unbounded PilotHistory growth | MED | `forge-pilot/src/history.rs` | Cap `prior_attempt_bundle_ids` and `prior_outcomes` at 20 entries | After 50 iterations, vectors capped at 20 |
| LIB-LOW-001 | DefaultHasher for deterministic digest | LOW | `forge-pilot/src/orient.rs:166` | Replace with `blake3::hash` | `grep DefaultHasher forge-pilot` returns zero |
| LIB-LOW-003 | Profile composition thin test coverage | LOW | `profile-runtime/src/compose.rs` | Add proptest for fold class invariants | 256 proptest cases pass |

### Exact fix for LIB-MED-002

In `forge-pilot/src/history.rs`, in `record_outcome()`:
```rust
const MAX_HISTORY_ENTRIES: usize = 20;

// After pushing to prior_attempt_bundle_ids:
if entry.prior_attempt_bundle_ids.len() > MAX_HISTORY_ENTRIES {
    entry.prior_attempt_bundle_ids.drain(..entry.prior_attempt_bundle_ids.len() - MAX_HISTORY_ENTRIES);
}
// After pushing to prior_outcomes:
if entry.prior_outcomes.len() > MAX_HISTORY_ENTRIES {
    entry.prior_outcomes.drain(..entry.prior_outcomes.len() - MAX_HISTORY_ENTRIES);
}
```

### Exact fix for LIB-LOW-001

In `forge-pilot/src/orient.rs`, replace `bounded_region_digest()`:
```rust
fn bounded_region_digest(observation: &Observation, target: &TargetKind) -> String {
    let mut hasher = blake3::Hasher::new();
    hasher.update(observation.scope_key.namespace.as_bytes());
    if let Some(ref domain) = observation.scope_key.domain {
        hasher.update(domain.as_bytes());
    }
    hasher.update(target.stable_key().as_bytes());
    if let Some(ref row) = observation.import_log {
        hasher.update(row.imported_at.as_bytes());
    }
    for claim in &observation.claim_versions {
        hasher.update(claim.claim_version_id.as_str().as_bytes());
    }
    format!("region:{}", &hasher.finalize().to_hex()[..16])
}
```

Remove the `use std::collections::hash_map::DefaultHasher;` and `use std::hash::{Hash, Hasher};` imports.

---

## Phase 3: Decomposition (3 hr)

| ID | Title | Severity | Location | Fix | Acceptance |
|---|---|---|---|---|---|
| LIB-MED-001 | main_support/mod.rs monolith | MED | `forge-pilot/src/main_support/mod.rs` | Split into 5 modules | No file > 600 lines |

### Module split plan

| New file | Functions moved | Lines (est) |
|---|---|---|
| `main_support/cli.rs` | `CommandArgs`, `parse`, `help`, `run_command_cli` | ~250 |
| `main_support/tui.rs` | `run_tui`, `start_closed_loop`, `stop_closed_loop`, `show_status_screen`, `ActiveLoop`, `LoopStatusSnapshot`, `AppState` | ~400 |
| `main_support/provider.rs` | `chat_with_ollama`, `chat_with_openai`, `configure_provider`, `chat_loop`, `chat_with_provider`, `augment_history_with_grounding`, `stream_json_lines`, `stream_sse_lines`, `trim_trailing_slash` | ~350 |
| `main_support/explain.rs` | `explain_observation`, `explain_candidates`, `explain_import_report`, `explain_bootstrap_report`, `explain_loop_report`, `explain_loop_report_detailed` | ~350 |
| `main_support/storage.rs` | `open_resources`, `open_memory_store`, `open_forge_store`, `build_loop_config`, `normalize_user_path`, `managed_storage_root_for_workspace`, `uses_managed_or_legacy_storage_defaults`, `detect_project_root`, `looks_like_project_root` | ~250 |
| `main_support/mod.rs` | `run()` entry point, re-exports | ~30 |
