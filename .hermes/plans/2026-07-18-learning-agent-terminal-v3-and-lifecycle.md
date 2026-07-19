# AiDENs bounded learning terminal V3 and exact-source lifecycle closure

**Date:** 2026-07-18
**Authority:** live canonical owner APIs in `living-memory`, `forge-memory-bridge`, and `semantic-memory`
**Scope:** finish a bounded exact-source retained-reuse proof; do not claim cross-task learning

## Invariants

1. The v2 evaluator-oracle corpus is qualification-only and never supplies procedure promotion evidence.
2. One `ProceduralMemoryArtifactV1` embeds one exact `StructuredPatch` and requires its exact `source_tree_digest`.
3. The real sandbox report, canonical effectful receipt, Forge bundle, export envelope, projection import, lifecycle receipts, and replay report must all bind that same artifact/patch/source identity.
4. Evaluation, terminal publication, lifecycle promotion, and replay execution are separately invoked authority boundaries.
5. AiDENs composes canonical stores and receipts; it creates no lifecycle, export, statistics, or projection truth store.
6. A successful run remains `Tested` until a separate single-use `ProcedureLifecyclePermitV1` is supplied.
7. V3 publication is complete only after Forge persistence/readback, canonical export, V3 bridge transform, semantic-memory atomic import, and fresh readback of the matching envelope/evidence bundle.
8. Recovery retries are idempotent for identical material and fail on conflicting identity reuse.

## Task 1 — Forge typed bundle persistence and evidence semantics

### RED

- Typed bundle persistence round-trips every canonical and Forge authoring field after store reopen.
- Reusing a bundle ID with different canonical content fails instead of replacing evidence.
- A non-comparative execution bundle cannot serialize as a paired causal claim.

### GREEN

- Add `ClaimStrength::ExecutionVerifiedNoComparison` while preserving `ProvisionalSinglePair` as the backward-compatible default.
- Add owner-native `ForgeStore::insert_canonical_evidence_bundle(&ExperimentEvidenceBundle)` and `get_canonical_evidence_bundle` APIs using `to_canonical_evidence_bundle` / `from_canonical_evidence_bundle`.
- Store canonical JSON plus compatibility projections in one transaction; identical retry is a no-op, conflicting retry is typed failure.

## Task 2 — Real-run artifact-bound effectful receipt and pending bundle

### RED

- A publication-complete real report records `ProcedureEffectfulEvaluationReceiptV1` for the exact tested artifact.
- The artifact remains `Tested` and cannot be retrieved before promotion.
- The controller returns a durable Forge bundle ID and typed readback proof.
- Bundle claim strength is `ExecutionVerifiedNoComparison`, promotion state is `NotPromoted`, and known threats explicitly reject comparative/generalization claims.

### GREEN

- Extend the controller result with canonical effectful receipt and pending terminal bundle projection.
- Add explicit Forge store path and publication namespace to real-run configuration/request identity.
- Build and seal one owner-native `ExperimentEvidenceBundle` from the real report and canonical backpointers; persist and reopen/read back before returning.
- Do not bridge/import or promote inside `run_real_sandbox`.

## Task 3 — Separate terminal V3 publication with recovery

### RED

- Missing bundle/store/namespace fails before semantic-memory writes.
- First publication returns V3 envelope/import/readback evidence.
- Identical retry returns the same envelope digest and an idempotent memory result.
- Conflicting bundle reuse is rejected.
- A simulated import failure leaves the persisted Forge bundle available for retry and never reports publication complete.

### GREEN

- Add an AiDENs publication adapter and `learn publish` CLI command.
- Load the typed bundle from Forge, call `forge_engine::export_bundle`, transform only with `forge_memory_bridge::transform_envelope_v3`, import only with `MemoryStore::import_projection_batch`, then query projection imports and verify envelope ID, content digest, namespace, V3 schema, evidence bundle ID, and rebuildable kernel payload.
- Return owner-native IDs/digests and a `Published` state only after readback.

## Task 4 — Exact-source promotion, governed retrieval, replay, rollback/revoke

### RED

- Promotion rejects missing/wrong/expired/reused lifecycle permits.
- Correct permit promotes only the exact artifact.
- Governed action retrieval rejects wrong source-tree digest and absent/wrong action permit.
- Exact matching context retrieves the immutable patch without invoking it.
- Fresh replay must use a separate execution permit and reproduce patch/source/check/rollback identities.
- Rollback removes the artifact from all retrieval paths; permit reuse and stale direct-ID/cache/replay retrieval remain denied.
- Revocation is tested on a separately promoted run or as the terminal alternative; rollback and revoke state transitions are not conflated.

### GREEN

- Extend CLI lifecycle support to canonical `rollback` while retaining unsupported `stop`.
- Add a typed governed-retrieval command accepting canonical request JSON and returning the owner report unchanged.
- Add a replay adapter that validates the retrieved exact structured patch and runs it through the existing sealed effectful evaluator with a new execution permit; retain canonical comparison receipts.
- Run the real operator-rerunnable drill: promote → retrieve exact → reject wrong source → replay exact → rollback → prove all retrieval paths deny stale selection.

## Verification

1. Focused RED/GREEN tests for Forge store, evidence projection, controller, CLI, and lifecycle.
2. `cargo fmt --manifest-path AiDENs/Cargo.toml --all -- --check`.
3. Affected AiDENs tests and strict Clippy.
4. Forge owner tests, semantic-memory procedural-memory tests, bridge tests.
5. Root `cargo check --workspace` and supported release lane.
6. Live pinned-Podman controller and exact replay only; no second 60-pair corpus run unless source affecting that adapter changes.
7. Fresh-process publication retry/readback.
8. Independent hostile P0/P1 review against live HEAD.
9. Scoped local commits; no push.

## Evidence-safe terminal claim

> AiDENs demonstrates bounded retained reuse of one immutable, exact source-bound Rust patch: real sealed evaluation, canonical evidence persistence and V3 publication, explicit operator promotion, governed exact-context retrieval, fresh sealed replay, and rollback with stale-selection denial. It does not demonstrate cross-task learning, held-out generalization, online RL, or production autonomous engineering.
