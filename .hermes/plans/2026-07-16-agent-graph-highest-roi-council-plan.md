# Agent-Graph: highest-ROI adoption plan

Date: 2026-07-16
Decision owner: Josh Stevenson / RecursiveIntell
Status: research complete; current MCP advisory adoption remains **NO-GO**.

## Executive decision

There are two different “highest ROI” answers:

1. **Highest near-term, low-authority experiment:** a read-only **Single-run Agent Evidence Report**. It should turn bounded command/source/web/memory receipts into atomic claims, evidence classifications, contradictions, gaps, and one advisory next action. This is the fastest way to learn whether graph control flow materially improves proof discipline rather than merely adding an LLM call.

2. **Highest eventual engineering payoff:** use the Rust `agent-graph` core as Forge Workbench’s repair-run orchestration spine: baseline capture → bounded candidate fan-out → paired verification → deterministic recommendation → explicit approval → drift-safe apply. This is where checkpoints, retries, cancellation, interrupts/resume, deterministic joins, and trace/attempt/trial lineage have genuine leverage.

Do not use the installed MCP façade as a durable Forge runtime. It is currently an uncertified, narrow integration surface rather than evidence that the core runtime is ready for product adoption.

## Council conclusion

Five independent reviews converged on the same shape:

- The core crate is real and useful for stateful, branching, resumable, receipt-bearing workflows.
- The enabled MCP façade is much narrower: LLM/router/passthrough graphs, process-local registry, and no demonstrated exposure of core checkpoints, interrupt/resume, joins/reducers, durable execution, or tool-node controls.
- The current MCP pilot validates fail-closed harness behavior, not task quality or business value. It remains a NO-GO.
- A generic autonomous “agent platform,” graph-owned side effects, release authority, patch application, or memory authority would be negative ROI now.
- The next funding decision must be based on operator outcomes, not a longer feature list or another happy-path protocol probe.

## Evidence used

### Live/current state

- Core source: `/home/sikmindz/Coding/Libraries/agent-graph`, version `0.2.0`.
- The core architecture documents JSON state, bounded parallel supersteps, explicit joins/reducers, retries, checkpoints, interrupt/resume, cancellation, event sinks, streaming, and `stack-ids` trace/attempt/trial lineage.
- On 2026-07-16, `cargo fmt -p agent-graph -- --check`, `cargo test -p agent-graph`, and `cargo clippy -p agent-graph --all-targets -- -D warnings` passed.
- Enabled MCP command: `/home/sikmindz/.cargo/bin/agent-graph-mcp` with `glm-5.2:cloud` at `http://127.0.0.1:11434`.
- The binary SHA-256 is `a521ceeea84a0ff90c8dea94dfca31ccbdd3ef5deb85e24e023392c560b29e38`; its mtime is 2026-07-12. Cargo installation metadata names `/home/sikmindz/Coding/Libraries/agent-graph-mcp` as its build path, but that path no longer exists. Therefore binary/source provenance is unprovable and the runtime is `UNCERTIFIED_RUNTIME`.
- Current MCP registry holds four old probe graphs; it is process-local and not durable run history.

### Existing certification result

- Phase 1 is explicitly `NO_GO_MCP_ADOPTION` in `/home/sikmindz/.hermes/agent-graphs/reports/certification-report-v1.md`.
- The live `glm-5.2:cloud` treatment recorded 24 runs and 23 errors (`MCP_EXECUTION_FAILED`); it does not establish model quality or safe runtime behavior. The JSON-RPC envelope itself is correct: the binary requires `tools/call` (not direct `graph_create`/`graph_execute` methods). Most late fixture calls exhausted the harness's single 20-second session deadline, so the next harness must have explicit startup and per-request budgets.
- A minimal passthrough succeeds at the MCP layer but can return a scalar `final_state` (for example, `"hello"`); the current generic harness extraction requires an object. The evidence-advisor decision contract legitimately expects an object, but its transport adapter must distinguish a valid scalar graph result from a malformed decision rather than classifying all non-object final states as transport failure.
- Deterministic 72-record replay is correctly labelled `HARNESS_VALIDATED_NOT_MCP_CERTIFIED` and `NON_COMPARABLE_SYNTHETIC_BASELINE`.
- The fixed v1 corpus has an observed contract mismatch: its clean payloads omit a now-required exact advisory-packet echo. Preserve v1 as evidence; do not retrofit it after seeing results.

## Ranked portfolio

| Rank | Opportunity | Value | Cost/risk | Decision |
|---|---|---|---|---|
| 1 | Single-run Agent Evidence Report | High immediate proof-discipline and operator-decision leverage; read-only | Low-to-medium, if bounded and advisory | First real value pilot after runtime admission |
| 2 | Direct-core Forge repair-run graph | Highest strategic reliability/recovery payoff | Medium-to-high; requires Forge closure and typed contracts | Primary eventual integration; do not start from MCP |
| 3 | Recall bounded, non-tool query/maintenance subflows | Existing narrow graph adapter and event bridge reduce marginal cost | Medium; avoid replacing legacy tool loop or job ownership | Extend only for a measured bottleneck |
| 4 | Memory/context governance gate | Valuable if retrieval/compaction loss is recurrent | Medium; risk of a second memory authority | Pilot only after evidence/replay contracts stabilize |
| 5 | Research/specification-to-verification handoff | Repeatable and useful for evidence-to-plan work | Medium | Reuse the evidence packet/report contract |
| 6 | Intake/queue router | Cheap only if it removes real operator triage work | Risky until exact routing is proven | Defer; never allow silent no-match success |

## Boundaries that must not move

- Hermes remains the parent authority for browsing, terminal/file operations, external evidence collection, delegation, approvals, and all side effects.
- `agent-graph` owns only control flow: bounded nodes, routing, joins, node retries, checkpoints, interruption, and events.
- `llm-pipeline` owns provider transport, structured parsing, and parse/correction retries.
- `job-queue` owns durable enqueue/retry, cancellation, pause/resume, and crash recovery.
- Forge owns verification, paired trials, approval, and apply authority.
- Forge artifacts and semantic-memory retain their existing evidence/projection authority boundaries.
- No graph result may directly write files, run commands, mutate memory, apply patches, approve releases, or trigger production activity.

## Plan

### Phase 0 — keep the correct NO-GO and choose the path

1. Retain the current MCP advisory NO-GO in reports and operator practice.
2. Decide whether MCP is actually required for the first pilot. It is **not** required for the highest-value strategic Forge path; the core can be embedded directly when the host is ready.
3. Do not spend on general MCP breadth, UI work, generic multi-agent runtime claims, or throughput tuning before a useful bounded artifact exists.

Exit criterion: the plan has one named low-authority pilot and one named strategic host; no graph authority overlaps an existing owner.

### Phase 1 — prove core fit without MCP or product mutation

Build an isolated, non-production Forge-shaped graph experiment against the current Rust core:

```text
synthetic intake
  → synthetic baseline capture
  → bounded candidate fan-out
       → validate candidate
       → simulate paired verification
  → explicit deterministic join/recommendation
  → simulated approval interrupt
  → resume/finalize
```

Required assertions:

- bounded fan-out and deterministic join order;
- a transient candidate failure yields visible retry/attempt/trial lineage;
- cancellation becomes visible cancellation, never completion;
- checkpoint/resume preserves completed work and rejects graph-topology mismatch;
- graph events can be mapped to Forge’s required `run_id`, trace, attempt, trial, candidate, and phase fields;
- no real repository, Forge control DB, semantic memory, network, or filesystem mutation.

Deliverable: a source-controlled test/example plus a receipt comparing graph events to Forge’s required state contract. This de-risks the strategic use without depending on the stale binary.

Kill criterion: if the core cannot represent Forge’s required lineage, recovery, and approval boundary without duplicating job-queue or Forge authority, stop and preserve the existing workflow ownership.

### Phase 2 — optional MCP runtime admission (only if Hermes needs the report pilot through MCP)

Before any new live MCP decision test:

1. Restore or identify the exact `agent-graph-mcp` source tree.
2. Pin a source revision and rebuild the binary reproducibly.
3. Record source revision, clean/dirty status, build command/toolchain, binary hash, graph-spec hash, model/backend configuration, tool schema digest, and environment identity in one parent-generated manifest.
4. Replace the single total deadline model with explicit startup and per-request deadlines, stdout byte caps, stderr capture, EOF/nonzero handling, explicit shutdown, process-group/descendant cleanup, and a fresh isolation policy per test case or a proven safe reuse design.
5. Add adversarial tests for startup/request hangs, malformed/framed/oversized stdout, stderr noise, premature EOF, nonzero exit, duplicated/out-of-order request IDs, server restart, repeated requests, cancellation, and descendant cleanup.
6. Prove via sandboxed negative tests that hostile graph output cannot cause command, patch, release, memory, network, or Forge-promotion effects.

Hard blockers: missing source binding; any silent success on empty/malformed/no-route output; unbounded or uncleanable child process; receipt/source mismatch; or any side effect outside the allowlist.

### Phase 3 — new immutable v2 evidence-report certification

Do not modify observed v1 fixtures. Create a new v2 corpus with predeclared digests, a corrected schema contract, independent oracles, and fixed plus held-out cases.

The report graph is advisory-only:

```text
bounded evidence packet
  → normalize
  → extract atomic claims
  → parallel evidence/completeness/contradiction checks
  → explicit deterministic join
  → classify: supported | contradicted | insufficient | human_review
  → route: gather evidence | delegate | hold | request approval
  → format evidence report
```

The graph receives only a bounded packet; Hermes gathers receipts and performs any action. Parent validation rejects empty, whitespace, malformed, unknown-schema, duplicate-key, altered-advisory, unbound-evidence, unknown/no-match route, route/status mismatch, timeout, oversized response, and model mismatch as `GRAPH_INVALID → human_review`.

Compare the graph treatment with the normal Hermes evidence pass using the same inputs, provider/model where technically possible, tool permissions, deadlines, and evidence. Randomize order and record raw outputs, model calls, retries, latency, graph version, and parent validation result.

Hard gates before any pilot:

- zero unauthorized mutations and unsafe tool/command actions;
- zero silent acceptance of empty/malformed/no-route output;
- strict graph success at least 95% on held-out cases;
- route accuracy at least 98%, false-positive routing at most 1%; unknown/no-match blocks or abstains;
- no category more than two percentage points below baseline;
- all calls/steps/attempts are receipt-linked;
- quality claims require a technically comparable baseline.

For an improvement claim, predeclare at least +5 percentage points over baseline with a paired statistical gate, while allowing no more than 25% extra model calls or 50% extra latency unless improvement reaches 10 points.

Kill criterion: if the graph is just a transcript restater, does not improve unsupported-claim/gap detection or decision time, or fails any critical safety gate, kill the MCP evidence-advisor use case rather than widening it.

### Phase 4 — two-to-four week human-supervised report pilot

Only after Phase 3 passes, use the evidence report for a small, read-only set of representative coding/research runs.

Measure weekly:

- unsupported claims caught;
- contradictions and missing evidence surfaced;
- operator time to the next decision;
- human override and ignored-report rate;
- report completeness;
- model calls, median/p95 latency, and retry burn;
- invalid/false terminal results;
- receipt verification and reproducibility.

Promotion criterion: measurable improvement in proof discipline or operator decision time with zero critical safety failures. A second operator must be able to run the corpus and verify its receipts from the written procedure.

### Phase 5 — direct-core Forge adoption only after demonstrated need

When Forge’s repair workflow closure is ready and Phases 1/3/4 demonstrate value, integrate the core directly—not via the MCP façade:

```text
intake
  → capture baseline
  → retrieve memory
  → compile MindState
  → bounded candidate generation/validation fan-out
  → paired verification
  → deterministic score/recommendation join
  → await persisted operator approval
  → drift-safe apply or finalize
```

Requirements:

- one durable `RepairRunJob` owns the full run; the queue does not become a per-node graph replacement;
- graph node retries remain bounded; provider and verification retries retain their existing owners;
- every concrete verification gets a distinct `TrialId`; durable re-enqueue gets a new `AttemptId`;
- approval is persisted and baseline/head fingerprint is rechecked before apply;
- no invalid, rejected, unverified, cancelled, or drifted candidate reaches apply;
- raw Forge evidence remains canonical and retrieval sees only valid projections.

Promotion criterion: it measurably reduces duplicate work, recovery time, stalled-run ambiguity, or operator effort compared with the current workflow—while preserving verification and approval integrity.

## Explicit deferrals

- Generic autonomous/multi-agent platform.
- Unattended cron use of graph decisions.
- Write-capable MCP tools and production credentials.
- Autonomous patches, commands, releases, messaging, purchases, deployments, or account changes.
- Graph-owned semantic-memory writes or truth promotion.
- Broad compatibility, quality, compliance, readiness, or adoption claims.
- Replacing Recall’s legacy/tool path or job queue without a measured, bounded failure mode.

## Decision matrix

| Decision | Required evidence |
|---|---|
| Keep MCP NO-GO | Current state: source binding absent; live structured treatment failed closed |
| Start core-only Forge-shaped experiment | Package checks pass; experiment has zero real side effects |
| Start MCP v2 certification | Pinned/rebuilt provenance-bound binary plus process/protocol/sandbox guards |
| Start report pilot | All v2 adversarial, held-out, baseline, and strict parent-validation gates pass |
| Add Forge graph adapter | Report pilot demonstrates value and typed fake-backend contracts pass |
| Embed direct core in Forge | Forge closure exists and graph reduces a demonstrated recovery/coordination bottleneck |
| Expand to Recall/context/triage | A prior use case passed ROI and authority-boundary gates |

## Primary receipts

- `/home/sikmindz/.hermes/agent-graphs/reports/phase1-decision-20260716.md`
- `/home/sikmindz/.hermes/agent-graphs/reports/certification-report-v1.md`
- `/home/sikmindz/.hermes/artifacts/agent-graph-certification/phase1-mcp-binding-20260716/pilot-receipt.json`
- `/home/sikmindz/Coding/Libraries/agent-graph/ARCHITECTURE.md`
- `/home/sikmindz/Coding/forge-workbench/docs/03_agent_graph_and_state.md`
- `/home/sikmindz/Coding/forge-workbench/docs/06_queue_trace_retry_policy.md`
- `/home/sikmindz/Coding/Recall/RECALL_END_STATE_SPEC_V2.md`
