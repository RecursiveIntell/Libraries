# Closed-Loop Learning Agent Complete Finish Plan

> **For Hermes:** Execute with `subagent-driven-development`, strict RED/GREEN, canonical-owner reuse, fresh-source verification, and a final hostile council. Do not connect corpus-oracle qualification evidence to procedure promotion.

**Goal:** Finish an operator-runnable, crash-resumable, receipt-grounded bounded exact-source learning agent from real sandbox execution through governed promotion, replay, rollback/revocation, terminal publication, and fresh-process verification.

**Architecture:** AiDENs remains the composition and CLI layer. `typed-patch` owns patch semantics, `check-runner` owns sealed execution/capability truth, CEA owns effect attribution, `living-memory`/Forge owns experiment evidence/export, `semantic-memory` owns procedure lifecycle/retrieval/replay retention, `forge-memory-bridge` owns projection transformation, `aidens-contracts` owns AiDENs terminal contracts, and `aidens-receipts` owns bundle/index publication. No new truth store, lifecycle authority, experiment authority, or receipt family may be introduced without a field-level owner-gap review.

**Target:** Bounded exact-source MVP. This plan does not claim cross-task learning, generalized coding skill, online RL, arbitrary untrusted-code containment, or production autonomy.

---

## 1. Planning checkpoint

Observed 2026-07-19:

- Repository: `/home/sikmindz/Coding/Libraries-medusa`
- Branch: `feat/medusa`
- HEAD: `44a0a68deb15727c7b6b9886f8b21a9621be0539`
- Worktree: clean before this plan; this plan is the only intended new file.
- Public learning surface: 10 commands — `run`, `inspect`, `compare`, `promote`, `revoke`, `rollback`, `replay`, `publish`, `terminal`, `stop`.
- Fresh bounded verification:
  - `cargo test -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-cli`: PASS.
  - Results include contracts 102 passed; receipts 20 passed; runner 83 passed and 2 ignored; CLI 69 passed and 1 ignored; integration/doc tests passed.
  - `cargo fmt --all -- --check`: PASS.
  - strict Clippy for the four affected crates with `-D warnings`: PASS.
  - Python v2 corpus tests: 14 passed.
  - Static v2 corpus validation: PASS as a non-executed validation, corpus digest `2736bcbf625b495187e8d96dbf6ce4140b21cd3a244d833b2294fc59a330fb3f`; zero sides executed in this planning pass.
- Historical live evidence exists for the pinned Podman image and 60 pairs/120 sides, but it is source-reported historical evidence, not reproduced in this planning pass.
- Pinned image: `localhost/aidens-rust-checks@sha256:96f6610f945d10b523a303848610bd6fbef241762c59c0e44d47af9089cb6d6b`.

Refresh branch, HEAD, status, dependency locks, image availability, and receipt digests at implementation preflight.

## 2. Verdict: how much is left

### Bounded exact-source target

| Lifecycle surface | Current state | Decisive evidence |
|---|---|---|
| Typed request/preflight/permit | Implemented | `aidens-cli/src/lib.rs:298-322`; `learning_controller.rs:156-187` |
| Sealed act/check/verify/CEA | Implemented; live test conditional | `learning_controller.rs:189-224` |
| Exact candidate artifact | Implemented | `learning_controller.rs:490+` |
| One-run candidate remains inactive | Implemented | controller returns `Tested`/pending rather than auto-promoting |
| Forge evidence persistence/readback | Implemented | controller evidence bundle path; publication adapter |
| V3 export/import/readback | Implemented as separate adapter | `learning_publication.rs:56-149` |
| Lifecycle promote/revoke/rollback | Implemented as owner adapters and CLI commands | `learning_lifecycle.rs`; CLI lifecycle dispatch |
| Governed exact retrieval and sealed replay | Implemented as runner library function | `learning_controller.rs:234-336` |
| Terminal bundle/index/projection closure | Implemented as explicit closure API | `learning_terminal.rs:198+` |
| Fresh terminal inspection | Implemented and fail-closed | `aidens-cli/src/lib.rs:1107+` |
| Operator replay command | Absent/stub | CLI returns unavailable |
| Operator compare command | Absent/stub | CLI returns unavailable |
| Quarantine command | Absent | no CLI variant despite owner adapter |
| Stop/cancellation owner | Absent | command is intentionally unsupported |
| Durable resume state machine across split phases | Partial/absent | normal run ends pending; later phases are manually sequenced |
| Promotion-grade candidate adjudication | Absent | corpus adapter returns raw owner results and explicitly does not adjudicate |
| Current-HEAD live closure receipt | Not reproduced in this pass | relevant Podman tests remain ignored by default |
| Operator runbook/release gate | Partial | learning lifecycle is absent from the quickstart and broad gate |

**Controller verdict:** the bounded proof has most canonical primitives, but the remaining work is concentrated at the highest-value boundary: composition, adjudication, recovery, and operator execution. A count-based view is roughly 10 source-level surfaces implemented, 3 partial, and 5 absent/unverified. That is not equivalent to 70% product completion: until the missing sequence is composed and proven, the operator has no complete closed loop.

### Product gates

- **POC:** Source-implemented and historically live-demonstrated, but current-HEAD live evidence must be refreshed before certifying this exact checkpoint.
- **Bounded MVP:** **NO-GO.** Public replay/compare are unavailable; no promotion-grade adjudication package exists; no durable resumable coordinator composes the full loop.
- **Production:** **NO-GO.** Generalization, broad containment, unattended authority, service operation, monitoring, upgrade/migration, and production incident recovery are outside current proof.

## 3. Council convergence

| Topic | Council agreement | Dissent | Controller decision |
|---|---|---|---|
| Core primitives | Strong canonical primitives exist | Plans overstate product-boundary completion | Treat source primitives as implemented, product loop as incomplete |
| Publication CLI | One lane called unconditional `published-verified` a P0 | `publish_terminal_evidence` returns only after exact Forge receipt and semantic-memory import readback checks (`learning_publication.rs:56-135`) | **Reject P0.** Add a CLI regression, but do not redesign the already fail-closed owner path |
| Statistics/promotion | Raw repeated-paired results are qualification-only | Existing 60-pair evidence may look promotion-grade | Require a new exact-candidate adjudication package; never promote from oracle corpus results |
| Replay | Runner replay exists | Public CLI replay is unavailable | Wire the existing canonical retrieval/replay path; do not create a second replay engine |
| Live verification | Prior live runs exist | Current audit did not reproduce them | Preserve historical receipts, add bounded preflight, rerun only closure smoke unless source-affecting changes require the 60-pair workload |
| Corpus default | CLI defaults to metadata-only v1 | v1 remains useful for legacy inspection | Make v1 explicitly inspection-only and v2 explicit/default for executable qualification |
| Stop | Fail-closed unsupported command is honest | Public command implies capability | Either implement a canonical cancellation owner or remove/hide `stop` from supported MVP claims |

## 4. Hard no list

- No promotion from one green run, fixture simulation, oracle corpus score, or CEA confidence alone.
- No model-weight updates.
- No authority/tool widening, evaluator edits, receipt-policy edits, verification edits, or holdout access by the learner.
- No AiDENs-local experiment, lifecycle, memory, export, receipt, or statistics truth store.
- No mock/fixture/degraded/unpersisted outcome mapped to verified success.
- No automatic lifecycle permit minting by the producer requesting promotion.
- No deletion or overwrite of owner evidence during rollback or recovery.
- No 60-pair rerun merely to inflate evidence; rerun only when the corpus/adapter/check backend/image contract changes or the user explicitly authorizes it.

---

## Phase 0 — Scope lock and current-state receipt

### Task 0.1: Add a learning completion ledger

**Owner:** documentation/evidence projection only.

**Files:**
- Create: `AiDENs/docs/learning-agent/COMPLETION_LEDGER.md`
- Create: `AiDENs/docs/learning-agent/claim-boundary.json`
- Test: `AiDENs/scripts/tests/test_learning_completion_ledger.py`

**RED:** Ledger validator fails if branch/HEAD, source digests, test counts, ignored live tests, promotion evidence state, or claim boundary are absent or stale.

**GREEN:** Generate a projection containing current HEAD/tree state, exact source-affecting file digests, historical-vs-current receipt labels, tests passed/ignored/skipped, and permitted claims. It must not become truth authority.

**Gate:** `python3 -m pytest -q AiDENs/scripts/tests/test_learning_completion_ledger.py`.

**Evidence:** completion ledger plus digest.

**Rollback:** delete the projection; owner receipts remain untouched.

## Phase 1 — Public command truth and corpus routing

### Task 1.1: Make v1 inspection-only and v2 executable by explicit contract

**Files:**
- Modify: `AiDENs/crates/aidens-cli/src/lib.rs:794-919`
- Modify: `AiDENs/crates/aidens-cli/src/tests.rs`
- Modify: `AiDENs/docs/OPERATOR_QUICKSTART.md`

**RED:** Executable learning operation using default/implicit v1 must fail with `metadata-only-corpus`; executable v2 must validate through the v2 owner consumer.

**GREEN:** Add a typed corpus version/mode boundary. Preserve v1 inspection. Default executable qualification to v2 or require explicit `--source`/`--corpus-version v2`.

**Gate:** CLI learning tests plus 14 Python v2 tests.

**Rollback:** restore explicit source requirement, never silently fall back to v1.

### Task 1.2: Harden publication and terminal CLI projections

**Files:**
- Modify: `AiDENs/crates/aidens-cli/src/lib.rs:1048-1190`
- Modify: `AiDENs/crates/aidens-cli/src/tests.rs`

**RED:** Inject missing Forge export receipt, incomplete semantic-memory import, wrong bundle ID, corrupt index, or wrong projection backpointer; CLI must return nonzero and never serialize `published-verified`/`succeeded-verified`.

**GREEN:** Derive rendered state from typed owner outcomes. Preserve current owner readback checks; add no duplicate publication semantics.

**Gate:** focused publication/terminal CLI tests.

**Rollback:** hide publication success rendering rather than weakening readback.

## Phase 2 — Canonical promotion-grade adjudication

### Task 2.1: Inventory the existing Forge/statistics contract family

**Owner:** non-mutating architecture gate.

**Files inspected:**
- `living-memory/living-memory/src/**`
- `semantic-memory/src/procedural_memory.rs`
- `semantic-memory-forge/src/**`
- `AiDENs/crates/aidens-runner/src/learning_experiment.rs`

**Gate:** produce a field-gap matrix proving whether an existing owner-native adjudication receipt can bind candidate, source, patch, verifier, environment, partitions, denominator, uncertainty, thresholds, and decision.

**Failure behavior:** If an owner exists, evolve/reuse it. If not, add a versioned owner contract in Forge/semantic-memory—not AiDENs.

### Task 2.2: Add exact-candidate evaluation/adjudication

**Files:** exact owner files determined by Task 2.1; thin AiDENs adapter only in `aidens-runner/src/learning_experiment.rs` or a narrowly named sibling module.

**RED:** Promotion package is rejected when any of candidate digest, patch, source, policy, verifier, environment, assignment, denominator, holdout, or permit differs. Total score passing while holdout/per-family gate fails must quarantine.

**GREEN:** Owner emits immutable assignment plus typed adjudication. Predeclare pair/family units, negative controls, stopping rule, uncertainty, thresholds, holdout non-regression, and rollback trigger. Exact-source candidate evaluation must be separate from evaluator-oracle corpus qualification.

**Gate:** owner tests plus runner adapter tests; no live workload yet.

**Evidence:** adjudication receipt and explicit `Eligible` or `Quarantined` decision.

**Migration:** additive version; existing candidates remain `Tested`/inactive.

**Rollback:** disable eligibility path; preserve all evidence.

### Task 2.3: Require adjudication in lifecycle promotion

**Owner:** `semantic-memory`.

**RED:** Existing candidate ID + valid lifecycle permit but missing/wrong adjudication package must fail.

**GREEN:** Promotion verifies the owner package, exact artifact digest, lifecycle predecessor, single-use independent permit, holdout decision, and policy version.

**Gate:** procedural-memory lifecycle tests including permit reuse and altered-digest denial.

**Rollback:** quarantine-only mode.

## Phase 3 — Durable coordinator and recovery

### Task 3.1: Define a versioned resumable workflow projection

**Owner:** orchestration projection, not truth.

**Files:**
- Create/modify under `AiDENs/crates/aidens-contracts/src/` only after confirming no overlapping workflow contract.
- Modify: `AiDENs/crates/aidens-runner/src/learning_controller.rs`

**States:** `Preflighted → ExecutedVerified → CandidateTested → EvidencePersisted → Published → Evaluated → Eligible/Quarantined → Promoted → Retrieved → Replayed → RolledBack/Revoked → TerminalPublished`; plus `Pending`, `Blocked`, `Indeterminate`.

**RED:** Crash after every transition cannot skip a predecessor, duplicate owner effects, or produce success.

**GREEN:** Coordinator resumes solely from owner receipts/IDs/digests. Projection remains rebuildable. Identical retry is idempotent; conflicting retry fails closed.

**Gate:** table-driven crash/failure injection tests.

**Rollback:** disable coordinator; retain split commands and owner evidence.

### Task 3.2: Wire publication into resume flow

**Files:**
- Modify: `aidens-runner/src/learning_publication.rs`
- Modify: coordinator/controller

**RED:** Crash after Forge export but before memory import; retry must reuse persisted export identity and complete exactly once.

**GREEN:** Reuse current `export_bundle → transform_envelope_v3 → import_projection_batch → query readback` path.

**Gate:** fresh-store recovery/idempotency/conflict tests.

## Phase 4 — Public governed retrieval and replay

### Task 4.1: Replace `learn replay` stub with typed exact-source replay

**Files:**
- Modify: `AiDENs/crates/aidens-cli/src/lib.rs:270-273, 973-981`
- Modify: `AiDENs/crates/aidens-cli/src/tests.rs`
- Reuse: `aidens-runner/src/learning_controller.rs:234-336`

**RED:** Missing retained material, wrong source, wrong patch, wrong action permit, expired/reused permit, wrong lifecycle receipt, verifier/environment drift, or revoked candidate cannot execute.

**GREEN:** Typed replay request carries store, artifact ID, expected source digest, separate action permit, and real-sandbox request. CLI calls the existing governed retrieval and sealed replay function and returns owner-linked receipts.

**Gate:** focused CLI/runner tests and one live pinned-Podman replay smoke.

**Evidence:** governed retrieval receipt, promotion receipt, new preflight/effectful receipt, replay result.

**Rollback:** restore unavailable state; do not add metadata-only fallback.

### Task 4.2: Replace `learn compare` stub with owner-result comparison

**RED:** Caller-supplied metadata without owner receipts cannot return a comparison claim.

**GREEN:** Compare only canonical original/replay receipts through the existing replay classification owner; produce `ExactMatch`, `Drift`, `Mismatch`, `Inconclusive`, or `NotAvailable`.

**Gate:** mutation matrix across source, patch, verifier, environment, store, input, and output.

## Phase 5 — Lifecycle controls

### Task 5.1: Add operator quarantine

**Files:**
- Modify: CLI enum/dispatch/tests
- Reuse: `ProcedureLifecycleAdapter::quarantine`

**RED:** Quarantine without correct independent permit or artifact binding fails.

**GREEN:** Add `learn quarantine` with typed permit and owner receipt returned unchanged.

**Gate:** lifecycle CLI and semantic-memory tests.

### Task 5.2: Close rollback and revocation denial paths

**RED:** After rollback/revoke, direct ID, governed retrieval, cache, search, and replay paths must all deny stale selection.

**GREEN:** Use canonical owner transitions and invalidate/recheck every access path without deleting history.

**Gate:** separate rollback and revoke drills; do not conflate their semantics.

### Task 5.3: Resolve `stop`

**Decision gate:** Identify a canonical active-run/process/container cancellation owner. If none exists, remove `stop` from supported learning claims and hide it behind an explicit unsupported/deferred surface.

**RED if implemented:** cancellation permit mismatch, missing run, descendant process, timeout, or repeated stop must fail/resolve idempotently with receipts.

**GREEN:** Terminate and observe the process/container tree, verify cleanup, emit canonical cancellation receipt, and leave lifecycle state semantically distinct from revoke/rollback.

## Phase 6 — Terminal publication and operator one-shot workflow

### Task 6.1: Wire the existing terminal closure API

**Files:**
- Reuse/modify: `aidens-runner/src/learning_terminal.rs`
- Modify: coordinator/controller and CLI

**RED:** Missing/duplicate/open/wrong-owner/wrong-role/digest-invalid child; bundle without index; index without bundle; corrupt log; wrong promotion/replay identity; or missing durable projection cannot reach `SucceededVerified`.

**GREEN:** Invoke `close_real_sandbox_terminal` only after publication, promotion, governed replay, and required owner receipts. Fresh store reads bundle/index/projection before zero exit.

**Gate:** existing closure tests plus new coordinator integration tests.

### Task 6.2: Add an operator-runnable `learn close`/`learn resume`

**RED:** Mixed run IDs, stores, namespaces, policies, or permit families fail before side effects.

**GREEN:** One typed request resumes the workflow from durable state, asks for no authority it does not possess, and stops at the next operator-required permit boundary. A fully supplied authorized request can complete the bounded loop.

**Evidence:** one terminal bundle/index/projection plus all child owner receipts.

## Phase 7 — Verification, release, and documentation

### Task 7.1: Add `verify_learning_agent.sh`

**Files:**
- Create: `AiDENs/scripts/verify_learning_agent.sh`
- Create: script tests
- Integrate: `AiDENs/scripts/verify_current.sh`

**Required classifications:** `PASS`, `FAIL`, `SKIP(reason)`. A skipped live test never licenses live/production claims.

**Gate contents:**
1. clean/source checkpoint;
2. format and strict Clippy;
3. contracts/receipts/runner/CLI tests;
4. v2 Python and Rust validators;
5. publication/recovery/lifecycle tests;
6. receipt provenance verification;
7. rootless Podman preflight;
8. optional live closure smoke.

### Task 7.2: Write the operator runbook

**Files:**
- Modify: `AiDENs/docs/OPERATOR_QUICKSTART.md`
- Create: `AiDENs/docs/learning-agent/RUNBOOK.md`

Document request schemas, commands, exit codes, pending/blocked/indeterminate states, permit boundaries, recovery, rollback/revoke, evidence locations, and forbidden claims.

### Task 7.3: Run the verification gauntlet

Run at final HEAD:

```bash
cargo fmt --manifest-path AiDENs/Cargo.toml --all -- --check
cargo test --manifest-path AiDENs/Cargo.toml -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-cli
cargo clippy --manifest-path AiDENs/Cargo.toml -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-cli --all-targets -- -D warnings
python3 -m unittest discover -s AiDENs/scripts/tests -p 'test_learning_corpus*.py' -v
python3 AiDENs/scripts/validate_learning_corpus_v2.py AiDENs/fixtures/learning-coding-agent/v2
bash AiDENs/scripts/verify_learning_agent.sh
cargo check --workspace
```

Then run the single live closure/replay smoke with the pinned image. Rerun the 60-pair workload only if its source-affecting digest changed or Josh explicitly authorizes it.

### Task 7.4: Final hostile council and commit

- Dispatch non-overlapping authority, recovery, statistics, containment, and operator lanes.
- Verify decisive citations against live HEAD.
- Resolve every P0/P1.
- Run `git diff --check`, secret/path leakage scan, status, and final log.
- Create scoped local commits; do not push unless separately requested.

---

## 5. Completion gates

### POC certified only when

- current-HEAD live sealed effectful run passes;
- exact candidate remains inactive without independent permit;
- owner evidence and publication read back successfully;
- claim remains exact-source only.

### Bounded MVP certified only when

- promotion requires exact-candidate adjudication and independent single-use authority;
- public governed retrieval and sealed replay work;
- rollback and revocation deny every stale access path;
- resume/recovery survives failure injection;
- terminal bundle/index/projection close and read back in a fresh process;
- operator CLI can execute/resume the entire sequence;
- current-HEAD release gate and hostile council pass with no P0/P1.

### Production remains blocked until a separate program proves

- multiple-family learner-generated treatment evaluation and credible held-out generalization;
- hostile real containment across filesystem, process, network, secrets, resources, and cleanup;
- unattended authority policy, incident response, monitoring, migrations, backups, upgrades, and revocation at service scale;
- external workload evidence and operational SLOs.

## 6. Evidence-safe final claim

> AiDENs provides a bounded, operator-authorized exact-source learning loop: a typed patch is executed and checked in a sealed local sandbox, attributed and persisted through canonical owners, retained as an inactive procedure, independently evaluated and promoted, retrieved under exact context and authority, replayed in a fresh sandbox, rolled back or revoked with stale-selection denial, and closed through durable bundle/index/projection readback. This does not establish cross-task learning, broad generalization, online RL, arbitrary untrusted-code safety, or production autonomous engineering.
