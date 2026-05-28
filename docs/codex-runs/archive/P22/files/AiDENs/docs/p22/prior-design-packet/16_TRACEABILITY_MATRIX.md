# 16 — Traceability Matrix

This matrix maps current Recall source surfaces to proposed AiDENs crates.

| Current Recall source | Proposed AiDENs target | Notes |
|---|---|---|
| `recall-contracts/src/lib.rs` | `aidens-contracts`, semantic owners | Split shared primitives from semantic artifact owners |
| `recall-session/src/config.rs` | `aidens-config`, `aidens-capability-kit` | Separate config from runtime truth |
| `recall-session/src/provider.rs` | `aidens-provider-kit` | Preserve providers; fix unknown native default |
| `recall-session/src/provider_bridge.rs` | `aidens-provider-kit` | Pipeline provider bridge |
| `deps/llm-pipeline/src/tool_loop.rs` | `aidens-provider-kit`, `aidens-runner` | Native tool loop happy path |
| `deps/llm-output-parser` | `aidens-boundary-kit` | Strict/repair parser boundary |
| `llm-tool-runtime` | `aidens-tool-kit`, `aidens-receipts` | Tool registry/runtime/receipts |
| `recall-session/src/tool_catalog.rs` | `aidens-tool-kit`, `aidens-capability-kit` | Tool descriptors/exposure/status |
| `recall-session/src/session/tool_dispatch.rs` | `aidens-runner`, `aidens-boundary-kit`, `aidens-tool-kit` | Split native loop, parser fallback, execution |
| `recall-session/src/session/arbiter.rs` | `aidens-arbiter-kit` | Route law |
| `recall-session/src/session/arbiter_fast_signals.rs` | `aidens-arbiter-kit` | Fast route signals |
| `recall-session/src/session/arbiter_intents.rs` | `aidens-arbiter-kit` | Intent scoring |
| `recall-session/src/control.rs` | `aidens-receipts`, `aidens-budget-kit`, `aidens-governance-kit` | Control artifacts and receipts |
| `recall-session/src/approval.rs` | `aidens-permit-kit`, `aidens-security-kit` | Approval and auto-approval policy |
| `recall-session/src/path_safety.rs` | `aidens-security-kit`, `aidens-boundary-kit` | Home/path/symlink policy |
| `recall-session/src/scope_governance.rs` | `aidens-governance-kit` | Scope decisions |
| `recall-session/src/governance.rs` | `aidens-governance-kit` | Query governance |
| `recall-session/src/memory_policy.rs` | `aidens-memory-kit`, `aidens-receipts` | Memory write receipts/dedupe |
| `recall-session/src/session/memory.rs` | `aidens-memory-kit` | Memory write execution |
| `recall-session/src/search_core.rs` | `aidens-memory-kit` | Search abstraction |
| `recall-session/src/graph_query.rs` | `aidens-kernel-kit`, `aidens-arbiter-kit` | Graph/query routing split |
| `recall-session/src/scheduler.rs` | `aidens-schedule-kit`, `aidens-plan-kit`, `aidens-permit-kit`, `aidens-repair-kit` | Very overloaded; split carefully |
| `recall-session/src/scheduler_store.rs` | `aidens-schedule-kit`, `aidens-queue-kit` | Durable schedule/queue state |
| `recall-session/src/scheduler_migration.rs` | `aidens-config`, `aidens-schedule-kit` | Migration logic |
| `recall-session/src/jobs.rs` | `aidens-queue-kit`, `aidens-runner` | Job execution and future actions |
| `recall-daemon/src/core.rs` | `aidens-daemon-kit` | Daemon runtime ownership |
| `recall-daemon/src/ipc/*` | `aidens-daemon-kit`, `aidens-cli`, `aidens-tauri-kit` | IPC shell adapters |
| `recall-daemon/src/scheduler/*` | `aidens-queue-kit`, `aidens-schedule-kit`, `aidens-daemon-kit` | Runtime loops |
| `recall-daemon/src/host_wake.rs` | `aidens-wake-kit` | Host wake adapter only |
| `recall-app/src/daemon_client.rs` | `aidens-tauri-kit`, `aidens-daemon-kit` | IPC client and event bridge |
| `recall-app/src/state.rs` | `aidens-tauri-kit`, `aidens-app-kit` | UI state adapter, not runtime truth |
| `recall-app/src/commands.rs` | `aidens-tauri-kit` | Tauri commands |
| `recall-cli` | `aidens-cli` | Scaffolding/doctor/check commands |
| `recall-embedder` | `aidens-memory-kit` | Optional embedder adapter |
| `recall-ingest` | `aidens-memory-kit`, app-specific ingestion profile | Keep domain ingestion separate from app kit |
| `recall-web` | `aidens-tool-kit`, `aidens-security-kit` | Network tool bundle, gated by policy |

## Extraction priority from Recall

1. `recall-contracts` → contracts/capability/receipt base.
2. `provider.rs` + `provider_bridge.rs` → provider kit.
3. `tool_catalog.rs` + `llm-tool-runtime` → tool kit.
4. `tool_dispatch.rs` native loop integration → runner.
5. `approval.rs` + path safety → permit/security.
6. `config.rs` → config/capability.
7. `control.rs` → receipts/budget/governance.
8. `scheduler.rs`/`jobs.rs`/daemon loops → schedule/queue/daemon.
9. app/tauri → shell adapter.
10. graph/memory/kernel → advanced adapters.
