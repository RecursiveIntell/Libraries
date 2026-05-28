# 06 — Artifact and Contract Model

## Contract ownership

AiDENs must not let each crate invent its own receipt, status, approval, route, and config shapes.

`aidens-contracts` provides shared primitives and schema governance. Specific crates own their semantic artifacts.

Example:

| Artifact | Semantic owner |
|---|---|
| `RuntimeCapabilityTruthV1` | `aidens-capability-kit` |
| `RunReportV1` | `aidens-runner` / `aidens-contracts` app report |
| `BoundaryRepairReceiptV1` | `aidens-boundary-kit` + `aidens-receipts` |
| `ProviderRouteReceiptV1` | `aidens-provider-kit` + `aidens-receipts` |
| `ToolExposureSetV1` | `aidens-tool-kit` |
| `ArbiterDecisionV1` | `aidens-arbiter-kit` |
| `ApprovalGrantV1` | `aidens-permit-kit` |
| `TriggerSpecV1` | `aidens-schedule-kit` |
| `QueueHopReceiptV1` | `aidens-queue-kit` + `aidens-receipts` |
| `PlanBundleV1` | `aidens-plan-kit` |
| canonical repair records | `verification-control` / `semantic-memory-forge` via `aidens-repair-kit` adapter |
| `AiDENsAppPlanV1` | `aidens-app-kit` |

## Schema rules

1. Rust types are the source of truth.
2. All wire-visible artifacts derive `Serialize`, `Deserialize`, and `JsonSchema`.
3. Every schema is generated.
4. Every generated schema is meta-validated.
5. Breaking changes require a new artifact version.
6. Compatibility tests compare historical schemas.
7. No hand-edited schema file may be authoritative.
8. Unknown fields are rejected unless explicitly allowed for forward compatibility.
9. Canonicalization is defined for hashable artifacts.
10. Self-hash fields are avoided; content digests bind outside the content being hashed.

## Minimal artifact base

Every durable artifact should include:

```text
schema_version
artifact_id
created_at
created_by
trace_id or trace_ctx
config_generation_id, when runtime-derived
app_plan_id, when app-derived
parent_artifact_refs
content_digest, when applicable
degraded flag
notes/warnings
```

## Run report model

`RunReportV1` should contain or reference:

```text
run_id
trace_id
attempt_family_id
attempt_id
started_at
finished_at
app_plan_id
config_generation_id
capability_truth_id
provider_route_receipt
arbiter_decision
exposed_tool_set
tool_attempt_receipts
approval_receipts
boundary_repair_receipts
queue_hop_receipts
schedule_receipts
memory_write_receipts
view_disclosures
budget_debits
stop_reason
degraded
warnings
```

## App run context model

`AidensRunContextV1` should remain app-local run/config context and convert to canonical stack trace/attempt ids:

```text
execution_id
trace_id
attempt_family_id
attempt_id
trial_id optional
created_at
truth_id / capability_truth_id
config_generation_id
app_plan_id
route
provider_backend_kind
provider_model
caller_class
query_class
workload_class
deadline
budget_context
parent_execution_id
queue_lease_id optional
schedule_trigger_id optional
degraded
```

## Boundary repair model

When parser/repair changes raw output:

```text
BoundaryRepairReceiptV1
  repair_id
  raw_digest
  repaired_digest
  dialect
  repair_strategy
  changed_paths
  rejected_paths
  schema_id
  tool_or_artifact_target
  treatment_integrity_status
  created_at
```

Tools that mutate files or memory must receive the repaired artifact digest, not raw model text.

## Capability truth model

Capability truth should be snapshot-based and supersedable:

```text
RuntimeCapabilityTruthV1
  truth_id
  supersedes_truth_id
  observed_at
  app_id
  session_id
  config_generation_id
  runtime_mode
  initialized
  blocking
  provider_statuses
  tool_statuses
  memory_statuses
  daemon_status
  queue_status
  schedule_status
  receipt_ledger_status
  notes
```

## Bitemporal discipline

Truth-bearing memory/evidence artifacts must preserve valid time and recorded time separately.

Execution receipts use operational time but may reference bitemporal memory snapshots. A run over memory should record:

```text
valid_as_of
recorded_as_of
temporal_mode
memory_snapshot_id
view_policy_id
widening_disclosures
```

## Contract anti-patterns

Forbidden:

```text
free-form JSON as durable artifact
schema generated but not validated
runtime status inferred from UI state
receipt implied by logs
config change without generation ID
repair without changed-path evidence
profile expansion without visible AppPlan
```
