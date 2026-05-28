# Crate Rewrite Map

Do not preserve the current 31-crate layout by inertia. The target shape is
plane-oriented and adapter-first.

| Target crate | Owns | Must not own | Notes |
|---|---|---|---|
| `aidens` | top-level facade/prelude | truth semantics | Re-export app/adapters only. |
| `aidens-core` | app-only orchestration traits and run/session abstractions | IDs, evidence, memory, verification, kernel law | No stack truth. |
| `aidens-config` | config/profile loading and app settings | canonical semantics | Config may point to stack components, not define them. |
| `aidens-cli` | CLI UX and operator commands | direct local memory/evidence writes | Current command inventory is useful. Route through adapters. |
| `aidens-runner` | turn/session orchestration | canonical receipts or provider truth | Emits/carries canonical receipts through adapters. |
| `aidens-daemon` | process lifecycle after gates | domain truth/control truth | Block until phase 7. |
| `aidens-provider-adapter` | provider transport/rendering glue | tool/evidence receipts | Use `llm-tool-runtime` provider surfaces. |
| `aidens-tool-adapter` | tool registry/dispatch glue | local tool receipt truth | Use `llm-tool-runtime::ToolRuntime`. |
| `aidens-memory-adapter` | Forge/bridge/memory/runtime integration | memory truth | Must call `semantic-memory-forge`, `forge-memory-bridge`, `semantic-memory`, `knowledge-runtime`. |
| `aidens-governance-adapter` | verification/policy/adjudication/delegation integration | promotion law | Must call verification crates. |
| `aidens-kernel-adapter` | kernel facade | solver/convergence/oracle truth | Must call canonical kernel crates; phase 8 only. |
| `aidens-compat` | forbidden compatibility escape hatch | all behavior | Do not create this crate; remove local duplicate DTOs instead. |
| `aidens-testkit` | fixtures and cross-crate proof tests | production behavior | Expand canonical proof tests. |

## Mapping from current crates

See `CURRENT_AIDENS_SURFACE_MAP.md`. Current `aidens-contracts`, `aidens-memory-kit`, `aidens-receipts`, `aidens-repair-kit`, and local governance/kernel surfaces are the priority rewrite targets.
