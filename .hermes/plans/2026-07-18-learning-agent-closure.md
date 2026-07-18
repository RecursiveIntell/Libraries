# Receipt-Grounded Learning Agent Closure Plan

> **For Hermes:** Execute with `subagent-driven-development`, strict RED/GREEN tests, canonical-owner reuse, controller-owned integration, and final hostile review.

**Status:** Council converged with a hostile **NO-GO**. Implementation is active only on the prerequisite repairs below; terminal success and promotion remain blocked until their named owner evidence exists.

**Goal:** Close the bounded real-sandbox learning-agent POC without promoting from insufficient evidence, then add a separately gated real multi-family evaluation path sufficient for an MVP promotion decision.

**Architecture:** AiDENs remains a thin composition layer. `semantic-memory` owns procedure lifecycle, `semantic-memory-forge::ExportEnvelopeV3` owns export format, `aidens-contracts::AiDENsRunBundleV3` owns operator composition, `aidens-receipts::RunBundleStore` owns child-first atomic bundle/index persistence, and rootless Podman/check-runner owns execution evidence. No parallel experiment engine, truth store, receipt database, lifecycle, or terminal schema will be created.

**Tech stack:** Rust, AiDENs nested workspace, semantic-memory, semantic-memory-forge, aidens-receipts, check-runner, CEA, verification crates, rootless Podman.

---

## Mutable planning checkpoint

Observed 2026-07-18 12:53 CDT:

- Repo: `/home/sikmindz/Coding/Libraries-medusa`
- Branch: `feat/medusa`
- HEAD: `744a812`
- Dirty files before this plan:
  - `AiDENs/crates/aidens-runner/src/learning_controller.rs`
  - `AiDENs/crates/aidens-cli/src/lib.rs`
  - `AiDENs/crates/aidens-cli/src/tests.rs`
- Latest verified gates before this plan: runner/CLI unit suites, strict clippy, full root workspace check, live controller and CLI rootless-Podman tests.
- Pinned image: `localhost/aidens-rust-checks@sha256:96f6610f945d10b523a303848610bd6fbef241762c59c0e44d47af9089cb6d6b`

Refresh this checkpoint before implementation and before any local commit.

## Scope lock

| Exact outcome | Governing source | Council lane | Non-substitute |
|---|---|---|---|
| Child-first terminal V3 publication and recovery | Prior plan §7.4, §8.3, Tasks 2–3/12 | Terminal architecture | A JSON report or lifecycle receipt alone |
| No promotion from one successful run | Prior plan Tasks 7–8 | Hostile integration | Green checks, CEA confidence, or fixture test |
| Real multi-family paired evaluation | Prior plan §4.3, Task 7/11/12 | Evaluation/benchmark | Metadata-only corpus or repeated identical fixture |
| Honest terminal projection | `aidens-contracts::CodingLearningEvidenceV1` | Terminal architecture + hostile review | CLI exit code or prose |
| Canonical export lineage | `semantic-memory-forge::ExportEnvelopeV3` | Terminal architecture | New local export envelope family |

## Verified current state

1. `AiDENsRunBundleV3::new_material_bound` and required learning-owner-role validation exist in `aidens-contracts`.
2. `RunBundleStore` already implements atomic bundle write, index-last append, digest chain, inspection, and reconciliation.
3. The real controller persists preflight and raw effectful terminal events, executes sealed checks, and verifies rollback/CEA. A single run deliberately does not write semantic-memory's effectful promotion prerequisite.
4. The prior dirty controller called `promote_procedure` after one effectful evaluation. The first correction used a locally minted quarantine permit; hostile review correctly rejected that as authority widening. The controller now compiles/tests the candidate and persists raw effectful evidence without creating a controlled lifecycle prerequisite or transition. The candidate remains `Tested` and inactive.
5. The prior blueprint embedded a hard-coded replacement unrelated to the executed patch. The blueprint now serializes the exact `StructuredPatch`, binds source/policy/image/capability/permit-use material, removes host-absolute fixture paths, and raises write risk from low to medium. Focused and live pinned-Podman regressions pass.
6. Executable corpus v2 now contains 15 digest-pinned Rust fixtures across five family-clustered 9/3/3 partitions. Python and Rust consumers reject drift, leakage, duplicate identities, public oracle material, incomplete fixtures, and unsafe paths.
7. A refreshed live receipt records 15 expected baseline failures and 15 treatment passes through the pinned sealed Podman image. It explicitly marks timing, paired statistics, and promotion evidence unavailable.
8. `living-memory` contains repeated-paired execution, but runtime adoption remains blocked by the binding agent-graph/Forge direct-integration NO-GO unless the canonical owner council identifies a separately authorized seam.

### V3 field-gap matrix

| Needed publication fact | Existing canonical surface | Gap/risk | Planned treatment |
|---|---|---|---|
| Material bundle identity | `AiDENsRunBundleV3::new_material_bound` | None observed | Reuse |
| Exact learning-owner roles | `validate_coding_learning_lineage` | None observed | Reuse |
| Closed child receipts | `AiDENsRunChildReceiptV1` plus `RunBundleStore` prepublication validation | Additive typed V3 closure implemented; terminal writer still lacks real owner children | Require closed digest-valid, unique-owner, canonically ordered exact child sets |
| Required-child equality | `AiDENsRunRequiredChildV1` and exact typed set validation | Generic V3 compatibility intentionally permits both lists absent | Coding-learning publication must supply both lists and pass exact equality |
| Atomic bundle/index write | `RunBundleStore::write_bundle_value` | Schema-named malformed V3 now fails typed deserialization before publication; coding-learning writer remains absent | Require a coding-learning writer to supply complete owner-native child closure before publication |
| Terminal projection | `CodingLearningTerminalProjectionV1` | Publication state is known only after bundle/index readback; naïve embedding is circular | Persist a deterministic post-publication projection receipt and support crash-resume |
| Forge export | `ExportEnvelopeV3` | No current run-bound export artifact in the controller | Build/validate a real owner envelope; no causal or paired fields unless owner evidence exists |

## Hard no list

- No one-run or fixture-only promotion.
- No fabricated Forge envelope, replay handle, corpus result, child receipt, or owner ID.
- No `SucceededVerified` before bundle and index read back successfully in a fresh store.
- No dynamic JSON substitute for an existing typed owner contract unless the existing V3 compatibility boundary explicitly requires a projection wrapper.
- No weakening sealed Podman constraints to improve latency.
- No direct agent-graph runtime adoption.
- No held-out/generalization claim from the metadata-only corpus.

---

## Phase 1 — P0 lifecycle correction

### Task 1.1: Replace one-run auto-promotion with an inactive Tested lifecycle

**Owner:** semantic-memory lifecycle; AiDENs runner forwards only.

**Files:**
- Modify: `AiDENs/crates/aidens-runner/src/learning_controller.rs`
- Test: controller unit/live tests in the same file

**RED:** Update/add a test proving one effectful verified run cannot return `ProcedureLifecycleDispositionV1::Promoted`.

**GREEN:** Compile and fixture-test the candidate while retaining the raw real-sandbox report in the canonical event log. A single run must not write semantic-memory's effectful promotion prerequisite. Leave the candidate `Tested` and unavailable for governed action retrieval.

**Focused gate:**
`cargo test --manifest-path AiDENs/Cargo.toml -p aidens-runner --lib learning_controller`

**Integration gate:** live pinned-Podman controller test proves execution remains verified while candidate is not promoted.

**Evidence:** lifecycle receipt disposition, artifact ID/digest, canonical preflight and raw terminal-event backpointers.

**Rollback:** revert the controller adapter; do not restore one-run promotion.

**Licensed claim:** one real run can generate and fixture-test an inactive procedure candidate while persisting raw effectful evidence; it cannot create the promotion prerequisite or authorize a lifecycle transition.

### Task 1.2: Bind the procedure payload to the exact executed material

**Owner:** typed-patch for patch representation; semantic-memory for the procedure artifact.

**RED:** Two different `StructuredPatch` values must produce different step payloads; no host-absolute fixture path or hard-coded operation may appear; artifact identity must include source tree, policy, image/capability, and permit-use identity.

**GREEN:** Serialize `config.patch` directly as the procedure step argument under `typed-patch:structured-apply:1`; bind exact source-tree digest, patch policy, sandbox image/capability digest, and permit-use receipt into artifact material; classify write risk as medium; use no fake 2999 expiry.

**Gate:** focused payload regression, semantic-memory artifact validation, and the live pinned-Podman controller test.

**Rollback:** quarantine/disable every artifact created by the earlier synthetic blueprint path; never make it action-selectable.

---

## Phase 2 — Terminal V3 publication

### Task 2.1: Produce canonical export and exact owner backpointers — **BLOCKED pending owner input/certification**

**Owner:** semantic-memory-forge export contract; AiDENs composition only.

**Files:**
- Modify: `AiDENs/crates/aidens-runner/Cargo.toml` only if no existing canonical re-export suffices
- Modify: `AiDENs/crates/aidens-runner/src/learning_controller.rs`
- Test: controller tests

**RED:** terminal bundle construction fails when any required role (`task`, `source-tree`, `policy`, `sandbox`, `patch`, `checks`, `verification`, `cea`, `procedure-lifecycle`, `forge-export-envelope-v3`, `replay`, `terminal-projection`) is absent, duplicated, or non-durable.

**GREEN:** Obtain a real owner-native Forge evidence bundle, call the canonical `forge_engine::export_bundle → ExportEnvelopeV3` path, validate/digest it, and construct exactly one durable backpointer per required role. The controller may not serialize its effectful report into a look-alike envelope.

**Blocker:** the real-sandbox controller currently has no `ForgeStore`/`ExperimentEvidenceBundle` input, and the binding agent-graph decision is NO-GO for direct Forge integration. Until a new certification or an already-authorized owner API supplies this evidence, terminal publication must remain pending.

**Gate:** focused runner tests plus semantic-memory-forge envelope validation/roundtrip.

**Migration:** additive V3 writer; existing readers remain valid.

**Rollback:** disable publication; preserve child receipts and leave terminal state pending.

### Task 2.2: Publish child-first bundle/index and derive terminal state after readback

**Owner:** `aidens-receipts::RunBundleStore` and `aidens-contracts` terminal projection.

**Files:**
- Modify: `AiDENs/crates/aidens-runner/src/learning_controller.rs`
- Modify only if contract gap proven: `AiDENs/crates/aidens-contracts/src/agent_bundle.rs`
- Tests: runner and aidens-receipts focused tests

**RED:** Missing child, digest mismatch, bundle-without-index, index-without-bundle, corrupt index tail, and crash after effect cannot yield `SucceededVerified` or CLI exit zero.

**GREEN:** Verify closed child receipts, build material-bound V3, write via `RunBundleStore`, reopen and inspect, reconcile recovery state, then derive/persist the terminal projection. Handle idempotent restart after bundle publication but before terminal projection.

**Gate:** `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-receipts -p aidens-runner` plus fresh-process recovery test.

**Evidence:** bundle path/digest/index record/recovery state/terminal projection receipt.

**Rollback:** keep bundle and children; mark pending/indeterminate; never delete effect evidence.

### Task 2.3: Make CLI terminal rendering evidence-driven

**Owner:** aidens-cli projection only.

**Files:**
- Modify: `AiDENs/crates/aidens-cli/src/lib.rs`
- Test: `AiDENs/crates/aidens-cli/src/tests.rs`

**RED:** CLI returns zero when lifecycle is promoted but terminal bundle/index is absent.

**GREEN:** Return zero only when canonical terminal projection is `SucceededVerified` and the referenced V3 bundle/index inspect successfully. Render quarantine separately from execution failure.

**Gate:** CLI unit suite and live pinned-Podman smoke test.

---

## Phase 3 — Real paired evaluation and promotion gate

**Council decision required:** Whether `living-memory::PairedExperimentRunner` may be adopted under the existing direct-Forge NO-GO. If blocked, this phase remains offline/advisory and cannot promote.

### Task 3.1: Replace metadata-only task descriptors with runnable immutable fixture packages — **IMPLEMENTED**

**Owner:** independent evaluation corpus.

**Files:** create a superseding corpus version under `AiDENs/fixtures/learning-coding-agent/v2/`; do not mutate observed v1.

**RED:** validator rejects non-runnable fixture, family leakage, oracle exposure, digest drift, duplicate patch, empty denominator, and baseline that does not exhibit the declared failure.

**GREEN:** At least five distinct Rust task families with frozen source tree, typed patch, declared baseline outcome, hidden evaluator/oracle separation, 60/20/20 family-aware partitions, and canonical digests.

**Gate:** deterministic validator twice with identical digest; each baseline and candidate executes through sealed Podman.

**Claim:** immutable local benchmark corpus, not external generalization.

### Task 3.2: Run at least 20 paired trials across five families — **BLOCKED pending canonical owner certification**

**Owner:** existing canonical experiment substrate if certified; otherwise blocked.

**RED:** changed verifier/environment/budget, duplicate trial, missing side, unrandomized order, incomplete denominator, or timing-inadmissible result cannot satisfy promotion.

**GREEN:** Freeze all material digests; run baseline/candidate in randomized paired order with a recorded seed; emit all failures; compute family-clustered outcome summary and conservative non-regression gate from owner receipts.

**Evidence:** immutable assignment, 40 side-execution receipts minimum, denominators, failure inventory, timing-admissibility state, analysis digest.

### Task 3.3: Gate promotion on paired evidence and drill revocation/rollback

**Owner:** semantic-memory lifecycle.

**RED:** one-run, fewer than 20 pairs, fewer than five families, holdout regression, degraded verification, wrong/expired permit, or replay drift cannot promote.

**GREEN:** Only a predeclared passing experiment decision plus explicit lifecycle permit can call `promote_procedure`; then inject a distinct-family regression and prove revoke/rollback plus stale-selection denial.

**Claim:** bounded promotion/revocation loop on frozen local fixtures.

---

## Phase 4 — Verification and closure

1. Focused RED/GREEN tests after each task.
2. `cargo fmt --manifest-path AiDENs/Cargo.toml --all -- --check`
3. `cargo test --manifest-path AiDENs/Cargo.toml -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-cli`
4. `cargo clippy --manifest-path AiDENs/Cargo.toml -p aidens-contracts -p aidens-receipts -p aidens-runner -p aidens-cli --all-targets -- -D warnings`
5. `cargo check --workspace`
6. Live pinned-Podman controller and CLI tests.
7. Fresh-process bundle/index/replay inspection.
8. Independent hostile review from current live HEAD; no unresolved P0/P1.
9. Scoped local commit only; no push.

## Claim boundary

- **After Phase 2:** bounded real-sandbox run with durable terminal V3 publication; candidate remains `Tested` and inactive until separate lifecycle authority is supplied.
- **After Phase 3 only if every gate passes:** bounded closed-loop procedure promotion/revocation on a frozen local multi-family corpus.
- **Still forbidden:** production autonomous engineer, safe arbitrary untrusted-code sandbox, external benchmark superiority, online RL, unrestricted self-improvement.

## Council convergence

| Lane | Verified result | Controller decision |
|---|---|---|
| Terminal architecture | Existing owners are sufficient for V3 identity, required roles, terminal projection, atomic bundle/index persistence, and recovery. Missing seams are owner-native Forge export input and typed child-closure representation. | Reuse `RunBundleStore`, `AiDENsRunBundleV3`, and `CodingLearningTerminalProjectionV1`; do not create an AiDENs-local export or publication store. Keep terminal pending until a real Forge envelope and all children exist. |
| Evidence/benchmark | Corpus v1 is metadata-only. The canonical repeated-paired substrate already exists in living-memory, but there is no executable v2 corpus, benchmark statistics owner, replay integration, or admissible timing evidence. | Preserve v1; design v2 as 15 executable tasks across five family clusters, 9/3/3 partitions, hidden oracles, and 20+ fixed paired trials. Do not claim held-out learning before those gates pass. |
| Hostile integration | **NO-GO.** P0 synthetic blueprint and locally manufactured lifecycle authority; P1 partial publication, fixture simulation, replay/retention absence, permit/destination drift, and misleading partial success. | Fixed both P0s first. No lifecycle promotion/quarantine without an independently issued permit. No V3 success until child-first publication and fresh-process readback. No Phase 3 promotion until replay/statistical gates and authority are supplied. |

### Dissent resolution

The architecture lane described the pre-council procedure as promoted; the hostile lane showed that promotion itself was unauthorized and insufficiently bound. Higher-authority live source plus the hostile evidence wins: the corrected controller leaves the candidate `Tested` and inactive. The architecture lane also proposed canonical Forge export, while the binding pilot decision forbids direct Forge integration. Therefore Task 2.1 is blocked rather than replaced with a fabricated envelope.

### Implementation order after convergence

1. Fix and verify exact patch/source/policy/image binding. **Implemented; focused and live tests pass.**
2. Remove locally minted lifecycle authority. **Implemented; candidate remains `Tested`; live test passes.**
3. Add destination/replay/retention material binding and typed blocked reason codes.
4. Resolve the V3 child-closure field gap in canonical contracts and strengthen learning-writer validation.
5. Obtain/certify the real Forge owner adapter; until then terminal publication remains pending.
6. Build executable corpus v2 and owner-native statistics/replay evidence offline; do not connect it to promotion until certified.
7. Run crash/retry/replay/permit tests, broad gates, and a new hostile review. Commit only if no unresolved P0/P1 remains.

## Current execution receipts

- Python v2 hostile validator: 13 tests passed.
- Rust corpus consumer: 5 v2-specific tests plus the legacy consumer test passed; no oracle file is opened or projected.
- Pinned sealed Podman corpus: 15/15 declared baseline failures and 15/15 treatment passes; 30 side executions; corpus digest `225f6ec950afa2c3c2a46bf75a55da3f5606505bc73bddc6527e357186c2a9bb`.
- The corpus receipt directly records the Podman/Python/validator digests, observed rootless mode, local image digest, exact sealed argv template, no-pull policy, keep-id user namespace, no-new-privileges, network isolation, read-only root, dropped capabilities, timeout and cleanup contract, and UTC recording time.
- Live pinned-Podman controller: passed and left the candidate non-promoted.
- Live public CLI: passed and remained `blocked-evidence-insufficient` with terminal publication pending.
- Affected packages: strict Clippy passed; `aidens-contracts` 101 passed; `aidens-receipts` 20 passed; `aidens-runner` 72 unit tests passed with 2 ignored live tests and the controller live test separately exercised; all runner integrations passed; `aidens-cli` 66 passed with its live test separately exercised plus 5 integrations.
- Full root and AiDENs workspace checks passed. The root supported release lane passed. Second high-effort hostile review returned GO with all six prior P1 findings resolved and no new P0/P1.
