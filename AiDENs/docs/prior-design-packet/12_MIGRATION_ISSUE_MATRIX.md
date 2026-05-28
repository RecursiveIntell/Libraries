# 12 — Migration Issue Matrix

| ID | Work item | Source | Target | Acceptance |
|---|---|---|---|---|
| AIDENS-001 | Create workspace and root umbrella crate | new | `aidens` | root crate re-exports only safe prelude |
| AIDENS-002 | Extract shared contract primitives | `recall-contracts` | `aidens-contracts` | schemas generate and meta-validate |
| AIDENS-003 | Move runtime/capability truth ownership | `recall-contracts`, `config.rs` | `aidens-capability-kit` | disabled/exposed/executable states distinct |
| AIDENS-004 | Extract strict boundary parser/repair | `llm-output-parser`, `tool_dispatch.rs` | `aidens-boundary-kit` | repaired output emits receipt |
| AIDENS-005 | Extract config model and atomic apply | `config.rs` | `aidens-config` | config generation pinned per run |
| AIDENS-006 | Extract receipt ledger | `control.rs`, `QueryReceiptV2`, tool receipts | `aidens-receipts` | every run path emits receipt |
| AIDENS-007 | Extract provider factory | `provider.rs`, `provider_bridge.rs` | `aidens-provider-kit` | unknown native provider rejected |
| AIDENS-008 | Wrap `ToolLoopRunner` | `llm-pipeline` | `aidens-provider-kit`/`aidens-runner` | native tool path is canonical |
| AIDENS-009 | Extract tool registry/exposure planner | `tool_catalog.rs`, `llm-tool-runtime` | `aidens-tool-kit` | disabled tools absent |
| AIDENS-010 | Extract sandbox/path capability policy | `path_safety.rs`, tool descriptors | `aidens-security-kit` | path traversal/symlink escape tests pass |
| AIDENS-011 | Extract approval/permit policy | `approval.rs`, scheduler permits | `aidens-permit-kit` | UI does not own approval truth |
| AIDENS-012 | Extract route arbiter | `arbiter*.rs`, `graph_query.rs` | `aidens-arbiter-kit` | no-tool/native/parser routes first-class |
| AIDENS-013 | Extract budget/stop/retry policy | `control.rs`, scheduler/job retry | `aidens-budget-kit` | retry storms blocked |
| AIDENS-014 | Extract governance/risk/scope | `governance.rs`, `scope_governance.rs` | `aidens-governance-kit` | risk-bearing outputs require plan |
| AIDENS-015 | Build one-run coordinator | `session/mod.rs`, `tool_dispatch.rs` | `aidens-runner` | no daemon/tauri dependency |
| AIDENS-016 | Define app plan/profile expansion | new + Recall profile config | `aidens-app-kit` | profile expands to visible plan |
| AIDENS-017 | Build CLI scaffolding | `recall-cli` patterns | `aidens-cli` | `aidens new coding-agent` works |
| AIDENS-018 | Extract memory adapter | `session/memory.rs`, memory libs | `aidens-memory-kit` | memory disabled/optional/required works |
| AIDENS-019 | Extract queue lease/attempt law | `jobs.rs`, `job-queue` | `aidens-queue-kit` | stale lease skipped |
| AIDENS-020 | Extract schedule law | `scheduler.rs` | `aidens-schedule-kit` | DST/misfire/overlap tests pass |
| AIDENS-021 | Extract host wake adapters | `host_wake.rs` | `aidens-wake-kit` | wake is projection only |
| AIDENS-022 | Extract daemon lifecycle | `recall-daemon` | `aidens-daemon-kit` | daemon owns runtime in daemon mode |
| AIDENS-023 | Extract Tauri shell | `recall-app` | `aidens-tauri-kit` | UI reads truth and displays approvals only |
| AIDENS-024 | Extract graph/kernel adapter | `graph_query.rs`, kernel libs | `aidens-kernel-kit` | graph inputs snapshot pinned |
| AIDENS-025 | Extract plan model | `scheduler.rs` plan types | `aidens-plan-kit` | plan revisions supersede cleanly |
| AIDENS-026 | Extract repair/quarantine model | `control.rs`, future repair logic | `aidens-repair-kit` | repair never self-promotes |
| AIDENS-027 | Build testkit | tests + new fixtures | `aidens-testkit` | generated app tests included |
| AIDENS-028 | Convert Recall app to AiDENs | all Recall app crates | Recall as AiDENs app | Recall no longer owns generic wiring |

## Prioritization

### P0

```text
AIDENS-001 through AIDENS-017
```

These make the easy app path real.

### P1

```text
AIDENS-018 through AIDENS-023
```

These make Recall itself cleanly portable.

### P2

```text
AIDENS-024 through AIDENS-028
```

These complete advanced runtime extraction and conversion.
