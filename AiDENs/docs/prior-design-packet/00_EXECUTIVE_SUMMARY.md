# 00 — Executive Summary

## Correction from prior direction

The extraction source is **current Recall**, not Recall-Coding. Recall-Coding can remain useful as an anti-pattern reference, but AiDENs should be extracted from the actual Recall workspace:

```text
recall-app
recall-cli
recall-contracts
recall-daemon
recall-embedder
recall-ingest
recall-ipc
recall-session
recall-web
```

and the current bundled libraries:

```text
llm-pipeline
llm-output-parser
llm-tool-runtime
knowledge-runtime
semantic-memory
semantic-memory-forge
forge-memory-bridge
profile-runtime
job-queue
agent-graph
verification-*
forge-pilot
stack-ids
```

## What AiDENs should become

AiDENs should be the **main creation layer** for anything built on these libraries:

- desktop apps,
- CLI apps,
- daemon apps,
- coding agents,
- memory-backed assistants,
- research workbenches,
- scheduled/autonomous agents,
- graph/kernel/proof-governed runtimes,
- future federated/mechanism runtimes.

But AiDENs itself should not be one giant crate. It should be a public umbrella over a crate family divided by **failure boundary**.

## Why not one crate

Recall already shows why one crate is dangerous. `recall-session` is useful, but it is also overloaded. It owns or wires:

- provider setup,
- embedder setup,
- memory store,
- knowledge runtime,
- ingest pipeline,
- Forge store,
- causal graph,
- compiled graph state,
- profile store,
- bundle store,
- reranker,
- tool registry,
- tool runtime,
- native/parser tool dispatch,
- approval policy,
- scheduler integration,
- memory write policy,
- query provenance,
- control receipts,
- runtime truth.

That is not wrong for an unfinished app. It is wrong as the reusable public design.

## Main design law

AiDENs must keep these domains separate:

```text
contracts       = what artifacts mean
boundary        = what inputs are allowed to become artifacts
config          = what the app intends to run
receipts        = what actually happened
capability      = what is actually possible now
provider        = how models are reached
tools           = what the model may call
security/permits= what side effects are allowed
arbiter         = what route should run
budget          = how execution is bounded
memory          = what projected truth is available
kernel          = what inference graph executes
queue/schedule  = what runs later and why
daemon/ui       = host lifecycle and presentation only
runner          = one run/turn coordinator
app-kit         = easy builder and profiles
```

## The essential abstraction

Profiles must expand into an explicit plan before they run:

```rust
let plan = AiDENsProfile::CodingAgent.expand();
plan.validate()?;
let app = AiDENsApp::from_plan(plan).build().await?;
```

The plan is the user's chance to see and override:

- providers,
- tools,
- memory,
- approval policy,
- daemon/queue/schedule behavior,
- receipt level,
- dangerous capabilities,
- local-only/network posture,
- conformance expectations.

## The most important footguns to prevent

1. Parser-fallback pretending to be native tool execution.
2. Disabled tools still registered or invocable.
3. UI becoming approval truth.
4. Daemon and local runtime split-brain.
5. Queue retries becoming duplicate jobs.
6. Host wake mechanisms becoming schedule truth.
7. Profiles silently enabling shell, write, web, queue, or auto-approval.
8. Runtime becoming a shadow memory database.
9. Bridge or adapter code becoming a policy engine.
10. Structured-output repair changing treatment without receipt.
11. Memory/reranker returning temporally invalid evidence.
12. Graph/kernel execution reading moving memory state.
13. Config applying midway through a run without generation pinning.
14. Tool name collisions across bundles.
15. Observability traces being treated as authoritative receipts.

## Build strategy

Do not build every crate at once.

Start with the hard foundation:

```text
aidens-contracts
aidens-boundary-kit
aidens-config
aidens-receipts
aidens-capability-kit
aidens-testkit
```

Then build the usable agent core:

```text
aidens-provider-kit
aidens-tool-kit
aidens-security-kit
aidens-permit-kit
aidens-arbiter-kit
aidens-budget-kit
aidens-runner
```

Then build the speed layer:

```text
aidens-app-kit
aidens-cli
aidens-profile-coding
```

Only after that should memory, daemon, schedule, queue, and kernel capabilities be extracted.
