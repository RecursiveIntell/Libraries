# Claude Code Execution Prompt — Libraries Polish Sprint

## Context

You are working on a collection of utility crates in the `Libraries/` directory. Read `CLAUDE.md` for architecture invariants and conventions. Read `MASTER_ISSUE_MATRIX.md` for the full list of findings. This prompt guides you through fixing them in dependency order.

**CRITICAL RULE**: After each phase, run `cargo test --workspace` (or the applicable subset) and verify all tests pass before proceeding. Do not proceed to the next phase with broken tests.

---

## Phase 1: Compilability Foundation

**Goal**: All crates compile from a single workspace.

### Step 1.1: Normalize Path Dependencies

For every `Cargo.toml` in the workspace, change all path dependencies to use `../crate-name` format:

**Files to fix** (grep for `../../Libraries/` and `../../` in Cargo.toml files):
- `job-queue/Cargo.toml`: `stack-ids = { path = "../../Libraries/stack-ids" }` → `stack-ids = { path = "../stack-ids" }`
- `agent-graph/Cargo.toml`: `stack-ids = { path = "../../Libraries/stack-ids" }` → `stack-ids = { path = "../stack-ids" }`

Verify: `grep -rn '../../' */Cargo.toml` should return nothing.

### Step 1.2: Create Workspace Cargo.toml

Create `Libraries/Cargo.toml`:

```toml
[workspace]
resolver = "2"
members = [
    "stack-ids",
    "job-queue",
    "Tauri-Queue",
    "AI-Batch-Queue",
    "LLM-Pipeline",
    "ComfyUI-RS",
    "Ollama-Vision-RS",
    "agent-graph",
    "constraint-compiler",
    "discovery-portfolio",
    "spec-execution",
    "federated-settlement",
    "profile-runtime",
    "remote-oracle-admission",
]

[workspace.dependencies]
serde = { version = "1", features = ["derive"] }
serde_json = "1"
thiserror = "2"
tokio = { version = "1", features = ["full"] }
tracing = "0.1"
chrono = { version = "0.4", features = ["serde"] }
uuid = { version = "1", features = ["v4", "serde"] }
anyhow = "1"
schemars = "0.8"
blake3 = "1"
```

Then update individual `Cargo.toml` files to use `workspace = true` where applicable. Start with the crates that already use `{ workspace = true }` syntax (constraint-compiler, discovery-portfolio, etc.) and extend to others.

**Verification gate**: `cargo check --workspace` succeeds (may need to exclude crates with missing external deps initially).

---

## Phase 2: Correctness Fixes

### Step 2.1: Fix cancel_job TOCTOU (JQ-1)

**File**: `job-queue/src/db.rs`, function `cancel_job`

Replace the two-step SELECT+UPDATE with a single atomic UPDATE:

```rust
pub fn cancel_job(conn: &Connection, job_id: &str) -> Result<String> {
    let now = chrono::Utc::now().to_rfc3339();
    
    // Read the current status first for the return value
    let prev_status: String = conn
        .query_row(
            "SELECT status FROM queue_jobs WHERE id = ?1",
            params![job_id],
            |row| row.get(0),
        )
        .map_err(|_| anyhow::anyhow!("Job '{}' not found", job_id))?;

    // Atomically update only if still cancellable
    let affected = conn
        .execute(
            "UPDATE queue_jobs SET status = 'cancelled', completed_at = ?1
             WHERE id = ?2 AND status IN ('pending', 'processing')",
            params![now, job_id],
        )
        .context("Failed to cancel job")?;

    if affected == 0 {
        // Re-read to get accurate current status for error message
        let current_status: String = conn
            .query_row(
                "SELECT status FROM queue_jobs WHERE id = ?1",
                params![job_id],
                |row| row.get(0),
            )
            .unwrap_or_else(|_| "unknown".to_string());
        anyhow::bail!(
            "Job '{}' is not cancellable (current status: {})",
            job_id,
            current_status
        );
    }

    Ok(prev_status)
}
```

**Verification**: Run `cargo test -p job-queue`. All existing cancel tests must still pass.

### Step 2.2: Fix AI-Batch-Queue mark_running Split Lock (ABQ-2)

**File**: `AI-Batch-Queue/src/queue.rs`, function `mark_running`

Hold a single combined lock for the entire operation. Create a struct:

```rust
struct SchedulingState {
    last_resource_key: Option<String>,
    consecutive_same_key: usize,
    last_resource_switch: Option<Instant>,
}
```

Replace the three separate `Mutex<T>` fields with `Mutex<SchedulingState>`. Update `mark_running` to hold the jobs lock while updating scheduling state (or better, move scheduling state into the jobs lock scope).

**Verification**: `cargo test -p ai-batch-queue`

### Step 2.3: Fix AgentState Transaction Lost-Update (AG-1)

**File**: `agent-graph/src/state.rs`

Add a version counter to `AgentState`:

```rust
pub struct AgentState {
    data: Arc<RwLock<HashMap<String, Value>>>,
    version: Arc<std::sync::atomic::AtomicU64>,  // ADD THIS
    // ... rest unchanged
}
```

In `transaction()`, capture the current version. In `commit()`, check that the version hasn't changed:

```rust
pub async fn commit(mut self) -> Result<()> {
    let current_version = self.state.version.load(Ordering::SeqCst);
    if current_version != self.snapshot_version {
        return Err(AgentGraphError::StateError(
            "Transaction conflict: state was modified concurrently".to_string()
        ));
    }
    let next = self.working.read().await.clone();
    self.state.replace_data(next).await;
    self.state.version.fetch_add(1, Ordering::SeqCst);
    self.committed = true;
    Ok(())
}
```

**IMPORTANT**: This changes `commit()` from `async fn commit(mut self)` to `async fn commit(mut self) -> Result<()>`. Update all callers.

**Verification**: `cargo test -p agent-graph`. Add a new test that verifies concurrent modification is detected.

---

## Phase 3: Architecture Improvements

### Step 3.1: Decompose agent-graph/src/graph.rs (AG-2)

Split into:
- `graph.rs` — `AgentGraph` struct definition, `START`/`END` constants, `builder()` method
- `builder.rs` — `AgentGraphBuilder` and all `.add_node()`, `.add_edge()`, `.build()` methods
- `engine.rs` — `execute()`, `execute_with_config()`, `execute_with_run()`, superstep logic
- `checkpoint_coord.rs` — checkpoint save/restore coordination during execution

Move the `impl AgentGraph` blocks that handle execution into `engine.rs`. The builder is a separate concern.

**Verification**: `cargo test -p agent-graph` — all 2700+ lines of tests must still pass.

### Step 3.2: Per-Request Timeouts in LLM-Pipeline (LP-1)

**File**: `LLM-Pipeline/src/exec_ctx.rs`

Remove the timeout from the `Client` builder. Instead, apply timeouts at the request level in `backend/ollama.rs` and `backend/openai.rs`:

```rust
// In the backend's send_request implementation:
let response = client
    .post(&url)
    .timeout(ctx.limits.request_timeout)  // per-request timeout
    .json(&body)
    .send()
    .await?;
```

Also add a `timeout` field to `LlmCall` so individual payloads can override the default:

```rust
pub struct LlmCall {
    // ... existing fields
    timeout: Option<Duration>,
}
```

**Verification**: `cargo test -p llm-pipeline`

---

## Phase 4: Testing & Coherence

### Step 4.1: Add TauriEventEmitter Error Logging (TQ-1)

**File**: `Tauri-Queue/src/lib.rs`

Replace all `let _ = self.app_handle.emit(...)` with:

```rust
if let Err(e) = self.app_handle.emit("queue:job_started", event) {
    tracing::debug!(error = %e, "Failed to emit queue:job_started event");
}
```

### Step 4.2: Add stack-ids to ComfyUI-RS (CUI-3, TRACE-1)

Add `stack-ids = { path = "../stack-ids" }` to `ComfyUI-RS/Cargo.toml`.

Add an optional `trace_ctx` parameter to key methods:

```rust
pub async fn queue_prompt_traced(
    &self,
    workflow: &Value,
    trace_ctx: Option<&stack_ids::TraceCtx>,
) -> Result<String> {
    // Include trace_ctx in logging if present
    if let Some(ctx) = trace_ctx {
        tracing::debug!(trace_id = %ctx.trace_id, "Queuing prompt to ComfyUI");
    }
    self.queue_prompt(workflow).await
}
```

### Step 4.3: Fix cancel_job TOCTOU Test

Add a test to `job-queue/src/db.rs` that validates the fix:

```rust
#[test]
fn test_cancel_completed_returns_current_status() {
    let conn = setup();
    insert_job(&conn, "job-1", 2, &serde_json::json!({})).unwrap();
    mark_processing(&conn, "job-1").unwrap();
    mark_completed(&conn, "job-1").unwrap();

    let result = cancel_job(&conn, "job-1");
    assert!(result.is_err());
    let err_msg = result.unwrap_err().to_string();
    assert!(err_msg.contains("completed"), "Error should mention current status, got: {}", err_msg);
}
```

---

## Phase 5: Documentation Polish

### Step 5.1: Update All README.md Files

Each crate's README should include:
1. One-sentence description
2. Minimum viable example (compiles and runs)
3. Link to the ecosystem (other crates that compose with it)
4. `stack-ids` integration status

### Step 5.2: Add CHANGELOG.md to Each Crate

Standard format:
```markdown
# Changelog

## [Unreleased]
### Fixed
- [JQ-1] cancel_job() TOCTOU race condition
### Changed
- Path dependencies normalized to `../crate-name` format
```

---

## Verification Gates

After ALL phases complete:

1. `cargo check --workspace` — clean
2. `cargo test --workspace` — all pass
3. `cargo clippy --workspace -- -D warnings` — no warnings
4. `grep -rn '../../' */Cargo.toml` — no legacy paths
5. Every `#[deprecated]` item has a removal condition documented
6. Every error type has a `.kind()` method

**Do not declare completion until all 6 gates pass.**
