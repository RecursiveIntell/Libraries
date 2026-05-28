# 02 — Crate Boundary Map

## Public facade

```text
aidens
```

Re-exports safe app-building APIs and common prelude. It must not own deep semantics.

## Foundation crates

| Crate | Owns | Must not own |
|---|---|---|
| `aidens-contracts` | versioned base artifacts, IDs, schema helpers | runtime behavior |
| `aidens-boundary-kit` | strict JSON, schema validation, canonicalization, repair receipts, patch gates | app policy |
| `aidens-config` | config load/migrate/redact/atomic apply | provider construction |
| `aidens-receipts` | append-only execution/control receipt ledger | domain truth |
| `aidens-capability-kit` | runtime capability/status truth | routing decisions |

## Capability crates

| Crate | Owns | Must not own |
|---|---|---|
| `aidens-provider-kit` | provider construction, route truth, native/fallback mode | tool registry |
| `aidens-tool-kit` | tool descriptors, registry, exposure planner | approval truth |
| `aidens-security-kit` | capability risk classes, sandbox posture | route planning |
| `aidens-memory-kit` | memory/forge/knowledge adapters | domain truth invention |
| `aidens-kernel-kit` | graph/kernel/oracle adapters | durable truth |
| `aidens-queue-kit` | durable jobs, leases, attempts, cancellation | schedule truth |

## Control crates

| Crate | Owns | Must not own |
|---|---|---|
| `aidens-arbiter-kit` | route decisions and fallback ladders | provider construction |
| `aidens-permit-kit` | approvals, grants, inherited authority | UI prompts |
| `aidens-budget-kit` | stop rules, retry/fanout/deadline ceilings | provider retry internals |
| `aidens-governance-kit` | verification plans, risk classes, downgrade law | raw evidence truth |
| `aidens-schedule-kit` | recurrence, triggers, misfire/overlap law | queue execution |
| `aidens-delegation-kit` | bounded sub-agent contracts | permission expansion |
| `aidens-plan-kit` | app/agent plan revisions and supersession | execution |
| `aidens-repair-kit` | retry vs repair vs quarantine records | hidden mutation |

## Composition and shells

| Crate | Owns | Must not own |
|---|---|---|
| `aidens-runner` | turn/run execution coordination | memory truth, daemon lifecycle |
| `aidens-app-kit` | app builder, profiles, templates, doctor integration | canonical law |
| `aidens-cli` | `new`, `doctor`, `check-config`, `list-tools`, `provider-check` | runtime law |
| `aidens-daemon-kit` | socket/IPC/service lifecycle | queue/schedule semantics |
| `aidens-tauri-kit` | UI event/command adapters | approval truth |
| `aidens-testkit` | conformance fixtures/reference checks | production behavior |

## Dependency rules

```text
aidens-contracts
  ↑
aidens-boundary-kit
  ↑
aidens-config
  ↑
aidens-receipts
  ↑
aidens-capability-kit
  ↑
provider/tool/security/memory/kernel/queue adapters
  ↑
arbiter/permit/budget/governance/schedule/delegation/plan/repair
  ↑
aidens-runner
  ↑
aidens-app-kit
  ↑
aidens / aidens-cli / aidens-daemon-kit / aidens-tauri-kit
```

CI should eventually check this with a dependency-boundary script.
