# Closed-Loop Learning Agent Canonical Closure Specification

> **For Hermes:** Execute this specification with Codex `gpt-5.6-luna` at low reasoning effort for clear coding tasks. Use TDD, two-stage review, canonical-owner reuse, and no speculative shadow authority.

**Status:** Council-converged design; implementation pending.

**Phase A audit amendment (2026-07-19):** The preliminary Forge v6 persistence patch was
reverted. `semantic-memory-forge` currently exposes evidence schemas/export types but no
database persistence API, and `semantic-memory` cannot depend directly on `ForgeStore`
without violating the crate dependency direction. Do not add a new cross-crate authority in
Phase A. Persistence remains blocked until the canonical owner publishes a typed persistence
boundary; the existing verification-adjudication contract remains the tested decision owner.

**Repository checkpoint:** `/home/sikmindz/Coding/Libraries-medusa`, branch `feat/medusa`, HEAD observed by council as `8d4a31fc09b9e91be21a751a257631a7f24de933`.

**Goal:** Complete the bounded exact-source learning MVP without claiming generalization, online RL, production autonomy, or arbitrary untrusted-code containment.

## 1. Council decision and authority map

The council rejected both single-owner overload and AiDENs-local truth.

| Semantic responsibility | Canonical owner | AiDENs role |
|---|---|---|
| Adjudication rules and deterministic gate evaluation | `verification-adjudication` | Call/serialize only |
| Immutable candidate adjudication evidence and Forge linkage | `semantic-memory-forge` | Pass IDs/digests only |
| Experiment records/statistics inputs | `living-memory` / existing Forge experiment owners | Supply evidence; no lifecycle mutation |
| Procedure lifecycle, permits, promotion/revoke/rollback | `semantic-memory` | Forward typed requests; display receipts |
| Retained replay inputs/admissions/results | `semantic-memory` | Request/replay projection only |
| Sealed execution/check/backend truth | `check-runner`, CEA, `aidens-runner` existing owners | Compose existing receipts |
| Terminal bundle/index/projection publication | `aidens-receipts` and existing terminal owner | Compose child receipts only |
| CLI/workflow/recovery projection | `aidens-cli` / `aidens-runner` | No new truth store |

Existing evidence cited by the council:

- `verification-adjudication` is already a workspace dependency and `CanonicalGovernanceAdapter::adjudicate_case` delegates to it in `AiDENs/crates/aidens-contracts/src/lib.rs:222-245`.
- `semantic-memory-forge` currently owns canonical evidence bundles, verification trials, comparability, uncertainty, and promotion summaries.
- `semantic-memory` currently owns `ProcedureEffectfulEvaluationReceiptV1`, `ProcedureLifecycleReceiptV1`, `ProcedureLifecyclePermitV1`, and one-shot lifecycle transitions in `semantic-memory/src/procedural_memory.rs:419-504`.
- Existing per-run learning output can be marked `EligibleForPromotion`, but the controller explicitly says a single run is not paired promotion evidence; adjudication must aggregate before lifecycle mutation.
- Existing semantic-memory replay APIs retain semantic-search inputs by receipt ID (`semantic-memory/src/lib.rs:2703-2744`, `semantic-memory/src/db.rs:2495-2585`), but coding-procedure replay lacks a unified owner-issued identity bundle.
- Existing schema is V37 (`semantic-memory/src/db.rs:891+`, `:972+`); additive replay retention uses V38.
- Existing terminal publication is child-first and fail-closed in `aidens-receipts` and `learning_terminal.rs`; the coordinator must reuse it.

### Non-negotiable invariants

1. No lifecycle transition from a single green run, fixture/oracle score, CEA confidence, or Forge projection alone.
2. A passing adjudication means `EligibleForLifecycleConsideration`, never `Promoted`.
3. Only semantic-memory can consume the independent single-use lifecycle permit and change procedure state.
4. Every identity used for eligibility/replay is immutable, typed, and digest-bound.
5. AiDENs may not persist a second adjudication, replay, lifecycle, experiment, or memory truth store.
6. Missing, ambiguous, stale, drifted, or conflicting evidence yields `Inconclusive`, `Quarantined`, `Pending`, `Blocked`, or `Indeterminate`, never success.
7. Historical artifacts lacking reconstructable identities remain inactive/unverifiable; no backfill fabrication.
8. Rollback/revoke append compensating receipts and never delete evidence.

## 2. Versioned contract design

### 2.1 `CandidatePromotionAdjudicationV1`

Owner: `verification-adjudication` defines the typed contract and deterministic validation; `semantic-memory-forge` persists the immutable artifact.

Required fields:

```rust
pub struct CandidatePromotionAdjudicationV1 {
    pub schema_version: String,                 // exact version, e.g. "CandidatePromotionAdjudicationV1"
    pub adjudication_id: String,
    pub adjudication_digest: String,
    pub candidate_id: String,
    pub candidate_digest: String,
    pub patch_digest: String,
    pub source_tree_digest: String,
    pub verifier_digest: String,
    pub check_policy_digest: String,
    pub environment_digest: String,
    pub image_digest: String,
    pub experiment_id: String,
    pub evidence_bundle_id: String,
    pub evidence_bundle_digest: String,
    pub assignment_digest: String,
    pub paired_denominator: u64,
    pub admissible_pairs: u64,
    pub excluded_pairs: u64,
    pub uncertainty: UncertaintyV1,
    pub family_results: Vec<FamilyGateV1>,
    pub holdout_result: HoldoutGateV1,
    pub thresholds: FrozenPromotionThresholdsV1,
    pub decision: AdjudicationDecisionV1,       // EligibleForLifecycleConsideration | Quarantined | Inconclusive
    pub reason_codes: Vec<String>,
    pub source_receipt_refs: Vec<ReceiptRef>,
    pub created_at: String,
}
```

The exact field names may reuse existing typed families where compatible. Do not use `serde_json::Value` for the decision-bearing contract.

Validation rules:

- All required identity strings are non-empty and digest formats are canonical.
- `paired_denominator > 0`; `admissible_pairs <= paired_denominator`.
- No family or holdout result may be omitted when required by frozen thresholds.
- Family and holdout gates run independently before aggregation.
- Any identity mismatch, changed threshold, missing source receipt, failed holdout, failed family, insufficient denominator, ambiguous assignment, stale/superseded evidence, or unverifiable receipt yields non-eligible disposition.
- `adjudication_digest` covers canonical serialized material fields, excluding only explicitly volatile metadata.
- Same material inputs and policy produce the same decision and reason codes.
- The artifact is immutable and idempotent by `adjudication_id` + digest; conflicting retry is rejected.

### 2.2 Forge persistence boundary

Add the artifact to the existing Forge evidence store/family rather than creating an AiDENs store. Persist:

- canonical adjudication JSON;
- `adjudication_id`, digest, candidate identity, bundle identity;
- immutable source receipt references;
- content digest and creation time;
- idempotency/conflict detection.

The Forge API must expose:

```text
persist_adjudication(adjudication) -> persisted receipt
read_adjudication(adjudication_id) -> verified artifact
verify_adjudication_binding(adjudication_id, expected identities) -> typed result
```

A Forge `EligibleForLifecycleConsideration` is evidence only. It cannot call or mutate semantic-memory lifecycle state.

### 2.3 Semantic-memory lifecycle gate

Extend the existing lifecycle request/receipt path additively. Promotion requires:

```text
candidate_id
candidate_digest
adjudication_id
adjudication_digest
evidence_bundle_id/digest
independent ProcedureLifecyclePermitV1
permit scope digest
idempotency key
```

Semantic-memory must:

1. Read and verify the Forge adjudication artifact through the owner boundary.
2. Verify candidate/artifact/evidence/permit scope equality.
3. Verify adjudication decision is eligible.
4. Verify lifecycle predecessor and effectful evaluation receipt.
5. Consume permit atomically once.
6. Emit a lifecycle receipt linking adjudication and permit digests.
7. Reject reuse, altered identity, expired permit, wrong operation, or conflicting retry.

No AiDENs-side eligibility check can substitute for this owner gate.

### 2.4 Exact replay contract and V38 retention

The council found no existing owner-issued unified coding replay identity. Additive semantic-memory V38 owner tables/contracts are required before enabling public replay:

- `procedure_replay_inputs`
- `procedure_replay_admissions`
- `procedure_replay_results`
- `procedure_replay_permit_uses`

All rows are immutable. Identical retry is idempotent; conflicting material is rejected.

Required retained identity fields:

```text
replay_id
original_artifact_id/original_artifact_digest
patch_digest
source_tree_digest
verifier_digest
check_policy_digest
environment_digest
image_digest
store_identity_digest
retained_input_digest
promotion_receipt_ref
action_permit_ref
result_digest
outcome
reason_codes
```

Canonical owner API:

```text
admit_procedure_replay(request, action_permit) -> replay admission receipt
load_retained_replay_inputs(replay_id) -> owner-verified inputs
record_replay_result(replay_id, result) -> owner replay receipt
compare_replay(original_receipt, replay_receipt) -> ExactMatch | Drift | Mismatch | Inconclusive | NotAvailable
```

`aidens-runner` supplies a fresh sealed execution only after owner admission and returns owner-linked receipts. It must not compute canonical store/verifier identity from local paths or caller metadata.

CLI changes are deferred until this owner API exists:

- `learn replay --request REQUEST.json --out REPORT.json`
- `learn compare --store STORE --original RECEIPT --replay RECEIPT`

Missing retention, changed source/patch/verifier/store/environment, expired/reused permit, or revoked/rolled-back candidate must return nonzero typed non-success.

Migration: V38 is additive; old readers fail with `SchemaAhead` rather than dropping evidence. Rollback requires a pre-migration DB snapshot or forward-compatible binary. No table drops and no historical backfill from path/JSON digests.

## 3. Durable coordinator

The coordinator is a rebuildable orchestration projection, not truth authority.

States:

```text
Preflighted
→ ExecutedVerified
→ CandidateTested
→ EffectfulEvidencePersisted
→ ForgePublished
→ Adjudicated
→ EligibleForLifecycleConsideration | Quarantined
→ Promoted
→ ReplayAdmitted
→ Replayed
→ RolledBack | Revoked
→ TerminalPublished
```

Side states:

```text
Pending | Blocked | Indeterminate | Failed
```

Each transition stores only owner receipt IDs/digests, run identity, and the next required action. On resume, the coordinator reads owner stores and refuses mixed run IDs, stores, namespaces, policies, permits, or conflicting material.

Failure matrix:

| Injection point | Required result |
|---|---|
| after preflight before backend | no backend effect; retry same identity |
| after sealed execution before candidate persistence | pending; preserve execution receipt |
| after Forge write before import | retry same Forge identity; no duplicate |
| after memory import before terminal projection | verify import/readback; resume projection |
| after adjudication before permit consumption | no lifecycle transition |
| after permit consumption before lifecycle receipt | owner atomic recovery or indeterminate; never second transition |
| after promotion before replay admission | candidate remains promoted; replay can resume |
| after replay execution before result receipt | preserve run; retry identical result binding |
| after rollback/revoke | every stale selector denied; history preserved |
| corrupt bundle/index/log | quarantine/indeterminate; no success |

Terminal closure invokes existing `close_real_sandbox_terminal` only after all required child owner receipts exist and fresh bundle/index/projection readbacks succeed.

## 4. Implementation order

### Phase A — Contract and owner tests

1. Add RED tests for `CandidatePromotionAdjudicationV1` identity binding, deterministic digest, denominator/admissibility, family/holdout gates, immutability, and conflicting retry.
2. Implement/reuse deterministic adjudication rules in `verification-adjudication`.
3. Add Forge persistence/readback and migration tests.
4. Add semantic-memory lifecycle gate tests for missing/wrong adjudication, permit mismatch/reuse, and receipt linkage.
5. Add V38 replay schema/owner tests for retention, admission, immutable results, and drift.

### Phase B — Thin AiDENs adapters

6. Adapt `learning_experiment` to produce owner-consumable evidence, never promotion truth.
7. Adapt `learning_controller` to persist exact candidate/effectful identities and request Forge adjudication.
8. Add coordinator projection and failure-injection tests.
9. Wire publication/resume using existing Forge bridge and terminal closure APIs.

### Phase C — Public replay/lifecycle

10. Add typed CLI replay/compare requests only after V38 owner APIs pass.
11. Wire quarantine, promotion, rollback, revoke, and stale-selection denial drills.
12. Implement `learn stop` only if a canonical cancellation/process-tree owner is identified; otherwise remove it from supported MVP claims.
13. Add `learn resume`/`learn close` as explicit projection commands that stop at missing authority boundaries.

### Phase D — Certification

14. Add current-head receipt verifier and update completion ledger.
15. Run focused tests, workspace checks, strict Clippy, and learning release gate.
16. Run one current-head pinned rootless-Podman closure/replay smoke.
17. Rerun the 60-pair workload only if source-affecting corpus/adapter/check/image digests changed or explicitly authorized.
18. Hostile review: authority, identity, containment, recovery, operator truth, and claim boundary.

## 5. RED/GREEN acceptance gates

### Adjudication RED

- Alter one candidate, patch, source, verifier, policy, environment, image, threshold, or evidence digest.
- Remove required family/holdout evidence.
- Use zero/ambiguous denominator.
- Reuse a consumed permit.
- Submit total score passing but holdout/family gate failing.
- Retry same ID with different material.

All must reject or produce `Quarantined`/`Inconclusive` with no lifecycle effect.

### Adjudication GREEN

A learner-generated exact candidate produces an immutable owner adjudication receipt containing all required identities, paired statistics, uncertainty, family/holdout results, thresholds, and evidence references. A separate independent permit then produces a semantic-memory lifecycle receipt linking both digests.

### Replay RED

- Missing retained inputs.
- Caller-supplied substitute patch/source/verifier/store/environment.
- Drift or revoked/rolled-back candidate.
- Expired/reused/wrong-scope action permit.

### Replay GREEN

Owner admission loads retained inputs, a fresh sealed execution completes, and the owner persists a linked replay receipt. Comparison classifies only owner receipts.

### Coordinator GREEN

Every failure-injection point resumes idempotently or stops `Pending`/`Indeterminate`; no duplicate transition, mixed run, false terminal success, or evidence deletion occurs.

## 6. Claim gates

### POC

Current-head sealed effectful run, candidate quarantine, Forge publication/readback, and exact terminal closure must pass. Claim remains one exact patch/source tree.

### Bounded MVP

Requires owner adjudication, independent promotion permit, V38 retained-input replay, stale-selection denial after rollback/revoke, crash matrix green, terminal fresh-process closure, and operator resume/close.

### Production

Still blocked by multi-family learner-generated generalization, hostile containment proof, service-scale authority/revocation/monitoring, migrations/backups/upgrades, and external workload evidence.

## 7. Rollback and migration

- Additive schemas only; retain old readers and reject unsupported newer versions safely.
- Observation mode precedes enforcement.
- Existing candidates remain inactive/unverifiable until reevaluated.
- Failed Forge/import/replay recovery leaves evidence and state pending/indeterminate.
- Disable new CLI eligibility/replay routes to roll back; do not delete owner artifacts.
- Database rollback requires snapshot/forward-compatible binary; never drop V38 evidence tables.
- Never reinterpret historical receipts as promotion-grade evidence.

## 8. Current known blockers

1. Forge adjudication persistence/API does not yet exist.
2. Semantic-memory lifecycle gate does not yet consume Forge adjudication.
3. V38 coding replay retention/admission/result owner APIs do not yet exist.
4. Current completion ledger must be regenerated after this spec/HEAD update.
5. Current live Podman closure/replay evidence remains unverified.

These are implementation work items, not permission to weaken the claim boundary.
