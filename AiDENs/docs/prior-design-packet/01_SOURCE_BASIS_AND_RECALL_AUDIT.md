# 01 — Source Basis and Current Recall Audit

## Archives inspected

The design basis includes the current uploaded Recall archive plus bundled libraries and research:

```text
/mnt/data/aidens_src/recall
/mnt/data/aidens_src/lib1
/mnt/data/aidens_src/lib2
```

The inspection mode was static source review. This packet is a design/extraction baseline, not a green-build claim.

## Current Recall workspace

Root workspace members:

```text
recall-embedder
recall-ingest
recall-session
recall-cli
recall-app
recall-web
recall-daemon
recall-ipc
recall-contracts
```

Important bundled dependencies in current Recall:

```text
agent-graph
llm-pipeline
llm-output-parser
llm-tool-runtime
job-queue
knowledge-runtime
semantic-memory
semantic-memory-forge
forge-memory-bridge
profile-runtime
verification-control
verification-policy
verification-calibration
verification-adjudication
forge-pilot
stack-ids
```

## Static size snapshot

Approximate Rust source inventory from the current Recall archive:

| Crate/path | Rust files | Rust LOC |
|---|---:|---:|
| `recall-session` | 82 | 39,360 |
| `recall-app` | 7 | 5,646 |
| `recall-daemon` | 15 | 5,400 |
| `recall-ingest` | 10 | 2,264 |
| `recall-cli` | 4 | 1,645 |
| `recall-web` | 11 | 1,427 |
| `recall-ipc` | 1 | 1,102 |
| `recall-contracts` | 2 | 992 |
| `recall-embedder` | 5 | 914 |
| `deps/llm-pipeline` | 30 | 11,023 |
| `deps/llm-output-parser` | 11 | 3,429 |
| `llm-tool-runtime` | 6 | 3,180 |
| `knowledge-runtime` | 26 | 5,870 |
| `semantic-memory` | 28 | 16,872 |
| `semantic-memory-forge` | 10 | 4,919 |
| `forge-memory-bridge` | 6 | 2,618 |
| `job-queue` | 8 | 3,543 |
| `agent-graph` | 24 | 4,725 |

## High-level current structure

### `recall-session`

This is the main source for extraction. It already contains many AiDENs-relevant seams:

```text
approval.rs
config.rs
control.rs
governance.rs
graph_query.rs
jobs.rs
memory_policy.rs
path_safety.rs
profile.rs
provider.rs
provider_bridge.rs
query_observer.rs
query_safety.rs
scheduler.rs
scheduler_migration.rs
scheduler_store.rs
scope_governance.rs
search_core.rs
session/*
tool_catalog.rs
tools/*
working_memory.rs
```

Key exported concepts:

- provider build/selection,
- runtime status/truth,
- path safety,
- approval handler and decisions,
- memory write policy,
- query observer/events,
- scope governance,
- query provenance,
- control receipts and ledger types,
- tool dispatch and parsing,
- scheduler/future-action state.

### `recall-contracts`

This is a strong seed for `aidens-contracts`, but should not remain a dumping ground. It currently defines many versioned types:

```text
RuntimeCapabilityTruthV1
RuntimeTruthV1
HostWakeBackendStatusV1
ApprovalMode
ApprovalDispositionV1
AutoApproveStatusV1
ApprovalEventV1
ExposedToolSetV1
ArbiterDecisionV1
AidensRunContextV1
ToolCapabilityStatusV1
ProviderDirectiveViewV1
ToolAttemptV1
QueueHopReceiptV1
RunClosureV1
OperatorActionReceiptV1
ToolCallSummaryV1
canonical memory write receipt/report
ToolDescriptorView
PlanSummaryView
RunView
FutureActionWhyChainV1
TriggerFireReceiptV1
PlanDetailView
RunDetailView
RepoManifestV1
ReportBundleV1
RuntimeStatusResultV1
QueryReceiptV2
QueryResultEnvelopeV1
ApplyConfigResultV1
```

These types prove the right artifact direction already exists. AiDENs should refine ownership and compatibility governance.

### `recall-daemon`

Extraction source for:

- daemon lifecycle,
- IPC framing,
- subscribed event connections,
- due-action loop,
- future action execution,
- monitor/reminder/reembed loops,
- host wake bindings,
- daemon-side approval forwarding.

### `recall-app`

Extraction source for:

- Tauri command adapters,
- daemon client,
- UI event bridge,
- approval event display,
- status projection.

It should **not** own runtime truth.

### `llm-pipeline`

Important existing asset. It already contains `ToolLoopRunner` with native tool paths:

```text
run_openai_responses
run_openai_chat
run_anthropic
run_ollama
run_ollama_stream
```

AiDENs should make this the canonical happy path where supported.

### `llm-tool-runtime`

Important existing asset. It already owns:

- `ToolRegistry`,
- `ToolDescriptor`,
- exposure planning,
- `ToolRuntime`,
- approval state,
- `ToolReceipt`,
- `ToolReceiptSink`,
- starter tools.

AiDENs should build app ergonomics around this, not replace it.

## Current Recall extraction reading

Current Recall is not a clean reusable app kit. It is a feature-rich unfinished app that already contains the right ideas but concentrates them in a few large modules.

AiDENs should extract:

- contracts,
- runtime truth,
- app plan/profile expansion,
- provider mode truth,
- tool exposure planning,
- approval/permit safety,
- receipt ledger behavior,
- config validation,
- scheduler/queue law,
- daemon/UI shell adapters.

AiDENs should not extract:

- app-specific UI behavior as core law,
- Recall-specific tool set as default global tools,
- `RecallSession` as public API,
- parser fallback as happy path,
- daemon-local socket behavior as universal semantics,
- scheduler host wake behavior as canonical schedule truth.
