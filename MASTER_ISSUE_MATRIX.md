# Master Issue Matrix — Utility Libraries Ecosystem

**Generated**: 2026-03-25  
**Covers**: All 13 Rust crates + Tauri-React-Hooks TypeScript package

## Issue Key

- **Severity**: CRITICAL > HIGH > MEDIUM > LOW
- **Category**: CORRECTNESS | ARCHITECTURE | PERFORMANCE | SAFETY | COHERENCE | TESTING | DEBT
- **Status**: OPEN (all — this is the initial audit)

---

## CRITICAL Issues

| ID | Crate | Category | Title | Description | Fix Effort |
|----|-------|----------|-------|-------------|------------|
| PATH-1 | ecosystem | COHERENCE | Inconsistent relative path deps | `job-queue` and `agent-graph` use `../../Libraries/stack-ids`, others use `../stack-ids`. Cannot compile from same layout. | 1h — pick one convention, update all Cargo.toml files |
| PR-1 | profile-runtime | COHERENCE | 5 missing path dependencies | Depends on `assurance-runtime`, `attestation-exchange`, `authority-delegation`, `continuity-runtime`, `verification-policy` — none shipped | 0h (ship them) or exclude crate |

## HIGH Issues

| ID | Crate | Category | Title | Description | Fix Effort |
|----|-------|----------|-------|-------------|------------|
| PATH-2 | ecosystem | ARCHITECTURE | No workspace Cargo.toml | No shared dep resolution, no `cargo test --workspace`, no compilation validation across crates | 2h — create workspace root, unify dep versions |
| PATH-3 | ecosystem | COHERENCE | Missing sibling crates | LLM-Pipeline needs `llm-output-parser`/`llm-tool-runtime`; constraint-compiler needs 3 crates; remote-oracle-admission needs `attestation-exchange` | 0h (ship them) or document as external |
| ABQ-1 | AI-Batch-Queue | PERFORMANCE | Global Mutex on job list | `Mutex<Vec<BatchJob<D>>>` locks entire list for every item update. Contention under concurrent UI + executor access | 3h — switch to `RwLock` or per-job lock |
| ABQ-2 | AI-Batch-Queue | CORRECTNESS | Split lock in mark_running | Drops jobs lock, then acquires 3 scheduling locks. Scheduling metadata can become inconsistent with job state | 2h — hold jobs lock while updating scheduling state, or use a single combined lock |
| AG-1 | agent-graph | CORRECTNESS | Transaction lost-update anomaly | `commit()` does wholesale `replace_data()`. Concurrent state changes between `transaction()` and `commit()` are silently lost | 4h — add version stamp + check-on-commit, or implement diff-based merge |
| AG-2 | agent-graph | ARCHITECTURE | graph.rs monolith (1539 lines) | Builder, execution engine, superstep scheduler, checkpoint coordination, event emission all in one file | 6h — decompose into builder.rs, engine.rs, superstep.rs, checkpoint_coord.rs |
| LP-1 | LLM-Pipeline | ARCHITECTURE | Per-client timeout baked at build time | `ExecCtx` client timeout is fixed at construction. Can't serve mixed-latency payloads (5s classifier + 120s generator) | 3h — move timeout to per-request via `reqwest::RequestBuilder::timeout()` |
| TRH-1 | Tauri-React-Hooks | TESTING | Zero test coverage | No test files exist. Async subscription cleanup, stale closure bugs, and memory leaks have no regression protection | 8h — set up vitest/react-testing-library, write tests for all 6 hooks |

## MEDIUM Issues

| ID | Crate | Category | Title | Description | Fix Effort |
|----|-------|----------|-------|-------------|------------|
| JQ-1 | job-queue | CORRECTNESS | cancel_job() TOCTOU | Two-step SELECT then UPDATE allows race. Error message can be wrong. | 1h — single UPDATE with WHERE + affected check |
| JQ-2 | job-queue | SAFETY | requeue_interrupted() no staleness check | Blindly resets all processing jobs. Could yank active jobs from running workers if called at wrong time | 1h — add heartbeat staleness guard or restrict to startup-only call |
| TQ-1 | Tauri-Queue | SAFETY | Silent event emission failure | `let _ = self.app_handle.emit(...)` discards errors without logging | 0.5h — add `tracing::debug!` on emit failure |
| TQ-2 | Tauri-Queue | CORRECTNESS | Stale ETA on coalesced events | Pending progress events carry old ETA data when flushed with newer events | 1h — recalculate ETA at flush time or drop stale pending events |
| ABQ-3 | AI-Batch-Queue | PERFORMANCE | Reorder clones all queued jobs | `reorder_queued_jobs` clones entire job payloads just to sort. O(n*item_count) memory spike | 2h — sort by index/key pairs instead |
| ABQ-4 | AI-Batch-Queue | SAFETY | Executor silent spin on missing state | If `BatchQueue` never registered in Tauri state, executor polls forever with no warning | 0.5h — add tracing::warn after N failed lookups |
| AG-3 | agent-graph | CORRECTNESS | max_parallelism not enforced at runtime | Config says max 32, but `tokio::spawn` has no semaphore. Fan-out graphs can exceed limit | 2h — add `tokio::sync::Semaphore` gated by config |
| AG-4 | agent-graph | SAFETY | Lock timeout during node execution | If node holds state lock > 5s (LLM call while locked), other nodes get timeout errors | 2h — document "don't hold locks across awaits" or enforce via API design |
| AG-5 | agent-graph | COHERENCE | stack-ids path inconsistency | Uses `../../Libraries/stack-ids` while siblings use `../stack-ids` | 0.5h — fix path (subsumed by PATH-1) |
| LP-2 | LLM-Pipeline | ARCHITECTURE | tool_loop.rs too large (1125 lines) | Mixes tool definition, invocation, and orchestration | 3h — decompose |
| LP-3 | LLM-Pipeline | SAFETY | Hardcoded default model `llama3.2:3b` | Wrong for non-Ollama backends. May not be installed. | 0.5h — remove default or use sentinel |
| LP-4 | LLM-Pipeline | CORRECTNESS | anyhow→PipelineError strips error chain | `From<anyhow::Error>` converts to string, losing backtrace and source chain | 1h — preserve source or use `anyhow` consistently |
| CUI-1 | ComfyUI-RS | SAFETY | Unbounded image download | `image()` downloads full response with no size limit | 1h — add default limit parameter |
| CUI-2 | ComfyUI-RS | PERFORMANCE | No connection pool config | Default reqwest pool for rapid request sequences | 0.5h — expose pool configuration |
| TRACE-1 | ecosystem | COHERENCE | ComfyUI-RS and Ollama-Vision-RS lack stack-ids | Trace lineage breaks at their boundary | 3h — add TraceCtx propagation to both crates |
| TRH-2 | Tauri-React-Hooks | CORRECTNESS | useTauriQuery re-fetches on unstable args | `JSON.stringify(args)` as dep key means unmemoized object literals trigger re-fetch every render | 1h — document requirement or add deep comparison |
| TRH-3 | Tauri-React-Hooks | CORRECTNESS | Handler removal doesn't unsubscribe immediately | Removing a key from bindings object only takes effect on next deps change | 0.5h — document behavior |
| CC-1 | constraint-compiler | COHERENCE | 3 missing path deps | Cannot compile from this distribution | 0h (ship deps) |
| CC-2 | constraint-compiler | CORRECTNESS | Non-canonical JSON hashing | blake3 hash over unsorted JSON keys — semantically equal objects hash differently | 2h — use canonical JSON serializer |
| OV-1 | Ollama-Vision-RS | COHERENCE | No stack-ids integration | Same as TRACE-1 | 1h |
| PR-2 | profile-runtime | TESTING | No tests shipped | 3965 lines with no test coverage in distribution | 4h |
| THIN-1 | remote-oracle-admission | COHERENCE | Missing attestation-exchange dep | Cannot compile | 0h (ship dep) |

## LOW Issues

| ID | Crate | Category | Title | Description | Fix Effort |
|----|-------|----------|-------|-------------|------------|
| JQ-3 | job-queue | SAFETY | Random worker_id on every restart | Makes heartbeat ownership useless across restarts | 0.5h — document or derive stable ID |
| JQ-4 | job-queue | PERFORMANCE | Unprepared statement in count_by_status | Creates new prepared statement each call | 0.5h — cache or document |
| JQ-5 | job-queue | PERFORMANCE | No index on heartbeat_at | reclaim_stale scans after status index | 0.5h — add composite index |
| TQ-3 | Tauri-Queue | ARCHITECTURE | Over-engineered buffer_size | Buffer keyed by job_id, typically only 1 active job | 0h — document, not worth changing |
| ABQ-5 | AI-Batch-Queue | PERFORMANCE | ETA tracker never evicts | Running average includes ancient data | 2h — add sliding window |
| LP-5 | LLM-Pipeline | DEBT | Deprecated TraceId still public | 150 lines of compat code | 1h — feature-gate |
| LP-6 | LLM-Pipeline | ARCHITECTURE | Dual JSON auto-completion paths | StreamingDecoder and output_parser both auto-complete JSON differently | 2h — consolidate |
| AG-6 | agent-graph | DEBT | node! macro doesn't document Arc clone cost | Users might avoid macro thinking clone is expensive | 0.5h — add doc comment |
| CUI-3 | ComfyUI-RS | COHERENCE | No stack-ids integration | Breaks trace lineage at ComfyUI boundary | 1h |
| OV-2 | Ollama-Vision-RS | SAFETY | Hardcoded 30s timeout | May be too short for large images | 0.5h — make configurable |
| TRH-4 | Tauri-React-Hooks | SAFETY | No TypeScript strict mode | tsconfig.json should have `"strict": true` | 0.5h |
| THIN-2 | thin crates | TESTING | Minimal test coverage | Only basic serde roundtrips | 2h — add property-based tests |
| DEPRECATION-1 | ecosystem | DEBT | Extensive migration annotations | Every event struct has ~6 deprecated fields. Correct but noisy | Ongoing — complete migration, then remove |

---

## Summary Statistics

| Severity | Count |
|----------|-------|
| CRITICAL | 2 |
| HIGH | 8 |
| MEDIUM | 22 |
| LOW | 13 |
| **TOTAL** | **45** |

| Category | Count |
|----------|-------|
| COHERENCE | 10 |
| CORRECTNESS | 9 |
| ARCHITECTURE | 6 |
| SAFETY | 8 |
| PERFORMANCE | 5 |
| TESTING | 4 |
| DEBT | 3 |

---

## Fix Order (Dependency-Aware)

**Phase 1: Compilability** (blocks everything else)
1. PATH-1: Normalize all path deps
2. PATH-2: Create workspace Cargo.toml
3. PATH-3, CC-1, PR-1, THIN-1: Ship or exclude missing deps

**Phase 2: Correctness** (highest user impact)
4. JQ-1: Fix cancel_job TOCTOU
5. AG-1: Fix transaction lost-update
6. ABQ-2: Fix mark_running split lock
7. CC-2: Fix non-canonical JSON hashing

**Phase 3: Architecture** (prevents future bugs)
8. AG-2: Decompose graph.rs
9. LP-1: Per-request timeouts
10. LP-2: Decompose tool_loop.rs
11. ABQ-1: Switch to RwLock for job list

**Phase 4: Testing & Polish**
12. TRH-1: Add React hooks tests
13. TRACE-1: Add stack-ids to ComfyUI-RS and Ollama-Vision-RS
14. Remaining MEDIUM and LOW items
