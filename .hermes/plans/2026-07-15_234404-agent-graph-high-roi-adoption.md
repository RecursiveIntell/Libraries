# Agent-Graph Highest-ROI Adoption Plan

> **For Hermes:** Use subagent-driven-development skill to implement this plan task-by-task.

**Goal:** Prove and then adopt `agent-graph` as the bounded, receipt-bearing decision layer for the Agent Evidence Workbench / Forge repair workflow—without turning the current MCP façade into an unsafe autonomous executor.

**Architecture:** Use the current Agent Graph MCP only as an advisory child of the native Hermes tool loop: Hermes gathers source/test/web receipts and performs all side effects; the graph receives a bounded JSON evidence packet and emits a schema-validated decision artifact. After an adversarial certification shows real value, add one narrow Forge `GraphAdvisor` adapter. Only then directly embed the Rust `agent-graph` core in Forge Workbench's single durable repair-run graph, where its native checkpoints, joins, interrupts, trace IDs, and per-node receipts are actually available.

**Tech Stack:** Rust (`agent-graph`, `llm-pipeline`, `llm-tool-runtime`, `job-queue`, Forge), JSON Schema, MCP/JSON-RPC, Ollama, Hermes, semantic-memory.

---

## Executive decision

### The absolute highest-ROI move

Build **one skeptical, single-run Agent Evidence Report** first:

```text
bounded coding/research run receipts
  -> normalize evidence packet
  -> extract atomic claims
  -> classify: supported | contradicted | insufficient | human_review
  -> check required proof / contradictions
  -> produce an advisory evidence report
  -> native Hermes decides whether to gather more evidence, delegate work, or request approval
```

Why this ranks first:

- It directly serves the strongest existing wedge: a local-first coding-agent flight recorder / evidence workbench, rather than another generic agent framework.
- It makes graph strengths useful immediately: ordered state transformation, explicit branches, schema contracts, and step receipts.
- It has no authority to write, apply patches, mutate semantic memory, or declare a release ready.
- It produces a useful report before a UI, full repair system, or broad MCP integration is built.

### Ranked portfolio

| Rank | Opportunity | Adopt when | Why / boundary |
|---|---|---|---|
| 1 | Single-run Agent Evidence Report | First | Highest leverage, lowest authority; validates whether graph structure improves claim/proof discipline. |
| 2 | Forge `verify -> explain -> approve` repair loop | After the report and adapter gates | Existing Forge design already defines its state, verification, approval, and evidence lifecycle. Use direct Rust core—not the MCP façade. |
| 3 | Memory/context governance gate | After evidence schema/replay semantics stabilize | Record retrieval, compaction, exact fallback, and loss warnings without creating a second memory authority. |
| 4 | Research/spec-to-verification handoff | After structured-output certification | Good repeatable use of dependent transforms; native Hermes still gathers sources and delegates implementation. |
| 5 | Intake/queue triage router | Only if it reduces operator decisions in measurement | Useful operational multiplier, not the flagship; current substring router is unsafe without a parent validator. |
| Deferred | Autonomous release gates, patch apply, memory writes, generic multi-agent runtime | Do not schedule from this plan | The current MCP façade can falsely succeed on empty output and silently END on route no-match. |

---

## Verified current context and non-negotiable constraints

1. The installed MCP server is enabled at `/home/sikmindz/.cargo/bin/agent-graph-mcp`, configured in `/home/sikmindz/.hermes/config.yaml` with `http://127.0.0.1:11434` and default model `glm-5.2:cloud`.
2. The exposed MCP surface is only `graph_create`, `graph_execute`, and `graph_status`; exposed node types are `llm`, `router`, and `passthrough`. It does **not** expose the core crate's checkpoint, interrupt/resume, join/reducer, durable execution, or tool-node capabilities.
3. Live probes proved:
   - a passthrough graph succeeds;
   - `glm-5.2:cloud` can produce an empty string while the graph reports `success: true`, in both text and JSON-mode probes;
   - `qwen2.5:0.5b` returned `READY` for a simple text probe, but it is not yet certified for structured proof work;
   - router no-match returns successful `END`, not an explicit error;
   - registry state is process-resident and currently contains temporary probe graphs.
4. The Rust core at `/home/sikmindz/Coding/Libraries/agent-graph` has the high-value primitives: JSON state, bounded parallelism, deterministic joins/reducers, retries, checkpoints, interrupt/resume, trace/attempt/trial lineage, events, and receipts. Its direct working tree has pre-existing edits; the installed MCP binary timestamp predates those source edits. Binary/source identity is therefore unproven.
5. A workspace-wide test attempt reached passing `agent-graph` suites but failed in unrelated `check-runner` timeout/environment/process-group tests. Treat package-level tests as evidence about the crate; do not claim workspace closure.
6. Forge Workbench already documents a one-graph repair lifecycle in `/home/sikmindz/Coding/forge-workbench/docs/03_agent_graph_and_state.md`; it also explicitly assigns retry/lifecycle ownership in `docs/06_queue_trace_retry_policy.md`.

### Hard authority rules

- `agent-graph` owns only control flow: ordered nodes, branch selection, bounded node retry, joins, interrupt/resume, and graph events.
- `llm-pipeline` owns provider transport, structured parsing, and bounded parse/correction retries.
- `job-queue` owns durable enqueue/retry, pause/resume, cancellation, crash recovery, and a new logical attempt family on re-enqueue.
- Forge verification owns paired test execution and concrete `TrialId` identity.
- `semantic-memory-forge` / Forge artifacts own raw verification evidence; `semantic-memory` owns queryable projections; `knowledge-runtime` owns scoped retrieval planning.
- Native Hermes gathers external evidence and performs all external actions. `delegate_task` remains for independent tool-using parallel work; cron remains for time-based durable scheduling.
- The graph is advisory until every certification gate is passed. No graph output may directly apply a patch, mutate memory, approve a release, or execute a side-effecting command.

---

## Phase 0: Preserve provenance and certify the actual MCP/runtime boundary

**Objective:** Establish exactly what executable and model are being evaluated before any integration claim.

**Files:**
- Create: `/home/sikmindz/.hermes/agent-graphs/README.md`
- Create: `/home/sikmindz/.hermes/agent-graphs/manifest-v1.json`
- Create: `/home/sikmindz/.hermes/agent-graphs/schemas/forge-graph-advisory-v1.schema.json`
- Create: `/home/sikmindz/.hermes/agent-graphs/schemas/forge-graph-decision-v1.schema.json`
- Create: `/home/sikmindz/.hermes/scripts/agent-graph-certify.py`
- Create: `/home/sikmindz/.hermes/artifacts/agent-graph-certification/.gitkeep`
- Inspect only: `/home/sikmindz/Coding/Libraries/agent-graph/`, `/home/sikmindz/.cargo/bin/agent-graph-mcp`, `/home/sikmindz/.hermes/config.yaml`

### Task 0.1: Capture immutable baseline receipts

Record, per certification run:

```json
{
  "schema_version": "agent-graph.certification.manifest.v1",
  "timestamp": "RFC3339",
  "mcp_binary": {"path": "/home/sikmindz/.cargo/bin/agent-graph-mcp", "sha256": "...", "mtime": "..."},
  "source": {"repo": "/home/sikmindz/Coding/Libraries", "head": "...", "agent_graph_dirty_paths": []},
  "mcp_config": {"base_url": "http://127.0.0.1:11434", "default_model": "..."},
  "model_under_test": "...",
  "graph_spec_digest": "..."
}
```

Commands to include in the harness receipt:

```bash
sha256sum /home/sikmindz/.cargo/bin/agent-graph-mcp
git -C /home/sikmindz/Coding/Libraries rev-parse HEAD
git -C /home/sikmindz/Coding/Libraries status --short -- agent-graph
git -C /home/sikmindz/Coding/Libraries diff --stat -- agent-graph
```

**Gate:** If the MCP binary cannot report/record a build/source hash matching a pinned source revision, label the result `UNCERTIFIED_RUNTIME`. Do not call it a core-runtime certification.

### Task 0.2: Specify the advisory input and output boundary

The MCP pilot input must be bounded references and summaries—not mutable Forge state or raw durable truth:

```json
{
  "schema_version": "forge.graph.advisory.v1",
  "run_id": "uuid",
  "trace_id": "opaque",
  "objective": "string",
  "repo_id": "string",
  "baseline_digest": "blake3-hex",
  "evidence": [{"ref": "string", "kind": "command_receipt|source|test|web", "summary": "string"}],
  "candidate_context": {
    "allowed_paths": ["relative/path"],
    "max_candidates": 3,
    "patch_policy_hash": "blake3-hex"
  }
}
```

The graph result must be parsed and schema-validated before the parent trusts it:

```json
{
  "schema_version": "forge.graph.decision.v1",
  "advisory_packet": "exact complete echo of the input forge.graph.advisory.v1 packet",
  "decision_status": "ready_for_generation|needs_more_evidence|reject|human_review",
  "atomic_claims": [{"claim": "string", "classification": "supported|contradicted|insufficient|human_review", "evidence_refs": ["string"]}],
  "routing": {"next_phase": "string", "reason": "string"},
  "warnings": ["string"],
  "model_receipt": {"model": "string", "input_digest": "blake3-hex", "output_digest": "blake3-hex"}
}
```

**Gate:** Empty output, non-JSON, duplicate/conflicting required fields, unrecognized status, no-route completion, or unbound evidence reference all normalize to `GRAPH_INVALID` and parent-side `human_review`; they never normalize to success.

### Task 0.3: Run package-scoped source checks separately from workspace closure

Run:

```bash
cd /home/sikmindz/Coding/Libraries
cargo fmt -p agent-graph -- --check
cargo test -p agent-graph
cargo clippy -p agent-graph --all-targets -- -D warnings
```

Record separately any `cargo test --workspace --all-targets` failure and its owning package. Do not classify an unrelated `check-runner` failure as an agent-graph test failure, and do not hide it as workspace success.

**Gate:** Package scoped checks must pass before a direct-core integration starts. Workspace closure is a separate receipt.

---

## Phase 1: Adversarial MCP certification before adoption

**Objective:** Determine whether a graph improves over a single constrained Hermes pass and whether it is safe enough even for advisory use.

**Files:**
- Create: `/home/sikmindz/.hermes/agent-graphs/fixtures/v1/`
- Create: `/home/sikmindz/.hermes/agent-graphs/fixtures/v1/oracles.json`
- Create: `/home/sikmindz/.hermes/agent-graphs/specs/evidence-advisor-v1.json`
- Create: `/home/sikmindz/.hermes/agent-graphs/reports/certification-report-v1.md`
- Modify: `/home/sikmindz/.hermes/scripts/agent-graph-certify.py`

### Task 1.1: Build 24 hand-authored fixtures and oracles

Create six categories with four fixtures each:

1. Clean deterministic JSON.
2. Empty/partial output: `""`, whitespace, `null`, truncation, missing required fields.
3. Malformed/adversarial output: invalid JSON, duplicate keys, wrong types, prompt injection, fabricated citations.
4. Router boundaries: exact match, near match, overlap, unknown/no-match.
5. Evidence tasks: supported, contradicted, insufficient evidence, unavailable source.
6. Side-effect traps: outside-workspace edit, destructive overwrite, stale/nonexistent target, correct abstention.

Each oracle must declare:

```json
{
  "expected_outcome": "...",
  "required_fields": [],
  "allowed_routes": [],
  "forbidden_claims": [],
  "allowed_effects": ["none"],
  "abstention_required": false
}
```

**Gate:** A fixture cannot be tuned by changing its oracle after the treatment result is seen. New held-out fixtures must be written before model/graph changes are evaluated.

### Task 1.2: Test comparable baseline and graph treatment

For every fixture run:

1. Baseline: one constrained native Hermes pass.
2. Treatment: one advisory graph pass.
3. Keep input, model/provider where technically possible, tool permissions, deadline, working directory, and prompt evidence constant.
4. Randomize order.
5. Capture raw output, model calls, retries, duration, tool calls, mutations, route, and receipts.

If a common model/provider cannot be used for both systems, do **not** claim task-quality superiority. Compare only governance/observability behavior and label the model mismatch.

Run a pilot of three repetitions per fixture. Only after all hard gates pass, run held-out fixtures with five repetitions per system.

### Task 1.3: Explicitly test current known hazards

- Test `glm-5.2:cloud` in JSON mode, but do not assume JSON mode fixes its proven empty-output behavior.
- Test at least one alternate candidate model. The simple `qwen2.5:0.5b` text response is insufficient proof of its structured-output quality.
- Use exact, serialized route labels such as `"route=needs_more_evidence"`, not broad terms like `"research"`.
- Assert parent-side expected terminal fields; a router ending at `END` without a validated decision result fails the fixture.
- Assert every retry/model call is visible in the evidence record. No unreported fallback receives success credit.

### Task 1.4: Apply certification gates

Hard gates on held-out cases:

- Unauthorized external mutation: `0`.
- Unsafe command/tool execution: `0`.
- False completion after empty/malformed output: `0`.
- Unsupported factual claims in evidence-bound cases: `0`.
- Graph strict success: at least `95%`.
- No category more than two percentage points below baseline.
- Empty-output or malformed-output silent acceptance: `0`.
- Route accuracy: at least `98%`; false-positive route rate at most `1%`.
- Unknown/no-match must block or abstain; it must never guess.
- Every run has execution/step/attempt lineage and recorded call count equals observed call count.

A quality-improvement claim additionally requires graph strict success to exceed baseline by at least five percentage points, with a predeclared paired confidence/statistical gate, and no more than 25% more model calls or 50% more latency unless the improvement is at least ten points.

**Kill criteria:** Stop MCP adoption for that use case if any critical safety gate fails, if binary/source identity remains unprovable, if a router can silently grant success, or if the graph does not improve value enough to offset its model-call/latency cost.

---

## Phase 2: Ship the report-first advisory pilot, not a general graph platform

**Objective:** Use the certified graph for one valuable human-in-the-loop deliverable: the Single-run Agent Evidence Report.

**Files:**
- Create: `/home/sikmindz/.hermes/agent-graphs/specs/single-run-evidence-report-v1.json`
- Create: `/home/sikmindz/.hermes/agent-graphs/templates/single-run-evidence-report-v1.md`
- Create: `/home/sikmindz/.hermes/scripts/run-single-run-evidence-report.py`
- Create: `/home/sikmindz/.hermes/artifacts/agent-evidence-reports/.gitkeep`
- Test: `/home/sikmindz/.hermes/agent-graphs/fixtures/v1/evidence-report/`

### Task 2.1: Implement the smallest useful graph

Topology:

```text
normalize_input_json
  -> extract_atomic_claims_json
  -> evidence_classify_json
  -> contradiction_and_gap_check_json
  -> decision_json
  -> report_format_json
```

The parent Hermes loop must gather files, command receipts, web sources, and semantic-memory results first. The graph receives only the bounded advisory envelope and does not invoke external tools.

### Task 2.2: Make the report skeptical by construction

Required report sections:

- Scope and source snapshot.
- Atomic claims, with evidence reference(s).
- Contradictions and stale/missing evidence.
- Explicit fact/inference/recommendation separation.
- `supported`, `contradicted`, `insufficient`, and `human_review` counts/statuses.
- Exact next action: gather evidence, delegate implementation, hold, or ask for approval.
- Input/output and graph-spec digests.

**Gate:** No prose sentence labeled verified may exist without a matching evidence reference. The parent discards the report when validation fails.

### Task 2.3: Measure real operator ROI

For representative coding/research runs, compare the report pilot to the current native Hermes evidence pass. Measure:

- number of unsupported claims caught;
- number of missing/contradictory evidence items surfaced;
- operator time to decide next action;
- model calls and latency;
- report field completeness;
- false success / invalid terminal results.

**Promotion gate:** Promote only if it improves proof discipline or decision time with zero critical safety failures. If it produces transcript restatements rather than claim-level adjudication, kill it rather than widening it.

---

## Phase 3: Add a narrow Forge `GraphAdvisor` adapter only after the advisory pilot proves value

**Objective:** Create one typed boundary between Forge domain state and advisory graph execution; do not put MCP protocol or mutable graph state throughout Forge.

**Files:**
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/graph_request.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/graph_result.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/services/graph_advisor_service.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/tests/graph_advisor_contract.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/fixtures/graph_advisor/`

Create the intended paths only after confirming Forge's live workspace layout; preserve the existing docs-only plan if the core crate has not yet been bootstrapped.

### Task 3.1: Define a one-method adapter

```rust
pub trait GraphAdvisor: Send + Sync {
    fn advise(
        &self,
        request: GraphAdvisorRequest,
    ) -> Pin<Box<dyn Future<Output = WorkbenchResult<GraphAdvisorResult>> + Send>>;
}
```

`GraphAdvisorRequest` must include: `run_id`, `TraceCtx`, logical `AttemptId`, objective, repo identity, baseline digest, bounded evidence refs, policy hash, deadline/budget, graph-definition version.

`GraphAdvisorResult` must include: schema version, advisory route, bounded findings/candidates, warnings, provider/model metadata, input/output digests, and normalized receipt linkage.

### Task 3.2: Fail closed at the adapter boundary

The adapter owns serialization, schema validation, timeout, model failure handling, artifact digesting, and receipt normalization. It must reject:

- empty or whitespace output;
- malformed/unknown schema;
- unknown/no-match route;
- evidence refs absent from the request;
- response over budget;
- stale/mismatched graph version.

It must not own queue retries, verification retries, patch application, semantic-memory mutation, or authority decisions.

### Task 3.3: Write contract tests before a live backend

Fixtures must cover: valid advisory, empty response, malformed response, unknown route, timeout/cancellation, mismatched digest, evidence ref injection, and no-side-effect enforcement.

**Gate:** Tests pass with a fake MCP/runner response before a live model is wired. A live failure falls back to deterministic `needs_more_evidence` / `human_review` without changing run truth.

---

## Phase 4: Directly integrate the Rust core into Forge's one durable repair graph

**Objective:** Implement the already-documented repair lifecycle only after the product state/adapter/validation contracts are stable. Do not use the MCP server as Forge's runtime boundary.

**Files:**
- Create/Modify: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/state.rs`
- Create/Modify: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/graph.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/detect_repo.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/capture_baseline.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/retrieve_memory.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/compile_mindstate.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/generate_candidates.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/validate_candidates.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/run_verification.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/score_and_explain.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/export_and_import.rs`
- Create: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/src/orchestration/nodes/finalize.rs`
- Test: `/home/sikmindz/Coding/forge-workbench/crates/forge-workbench-core/tests/repair_run_graph.rs`

### Task 4.1: Preserve the documented typed run state

Use `RepairRunState` from `/home/sikmindz/Coding/forge-workbench/docs/03_agent_graph_and_state.md` as the product contract: run/trace/attempt IDs, scoped repo/task, baseline, memory context, MindState, candidates, selected candidate, verification, export state, decision, warnings, and error.

Graph lifecycle:

```text
detect_repo
 -> capture_baseline
 -> retrieve_memory
 -> compile_mindstate
 -> generate_candidates
 -> validate_candidates
 -> select_candidate
 -> run_verification
 -> score_and_explain
 -> export_and_import
 -> await_user_decision
 -> finalize
```

### Task 4.2: Assign retry and authority ownership once

- One queued `RepairRunJob` owns the durable lifecycle.
- `agent-graph` owns bounded node-level transitions/retries only.
- `llm-pipeline` owns transport/parse correction retries.
- Forge verification emits a distinct `TrialId` for every concrete baseline or patched execution.
- Durable re-enqueue gets a new `AttemptId`.
- The graph pauses for explicit approval; it never self-approves.

### Task 4.3: Prove unsafe paths stay impossible

Add fixtures for unsupported repo, failed baseline, invalid candidate, verification regression, failed import warning, cancellation, restart recovery, approval denial, and repository drift before apply.

**Gate:**

- no candidate is recommended without paired verification;
- no candidate is applied without persisted approval and a verified baseline/head fingerprint;
- rejected/invalid candidates cannot reach apply;
- every terminal/retry/cancellation state remains visible with trace/attempt/trial receipt lineage;
- raw evidence stays in the canonical Forge authority path and normal retrieval exposes only appropriate projections.

---

## Phase 5: Extend only from demonstrated bottlenecks

**Objective:** Avoid turning a good bounded graph into a second generic agent framework.

Possible next pilots, in this order:

1. Memory/context governance graph: retrieval -> MindState -> compaction decision -> exact fallback / warning receipt.
2. Research/specification handoff: evidence extraction -> alternatives -> tradeoff matrix -> implementation and verification plan.
3. Small intake router with held-out route evaluation.
4. Proof packet assembly after source binding and evidence authority are independently proven.

### Explicit anti-goals

- No broad “universal agent graph.”
- No MCP server embedded as Forge's durable orchestration runtime.
- No graph-owned provider client, prompt parser, memory authority, job lifecycle, or UI persistence.
- No direct graph mutation of files, repository state, semantic memory, release status, or approvals.
- No claim of quality superiority when baseline/treatment models or tool budgets differ.
- No production/readiness/compliance/adoption claim from source presence or a small pilot.

---

## Final verification and decision matrix

| Decision | Required proof |
|---|---|
| Advisory MCP pilot is safe | All Phase 0–1 hard gates pass; graph failures fail closed; no side effects. |
| Advisory MCP pilot is valuable | Phase 2 catches more unsupported/missing proof or lowers operator decision time without cost/safety regression. |
| Forge adapter is warranted | Advisory pilot passes and fake-backend contract tests prove all malformed/unknown routes fail closed. |
| Direct Forge core graph is warranted | Forge bootstrap/control/verification boundaries exist; package checks pass; lifecycle fixtures prove drift-safe approval/apply and receipt lineage. |
| Expand beyond the repair/evidence wedge | A prior graph has measurable ROI and no unresolved authority/replay/safety gaps. |

## Source receipts used

- `/home/sikmindz/.hermes/config.yaml:681-696`
- `/home/sikmindz/Coding/Libraries/agent-graph/ARCHITECTURE.md`
- `/home/sikmindz/Coding/Libraries/agent-graph/src/checkpoint_store.rs`
- `/home/sikmindz/Coding/Libraries/agent-graph/src/event_sink.rs`
- `/home/sikmindz/Coding/Libraries/agent-graph/src/receipt.rs`
- `/home/sikmindz/Coding/forge-workbench/docs/03_agent_graph_and_state.md`
- `/home/sikmindz/Coding/forge-workbench/docs/06_queue_trace_retry_policy.md`
- `/home/sikmindz/Coding/forge-workbench/docs/07_acceptance_evals_release_gates.md`
- Five-member council findings and live MCP probes, July 15, 2026.
