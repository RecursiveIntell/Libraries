# 01 — Source and Extraction Map

## Source root

```text
~/Coding/Recall
```

This path is read-only extraction input.

## Destination root

```text
~/Coding/Libraries/AiDENs
```

Codex should create or modify files only under this path unless the user explicitly says otherwise.

## Source areas to inspect

| Recall path | What to extract | What not to preserve |
|---|---|---|
| `recall-session/src/provider.rs` | provider capability types, execution mode resolution, retry summary ideas | app-specific provider assumptions, unverified native route defaults |
| `recall-session/src/provider_bridge.rs` | `llm-pipeline` bridge concepts, multi-turn preservation tests | any flattening/legacy prompt behavior |
| `recall-session/src/session/tool_dispatch.rs` | exposure planning concepts, native vs fallback split, approval filtering | `RecallSession` methods as public API |
| `recall-session/src/tool_catalog.rs` | registry/bundle lessons | Recall-specific tool catalog as generic catalog |
| `recall-session/src/approval.rs` | approval request/decision concepts | UI/daemon-specific callbacks |
| `recall-session/src/control.rs` | control-plane receipt and lineage lessons | broad control god-module behavior |
| `recall-session/src/session/arbiter*.rs` | route decision tests and signal categories | substring routing as law |
| `recall-session/src/config.rs` | config model, redaction, defaults | half-applied runtime settings |
| `recall-session/src/path_safety.rs` | path canonicalization/sandbox ideas | model-provided raw paths |
| `recall-session/src/scheduler*.rs` | trigger/misfire/overlap bug lessons | scheduler truth inside daemon or queue |
| `recall-session/tests/*` | conformance tests and bug regressions | tests that assume Recall-specific UI/runtime names |
| `recall-contracts/src/lib.rs` | existing contract generation and schema conventions | dumping all AiDENs contracts into one blob |
| `deps/llm-pipeline` | provider-native tool loop integration | text-first tool path as happy path |
| `_vendor/Libraries/llm-tool-runtime` | registry, descriptors, runtime receipts | Recall-specific tool names |
| `_vendor/Libraries/knowledge-runtime` | runtime view/provenance ideas | domain truth persistence from runner |
| `_vendor/Libraries/semantic-memory*` | memory/forge/bridge adapters | memory truth inside AiDENs runner |
| `_vendor/Libraries/job-queue` | durable jobs, leases, attempt lineage | host scheduler semantics |
| `_vendor/Libraries/agent-graph` | graph/kernel adapters | collapsing all graphs into one |

## Extraction stance

Copying source line-for-line is usually wrong. Extract the failure-boundary concept and re-express it in the appropriate AiDENs crate.

Example:

- `ToolExecutionMode` concept belongs in `aidens-provider-kit` and receipt types.
- Tool exposure planning belongs in `aidens-tool-kit`.
- Approval grants belong in `aidens-permit-kit`.
- Runtime execution belongs in `aidens-runner`.
- App profiles belong in `aidens-app-kit` or profile crates.
