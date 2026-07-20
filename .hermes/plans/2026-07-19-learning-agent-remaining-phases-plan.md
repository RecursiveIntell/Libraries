# Learning Agent Remaining Phases Implementation Plan

> **SOL council design:** 2026-07-19. **Implementation:** gpt-5.3-codex-spark low-effort agents.
> **Repository:** `/home/sikmindz/Coding/Libraries-medusa`, branch `feat/medusa`, HEAD `dcb7ec5`.

## Safe commit sequence (from council)

1. Semantic-memory owner snapshot APIs
2. Adjudicated promotion adapter and legacy-path fencing
3. Owner-derived replay admission
4. Coordinator contract and checkpoint persistence
5. Resumable owner transitions and failure matrix
6. Sealed replay execution/result persistence
7. `learn resume` and `learn close`
8. Remove `learn stop`
9. Terminal linkage hardening
10. Release-gate refresh
11. Live closure/replay smoke
12. Completion ledger and claim boundary update

## Task 1: Semantic-memory owner snapshot APIs

**Crate:** `semantic-memory` (read-only additions over V34/V38 tables)

Add:
- `ProcedureOwnerSnapshotV1` — verified artifact, latest lifecycle receipt, latest effectful receipt
- `ProcedureReplaySnapshotV1` — verified retained inputs, admission, optional result
- `MemoryStore::load_procedure_owner_snapshot(artifact_id) -> ProcedureOwnerSnapshotV1`
- `MemoryStore::load_procedure_replay_snapshot(replay_id) -> ProcedureReplaySnapshotV1`
- `MemoryStore::admit_adjudicated_procedure_replay(forge, replay_id, adjudication_id, permit) -> ProcedureReplayAdmissionV1`

The new admission API derives candidate/patch/source/verifier/policy/environment/image/promotion-receipt/permit-ref/store-identity from owner state. AiDENs must not populate raw `ProcedureReplayInputsV1` fields.

Tests: owner snapshot readback, tamper/corruption failure, missing artifact, missing replay, owner-derived admission vs caller-filled, permit mismatch/reuse.

## Task 2: Adjudicated promotion adapter and legacy-path fencing

**Crate:** `aidens-runner`, `aidens-cli`

- Add `ProcedureLifecycleAdapter::promote_adjudicated` consuming bridge-verified adjudication + single-use permit
- Fence existing `promote` from the supported learning lane
- Route `learn promote` through `MemoryStore::promote_adjudicated_procedure`, not legacy `promote_procedure`
- RED test: `learn promote` cannot reach legacy adapter
- GREEN test: adjudicated promotion produces linked lifecycle receipt

## Task 3: Coordinator contract and checkpoint persistence

**Crate:** `aidens-runner` (new `learning_coordinator.rs`), `aidens-contracts`

- `LearningCoordinatorProjectionV1` with stages, disposition, owner receipt pointers, binding digests, next required action
- `OwnerReceiptPointerV1` — owner crate, artifact kind, receipt ID, receipt digest
- Pure workflow reducer: owner snapshot → stage/disposition/next action
- Checkpoint append/readback via `CanonicalEventLog::append_orchestration_report`
- Deterministic checkpoint ID; duplicate ID accepts only byte-identical material
- No owner payloads, permits, paths, or recomputed decisions in projection

Tests: projection validation, duplicate checkpoint, mixed run IDs, stale binding, checkpoint deletion → owner reconstruction yields same projection.

## Task 4: Resumable owner transitions and failure matrix

**Crate:** `aidens-runner`

- Expose existing controller steps as `pub(crate)` without changing semantics
- Split publication into resumable export and import seams
- Table-driven coordinator failure tests for every injection point:
  - after preflight, after sealed execution, after candidate test, after Forge write, after export, after import, after adjudication, during lifecycle transaction, after promotion, after replay execution, after rollback/revoke, corrupt checkpoint/log/bundle

## Task 5: Sealed replay execution and result persistence

**Crate:** `aidens-runner`, `aidens-cli`

- `OwnerAdmittedSealedReplayRequestV1` — carries no patch/source/verifier/image/policy; those come from owner state
- `OwnerAdmittedSealedReplayReportV1` — projection over owner receipts
- Execution sequence: load snapshot → verify identities → sealed executor → persist receipt → compare → record result → reload snapshot
- Crash-after-execution resume from execution receipt, not rerun Podman
- Replace CLI `NotAvailable` branch with real owner-admitted path
- One ignored live rootless-Podman replay smoke

## Task 6: `learn resume` and `learn close`

**Crate:** `aidens-cli`, `aidens-runner`

- `learn resume`: rebuild from owner stores, perform idempotent authorized transitions, stop at missing authority
- `learn close`: validate all owner receipts exist, invoke terminal closure
- Terminal hardening: require adjudication/permit linkage, replay result, lifecycle compatibility
- Neither treats checkpoint as authority

## Task 7: Remove `learn stop`

**Crate:** `aidens-cli`

- Remove `LearningCommand::Stop` variant and dispatch
- Update help, tests, runbook, claim boundary
- Add boundary test: queue cancellation is not execution cancellation

## Task 8: Release gate refresh and completion ledger

- Update `verify_learning_agent.sh` with all required checks
- Integrate into `verify_current.sh`
- Regenerate completion ledger with current HEAD/digests/test counts
- Update claim-boundary JSON and runbook
- Final hostile review