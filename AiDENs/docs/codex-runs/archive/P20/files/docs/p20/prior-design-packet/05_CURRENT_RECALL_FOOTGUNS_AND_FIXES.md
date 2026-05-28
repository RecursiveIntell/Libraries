# 05 — Current Recall Footguns and AiDENs Fixes

This file treats current Recall as unfinished source material. Some items are already partially fixed in Recall; AiDENs should make the fix canonical and impossible to regress silently.

## 1. `RecallSession` god object

### Current condition

`RecallSession::build_with_embedder_and_scheduler` initializes or wires memory directories, memory store, knowledge runtime, ingest pipeline, Forge store, CEA graph, compiled graph state, profile store, bundle store, reranker, tool registry, tool runtime, provider execution mode, approval policy, scheduler references, and query behavior.

### Risk

Any app using this shape inherits all responsibilities at once. One refactor can change provider truth, memory truth, tool exposure, approval semantics, and receipt generation.

### AiDENs fix

Split by failure boundary:

```text
provider-kit
memory-kit
tool-kit
permit-kit
receipts
capability-kit
runner
app-kit
```

The runner coordinates one run only.

## 2. Native tool execution and parser fallback in same large path

### Current condition

Recall has both native tool loop support through `llm-pipeline::ToolLoopRunner` and parser-fallback extraction/repair logic. It also has mode forcing via config.

### Risk

Parser fallback can appear operationally equivalent to native tool calling. This is not true. Parser fallback is useful, but it is degraded execution.

### AiDENs fix

Provider route labels must be exact:

```text
native_openai_responses
native_openai_chat
native_ollama
native_anthropic
openai_compatible
parser_fallback
no_tools
disabled
unavailable
```

Fallback emits `ProviderRouteReceiptV1` and degraded flag.

## 3. Unknown provider native-mode default

### Current condition

`resolve_execution_mode` warns and defaults an unrecognized native provider to `NativeOpenAiChat`.

### Risk

A provider can be mislabeled and then fail strangely or expose wrong tool protocol.

### AiDENs fix

Unknown native provider must resolve to:

```text
NativeModeUnknownRequiresExplicitMapping
```

or parser fallback if explicitly configured.

## 4. Tool state ambiguity

### Current condition

Recall already has `RuntimeCapabilityTruthV1`, `ExposedToolSetV1`, tool descriptor views, denied tool entries, and tool capability statuses. That is good, but the logic is distributed.

### Risk

UI or app code can confuse registered tools, exposed tools, executable tools, attempted tools, and healthy tools.

### AiDENs fix

`aidens-capability-kit` and `aidens-tool-kit` must preserve separate states:

```text
registered
enabled
eligible
exposed_this_turn
executable_this_turn
attempted
succeeded
failed
blocked_by_policy
requires_approval
disabled
```

## 5. Disabled tool callable by accident

### Current condition

Recall has gating/config and descriptor-level exposure policy. However, app code can still accidentally register a tool and rely on later gating.

### Risk

Defense-in-depth becomes the only defense.

### AiDENs fix

Disabled means:

```text
not registered
not exposed
not invocable
not shown as ready
```

Invocation-time denial remains defense in depth.

## 6. UI approval truth

### Current condition

The Tauri app keeps pending approval senders and emits approval requests over UI events. It includes timeout logic and subscriber leak guards.

### Risk

UI closure, reload, or stale event subscription can accidentally affect authority semantics.

### AiDENs fix

`aidens-tauri-kit` displays approval prompts only. `aidens-permit-kit` owns approval truth and ledger. UI decisions are inputs to the permit system, not authoritative state.

## 7. Subscription leaks and event duplication

### Current condition

Recall app has explicit guards against leaking subscribed event connections on reload/HMR.

### Risk

Without canonical stream IDs and sequence numbers, UI can duplicate or lose event semantics.

### AiDENs fix

Every shell event includes:

```text
run_id
stream_id
sequence_no
event_kind
receipt_ref
```

Shell adapters must be idempotent.

## 8. Daemon/local split-brain

### Current condition

Recall has app, daemon, IPC client, and local state paths. The daemon can own scheduler/runtime behavior while UI is a client.

### Risk

GUI or CLI can accidentally start a local runtime instead of using daemon runtime, creating two truth/status surfaces.

### AiDENs fix

In daemon mode:

```text
daemon owns runtime authority
GUI/CLI are clients
local fallback requires explicit override
config apply RPCs the daemon
```

## 9. Scheduler and host wake conflation

### Current condition

Recall scheduler state includes future actions, triggers, host wake bindings, trigger fire receipts, permits, plans, revisions, leases, and future action state.

### Risk

Host wake backend behavior can become canonical schedule truth.

### AiDENs fix

Split:

```text
aidens-schedule-kit = canonical trigger law
aidens-queue-kit    = durable job/lease law
aidens-wake-kit     = host wake adapters only
aidens-daemon-kit   = process/IPC lifecycle
```

## 10. Queue retry and stale lease hazards

### Current condition

Recall has lease TTLs, stale lease checks, and job IDs derived from action/lease. Good direction.

### Risk

Duplicate jobs, stale jobs, or failover can still create multiple executions of the same logical action if not globally modeled.

### AiDENs fix

Every queued run requires:

```text
job_id
lease_id
attempt_family_id
attempt_id
trace_id
config_generation_id
parent_receipt_id
```

Provider failover creates child attempt, not a new logical job.

## 11. Boundary repair can mutate treatment

### Current condition

Recall uses `llm-output-parser` repair/extraction in parser fallback.

### Risk

A repaired JSON tool call or patch can change the effective treatment without durable evidence.

### AiDENs fix

All model structured output enters through `aidens-boundary-kit` and emits a repair receipt when repaired. Patch tools cannot consume raw model JSON.

## 12. Memory dedupe is retrieval-dependent

### Current condition

Recall comments correctly note that dedupe can depend on retrieved candidates.

### Risk

Duplicate or update decisions can be missed if retrieval does not surface the correct candidate.

### AiDENs fix

`aidens-memory-kit` should support both:

```text
content-hash exact checks
retrieval-assisted semantic checks
```

and mark the dedupe mode in the receipt.

## 13. Reranker can violate temporal semantics

### Current condition

Recall has temporal provenance and reranking. The safety law should be explicit.

### Risk

Semantic reranking could surface time-invalid evidence if temporal filtering is not enforced first.

### AiDENs fix

Temporal/scoped filter occurs before reranking. Any widening emits a `ViewDisclosureV1` degradation.

## 14. Config applies during active run

### Risk

A run can start with one provider/config and finish under another.

### AiDENs fix

Every run pins:

```text
config_generation_id
capability_truth_id
app_plan_id
```

Hot-swap is allowed only if explicitly modeled.

## 15. Tool name collision

### Risk

Multiple tool bundles can define `search`, `read_file`, `write`, etc.

### AiDENs fix

Canonical identity:

```text
tool_namespace
tool_name
tool_version
capability_class
risk_class
```

Display name is not identity.
