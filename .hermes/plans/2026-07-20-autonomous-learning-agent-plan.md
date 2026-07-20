# SOL Council: Autonomous Learning Agent Implementation Plan

## Council findings

Two premise mismatches were discovered:

1. **Adjudication builder gap**: `learn run` currently produces fixture/oracle execution evidence, but does NOT produce the paired statistics, family results, holdout gates, and frozen thresholds that `CandidatePromotionAdjudicationV1` requires. The adjudication JSON must be assembled from run results — this is the primary autonomy blocker.

2. **Permit authority gap**: All permits are operator-supplied. For autonomy, an operator-approved "autonomy lease" can delegate narrow scoped child permits to the controller — this is not self-minted authority.

## 10-task implementation plan

### Task 1: Persist V2 qualification experiment receipts
- Add `V2CorpusQualificationOutcomeV1` with deterministic experiment/assignment IDs, per-task receipts, denominator/admissibility counts, family/split aggregation, uncertainty, Forge bundle binding
- Add `run_v2_qualification_persisted()` in `learning_experiment.rs`
- Add `learn run --mode qualification --corpus-version v2` CLI mode
- Tests: qualification persists receipts, reopen reproduces aggregates, frozen policy bound before execution

### Task 2: Build real candidate adjudication from owner evidence
- Add `learning_adjudication.rs` with `build_and_persist_exact_source_adjudication()`
- Reads and verifies: preflight, effectful, artifact, tested receipt, Forge bundle, image/environment/patch/source/verifier/policy identities
- Persists through `ForgeAdjudicationStore::persist_adjudication`, reopens, verifies binding
- Tests: binds all owner receipts, uses frozen policy digest, quarantines missing evidence

### Task 3: Remove in-memory adjudication witness from promotion
- Change `promote_adjudicated` to require real Forge owner, not in-memory witness
- Update `learn promote` and resume to open `ForgeStore` and accept adjudication ID
- Tests: promotion requires Forge-persisted adjudication, rejects unpersisted JSON

### Task 4: Add operator-approved autonomy lease
- Add `LearningAutonomyLeaseV1` in `authority-delegation` with scoped child permit derivation
- CLI: `learn authority request/approve/inspect/revoke`
- Lease delegates narrow operations: candidate, operation, namespace, store scope, budget, expiry
- Tests: rejects self-approval, derives only scoped permits, rejects expired/revoked/widened

### Task 5: Make coordinator reconstruction query every owner directly
- Add `LearningRunHandleV1` as a digest-bound locator/identity manifest
- Update `rebuild_learning_owner_snapshot` to query event log, semantic memory, Forge, replay store
- Fix `apply_adjudication` to accept `Tested` candidate when bindings close
- Tests: reads all owners before terminal bundle, rejects mixed run IDs

### Task 6: Implement bounded phase-chaining controller
- Add `learning_autonomy.rs` with `execute_learning_transition_once()` and `drive_learning_until_boundary()`
- Algorithm: rebuild → reduce → checkpoint → verify lease/budgets → dispatch one transition → rebuild → checkpoint → repeat
- Maps actions to APIs: ExecuteAndVerify→run_real_sandbox, PromoteProcedure→Forge-backed promotion, ReplayProcedure→sealed replay, PublishTerminalEvidence→owner-rebuilt closure
- Tests: stops at authority boundary, one transition per checkpoint, crash recovery, quarantine on mixed evidence

### Task 7: Automate replay admission and sealed replay
- Add `admit_replay_from_adjudication()` and `replay_admitted_candidate()` controller adapters
- Expose typed replay execution readback API
- Tests: drive admits then executes replay with bound permit, rejects wrong candidate/namespace

### Task 8: Refactor terminal closure around verified readback material
- Add `RealSandboxTerminalClosureMaterialV1` and `rebuild_real_sandbox_terminal_material()`
- Reconstruct publication by idempotently rerunning `publish_terminal_evidence` from Forge bundle
- Make `learn close` execute actual closure from owner stores
- Tests: rebuild from owners matches live, publishes verified terminal, rejects missing replay

### Task 9: Add Podman capability gating
- Add `inspect_real_sandbox_readiness()` verifying rootless Podman, digest-pinned image, namespace/store roots
- Podman unavailability is `awaiting-capability`, not permission to weaken evidence
- Tests: doesn't consume permit when Podman unavailable

### Task 10: Add autonomous CLI surface
- `learn auto plan --request --policy` — create autonomous plan
- `learn auto inspect --plan` — check readiness
- `learn auto run --plan --lease-id` — execute until boundary
- `learn auto resume --run-handle --lease-id` — resume from handle
- `learn auto status --run-handle` — read-only status
- `learn close --run-handle` — execute terminal closure
- Dispositions: completed, awaiting-authority, awaiting-capability, quarantined, failed
- Tests: stops at authority boundary, resumes from handle, never reports succeeded before readback

## Safety boundaries preserved

- Operator-originated authority (via autonomy lease)
- Exact candidate/run/action/namespace permit scopes
- Immutable pre-run policy and threshold digests
- Rootless sealed Podman, digest-pinned image, no host fallback
- Owner-store readback after every mutation
- Promotion and replay as distinct authorized effects
- Terminal success only after replay and terminal bundle readback
- Automatic quarantine on ambiguous/mixed/tampered evidence

## Never automate without new authority

- Approval of the autonomy lease itself
- Scope widening
- Threshold changes after execution begins
- Promotion outside approved candidate/plan lineage
- Replay into another namespace
- Host execution or unpinned-image substitution
- Treating oracle qualification as candidate evidence