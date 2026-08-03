# Receipt-Grounded Learning Coding Agent POC/MVP Specification and Implementation Plan

> **For Hermes:** Implement with `subagent-driven-development`, strict RED/GREEN TDD, one canonical owner per artifact, controller-owned integration, and an independent hostile review. Do not implement against the current dirty tree until Task 0 records and isolates every overlapping change. Do not begin agent-graph Phase 2, a Forge adapter, or direct Forge integration unless a new certification decision supersedes the 2026-07-16 NO-GO.

**Goal:** Build a bounded local coding agent that executes a complete observe → plan → authorize → sandbox-act → independently verify → attribute → quarantine → evaluate → promote/revoke → replay loop, and learns only governed procedural-memory artifacts plus later shadow-only retrieval-routing parameters.

**Architecture:** Extend the existing AiDENs coding profile, runner, CLI, and V3 run bundle as a thin composition layer. Use `semantic-memory` for witnessed retrieval and procedure lifecycle, `sandbox-workspace` + `typed-patch` + `check-runner` for effectful work, `verification-*` for independent disposition, `cea-core`/`cea-store` for attribution, `semantic-memory-forge::ExportEnvelopeV3` for canonical export, and `AiDENsRunBundleV3` for operator/run composition. Do not build another agent runtime, memory server, truth store, receipt database, verification oracle, causal store, or policy authority.

**Tech stack:** Rust, SQLite/rusqlite, serde/schemars, `stack-ids`, AiDENs, `forge-pilot`, `semantic-memory`, `semantic-memory-forge`, `claim-ledger`, `verification-*`, `Primitives/{sandbox-workspace,typed-patch,check-runner,cea-core,cea-store}`, rootless Podman, Cargo, JSON/JSONL receipts.

---

## 1. Executive decision

### Approved

A **bounded, local, receipt-grounded vertical slice** is approved for implementation:

1. Read a frozen coding task and a governed memory snapshot.
2. Produce a typed patch proposal.
3. Obtain an effect-specific permit.
4. Stage the task in an isolated workspace.
5. Apply only a `typed-patch::StructuredPatch`.
6. Run pinned checks through a sealed `check-runner` container backend.
7. Let verification crates—not the model—decide the outcome.
8. Attribute the outcome with CEA.
9. Compile any reusable procedure as a quarantined `ProceduralMemoryArtifactV1`.
10. Compare candidate and baseline on frozen paired trials and immutable holdouts.
11. Promote only through the semantic-memory lifecycle permit; otherwise quarantine or revoke.
12. Persist canonical child receipts before the terminal `AiDENsRunBundleV3` and support explicit replay modes.

### Blocked

The following claims and implementation shortcuts are blocked:

- unrestricted autonomous coding;
- production safety or autonomous release readiness;
- online reinforcement learning, self-training, or foundation-model weight updates;
- learning that can change evaluator logic, verification policy, authority policy, receipt semantics, available tools, or holdouts;
- direct host-repository mutation;
- direct shell execution outside the bounded execution backend;
- treating `agent-guard` as containment;
- treating fixture simulation, mock-provider output, checkpoint presence, or receipt-shaped metadata as execution proof;
- direct agent-graph/Forge integration under the current NO-GO;
- promotion because “tests passed,” CEA confidence is high, or the agent claims success;
- a new `LearningEpisode`, generic proof packet, memory store, or run-bundle family when existing owner artifacts can carry the boundary.

### Product claim boundary

After the POC gate, the safe claim is:

> A local vertical slice can execute a frozen coding task in a bounded external sandbox, independently verify it, persist source-bound receipts, generate a quarantined procedure candidate, and exercise promotion/revocation/replay controls.

After the MVP evaluation gate, the additional safe claim is:

> On a predeclared benchmark with frozen development/calibration/holdout partitions, a versioned procedural policy can be compared with a frozen baseline using independently witnessed outcomes and conservative promotion gates.

Neither gate licenses “production autonomous engineer,” “safe untrusted-code sandbox,” “online RL,” or “self-improving foundation model.”

---

## 2. Evidence basis and mutable baseline

### Planning checkpoint

Observed on 2026-07-16 after both council tranches:

- Repository: `/home/sikmindz/Coding/Libraries`
- Branch: `fix/hostile-remediation-20260715`
- HEAD: `03ca7e9911a3d2b30437092403cd55c262e33b2a`
- Dirty entries: 172 (`1 D`, `84 M`, `87 ??`)
- Current porcelain SHA-256: `1f8a05bb97552bde15d7f3fa9a11bd47e34f1b00817792f7734374e99743a520`
- Submodule check: blocked by `fatal: no submodule mapping found in .gitmodules for path 'cea-bridge'`

These are planning observations, not release receipts. Task 0 must recapture them before implementation, and release certification must use a clean attributable tree.

### Verified package-scoped baseline

| Command | Result | Narrow meaning |
|---|---|---|
| `cargo test -p typed-patch --lib` | 6 passed | typed-patch unit behavior only |
| `cargo test -p sandbox-workspace --lib` | 11 passed | staging/path tests only; not OS containment |
| `cargo test -p check-runner --lib` | 11 passed, 1 ignored | backend/env/timeout behavior; ignored backend-selection case remains |
| `cargo test -p cea-core --lib` | 11 passed | CEA core unit behavior |
| `cargo test -p cea-store --lib` | 5 passed | CEA persistence unit behavior |
| `cargo test -p forge-pilot --lib` | 22 passed | package-scoped Forge pilot tests |
| `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-runner --lib` | 41 passed | AiDENs runner package behavior |
| `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-receipts --lib` | 6 passed | receipt/store package behavior |
| `cargo test -p agent-graph --lib` | 0 tests | compilation only for this invocation; certification evidence is separate |
| `cargo test -p agent-guard` | 3 passed | initialization/state tests, not containment |
| `python3 AiDENs/scripts/p30_guard.py --repo . --json` | 13,066 warnings, 0 hard findings | not release evidence; historical path matching misses live `AiDENs/crates/...` paths |

### External containment available on the planning host

Live inspection found `/usr/bin/podman`, `/usr/bin/docker`, and `/usr/bin/bwrap`. Podman reported rootless mode, cgroup v2, and seccomp enabled. The POC therefore standardizes on **rootless Podman sealed execution**. Presence is not certification; Task 4 must prove denial and cleanup behavior.

### Binding agent-graph decision

The authoritative 2026-07-16 decision in `/home/sikmindz/.hermes/agent-graphs/reports/phase1-decision-20260716.md` states:

> **NO-GO: do not begin Phase 2, the Forge adapter, or direct Forge integration from this pilot.**

`agent-graph` may support read-only councils/design review outside the runtime. It must not enter this MVP execution path until a new versioned certification clears runtime/source binding, strict success, comparable baseline, held-out, process-boundary, and value gates.

---

## 3. Council convergence

Six read-only council lanes completed across two tranches. The controller verified decisive claims against live source.

| Topic | Council agreement/dissent | Controller decision |
|---|---|---|
| Runtime architecture | Architecture lanes favored thin AiDENs composition; hostile lane rejected a new autonomous substrate | Extend AiDENs runner/CLI; no new agent runtime |
| Closed loop | OODA, receipts, verification, CEA, and procedure lifecycle exist but are not fully wired | Build one procedural-learning vertical slice with canonical backpointers |
| Learning | Learning lanes approved offline procedural/routing adaptation; security lane rejected self-training | Procedure learning is effectful only after promotion; routing is shadow-only in MVP |
| Containment | Early hostile search incorrectly missed `Primitives/sandbox-workspace`, `typed-patch`, and `check-runner`; later live source found them | Reuse those crates, but distinguish path-safe staging from OS containment; require rootless Podman |
| Agent guard | All hostile reviews found placeholder enforcement | Exclude `agent-guard` from the POC trust boundary and from safety claims |
| Agent graph | Core is useful, but installed MCP adoption certification is NO-GO | Advisory/council use only; no runtime/Forge adapter |
| Run artifact | V3 already carries lineage and owner backpointers; some fields/IDs are weak | Retain V3 for POC; add material-bound IDs and canonical external backpointers. Require a field-gap matrix before any V4 |
| Procedure testing | Existing `test_procedure` is deterministic fixture simulation | Keep it as a static gate; add a separate real sandbox evaluator before promotion |
| Success semantics | Existing reports can expose degraded/blocked states, but false completion remains possible | Add a fail-closed derived terminal projection and a single success predicate backed by canonical receipts |
| Evaluation | Existing routing threshold of 10 examples is not scientific | Require paired baseline/treatment, immutable splits, confidence intervals, negative controls, and predeclared stopping |
| Production readiness | Systems/security lanes block release | Approve POC/MVP only; retain production release blockers |

The full council memoranda are retained at:

- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260716_053433_395242.txt`
- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-1-20260716_053433_400812.txt`
- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-2-20260716_053433_404112.txt`
- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-0-20260716_054745_822036.txt`
- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-1-20260716_054745_883892.txt`
- `/home/sikmindz/.hermes/cache/delegation/subagent-summary-2-20260716_054745_903146.txt`

---

## 4. Product definition

### 4.1 User and job

Primary user: a local operator/developer who wants a coding agent to improve reusable coding procedures while retaining authority over effects, evidence, promotion, rollback, and claims.

Primary job:

> Given a bounded repository fixture and task, produce a verified patch, explain exactly what executed, preserve independently checkable evidence, and learn only a reusable procedure that demonstrably improves held-out outcomes.

### 4.2 POC scope

The POC proves one complete procedural-learning cycle on frozen local fixtures:

- one Rust task family plus adversarial negative fixtures;
- read/search, typed patch proposal/application, pinned checks;
- external sealed sandbox;
- no network;
- no host-repository write;
- no package publication, deployment, credential access, or Git push;
- one operator-approved promotion and one forced revocation/rollback drill;
- explicit `no_replay`, `store_inputs`, and replay comparison behavior;
- mock/fixture/dry-run/real-sandbox modes visibly separated.

### 4.3 MVP scope

The MVP adds:

- at least 20 paired trials across at least 5 task families for a procedure promotion demonstration;
- immutable 60/20/20 development/calibration/holdout partitions;
- family-clustered statistics and negative suites;
- shadow-only routing candidate evaluation with at least 100 eligible examples before any promotion proposal;
- operator inspection, comparison, quarantine, promotion, revocation, and replay commands;
- crash recovery and atomic terminal publication;
- independent hostile review from a clean tree.

### 4.4 Explicit non-goals

- autonomous release or deployment;
- arbitrary repositories or languages in v1;
- unrestricted shell/network/package-manager access;
- model-weight updates;
- online self-modification;
- model-authored evaluators or hidden-test changes;
- general-purpose multi-agent runtime;
- agent-graph runtime adoption;
- production containment claims based on `agent-guard`;
- learning from unaudited user thumbs-up/down or synthetic CEA telemetry;
- new canonical truth stores.

---

## 5. Canonical ownership map

| Lifecycle responsibility | Canonical owner and live surface | MVP use |
|---|---|---|
| IDs, digests, trace, attempt/trial | `stack-ids/src/lib.rs` | material identity and lineage spine |
| Coding profile | `AiDENs/crates/aidens-profile-coding/src/lib.rs` | default deny, permit-required coding capability |
| Run/turn composition | `AiDENs/crates/aidens-runner/src/{lib.rs,execution.rs,finalization.rs,provider_tool.rs,receipts.rs,replay.rs}` | thin closed-loop application layer |
| Operator CLI | `AiDENs/crates/aidens-cli/src/lib.rs` and `src/agent.rs` | extend existing `agent` surface |
| Run bundle contract | `AiDENs/crates/aidens-contracts/src/agent_bundle.rs` | retain `AiDENsRunBundleV3`; add owner backpointers, not copied truth |
| Schema registry | `AiDENs/crates/aidens-contracts/src/schema_catalog.rs` | register only new non-authoritative projections if needed |
| Receipt persistence | `AiDENs/crates/aidens-receipts/src/lib.rs` | child-first, atomic bundle/index publication and recovery |
| Governed memory retrieval | `semantic-memory/src/{authority.rs,state_epistemics.rs,evidence_gap.rs}` | witnessed, scoped, current/historical retrieval |
| Procedure candidate/lifecycle | `semantic-memory/src/procedural_memory.rs` | compile/test/quarantine/promote/revoke/rollback |
| Routing learner | `semantic-memory/src/rl_routing.rs` | shadow-only candidate; existing 10-example threshold is not promotion |
| OODA and run report | `forge-pilot/src/{loop_runner.rs,config.rs,loop_runner_report.rs,act.rs,orient.rs,decide.rs,bundle_builder.rs}` | bounded plan/act/verify composition; tighten success predicate |
| Canonical Forge export | `semantic-memory-forge/src/envelope.rs` | `ExportEnvelopeV3` validation/digest/roundtrip |
| Path-safe staging | `Primitives/sandbox-workspace/src/lib.rs` | fixture copy and path confinement |
| Mutation representation | `Primitives/typed-patch/src/lib.rs` | sole patch representation/application |
| Command checks | `Primitives/check-runner/src/lib.rs` | sealed container backend and normalized check receipts |
| Causal attribution | `Primitives/cea-core/src/{lib.rs,attribution.rs}` | run hash and edit/effect attribution |
| Causal persistence | `Primitives/cea-store/src/lib.rs` | transactional/idempotent graph updates |
| Experiment substrate | `living-memory/living-memory/src/experiment.rs` | paired baseline/candidate trials and typed diffs |
| Verification | `verification-control`, `verification-policy`, `verification-calibration`, `verification-adjudication` | independent disposition and non-inferiority gates |
| Claim admission/proof debt | `claim-ledger` | reject unsupported promotion/marketing claims |
| OS/process containment | rootless Podman outside `agent-guard` | no network, bounded resources, no host mount except controlled fixture/result paths |

### Ownership rule

AiDENs may compose IDs and references to owner artifacts, but it may not reinterpret their truth. `AgentState`, Forge history, run reports, and UI status are projections. They may never promote memory, admit claims, change verification outcomes, or become an alternate causal graph.

---

## 6. Architecture

```text
Operator task + immutable corpus manifest
                  │
                  ▼
AiDENs profile/CLI ──→ material run/attempt/trial IDs
                  │
                  ▼
Governed observe/retrieve
semantic-memory authority + witnessed receipt + state generation
                  │
                  ▼
Plan and preflight
PlanActVerifyLoopV1 / forge-pilot + CheckPlan + permit digest
                  │
                  ├──── blocked/indeterminate → durable terminal bundle
                  ▼
Rootless Podman sandbox
sandbox-workspace → typed-patch → check-runner
                  │
                  ▼
Independent verification
verification-control → policy → calibration → adjudication
                  │
                  ├──── failed/degraded → quarantine/rollback
                  ▼
CEA attribution
cea-core → cea-store (exact patch/check/evaluation lineage)
                  │
                  ▼
Candidate extraction
ProceduralMemoryArtifactV1, always quarantined first
                  │
                  ▼
Paired baseline/treatment experiment
frozen task + randomized order + immutable holdout
                  │
                  ▼
Governed lifecycle
promote | quarantine | revoke | rollback
                  │
                  ▼
Export and terminal publication
ExportEnvelopeV3 + child receipts → AiDENsRunBundleV3 → index
                  │
                  ▼
Replay/evaluate next iteration
only admitted, scope-valid, non-revoked procedures are selectable
```

### Why this is “full closed loop”

The loop is closed only when a later run can select a previously promoted procedure, execute under the same authority/verification boundaries, detect regression or drift, and revoke/rollback it. Candidate generation alone is not closure. Passing tests alone is not closure. Writing a memory record alone is not closure.

### Why agent-graph is absent from runtime

The council used independent agent lanes to pressure-test the design. Runtime adoption of agent-graph is blocked by the binding NO-GO. A future certified agent-graph may replace only control-flow/checkpoint mechanics; it may not own payloads, memory, provider identity, verification, receipts, or promotion.

---

## 7. State, artifact, and success model

### 7.1 State classes

**Control state** (AiDENs/Forge; non-authoritative): current stage, retries, deadlines, in-flight handles, operator interrupt, derived status.

**Domain state** (canonical): memory/procedures, verification cases/dispositions, CEA graph, claim ledger, Forge envelope.

**Derived state** (rebuildable): indexes, cached routing weights, graph checkpoints, summaries, UI projections.

A stale or unverifiable derived artifact is never authority.

### 7.2 Lifecycle stages

Every run records these independently:

`proposed → authorized → attempted → executed → verification_pending → verified | degraded | failed → attributed | causal_unavailable → candidate_quarantined → tested → promoted | retained_quarantined → revoked | rolled_back`

These are event stages, not one mutable status field. Event history is append-only.

### 7.3 Terminal projection

Create a **non-authoritative derived projection** in `AiDENs/crates/aidens-contracts/src/coding_learning.rs` named `CodingLearningTerminalStateV1` with these closed variants:

- `SucceededVerified`
- `SucceededDegraded`
- `BlockedPolicy`
- `BlockedMissingPermit`
- `BlockedSandboxUnavailable`
- `BlockedReceiptPersistence`
- `BlockedEvidenceInsufficient`
- `BlockedStaleEvidence`
- `BlockedRevoked`
- `FailedExecution`
- `FailedVerification`
- `FailedRollback`
- `AbortedCancelled`
- `InterruptedAwaitingApproval`
- `Quarantined`
- `BudgetExhausted`
- `ReplayUnavailable`
- `ReplayMismatch`
- `ReplayDrift`
- `ProviderUnavailable`
- `MockOnly`
- `FixtureOnly`

This projection must store reason codes and canonical backpointers. It must never overwrite owner dispositions.

### 7.4 Success predicate

A run is `SucceededVerified` only if all conditions are true:

1. execution mode is `real_sandbox`;
2. preflight receipt was durably persisted before the effect;
3. every effect has a valid, unexpired, one-shot, scope/method/arguments/policy/attempt-bound permit;
4. typed patch validation and application succeeded in the staged workspace;
5. every required check was actually executed by the sealed backend;
6. verification adjudication is positive and neither advisory nor degraded;
7. source commit/tree, task, policy, tool registry, environment, patch, check, and verifier digests are present and valid;
8. terminal child receipts are durable and digest-valid;
9. no blocked, degraded, revoked, stale, mock, fixture-only, or indeterminate state exists;
10. bundle publication and index publication completed.

Agent text, provider status, attempted commands, green-looking stdout, and receipt shape contribute zero to this predicate without owner receipts.

---

## 8. Lineage and persistence contract

### 8.1 Required lineage

Each terminal `AiDENsRunBundleV3` must bind or point to:

- material `run_id`, `attempt_family_id`, `attempt_id`, `trial_id`, and `TraceCtx`;
- task specification and corpus-manifest digests;
- repository commit and complete tree/worktree digest;
- provider/model/config digest and actual route/mode;
- memory state generation and witnessed retrieval receipt;
- selected baseline/candidate procedure or routing policy digest;
- tool-registry/schema digest;
- policy snapshot and permit-use receipt IDs;
- sandbox image/config/workspace identity;
- patch digest and before/after tree digests;
- check plan, exact commands, outputs, exit status, and check receipt digests;
- verification case, attempt, calibration, adjudication, and control receipt IDs;
- CEA run hash/store record or explicit `causal_unavailable`;
- procedural candidate and lifecycle receipt IDs;
- `ExportEnvelopeV3` ID/digest;
- replay mode and replay handle/status;
- terminal projection with reason codes.

Use `CanonicalBackpointerV1::artifact` or `::external` for owner-native IDs. Do not copy owner payloads into a new AiDENs truth schema.

### 8.2 Material identity

`AiDENsRunBundleV3::new` currently uses `display_only_unstable_id`. Add a material-bound constructor using existing `generated_artifact_id_from_material` after canonical input material is available. Durable receipts, permits, bundles, replay handles, and canonical backpointers must mechanically reject display-only IDs.

### 8.3 Publication protocol

1. Persist run preflight and allocation receipts.
2. Persist tool/permit/sandbox/check receipts as each effect occurs.
3. Persist verification, CEA, procedure-lifecycle, and export receipts.
4. Verify every child digest and closed reference.
5. Write the terminal bundle to a temporary file, fsync, then atomically rename.
6. Append/fsync the index record last.
7. On startup, reconcile bundle-without-index, index-without-bundle, incomplete preflight, effect-without-outcome, and corrupt trailing records.
8. Never map recovery uncertainty to success; quarantine effects that cannot be reconciled.

### 8.4 Replay modes

- `no_replay`: retain IDs, digests, lineage, outcome, and explicit `ReplayUnavailable`; do not retain sensitive input.
- `store_inputs`: opt-in retention of canonical task input, filters, repository reference, policy/config/environment manifests, and required artifacts.
- `replay_evaluation`: re-execute retained input and compare normalized semantic outcome, patch/tree equivalence, verifier drift, reward vector, and attribution.

Replay result is exactly one of `replay_match`, `replay_mismatch`, `inconclusive`, or `not_available`.

---

## 9. Learning design

### 9.1 Learnable surfaces

**Procedure candidate** (`ProceduralMemoryArtifactV1`): may change preconditions, allowlisted tool sequence, schema-valid arguments, step order, bounded retries, rollback declaration, and applicability predicates.

**Routing candidate** (`semantic-memory::rl_routing::RoutingPolicy`): may change only bounded retrieval stage thresholds, enablement/order where supported, exploration probability, and retrieval budgets. It remains shadow-only in this MVP until a separate immutable lifecycle and promotion adapter are proven.

### 9.2 Forbidden learning surfaces

Model weights, arbitrary source, tool inventory, authority scope, permit policy, verification/evaluator code, tests/holdouts, receipt validation, CEA implementation, reward weights, promotion thresholds, and release controls.

### 9.3 Candidate extraction

Candidate extraction is deterministic and runs only after a completed witnessed run. It:

1. verifies all required lineage and receipt digests;
2. normalizes volatile data;
3. identifies reusable verified step subsequences and applicability predicates;
4. rejects unknown tools, unbounded commands, secrets, policy/evaluator changes, missing provenance, or unauthorized files;
5. constructs an immutable `ProceduralMemoryArtifactV1`;
6. deduplicates by canonical artifact digest;
7. calls `compile_procedure`, which may only compile or quarantine;
8. calls existing fixture simulation as a static gate, clearly labeled `fixture`;
9. sends effectful candidates to the real sandbox evaluator;
10. keeps the candidate quarantined until experiment and lifecycle gates pass.

Model explanations are hypotheses and contribute zero reward.

### 9.4 Reward vector

Store the vector, not only a scalar:

`(correctness, verification, causal_attribution, scope, efficiency, safety, reproducibility, regression)` in `[-1,1]`.

For ranking only:

`S = 0.30 correctness + 0.20 verification + 0.20 causal + 0.10 scope + 0.05 efficiency + 0.10 safety + 0.05 reproducibility - 0.40 regression_indicator`.

Hard overrides:

- safety or unauthorized effect → reject/revoke;
- missing verification → no promotion;
- missing causal attribution → quarantine;
- holdout regression → no promotion;
- efficiency never compensates for correctness or safety.

Reward components must be derived from independent receipts, never model/user self-score.

### 9.5 Experiment design

- Freeze task fixture, repository commit, tool allowlist, verifier manifest, resource budget, and provider/model config.
- Pair baseline and candidate on the same task with randomized order from a recorded seed.
- Partition by repository/template/semantic family: 60% development, 20% calibration, 20% untouched holdout.
- Use McNemar for paired correctness, paired bootstrap for cost/latency, Beta posteriors for signature outcomes, and family-clustered intervals.
- Predeclare the stopping rule; do not repeatedly peek.
- Include every failed/missing run in denominators.

### 9.6 Promotion thresholds

Procedure `candidate → tested` requires complete provenance, deterministic digest, static gates, at least 3 CEA-attributed samples per changed signature, and zero hard safety violation.

Procedure `tested → promoted` requires:

- at least 20 paired trials across at least 5 task families;
- all required verification checks pass in every promotion sample;
- zero unauthorized effects and zero evaluator/test suppression;
- 95% posterior lower bound for correctness at least 0.90;
- 95% posterior lower bound for non-regression at least 0.95;
- holdout degradation no worse than 2 percentage points;
- primary correctness improvement at least 5 points, or at least 10% efficiency gain with no correctness loss;
- replay semantic agreement at least 95%;
- no family worse than baseline by more than 5 points.

Routing promotion is out of scope until at least 100 eligible examples, 30 per materially changed bucket where feasible, at least 3% holdout utility improvement, no correctness regression, and a governed immutable routing lifecycle exist.

### 9.7 Automatic quarantine/revocation

Quarantine on missing/unverifiable receipt, low attribution, negative-case failure, replay mismatch, schema/tool/authority/verifier drift, unstable family performance, concentrated reward source, or disappearing paired benefit.

Revoke on unauthorized effect, secret leakage, destructive action, evaluator manipulation, two verified regressions in distinct families, holdout lower bound more than 2 points below baseline, receipt integrity failure, poisoning, or invalidating dependency/schema drift. Revocation removes selection eligibility immediately but preserves append-only history.

---

## 10. Sandbox and threat model

### 10.1 Required POC sandbox profile

Rootless Podman invocation must enforce:

- `--network=none`;
- read-only base image and no privileged mode;
- dropped capabilities;
- no host credentials, SSH agent, Docker socket, home directory, or ambient environment;
- explicit executable/tool allowlist and pinned image digest;
- one controlled input fixture mount and one controlled output/receipt path;
- cgroup v2 CPU/memory/PID limits, wall deadline, disk/file-count/output caps;
- process-group and descendant cleanup;
- before/after tree and external-effect snapshots;
- no host fallback in sealed mode.

`sandbox-workspace` remains the path-safe staging primitive inside this boundary. It is not the OS boundary.

### 10.2 Threats and controls

| Threat | Control | Expected failure state |
|---|---|---|
| path/symlink escape | `sandbox-workspace`, mount isolation, hostile fixtures | `BlockedSandboxUnavailable` |
| shell/network/credential escape | sealed Podman, no network/ambient env, deny tests | `BlockedSandboxUnavailable` |
| permit replay/wrong args | one-shot digest/method/scope/attempt binding | `BlockedMissingPermit` |
| test/evaluator tampering | pinned verifier manifest, forbidden-path diff | `Quarantined` |
| receipt loss after effect | child-first protocol and crash reconciliation | `BlockedReceiptPersistence` or `Quarantined` |
| self-confirming memory | origin separation and independent adjudication | `BlockedEvidenceInsufficient` |
| poisoned training data | witnessed runners, deduplication, source diversity, immutable holdout | `Quarantined` |
| reward hacking | external state checks, hidden negatives, no learner access to reward/policy | `FailedVerification` |
| stale/revoked evidence | bitemporal generation and revocation checks | `BlockedStaleEvidence` |
| mock/fixture confusion | explicit execution mode and forbidden success mapping | `MockOnly` or `FixtureOnly` |
| replay under drift | retained manifests and semantic comparator | `ReplayDrift` |
| rollback failure | snapshot/restore proof and quarantine | `FailedRollback` |

### 10.3 Mandatory failure injection

The POC test suite must inject store unavailability before action; crash after preflight; crash after effect; expired/replayed/wrong permit; path and symlink escape; grandchild process; network availability; real-to-mock fallback; malformed/duplicate-key/schema-invalid tool calls; boundary repair changing arguments; context/lineage loss; stale/revoked memory; missing replay inputs; changed-store replay; empty/reduced corpus; self-supporting claim; evaluator modification; terminal serialization failure; and rollback failure.

---

## 11. Operator experience

### 11.1 CLI surface

Extend existing `aidens agent` commands rather than create a competing binary:

- `aidens agent learn-run --spec <spec> --task <task> --out <dir> --mode real-sandbox`
- `aidens agent learn-inspect --run <run-dir>`
- `aidens agent learn-compare --baseline <id> --candidate <id> --corpus <manifest>`
- `aidens agent learn-candidate --run <id>`
- `aidens agent learn-promote --candidate <id> --permit-json <file>`
- `aidens agent learn-quarantine --candidate <id> --reason <text> --permit-json <file>`
- `aidens agent learn-revoke --candidate <id> --reason <text> --permit-json <file>`
- `aidens agent learn-replay --run <id> --mode <no-replay|store-inputs|evaluate>`
- `aidens agent learn-status --candidate <id>`
- `aidens agent learn-stop --run <id>`

These are planned commands; they do not exist at this checkpoint.

### 11.2 Required display

Every command and future UI shows:

- execution mode: `mock | fixture | dry_run | real_sandbox | real_host`;
- source HEAD/tree digest and dirty state;
- task/corpus split and fixture digest;
- provider/model/config and actual route;
- current stage and terminal projection;
- authority/permit scope and expiry;
- sandbox profile/image digest;
- checks attempted vs executed vs verified;
- evidence completeness and digest status;
- degraded/blocked/replay state;
- baseline/candidate policy IDs and assignment seed;
- reward vector and confidence intervals;
- candidate lifecycle, promotion reason, rollback/revocation state;
- receipts/backpointers and unresolved proof debt.

### 11.3 Human approval boundaries

Human approval is mandatory for patch application in non-fixture repositories, real command execution, candidate promotion, rollback/revocation override, replay with retained sensitive inputs, and any scope/tool/budget change. The model cannot grant or broaden authority.

The default screen is a live run summary—not raw logs. Raw outputs are linked as evidence; they do not substitute for structured receipts. Emergency stop must terminate descendants, freeze publication as interrupted/quarantined, and preserve partial evidence.

---

## 12. Immutable benchmark corpus

Create `AiDENs/fixtures/learning-coding-agent/v1/` with:

- `manifest.json`: corpus version, immutable fixture digests, family, split, required checks, language/toolchain, expected permitted effects, forbidden effects, baseline policy, verifier manifest, and provenance;
- `oracles.json`: operator-authored expected behavior and forbidden outcomes;
- `tasks/<family>/<case>/`: minimal repositories or deterministic source fixtures;
- `negative/`: path escape, symlink, process, network, secret, schema drift, evaluator tamper, stale dependency, failing baseline, malformed fixture, rollback, and reward-hacking cases;
- `README.md`: claim boundary and immutable versioning rule.

After any treatment observes v1, never edit it in place. Fixes require v2 with new digests and an explicit supersession note.

---

## 13. Zero-guesswork implementation plan

### Dependency graph

`0 → 1 → 2 → 3 → 4 → 5 → 6 → 7 → 8 → 9 → 10 → 11 → 12`; Task 13 follows Task 12; Task 14 requires all preceding tasks. Tasks 4 and 5 may be developed in parallel after Task 3 if they do not overlap files. Agent-graph work is not in this graph.

### Task 0: Isolate an attributable implementation baseline

**Owner:** controller/release engineer  
**Files:** inspect all current changes; create implementation worktree only after attribution; do not edit MVP source in the current mixed tree.  
**RED:** run `git submodule status --recursive`; expected current failure mentions missing `cea-bridge` mapping. Record branch, HEAD, porcelain, status digest, root/AiDENs lockfile digests, and nested workspace metadata.  
**GREEN:** select a clean fixed base/worktree; reconcile `.gitmodules` without deleting user work; write a baseline receipt under `target/learning-agent/preflight/`.  
**Focused gate:** `git status --short --branch`; `git rev-parse HEAD`; metadata for root and `AiDENs/Cargo.toml`.  
**Evidence:** immutable preflight JSON with command, cwd, exit, stdout/stderr digests, revision, status digest, lockfile digests.  
**Migration:** none.  
**Rollback:** delete only the newly created clean worktree after preserving receipts.  
**Licensed claim:** implementation started from an attributable source generation; no product claim.

### Task 1: Repair hostile-guard source coverage

**Owner:** AiDENs tooling  
**Files:** modify `AiDENs/scripts/p30_guard.py`; add/update its tests under `AiDENs/scripts/tests/`.  
**RED:** fixture places a known-hard pattern under `AiDENs/crates/aidens-runner/...`; current guard fails to emit a hard finding or missing-target error.  
**GREEN:** discover workspace roots, normalize targets relative to each root, fail when configured target paths do not exist, and self-test every hard rule with a known-bad fixture.  
**Focused gate:** `python3 -m pytest AiDENs/scripts/tests -k p30_guard -q`.  
**Integration gate:** `python3 AiDENs/scripts/p30_guard.py --repo . --json`; run separately for root and AiDENs workspace.  
**Evidence:** JSON guard receipt includes discovered roots, target count, missing targets, rule coverage, and findings.  
**Migration:** retain historical rule names and add normalized paths.  
**Rollback:** revert guard logic; do not reuse old zero-hard output as evidence.  
**Licensed claim:** guard coverage is mechanically exercised; not containment certification.

### Task 2: Make durable V3 lineage material-bound

**Owner:** `aidens-contracts`  
**Files:** modify `AiDENs/crates/aidens-contracts/src/agent_bundle.rs`, `src/lib.rs`, `src/schema_catalog.rs`, `src/tests.rs`; create `src/coding_learning.rs` for the non-authoritative terminal projection.  
**RED:** construct identical run material twice and show current V3 `bundle_id` differs/process-counters; attempt to persist a display-only durable ID; attempt `SucceededVerified` with a missing/degraded owner receipt.  
**GREEN:** add a material-bound V3 constructor using `generated_artifact_id_from_material`; reject `local-process-seq` IDs in durable fields; add canonical external backpointers for task, source tree, policy, sandbox, patch, checks, verification, CEA, procedure lifecycle, Forge envelope, replay, and terminal projection; implement the fail-closed terminal projection. Keep V3 wire compatibility.  
**Focused gate:** `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-contracts`.  
**Schema gate:** regenerate/check the schema catalog and roundtrip existing V3 fixtures.  
**Evidence:** deterministic V3 fixtures and schema-diff receipt.  
**Migration:** dual-read existing V3; only new writers require material IDs. Do not create V4 unless a committed field-gap matrix proves V3 cannot carry a required owner reference.  
**Rollback:** switch new writer off; old V3 reader remains.  
**Licensed claim:** new V3 bundles have material-bound local identity and typed owner references; owner truth remains external.

### Task 3: Make run-bundle publication crash-safe

**Owner:** `aidens-receipts`  
**Files:** modify `AiDENs/crates/aidens-receipts/src/lib.rs`; tests in the same crate.  
**RED:** inject failures between child receipt, bundle rename, and index append; produce bundle-without-index, index-without-bundle, corrupt tail, and effect-without-outcome cases.  
**GREEN:** implement child-reference verification, fsync/atomic rename, index-last publication, startup reconciliation, durable pending/indeterminate states, and terminal failure on persistence error. Never delete evidence after an effect merely because index append failed.  
**Focused gate:** `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-receipts`.  
**Integration gate:** runner crash-recovery tests reopen a fresh store and classify every injected state.  
**Evidence:** recovery matrix JSON plus canonical digest-chain verification.  
**Migration:** existing indexes remain readable; recovery scans existing bundle directories.  
**Rollback:** disable new writer but retain recovery reader and quarantined artifacts.  
**Licensed claim:** tested publication/recovery semantics, not filesystem durability on every platform.

### Task 4: Enforce the real sealed execution backend

**Owner:** `check-runner` + AiDENs tool adapter  
**Files:** modify `Primitives/check-runner/src/lib.rs` and backend modules; modify `AiDENs/crates/aidens-tool-kit/src/exposure.rs` and dispatcher wiring as required; tests under both crates.  
**RED:** sealed mode currently permits/attempts host fallback or does not prove network/env/process-tree denial. Add tests for no container runtime, network egress, credential/env inheritance, symlink escape, grandchild process, fork pressure, timeout, output cap, and host fallback.  
**GREEN:** require rootless Podman by pinned profile/image digest, `--network=none`, dropped capabilities, cleared environment, cgroup limits, descendant cleanup, controlled mounts, before/after tree snapshots, and no sealed-mode host fallback. Emit preflight and outcome receipts.  
**Focused gates:** `cargo test -p check-runner`; `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-tool-kit`.  
**Hostile gate:** execute all denial fixtures and verify no outside-root changes/network success.  
**Evidence:** sandbox capability-truth receipt with mechanism enumeration and denial outcomes.  
**Migration:** host backend remains available only for explicitly non-sealed modes; existing callers must choose mode.  
**Rollback:** disable `real_sandbox` if Podman/profile proof is unavailable; fail closed.  
**Licensed claim:** bounded rootless-Podman profile passed specified hostile tests; not general untrusted-code containment.

### Task 5: Add real effectful procedure evaluation

**Owner:** `aidens-runner` composition; semantic-memory retains lifecycle ownership  
**Files:** create `AiDENs/crates/aidens-runner/src/learning.rs`; modify `lib.rs`, `execution.rs`, `finalization.rs`, `receipts.rs`; minimally extend `semantic-memory/src/procedural_memory.rs` only for owner-native references if required.  
**RED:** prove existing `test_procedure` can return passed without invoking a command; show promotion input has fixture-only evidence.  
**GREEN:** keep `test_procedure` labeled fixture simulation; add a separate runner path that resolves a quarantined procedure, stages a frozen fixture, executes only allowed steps through Task 4, independently verifies effects, checks rollback, and returns owner receipt IDs to V3. Fixture-only evidence cannot satisfy effectful promotion.  
**Focused gates:** semantic-memory procedural lifecycle tests; `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-runner -k learning` or exact Rust test filters.  
**Integration gate:** one safe procedure executes in Podman; one fixture-only and one forbidden-effect procedure remain quarantined.  
**Evidence:** pre/post snapshots, tool/check/verification receipts, procedure test/lifecycle receipt references.  
**Migration:** existing `test_procedure` behavior and schema stay unchanged; new real evaluation is additive.  
**Rollback:** disable real evaluator; candidates remain quarantined.  
**Licensed claim:** real sandbox evaluation is distinct from fixture simulation.

### Task 6: Implement deterministic candidate extraction

**Owner:** `aidens-runner` composition + `semantic-memory` candidate owner  
**Files:** extend `AiDENs/crates/aidens-runner/src/learning.rs`; add tests; use existing `semantic-memory/src/procedural_memory.rs` APIs.  
**RED:** attempt extraction from mock, degraded, missing-receipt, secret-containing, unknown-tool, authority-widening, evaluator-modifying, and non-reproducible runs.  
**GREEN:** verify lineage/digests, normalize volatile fields, structurally extract allowlisted steps/preconditions/rollback/applicability, scan forbidden content, deduplicate by artifact digest, call `compile_procedure`, and preserve quarantine-first state.  
**Focused gate:** runner candidate-extractor tests.  
**Integration gate:** repeated extraction from the same verified run returns one idempotent candidate; tampered lineage is rejected.  
**Evidence:** source receipt IDs, artifact digest, compile lifecycle receipt, rejection reason codes.  
**Migration:** none; no active procedure is changed.  
**Rollback:** revoke/quarantine extracted candidates; append-only evidence remains.  
**Licensed claim:** deterministic quarantined procedure extraction from witnessed runs.

### Task 7: Wire paired experiments, reward adjudication, and CEA

**Owner:** `living-memory` experiment + CEA owners + runner adapter  
**Files:** modify `living-memory/living-memory/src/experiment.rs` only where existing structures lack immutable assignment/manifest fields; modify/create runner learning adapter; use `Primitives/cea-core/src/attribution.rs` and `Primitives/cea-store/src/lib.rs`; add tests.  
**RED:** candidate compared by historical before/after, self-reported reward, changed verifier, changed budget, duplicate run, unrandomized order, or synthetic `cea-bridge` telemetry.  
**GREEN:** freeze baseline/candidate/task/verifier/environment digests, randomize paired order from a recorded seed, derive reward vector from owner receipts, compute CEA run hash/attribution, update CEA store idempotently, and report all failures/missing trials. Mark causal unavailable explicitly.  
**Focused gates:** `cargo test -p forge-engine`; `cargo test -p cea-core`; `cargo test -p cea-store`; runner adapter tests.  
**Statistical gate:** golden fixtures verify McNemar inputs, paired bootstrap determinism under seed, Beta posterior, and family clustering.  
**Evidence:** immutable assignment, trial receipts, reward vector, CEA record, analysis report with denominators.  
**Migration:** old experiment records remain readable; new fields are versioned/defaulted.  
**Rollback:** candidate remains quarantined; baseline remains selected.  
**Licensed claim:** controlled paired evidence and attribution, not general causal proof.

### Task 8: Implement governed promotion/quarantine/revocation adapter

**Owner:** semantic-memory lifecycle; runner only requests and records  
**Files:** extend runner learning adapter; use `semantic-memory/src/procedural_memory.rs`; tests in both workspaces if owner APIs need strengthening.  
**RED:** promote with fixture-only evidence, missing adjudication, degraded verification, low sample/family count, holdout regression, expired/wrong principal permit, duplicate key with different material, or CEA confidence alone.  
**GREEN:** compute threshold decision from immutable experiment receipts; require explicit `ProcedureLifecyclePermitV1`; call only `promote_procedure`, `quarantine_procedure`, `revoke_procedure`, or `rollback_procedure`; invalidate selection caches immediately on revoke/rollback; persist canonical lifecycle backpointer.  
**Focused gates:** semantic-memory procedural tests and runner promotion tests.  
**Integration gate:** promotion drill, distinct-family regression revocation drill, rollback drill, stale-cache denial.  
**Evidence:** lifecycle permit, decision rationale, lifecycle receipt, cache invalidation receipt.  
**Migration:** no direct writes to active memory; append/supersession only.  
**Rollback:** call governed rollback/revoke; never delete history.  
**Licensed claim:** lifecycle transitions are permit-gated and evidence-bound.

### Task 9: Implement explicit replay and drift comparison

**Owner:** runner replay adapter + semantic-memory governed replay  
**Files:** modify `AiDENs/crates/aidens-runner/src/replay.rs`; extend `AiDENs/crates/aidens-receipts/src/lib.rs` as required; use semantic-memory governed replay APIs; tests.  
**RED:** rerun a prompt without retained inputs, changed verifier/current store, missing environment, or volatile timestamp differences and observe false match/false historical claim.  
**GREEN:** support exact three modes, retain inputs only by opt-in, compare normalized semantic outcome/tree/patch/reward/attribution, classify verifier/store drift, and emit exact replay state.  
**Focused gate:** runner replay tests.  
**Integration gate:** one match, mismatch, drift, inconclusive, and unavailable case in a fresh process.  
**Evidence:** original/replay digests, retention mode, drift fields, equivalence rule, result.  
**Migration:** old metadata-only runs map to `ReplayUnavailable`, never silent replay.  
**Rollback:** disable stored-input replay and preserve metadata-only mode.  
**Licensed claim:** replay capability only for runs with retained required inputs.

### Task 10: Add operator CLI and fail-closed status rendering

**Owner:** `aidens-cli`  
**Files:** modify `AiDENs/crates/aidens-cli/src/lib.rs` and `src/agent.rs`; tests in the CLI crate.  
**RED:** command reports completion for mock/fixture/degraded/missing-receipt run; lifecycle operation lacks permit; stop leaves descendants; inspect hides blocked checks.  
**GREEN:** add the planned `learn-*` commands; render mode, source, scope, phase, evidence, degradation, replay, candidate, confidence, promotion/revocation, and receipts; require lifecycle permit files; stop via backend process-tree control; return nonzero for every non-verified terminal state where command semantics require success.  
**Focused gate:** `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-cli`.  
**Smoke gate:** run help, dry-run, real-sandbox, inspect, compare, promote-denied, promote-approved, replay, revoke, and stop flows.  
**Evidence:** CLI golden snapshots plus run directories.  
**Migration:** existing `agent run`, `run-coding-agent`, and inspect commands remain compatible.  
**Rollback:** hide new commands behind an explicit experimental feature; no state loss.  
**Licensed claim:** operator can inspect and control the bounded loop without reading raw logs.

### Task 11: Build and freeze corpus v1

**Owner:** evaluation/release engineer independent from learner  
**Files:** create `AiDENs/fixtures/learning-coding-agent/v1/{manifest.json,oracles.json,README.md,tasks/,negative/}` and corpus validation tests.  
**RED:** duplicate/leaked family across splits, altered fixture after manifest, empty denominator, missing negative category, learner-readable holdout oracle, or unpinned verifier.  
**GREEN:** create at least five task families for the MVP, immutable 60/20/20 family-aware splits, negative suite, pinned digests, duplicate/leak detector, denominator assertions, and v1 immutability rule.  
**Focused gate:** deterministic corpus validator run twice with identical digest.  
**Integration gate:** baseline and candidate runner can consume manifests without accessing holdout oracle content.  
**Evidence:** corpus digest, split summary, family counts, leak-check receipt.  
**Migration:** never edit observed v1; supersede with v2.  
**Rollback:** quarantine corpus version; no candidate can cite it for promotion.  
**Licensed claim:** immutable local evaluation corpus, not external benchmark generalization.

### Task 12: Close the real procedural-learning vertical slice

**Owner:** controller integration  
**Files:** integrate Tasks 2–11 through `aidens-runner`, `aidens-cli`, owner crates, and fixtures; add end-to-end tests under the existing relevant crate/test layout.  
**RED:** start with no promoted procedure; run all failure injections and prove none can report verified success or promotion.  
**GREEN:** run baseline → real sandbox patch/check → independent verification → CEA → quarantined procedure → paired evaluation → approved promotion → later selection → injected regression → revocation/rollback → replay.  
**Focused gate:** one deterministic fixture end to end.  
**Integration gates:** all targeted package tests; root and nested workspace checks; hostile failure matrix.  
**Evidence:** one complete run tree with all child receipts, V3 bundle, Forge V3 export, lifecycle receipts, replay result, and revocation drill.  
**Migration:** experimental feature/default-off until gate passes.  
**Rollback:** disable feature and revoke every experimental promoted procedure.  
**Licensed claim:** full bounded local procedural-learning loop on frozen fixtures.

### Task 13: Add shadow-only routing evaluation

**Owner:** semantic-memory routing + evaluation adapter  
**Files:** modify `semantic-memory/src/rl_routing.rs`; runner adapter/tests; do not activate production routing.  
**RED:** current `is_trained` permits use after 10 examples; outcome caller can self-label; no policy version/holdout/drift lifecycle.  
**GREEN:** content-address immutable candidate policy, derive outcomes from verification receipts, require at least 100 examples/route-bucket/family gates, evaluate in shadow mode with 0% effect, and emit a promotion proposal only. No active route mutation in this MVP.  
**Focused gate:** semantic-memory routing tests.  
**Integration gate:** shadow predictions cannot affect executed route; poisoned/self-reported outcomes are rejected.  
**Evidence:** candidate policy digest, eligible denominator, holdout report, no-effect receipt.  
**Migration:** existing heuristic routing remains baseline.  
**Rollback:** delete/rebuild derived shadow cache; canonical receipts remain.  
**Licensed claim:** evaluated shadow routing candidate, not autonomous routing promotion.

### Task 14: Independent certification and release decision

**Owner:** independent hostile reviewer; controller fixes findings  
**Files:** no source changes during the final evidence run; write reports under a content-addressed evidence directory outside source or approved `evidence/` layout.  
**RED:** run complete gate from a dirty, source-mismatched, or unbound runtime and require BLOCK.  
**GREEN:** clean attributable tree; exact root/AiDENs/Primitives package/workspace commands; sandbox denial suite; crash recovery; corpus/evaluation; promotion/revocation/replay; schema compatibility; source/runtime hash binding; no unresolved P0/P1.  
**Commands:** `cargo fmt --all -- --check`; root package/workspace checks/tests/clippy as supported; `cargo check/test/clippy --manifest-path AiDENs/Cargo.toml`; targeted Primitives tests; P30 guard; corpus validator; end-to-end and failure matrix.  
**Evidence:** manifest of commands, exits, output digests, source/lock/image hashes, artifact inventory, and final `APPROVE_POC`, `APPROVE_MVP`, or `BLOCK`.  
**Migration:** none.  
**Rollback:** revoke experimental procedures, disable feature, preserve evidence.  
**Licensed claim:** only the exact decision and scope named by the independent report.

---

## 14. POC and MVP acceptance gates

### POC gate

All must pass:

- one real sandbox task completes with independently verified checks;
- every accepted artifact has complete run/task/source/policy/sandbox/patch/check/verifier lineage;
- duplicate writes are idempotent and tampering/wrong commit/wrong verifier are rejected;
- every negative case blocks or quarantines with zero unauthorized effect;
- fixture/mock modes can never map to verified success;
- candidate is quarantined before evaluation;
- explicit permit is required for promotion;
- revocation and rollback drill succeed;
- replay match plus unavailable/drift cases are correctly classified;
- crash recovery never invents success;
- terminal bundle and index digests verify in a fresh process.

### MVP evaluation gate

All POC gates plus:

- at least 20 paired trials across at least 5 task families;
- at least 3 CEA samples per changed signature;
- candidate correctness at least baseline +5 points, or equal correctness with at least 10% lower cost/latency;
- 95% lower correctness bound at least 0.90;
- holdout degradation no worse than 2 points;
- no family degradation over 5 points;
- replay semantic agreement at least 95%;
- p95 cost/latency regression no worse than 10% absent qualifying correctness gain;
- two consecutive clean evaluation windows;
- source diversity and poisoning checks pass;
- independent reviewer returns `APPROVE_MVP` from a clean tree.

### Production/autonomous release blockers retained after MVP

- `agent-guard` lacks real installed enforcement;
- generic untrusted-code containment is not certified;
- authority/permit coverage across every alternate executor is not proven;
- production routing lifecycle/promotion is absent;
- remote CI/deployment/release authority is out of scope;
- broad repository/language generalization is unproven;
- current dirty-tree and `.gitmodules` issues cannot support release certification;
- agent-graph direct integration remains NO-GO;
- no claim of production readiness until all blockers have independent evidence.

---

## 15. Validation gauntlet for implementation

Run from the correct workspace; do not assume root covers AiDENs:

```bash
# Root focused packages
cargo test -p sandbox-workspace
cargo test -p typed-patch
cargo test -p check-runner
cargo test -p cea-core
cargo test -p cea-store
cargo test -p forge-pilot
cargo test -p verification-control
cargo test -p verification-adjudication
cargo test -p semantic-memory
cargo test -p semantic-memory-forge

# Nested AiDENs workspace
cargo test --manifest-path AiDENs/Cargo.toml -p aidens-contracts
cargo test --manifest-path AiDENs/Cargo.toml -p aidens-receipts
cargo test --manifest-path AiDENs/Cargo.toml -p aidens-runner
cargo test --manifest-path AiDENs/Cargo.toml -p aidens-cli

# Workspace closure, only after focused gates
cargo fmt --all -- --check
cargo check --workspace --all-targets
cargo test --workspace --all-targets
cargo clippy --workspace --all-targets -- -D warnings
cargo check --manifest-path AiDENs/Cargo.toml --all-targets
cargo test --manifest-path AiDENs/Cargo.toml --all-targets
cargo clippy --manifest-path AiDENs/Cargo.toml --all-targets -- -D warnings

# Project-specific
python3 AiDENs/scripts/p30_guard.py --repo . --json
# plus corpus validator, Podman hostile suite, end-to-end loop, crash recovery,
# promotion/revocation/replay drills, and receipt digest verification.
```

If a workspace command is not supported by the current manifest/features, record the exact invalid command and replace it with package-scoped commands; never report a command that did not run as passed.

---

## 16. Final design verdict

This design uses the existing library portfolio rather than building a new autonomous-agent platform. It closes the learning loop at the only defensible MVP boundary: **offline, receipt-grounded, independently verified procedural adaptation under explicit authority**. It preserves canonical ownership, blocks model self-confirmation, distinguishes simulation from execution, requires external sealed containment, and makes success a conjunction of durable witnessed facts rather than agent narrative.

**Implementation decision:** `APPROVE_POC_WITH_GATES`.  
**MVP decision:** `CONDITIONAL_APPROVE_AFTER_POC_AND_STATISTICAL_GATES`.  
**Production autonomous release:** `BLOCK`.
