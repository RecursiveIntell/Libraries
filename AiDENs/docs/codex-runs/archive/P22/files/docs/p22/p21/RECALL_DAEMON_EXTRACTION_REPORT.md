# Recall Daemon Extraction Report

## Source Basis

Inspected read-only sources:

- `/home/sikmindz/Coding/Recall/recall-daemon/src/core.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/scheduler.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/scheduler_store.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/config.rs`
- `/home/sikmindz/Coding/Recall/recall-session/src/approval.rs`
- `/home/sikmindz/Coding/Recall/recall-daemon/tests/*`

Recall was treated as a pattern source only. No Recall crate, session type, scheduler store, DB file, socket path, Tauri command, UI event model, or memory representation was imported into AiDENs.

## Reusable Patterns

### Config Startup Truth

Useful pattern:

- config load status must distinguish loaded, created default, and invalid;
- invalid config is blocking and should not silently fall back to defaults;
- secrets are redacted before display;
- runtime status should show provider, tools, receipts, memory mode, and blocked/degraded state.

AiDENs mapping:

- `check-config`, `doctor`, `status`, `provider-check`, `tools inspect`, and `receipts` expose operator-facing runtime truth;
- provider unsupported/unavailable states remain explicit and tested.

### Daemon Lifecycle And Heartbeat

Useful pattern:

- daemon heartbeat is an observable liveness signal;
- config reload must be atomic from the operator's perspective;
- approval delivery must survive disconnected clients;
- safe mode must be a runtime switch that changes queue admission.

AiDENs mapping:

- `aidens-daemon-kit` currently owns a safe P11 controller facade over queue/schedule/wake/leases/safe mode;
- it does not import Recall IPC or UI state;
- safe mode and drain operations emit queue/safe-mode reports.

### Queue, Schedule, And Wake

Useful pattern:

- schedules and wakes should enqueue durable jobs with idempotency keys;
- duplicate schedule/wake occurrences must suppress storms and emit suppression receipts;
- leases prevent repeated execution;
- completed/cancelled jobs must survive restart;
- risky work must be blockable by safe mode.

AiDENs mapping:

- `aidens-queue-kit`, `aidens-schedule-kit`, `aidens-wake-kit`, and `aidens-daemon-kit` already prove namespace isolation, duplicate suppression, restart persistence, leases, cancellation, safe mode, and drain behavior;
- the Phase 07 daemon template documents those supported surfaces without claiming full Recall daemon parity.

## Quarantined Recall Assumptions

These were intentionally not extracted:

- Recall scheduler SQLite schema or file names;
- Recall IPC socket paths and event frames;
- Recall Tauri state bridge;
- Recall web cache and app shell state;
- Recall memory/search model;
- Recall `RecallSession` as an AiDENs public API;
- Recall host wake wrapper scripts as AiDENs daemon truth;
- app-specific future-action payload formats as canonical schemas.

## Extracted AiDENs Artifacts

- `examples/configs/daemon-safe.toml`
- `examples/templates/daemon-safe-operator.template.md`
- `crates/aidens-integration-tests/tests/phase_07_recall_extraction.rs`

## Residual Gaps

- AiDENs does not yet expose a long-running desktop daemon shell.
- AiDENs does not yet expose heartbeat/status over IPC.
- AiDENs does not yet implement Recall-style approval replay hubs.
- AiDENs daemon support remains a safe queue/schedule/wake controller proof, not a full Recall daemon replacement.
