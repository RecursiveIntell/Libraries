# CLAUDE.md — Utility Libraries Ecosystem

## Project Identity

This is a collection of reusable Rust crates and one TypeScript package that form the utility layer for a local-first AI agent runtime. The crates are designed to compose: they share identity primitives via `stack-ids`, use consistent error patterns, and follow the same API conventions.

## Directory Layout

All crates live as siblings in the `Libraries/` directory. The canonical path convention is:
```
Libraries/
  stack-ids/
  job-queue/
  Tauri-Queue/
  AI-Batch-Queue/
  LLM-Pipeline/
  agent-graph/
  ComfyUI-RS/
  Ollama-Vision-RS/
  Tauri-React-Hooks/
  constraint-compiler/
  discovery-portfolio/
  spec-execution/
  federated-settlement/
  profile-runtime/
  remote-oracle-admission/
```

**All path dependencies MUST use `../crate-name` relative paths.** The `../../Libraries/crate-name` convention is legacy and must not be used.

## Build & Test

```bash
# From workspace root (Libraries/)
cargo test --workspace
cargo clippy --workspace -- -D warnings

# TypeScript
cd Tauri-React-Hooks && npm run typecheck
```

## Architecture Invariants

1. **stack-ids is the identity foundation.** Every crate that handles trace correlation, retry lineage, or cross-crate identity MUST use `stack_ids::TraceCtx`, `stack_ids::AttemptId`, and `stack_ids::TrialId`. Never invent local ID types.

2. **Error types MUST have a `.kind() -> &'static str` method** for stable programmatic matching. This is the PRIMITIVES_CONTRACT §2 requirement.

3. **thiserror v2** across all crates. No thiserror v1. No anyhow in public APIs (internal only).

4. **Stable Rust only.** No nightly features.

5. **SQLite defaults:** WAL mode, `PRAGMA foreign_keys = ON`, `PRAGMA busy_timeout = 5000`. Every crate that opens a SQLite connection MUST set these.

6. **Lock safety:** Never hold a Mutex/RwLock across an `.await` point. Use `tokio::sync::RwLock` when you need async-compatible locking. Use `std::sync::Mutex` only for synchronous critical sections. Always handle mutex poisoning (`unwrap_or_else(|e| e.into_inner())` for non-critical state, or propagate as error).

7. **Deprecation protocol:** Deprecated items must have:
   - `#[deprecated(since = "X", note = "Use Y instead...")]`
   - Doc comment with "Phase status: compatibility / migration-only"
   - "Removal condition:" stating when it can be deleted
   - A conversion method to/from the canonical replacement

## Code Patterns

### Job/Queue Crates
- Atomic claim: SELECT + UPDATE in single transaction, check `affected > 0`
- Worker ownership: every DB mutation on a processing job must validate `worker_id`
- Retry lineage: `attempt_id` (one per re-enqueue) + `trial_id` (one per execution)

### LLM-Pipeline
- `Payload` trait is the extension point. All LLM operations implement `Payload`.
- `ExecCtx` is shared context — construct once, use across all payloads in a chain.
- `OutputStrategy` controls parsing. Default is `Lossy` (wraps raw text in JSON string).
- Transport retry (429/5xx) is separate from semantic retry (parse failures).

### agent-graph
- `AgentState` uses `tokio::sync::RwLock` with configurable lock timeouts.
- Reducers compose state changes during parallel execution.
- The `node!` macro is the primary way to create nodes. It handles cloning correctly.
- `START` and `END` are virtual node names for graph entry/exit.

### Tauri Integration
- `TauriEventEmitter` bridges job-queue events to Tauri's frontend event system.
- `CoalescingEmitter` wraps any emitter with backpressure and deduplication.
- Events use `queue:` prefix for job-queue, `ai_batch:` prefix for batch queue.

## Testing Requirements

- Every public function must have at least one test.
- State transitions must be tested: valid transitions succeed, invalid transitions fail with correct error.
- Concurrency must be tested: claim races, cancellation during execution, heartbeat expiry.
- Integration tests go in `tests/`, unit tests go in `#[cfg(test)] mod tests` at the bottom of each file.

## What NOT to Do

- Do not add `unsafe` code. There are zero unsafe blocks in this ecosystem and it should stay that way.
- Do not add runtime panics. All errors must be propagated via Result. The only acceptable `.expect()` is for infallible operations (e.g., building an HTTP client with valid configuration).
- Do not add `println!`. Use `tracing` macros (`tracing::info!`, `tracing::warn!`, etc.).
- Do not change the `stack-ids` API without updating every consumer crate.
- Do not introduce workspace-level features that change the public API of individual crates.
