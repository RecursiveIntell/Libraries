# 03 — Crate Boundary Map

## Foundation crates

### `aidens-contracts`

**Extract from:** `recall-contracts`, selected `verification-*`, `stack-ids` integration.

Owns:

```text
schema traits
shared ID wrappers
artifact family markers
stable V1/V2 wire contracts
schema generation helpers
compatibility metadata
```

Must not own:

```text
provider construction
runtime execution
memory store access
tool invocation
UI or daemon behavior
```

### `aidens-boundary-kit`

**Extract from:** `llm-output-parser`, `recall-session/src/session/tool_dispatch.rs` parser/repair logic, `path_safety.rs`, structured patch handling.

Owns:

```text
StrictJsonParser
SchemaGate
CanonicalJsonHasher
StructuredOutputRepair
BoundaryRepairReceipt
PatchGate
PathGate
BoundaryDialect
BoundaryFailure
```

Must not own app policy. It validates and records boundary behavior; policy crates decide whether to proceed.

### `aidens-config`

**Extract from:** `recall-session/src/config.rs`, config parts of `recall-app`, `recall-daemon` apply/reload logic.

Owns:

```text
AiDENsConfigV1
ConfigLoadResult
ConfigApplyPlan
ConfigApplyReceipt
ConfigGeneration
SecretRedactor
ConfigMigration
```

Must not instantiate providers or runtime handles.

### `aidens-receipts`

**Extract from:** `recall-contracts` receipt types, `recall-session/src/control.rs`, tool receipt adapters, daemon scheduler receipts.

Owns:

```text
ReceiptLedger
RunReportV1
ProviderRouteReceiptV1
ToolAttemptReceiptV1
ApprovalReceiptV1
QueueHopReceiptV1
ScheduleReceiptV1
BoundaryRepairReceiptV1
ConfigApplyReceiptV1
ReceiptQuery
```

Must not become domain truth.

### `aidens-capability-kit`

**Extract from:** `RuntimeCapabilityTruthV1`, `RuntimeTruthV1`, `build_runtime_status`, `tool_capability_statuses`, web/provider/scheduler status fields.

Owns:

```text
RuntimeCapabilityTruthV1
RuntimeTruthV1
ProviderCapabilityTruth
ToolCapabilityTruth
MemoryCapabilityTruth
DaemonCapabilityTruth
SchedulerCapabilityTruth
CapabilityDoctor
```

Must distinguish:

```text
configured
available
healthy
registered
exposed
executable
attempted
succeeded
failed
degraded
fallback_only
disabled
blocked_by_policy
requires_approval
```

## Capability adapter crates

### `aidens-provider-kit`

**Extract from:** `recall-session/src/provider.rs`, `provider_bridge.rs`, `deps/llm-pipeline`.

Owns:

```text
ProviderKind
ProviderConfig
ProviderStack
ProviderFactory
ProviderHealth
ProviderRouteTruth
ToolExecutionModeResolver
NativeToolMode
```

Fix current risk: unknown native provider kinds must not default to a misleading native OpenAI chat route. Unknown native support should be `UnknownNativeUnsupported` or `RequiresExplicitMapping`.

### `aidens-tool-kit`

**Extract from:** `llm-tool-runtime`, `recall-session/src/tool_catalog.rs`, generic parts of `recall-session/src/tools/*`.

Owns:

```text
ToolBundle
ToolRegistryBuilder
ToolExposurePlanner
ToolIdentity
ToolDescriptorView
ToolRiskClass
ToolInstallReceipt
ToolCatalogDigest
```

Must not own app-specific Recall tools as global defaults.

### `aidens-security-kit`

**Extract from:** path safety, sandbox policy, dangerous capability classification.

Owns:

```text
SandboxPolicy
NetworkPolicy
FileSystemPolicy
ShellPolicy
DangerousCapabilityClass
CapabilityGate
```

### `aidens-memory-kit`

**Extract from:** `semantic-memory`, `semantic-memory-forge`, `forge-memory-bridge`, `knowledge-runtime`, Recall memory policy wrappers.

Owns:

```text
MemoryRuntimeBuilder
MemoryMode
MemoryScopePolicy
ProjectionHealth
RuntimeQueryProvenance adapter
MemoryWritePolicy adapter
```

Must not own raw truth promotion beyond existing authoritative crates.

### `aidens-kernel-kit`

**Extract from:** `agent-graph`, `constraint-compiler`, `recursive-kernel-core`, `kernel-execution`, `kernel-oracles`, `kernel-conformance`, current `graph_query.rs` patterns.

Owns:

```text
KernelAdapter
RegionRuntimeAdapter
CompiledGraphHandle
SnapshotInput
OracleSliceRequest
KernelRunReceipt
```

Must enforce right-graph law: storage graph, retrieval graph, inference graph, repair graph, and control graph are not the same object.

### `aidens-queue-kit`

**Extract from:** `job-queue`, Recall future-action enqueue/lease logic.

Owns:

```text
DurableQueueAdapter
JobLease
AttemptFamily
QueueHopReceipt
CancellationLink
FutureActionExecutionEnvelope
```

Must not own schedule semantics; that belongs to `aidens-schedule-kit`.

## Control crates

### `aidens-arbiter-kit`

**Extract from:** `session/arbiter.rs`, `arbiter_fast_signals.rs`, `arbiter_intents.rs`, route portions of `graph_query.rs`.

Owns:

```text
ArbiterDecisionV1
RouteCandidate
FallbackLadder
NoToolRoute
NativeToolRoute
ParserFallbackRoute
BlockedRoute
```

### `aidens-permit-kit`

**Extract from:** `approval.rs`, scheduler permit types, tool approval policy wiring.

Owns:

```text
ApprovalRequest
ApprovalDecision
ApprovalGrant
FutureActionPermitV1/V2
PermitAttenuation
PermitLedger
```

### `aidens-budget-kit`

**Extract from:** max tool rounds, retry policy, queue cooldown, control budget debits.

Owns:

```text
BudgetGovernor
StopRule
RetryPolicy
FanoutLimit
CooldownPolicy
BudgetDebit adapter
```

### `aidens-governance-kit`

**Extract from:** `governance.rs`, `scope_governance.rs`, `verification-*` adapters.

Owns:

```text
canonical side-effect class
canonical verification-control plans
ScopeDecision
GovernanceDecision
PromotionPolicy
DowngradePolicy
```

### `aidens-schedule-kit`

**Extract from:** `scheduler.rs` trigger/plan/time structures.

Owns canonical schedule law:

```text
TriggerSpecV1
MisfirePolicy
OverlapPolicy
TimezonePolicy
TriggerFireReceiptV1
NextFireCalculator
```

### `aidens-delegation-kit`

**Extract from:** plan child-role/delegation patterns, future action permits.

Owns:

```text
DelegationContractV1
DelegationDepthLimit
ChildAgentAuthority
MergeContract
```

### `aidens-plan-kit`

**Extract from:** `PlanBundleV1`, `PlanRevisionV1`, plan node/edge structures.

Owns:

```text
PlanBundleV1
PlanRevisionV1
PlanNodeV1
PlanEdgeV1
PlanSupersession
PlanRuntimeState
```

### `aidens-repair-kit`

**Extract from:** control `RepairRecord`, scheduler recovery, kernel syndrome/repair future direction.

Owns:

```text
canonical repair records
RetryVsRepairDecision
QuarantineDecision
CompensationPlan
SupportCore
RemovalFrontier
```

## Composition and shell crates

### `aidens-runner`

Owns one run. It composes provider, tool, security, permit, arbiter, budget, memory, receipts.

Must not own:

```text
app lifecycle
daemon lifecycle
Tauri events
long-term memory truth
canonical schedule truth
```

### `aidens-app-kit`

Owns app builder, profile expansion, `AiDENsAppPlanV1`, project layout, safe defaults.

### `aidens-cli`

Owns `aidens new`, `doctor`, `check-config`, `list-tools`, `provider-check`, `receipts inspect`.

### `aidens-daemon-kit`

Owns process/IPC lifecycle. It does not own queue or schedule law.

### `aidens-tauri-kit`

Owns UI commands/events. It does not own runtime truth or approval truth.

### `aidens-testkit`

Owns reference fixtures and conformance assertions.
