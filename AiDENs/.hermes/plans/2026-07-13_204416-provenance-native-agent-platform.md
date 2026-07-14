# Provenance-Native Rust Agent Platform Implementation Plan

> **For Hermes:** Use `subagent-driven-development` only after the operator approves the scope gate. Keep AiDENs as an orchestration/adaptation layer; do not duplicate canonical truth owned by sibling Libraries crates.

**Goal:** Build a Rust-first, local-first agent gateway and operator workbench using AiDENs as the execution/control plane, while making every material action policy-gated, provenance-linked, receipt-bearing, inspectable, and replayable at the correct level of truth.

**Architecture:** Create a new product workspace, tentatively `aidens-agent-gateway`, adjacent to the canonical `Libraries/AiDENs` workspace. It depends on canonical AiDENs and sibling Libraries crates by explicit versions/path dependencies. The trusted kernel is a modular monolith: a durable event/receipt ledger, typed runtime state machine, policy enforcement, artifact references, and native-Rust tools operate in-process. Protocol adapters (MCP, messaging channels, OpenAI-compatible API, browser/process/WASI execution) sit outside the kernel.

**Tech stack:** Rust 2021/MSRV compatible with AiDENs (currently 1.76); Tokio; Axum + WebSocket/SSE; SQLite/WAL plus content-addressed artifact store; BLAKE3/`stack-ids`; RFC 8785 boundary handling through `boundary-compiler`; `claim-ledger`, `bitemporal-runtime`, `semantic-memory`, `knowledge-runtime`, `aidens-*`; `ratatui`; optional Leptos/Dioxus or a narrowly scoped TypeScript operator console only after CLI/TUI parity. Python is prohibited in the trusted path; a versioned Monte Carlo sidecar is allowed only after a benchmark proves Rust (`rand`/`rayon`/`ndarray`/C kernels) insufficient.

---

## 0. Evidence and scope boundary

### Current evidence

- Canonical AiDENs is a Rust workspace with 37 listed members, including `aidens-runner`, `aidens-autonomous`, `aidens-tui`, and `aidens-memory-tools`.
- `aidens-runner` already coordinates provider completion, tool dispatch, permit checking, memory grounding, and receipt emission; it expressly does not own domain truth.
- `aidens-memory-tools` already depends directly on `semantic-memory`, `knowledge-runtime`, `claim-ledger`, `stack-ids`, and `llm-output-parser`. Native memory/claim-tool integration is therefore an existing seam, not a greenfield invention.
- AiDENs’ active P32 source explicitly says `feature_expansion_allowed: false`, its current certification is blocked, and the current tree is dirty (33 paths at planning time). Feature work must **not** land in that tree until the active hardening run is closed or the product is created as an adjacent workspace.
- The verified local boundary is limited: the README records local lint/test/gate evidence and an Ollama tool-call exercise. It does **not** establish cloud readiness, durable autonomous-cycle receipt history, package/extracted replay certification, or authenticated OpenAI-compatible live-provider certification.
- OpenClaw’s documented strengths are a self-hosted multi-channel Gateway, agent sessions/memory/routing, tools/skills/cron/webhooks, a Control UI, and MCP bridging. Hermes’ documented strengths include provider-agnostic agent loops, tools, skills, persistent memory, sessions, cron, plugins/MCP, and a multi-platform messaging gateway.

### Non-negotiable doctrine constraints

1. AiDENs coordinates; it must not become a shadow owner of semantic-memory, claim/evidence, stable identity, verification, or kernel truth.
2. Every material action (provider route, tool call, approval, retry, cancellation, degradation, side effect, recovery, and replay) emits a typed receipt or a typed explicit non-durable/degraded outcome.
3. Truth is append-plus-supersession; valid time and recorded time remain distinct.
4. No unsafe “success” claim from advisory checks or inferred telemetry. A UI status must link to the actual governing receipt(s).
5. Model/tool output is hostile input. Policy and typed validation run before effects, not after.
6. Secrets are handles resolved only inside an execution boundary; never prompt context, receipt payload, ordinary logs, or UI defaults.

### Product boundary

**Build:** a new, compatible agent system that adopts useful *behaviors* from Hermes/OpenClaw (gateway, sessions, tools, skills, scheduling, MCP, operator UX), not source/brand/UI copies. Use upstream docs and licenses as capability baselines; preserve their trademarks and licenses.

**Do not build initially:** a clone of every social channel, a web UI before CLI/TUI work, multi-tenant SaaS, autonomous external writes, a bespoke vector DB, a second semantic-memory store, a second claim ledger, a fake deterministic replay guarantee for stochastic models, or a Python daemon.

---

## 1. Capability target matrix

| Capability | Hermes/OpenClaw baseline to adopt | Existing RecursiveIntell/AiDENs asset | Product commitment |
|---|---|---|---|
| Agent loop | tool-calling sessions, provider routing | `aidens-runner`, Plan/Act/Verify, budgets | Reuse runner semantics; add a durable run/event envelope around it rather than replacing it. |
| Tools | schemas, discovery, execution | `aidens-tool-kit`, permits, `llm-tool-runtime` | Typed manifests, version pinning, validation, capability-token dispatch, receipt per invocation. |
| Memory | persistent recall/context | `semantic-memory`, `knowledge-runtime`, `aidens-memory-tools` | Native direct calls for canonical local memory; MCP only as external interoperability boundary. |
| Claims/provenance | usually logging/audit only | `claim-ledger`, `bitemporal-runtime`, `stack-ids`, `boundary-compiler` | Receipts and evidence are execution semantics, with verification and supersession. |
| Gateway | multi-channel messaging and sessions | AiDENs daemon/queue/wake/schedule kits | Rust gateway core with one initial channel + local HTTP/WebSocket; adapters are independently sandboxed. |
| Scheduling | cron/webhooks/background tasks | `aidens-schedule-kit`, queue/daemon kits | Durable idempotent job acquisition/recovery, plus receipt-bearing wakeups and retries. |
| MCP | client/server bridge | Hermes/OpenClaw provide examples; native MCP capability needed | Rust MCP client/server adapter outside trusted kernel; pin/verify server manifests before high-risk tools. |
| Sandboxing | shell/browser/file tools | `aidens-tool-kit` sandbox concepts | WASI/component first; then process isolation with Linux Landlock/seccomp/cgroups/namespaces where available. |
| Replay | transcript/history | receipts and canonical IDs exist | Recorded replay first; re-execution/divergence second; deterministic replay only for pinned deterministic components. |
| Operator UX | CLI, terminal, control UI | `aidens-cli`, `aidens-tui` | CLI/TUI inspect/approve/replay/verify first. Web console only after it earns its cost. |

### Rust advantages that must be measured, not marketed

- bounded Tokio channels and structured cancellation under stream/tool backpressure;
- a single local binary/daemon with low idle memory and crash-safe recovery;
- typed contracts at the execution boundary, not dynamically interpreted tool JSON;
- in-process native tool/memory calls without MCP/HTTP serialization where protocol boundaries are unnecessary;
- parallel DAG execution only when policy, leases, and resource budgets prove independence;
- kernel-level resource and filesystem/network isolation where the host OS supports it;
- fast local receipt verification and event indexing.

No comparative performance/security claim is permitted until a reproducible benchmark and threat-specific test produce a receipt bundle.

---

## 2. Target architecture and ownership map

```text
Channels / CLI / TUI / optional Web Console / Webhooks
                         |
              aidens-agent-gateway (untrusted adapters)
                         |
         Authn + session routing + ingress receipt
                         |
        ┌──────── Durable Run Kernel ────────┐
        │ run state machine / cancellation    │
        │ resource budgets / leases / retries │
        │ policy decision + approval binding  │
        └──────────────┬──────────────────────┘
                       |
                  AiDENs Runner
       ┌───────────────┼───────────────────────┐
       |               |                       |
 Model adapters   Typed tool registry       Memory / claims
 local/remote     WASI/process/MCP          native direct canonical APIs
       |               |                       |
 provider receipt  effect + permit receipt    owner-crate receipts/facts
       └───────────────┴───────────────────────┘
                       |
          Receipt/event adapter + artifact references
                       |
 canonical owners: stack-ids, boundary-compiler, bitemporal-runtime,
 claim-ledger, semantic-memory, verification-* (no duplicate truth stores)
```

### Proposed adjacent workspace

Create only after Phase 1 approval:

```text
/home/sikmindz/Coding/aidens-agent-gateway/
├── Cargo.toml
├── AGENTS.md
├── README.md
├── crates/
│   ├── agent-gateway-contracts/   # typed public protocol only
│   ├── agent-gateway-kernel/      # lifecycle state machine; no network adapters
│   ├── agent-gateway-ledger/      # receipt/event adapter and artifact references
│   ├── agent-gateway-policy/      # grants, approvals, taint, budgets
│   ├── agent-gateway-artifacts/   # content-addressed encrypted payload references
│   ├── agent-gateway-tools/       # manifest registry and native-tool adapter
│   ├── agent-gateway-exec/        # WASI/process sandbox implementations
│   ├── agent-gateway-models/      # provider adapter/routing envelope
│   ├── agent-gateway-mcp/         # client/server protocol adapters
│   ├── agent-gateway-sessions/    # session projection, no canonical receipts
│   ├── agent-gateway-daemon/      # recovery, scheduler, durable queue binding
│   ├── agent-gateway-api/         # HTTP/WS/SSE control plane
│   ├── agent-gateway-cli/         # operator commands
│   └── agent-gateway-tui/         # ratatui operator surface
├── tests/                          # cross-crate scenario/fault tests
├── benches/                        # baseline + reproducible measurements
├── docs/
│   ├── adr/
│   ├── threat-model/
│   ├── protocols/
│   └── evidence/
└── scripts/                        # gates, fixture generation, receipt verification
```

### Canonical owner map

| Concern | Canonical owner | Gateway role |
|---|---|---|
| Stable IDs/digests | `stack-ids` | Consume only. |
| Canonical JSON/boundary validation | `boundary-compiler` | Validate ingress/egress, record refusal/quarantine. |
| Bitemporal fact semantics | `bitemporal-runtime` | Carry timestamps; do not collapse them. |
| Claims/evidence/provenance | `claim-ledger` | Create/query through its contracts; never mirror semantic state. |
| Semantic knowledge/memory | `semantic-memory` + `knowledge-runtime` | Direct in-process tool adapter for local use; MCP bridge for external servers. |
| Agent orchestration | AiDENs crates | Configure, invoke, bind outcomes to durable product receipts. |
| Verification policy | `verification-*`, `assurance-runtime` | Route/check/report; never relabel advisory observation as verification. |

---

## 3. Receipt, artifact, replay, and privacy specification

### 3.1 Event model

Every run is an append-only sequence of typed events. `RunStarted`, `IngressAccepted`, `ModelRequested`, `ProviderRouted`, `ToolCallProposed`, `PolicyDecided`, `ApprovalGranted|Denied`, `ToolStarted`, `ToolCompleted|Failed`, `RetryScheduled`, `Cancelled`, `RunFinalized`, and `Degraded` are distinct event variants. Each carries:

- deterministic `event_id`, run/task/session/actor/tenant IDs, causal-parent IDs;
- valid-time and recorded-time separately;
- schema + code/build identity, policy version, and canonical input/output artifact digests;
- capability grant/approval reference for any effect;
- provider/model/config/prompt-template digest and token/cost details when applicable;
- redaction/encryption disposition and artifact retention class;
- prior-chain digest, event digest, signature identity/key ID where signing is enabled.

Use the canonical owner stack for identity and canonicalization. Internal signed bytes must be an explicitly canonical typed encoding; JSON is a boundary/export format only. Do not introduce ad hoc hashing or random event IDs.

### 3.2 Artifact model

- Inline receipt payloads contain secret-free metadata and content digests.
- Prompts, tool args/results, files, and screenshots live as separately encrypted content-addressed artifacts with retention/destruction policy.
- Redaction is an explicit transformation event with original-artifact access restricted; redaction never silently replaces original evidence.
- Export creates a verifiable bundle containing receipt chain, public artifacts or encrypted references, schemas, verifier version, and a disclosure of unavailable/degraded data.

### 3.3 Replay taxonomy

1. **Recorded replay:** rebuild the observed run strictly from stored provider/tool outcome artifacts; no provider/tool side effects.
2. **Re-execution:** execute current/pinned components again under explicit policy and emit a divergence receipt against the recorded run.
3. **Deterministic replay:** only for components whose code, environment, inputs, clock/randomness, and artifacts are actually pinned and deterministic.

The product must never call a live LLM replay “deterministic.”

---

## 4. Security and execution policy

### Threat model to implement before capability expansion

- prompt injection and tool-result injection;
- confused deputy and overbroad delegated permission;
- compromised/poisoned MCP server or tool manifest;
- secret exfiltration through prompts/logs/artifacts/channel replies;
- filesystem traversal, symlink/hardlink escape, command injection, browser SSRF/private-address abuse;
- provider failure, rate-limit, timeout, content truncation, model substitution;
- receipt forgery, log truncation, duplicate side effect after crash, and replay confusion;
- runaway loops, spend/token exhaustion, resource exhaustion, and cancellation loss.

### Mandatory control rules

1. Deny-by-default capabilities; every tool invocation gets a narrow grant bound to run, tool version, input digest/scope, expiry, and intended effect.
2. A high-impact side effect requires an operator approval receipt. UI “Approve” is an operation that produces durable evidence, not a boolean toggle.
3. Typed schema validation, canonicalization, taint labeling, and policy evaluation occur before dispatch; rejected and malformed calls produce receipts.
4. No ambient shell PATH, broad network, wildcard file root, or arbitrary subprocesses in a high-assurance profile.
5. Use secret handles; execution resolves them after policy allows it. The model receives a capability description, never a secret value.
6. Default resource limits: deadline, token/cost ceiling, tool call ceiling, process count, memory, output bytes, filesystem quota, and egress domains.
7. Sandbox profiles truthfully advertise host support: Linux Landlock/seccomp/cgroup/namespaces; portable WASI restrictions; explicit degradation receipt on unsupported systems.
8. Side effects must be idempotency-keyed and recovery-aware. If correctness cannot be proven after a crash, quarantine and require operator resolution.

---

## 5. Delivery roadmap

### Phase 1 — Architecture freeze and clean product boundary

**Objective:** Author the design contracts without touching the current AiDENs hardening worktree.

**Files to create (new adjacent repo):**
- `docs/adr/0001-product-boundary-and-canonical-owners.md`
- `docs/adr/0002-receipt-and-artifact-model.md`
- `docs/adr/0003-replay-taxonomy.md`
- `docs/adr/0004-sandbox-and-capability-model.md`
- `docs/threat-model/v1.md`
- `docs/protocols/run-event-v1.md`
- `docs/protocols/tool-manifest-v1.md`
- `docs/capability-matrix.md`
- `docs/evidence/baseline-inventory-<timestamp>.md`

**Steps:**
1. Capture Git status/diff/log receipts of `Libraries/AiDENs` and all consumed canonical crates.
2. Read owner-crate public APIs and current capability documents; record version/digest/source paths in the inventory.
3. Convert the architecture above into ADRs with explicit non-goals and owner assignments.
4. Define public compatibility targets based on tested behavior, never API-name imitation.
5. Freeze first vertical-slice acceptance tests before code begins.

**Exit gate:** operator approves product name/path, single-user-first model, initial channel, initial provider pair, and whether external signature/anchor verification is required in v1.

### Phase 2 — Vertical slice: one auditable non-destructive run

**Objective:** Prove the entire execution semantics on one chat request, one provider, and one read-only native tool.

**Initial crates:** `agent-gateway-contracts`, `agent-gateway-kernel`, `agent-gateway-ledger`, `agent-gateway-policy`, `agent-gateway-models`, `agent-gateway-tools`, `agent-gateway-cli`.

**Steps:**
1. Write failing tests for valid/recorded time separation, deterministic material identity, receipt-chain continuity, and malformed tool-call refusal.
2. Implement typed run state transitions; every terminal state requires finalization or explicit degraded/abandoned receipt.
3. Bind `AiDENsRunner` output to a product-level durable event sink without claiming that AiDENs’ current in-memory autonomous ledger is durable.
4. Implement one local provider adapter and one OpenAI-compatible adapter; use provider fingerprinting and exact route receipts.
5. Register only `memory_search` or a sandboxed `repo-read` native tool as the first effect-free tool.
6. Implement policy denial/default grants, CLI inspection, `receipt verify`, and recorded replay that makes no network/tool calls.
7. Run crash/restart test immediately before/after receipt append and prove no unreceipted terminal status.

**Exit tests:** offline recorded replay; receipt verifier on an exported bundle; malformed tool call is refused with a receipt; killing and restarting does not emit a false completion; secret scanner finds no test secret in receipt/log artifact.

### Phase 3 — Durable effects and sandboxed tool plane

**Objective:** Make limited side effects safe, bounded, inspectable, and recoverable.

**Crates:** add `agent-gateway-artifacts`, `agent-gateway-exec`; extend policy/tools/kernel.

**Steps:**
1. Implement immutable tool manifests with owner, version, input/output schema, risk class, sandbox profile, and capability requirements.
2. Add input/output artifact references, encrypted storage, redaction transformation receipts, retention policy, and verifier support.
3. Implement WASI tool execution first. Add process backend only with canonical path checks, symlink/hardlink defense, sanitized environment, fixed executable resolution, resource limits, and receipts for spawn/exit/timeout/cancel.
4. Add Linux feature-gated Landlock/seccomp/cgroup/namespaces. Unsupported paths explicitly yield restricted or unavailable profiles, never pretend equivalence.
5. Add approval queue/TUI; approvals bind to exact proposed action and expire.
6. Add idempotency records and uncertain-outcome quarantine for externally visible effects.

**Exit tests:** path traversal/hardlink/symlink regression suite, command injection corpus, denied egress, cancelled process cleanup, duplicate effect prevention after injected crash, approval replay rejection after input digest changes.

### Phase 4 — Interoperability, messaging, and durable daemon

**Objective:** Match the useful gateway behavior of Hermes/OpenClaw while preserving the trusted kernel.

**Crates:** add `agent-gateway-mcp`, `agent-gateway-sessions`, `agent-gateway-daemon`, `agent-gateway-api`, `agent-gateway-tui`.

**Steps:**
1. Add a local HTTP/WS/SSE control plane with capability-restricted authentication and a receipt per ingress/outbound delivery.
2. Implement one messaging adapter (choose Telegram **or** Discord after approval); establish session isolation, identity binding, and per-channel policy.
3. Build MCP client support for low-risk external tools, then expose a narrowly selected read-only gateway surface as MCP server.
4. Treat MCP manifests/responses as untrusted; pin tool identity/version and prohibit inherited ambient grants.
5. Bind AiDENs schedule/queue/daemon kits to durable lease/idempotency/recovery receipts.
6. Add webhooks only after signed inbound verification, replay nonce handling, rate limiting, and target allowlists.

**Exit tests:** gateway restart session recovery, message delivery idempotency, remote MCP tool denied without a bound grant, forged webhook refusal, queue lease expiry/recovery without duplicate effect.

### Phase 5 — Memory, skills, delegation, and evaluation

**Objective:** Add the differentiators without turning observations into truth.

**Steps:**
1. Use direct embedded `semantic-memory`/`knowledge-runtime` through `aidens-memory-tools` for local canonical operations. Keep external MCP memory integrations as adapters.
2. Establish a capture policy: raw conversation/tool events are evidence artifacts, not automatically durable memory facts. Promotion goes through explicit evidence/claim policy.
3. Implement versioned signed skill bundles with manifest, source/provenance, capability declaration, tests, and revocation/supersession semantics.
4. Add multi-agent/DAG work only after durable leases, per-agent capabilities/budgets, causal receipt links, and handoff contracts are proven.
5. Implement operator-visible contradiction/proof-debt status by querying canonical `claim-ledger`/verification owners.
6. Implement Monte Carlo planning in Rust first. Benchmark candidate simulations against a defined decision-quality and latency baseline. Only then design a Python sidecar protocol with pinned interpreter/environment digest and full input/output receipts.

**Exit tests:** memory-promotion refusal for unsupported model text, skill manifest tamper refusal, delegation cancellation propagation, agent handoff causal-chain verification, Monte Carlo benchmark receipt and a proof that it changes an approved decision metric.

### Phase 6 — Operator experience and hardening

**Objective:** Make the system genuinely operable, then prove its bounds.

**Steps:**
1. Finish CLI commands: `chat`, `run`, `sessions`, `approve`, `deny`, `inspect`, `receipts verify/export`, `replay recorded`, `reexecute`, `doctor`, `policy explain`.
2. Build `ratatui` dashboards for run graph, receipt chain, pending approvals, budgets, effects, degraded state, and replay divergence.
3. Add a web console only when TUI usability evidence shows a browser surface is necessary; use Rust-rendered UI or minimal TypeScript solely at the presentation boundary.
4. Add property tests, fuzzing of boundary/tool schemas, fault injection at every durable transition, sandbox escape suite, supply-chain checks, compatibility tests, and reproducible benchmarks.
5. Produce independently executable receipt-verification fixtures and publish only bounded, evidence-backed capability claims.

**Exit gate:** a clean source tree; locked dependency build; format/lint/test/fuzz/timeboxed security suite; receipt verification on a fresh machine; package and extracted-replay gates; documented limits/degradations; independent hostile audit.

---

## 6. Test and benchmark contract

### Functional acceptance scenarios

1. **Inspectability:** A completed run shows its causal chain, provider route, tool args/result artifacts, policy decision, approval evidence, and finalization receipt.
2. **Recorded replay:** The exact observed run reconstructs offline, with network/tool dispatch disabled and verifier output matching the recorded chain.
3. **Re-execution:** Running with a changed model/tool produces an explicit machine-readable divergence report, not a silent replacement.
4. **Recovery:** Killing the daemon at each durable boundary produces either safe resumability or a quarantined uncertain effect—never duplicate side effects or false success.
5. **Least authority:** A compromised model/MCP/tool result cannot create a new capability grant or access ungranted paths/network/secrets.
6. **Privacy:** Secrets never appear in prompt messages, ordinary logs, receipts, or default exports; test uses synthetic canaries.
7. **Compatibility:** Selected MCP client/server and one channel adapter pass protocol fixtures with receipt evidence.

### Benchmarks (no vague Rust superiority claims)

- idle daemon RSS/CPU and startup time;
- bounded stream backpressure/cancellation latency;
- native direct-memory tool latency vs equivalent MCP round trip;
- receipt append/verify throughput and storage overhead;
- parallel DAG throughput under explicit CPU/memory budgets;
- crash/recovery time and duplicate-effect count (must be zero in supported scenarios);
- sandbox overhead versus unconfined tool baseline;
- Monte Carlo decision quality/latency against a pre-registered workload.

Each benchmark includes hardware, OS, build profile, commit/digest, workload fixtures, raw outputs, and a signed/verified evidence bundle.

---

## 7. Decisions already recommended; confirmation required before code

| Decision | Recommended default | Why |
|---|---|---|
| Product location | New `/home/sikmindz/Coding/aidens-agent-gateway` workspace | Current AiDENs P32 prohibits feature expansion and is dirty. |
| Deployment | Linux workstation, single operator first | Matches existing stack and enables truthful sandbox controls. |
| UI | CLI + Ratatui first; no TypeScript in v1 | Strong operator path with no browser/UI maintenance tax. |
| Provider slice | one local Ollama-compatible adapter + one OpenAI-compatible adapter | Tests local and remote boundary without provider explosion. |
| First channel | one only (Telegram or Discord) | Establishes correct session/identity/policy semantics before breadth. |
| Persistent data | SQLite/WAL metadata + encrypted content-addressed artifacts | Local-first, crash-safe, inspectable; no unnecessary distributed DB. |
| Memory | direct native canonical integration | Avoids needless MCP/HTTP serialization within the same trusted host. |
| Sandboxing | WASI first; Linux process isolation second | Portable constrained tools first, realistic shell/browser boundary second. |
| Receipts | local verification from day one; external anchoring deferred | Provides real evidence without introducing premature network/identity service complexity. |
| Monte Carlo | Rust first; Python only by benchmark exception | Honors language preference and eliminates a daemon until justified. |

---

## 8. Risks and explicit mitigations

| Risk | Mitigation / stop condition |
|---|---|
| Building a shadow semantic/provenance system | Owner map review required for every new contract; direct canonical owner use preferred. |
| Current AiDENs hardening work is destabilized | New workspace; no AiDENs edits during P32 without an explicit scope change. |
| “Receipts” become decorative logs | Make receipt emission a state transition precondition and test every failure/cancel/degraded path. |
| Replay promises overreach stochastic providers | Enforce recorded/re-execution/deterministic taxonomy in API and UI labels. |
| Model tool calls are unsafe | Deny-by-default grants, schema validation, taint propagation, approval binding, sandbox. |
| Tool count overwhelms local models | Profile-specific exposure, capability-based discovery, benchmark tool selection. |
| Python sidecar spreads into trusted runtime | Feature gate, protocol contract, receipt every exchange, benchmark justification, no direct store access. |
| Channel/MCP breadth creates untested attack surface | One adapter at a time; adapters live outside kernel; protocol test fixtures required. |
| Rust advantage becomes marketing theater | No claim without versioned workloads, raw results, and comparator methodology. |

---

## 9. First implementation milestone definition

**Milestone name:** `M0 — Receipted Read-Only Agent`

**Definition of done:**

- A Rust CLI accepts one prompt, invokes an AiDENs-backed model turn, exposes one read-only native tool, and terminates through a typed state machine.
- The run produces a durable receipt chain and separately stored artifact references; `receipts verify` succeeds offline.
- A policy denial, malformed tool request, provider timeout, cancellation, and recorded replay each produce typed evidence.
- No write, shell, browser, network-tool, channel, MCP-server, automated memory promotion, or Python feature exists in M0.
- The published claim is limited to this tested M0 scope and carries the exact verification receipt paths.

---

## Operator handoff

This is a planning-only artifact. No source implementation was started. Before any code, approve or amend the ten decisions in Section 7—especially product location, first messaging channel, primary user/security model, and receipt anchoring requirement. Then execute Phase 1 with source receipts, followed by M0 before feature breadth.

**Planning evidence:** witnessed semantic-memory retrieval receipt `aidens-rust-agent-plan-2026-07-13`; current AiDENs source inspected at `/home/sikmindz/Coding/Libraries/AiDENs` on 2026-07-13; current run authority `docs/codex-runs/CURRENT_RUN.json` states feature expansion is disallowed and certification is blocked.
