# Production consumer citation spec — 2026-03-18

This document freezes the minimum citation payload that downstream crates should carry once the v25 composition lane is consumed directly.

## Minimum citation set

Unless a narrower subset is explicitly justified, the preferred citation set is:

- `applicability_context_id: ApplicabilityContextId`
- `profile_set_id: ProfileSetId`
- `composition_receipt_id: CompositionReceiptId`
- `effective_constitution_id: EffectiveConstitutionId`
- `compiled_obligation_set_id: CompiledObligationSetId`
- `composition_conflict_set_id: Option<CompositionConflictSetId>`
- `profile_exception_bundle_ids: Vec<ProfileExceptionBundleId>`

In addition, decision artifacts should carry:

- `required_obligation_refs: Vec<String>`
- `blocking_obligation_refs: Vec<String>`
- `monitoring_obligation_refs: Vec<String>` when monitors are material to the decision

## Effect-runtime target surface

### `EffectIntentV1`
Convert the effect-owned IDs from raw `String` to `stack-ids` newtypes. `EffectIntentV1` itself does not need the full citation set, but it should stop using untyped IDs.

### `EffectPreflightReportV1`
Carry the full minimum citation set plus `required_obligation_refs`, `blocking_obligation_refs`, and `decision_basis_obligation_refs`.

### `EffectCommitDecisionV1`
Carry `composition_receipt_id`, `effective_constitution_id`, `compiled_obligation_set_id`, `profile_exception_bundle_ids`, and the approval-related obligation refs that justified commit authorization or refusal.

### `EffectExecutionReceiptV1`, `EffectObservationBundleV1`, `CompensationPlanV1`, `CompensationExecutionReceiptV1`
Carry at least `effective_constitution_id` and `compiled_obligation_set_id`; prefer the full minimum citation set where replay or cleanup decisions depend on the composition context.

## Verification-control target surface

Each of the following should carry the full minimum citation set:

- `EffectReviewCaseV1`
- `EffectBlockReceiptV1`
- `DelegationReviewCaseV1`
- `ReleaseGateCaseV1`
- `ContinuityReviewCaseV1`
- `ControlReceipt`

The review artifacts should also carry `required_obligation_refs` and `blocking_obligation_refs` so an operator can see which compiled obligations actually drove the state.

## Verification-policy target surface

`PolicyDecision` must **not** import `profile-runtime` types directly because that would create a cycle.
Instead it should carry `stack-ids` references only:

- `composition_receipt_id`
- `effective_constitution_id`
- `compiled_obligation_set_id`
- `profile_exception_bundle_ids`
- `required_obligation_refs`
- `blocking_obligation_refs`

`evaluate_policy` may receive these values as already-computed inputs or as a small local citation struct built from `stack-ids` only.

## Verification-adjudication target surface

The following should cite one `PolicyDecision` and one composite constitutional answer:

- `PromotionDecision`
- `RefutationDecision`
- `RollbackPlan`
- `EffectAdjudicationReceiptV1`
- `ReleaseRollbackDecisionV1`

At minimum add:

- `policy_decision_id`
- `composition_receipt_id`
- `effective_constitution_id`
- `compiled_obligation_set_id`
- `profile_exception_bundle_ids`

## Remote-oracle-admission target surface

`RemoteSliceRequestV1`, `RemoteSliceResultV1`, and `CrossRuntimeReplayTicketV1` should carry local constitutional refs so disclosure, trust-root, replay, and residency posture can be reconstructed without archaeology.

## Federated-settlement target surface

`SettlementCaseV1`, `SettlementReceiptV1`, `SharedReplaySliceV1`, and `SharedDivergenceReportV1` should preserve the local constitutional lane behind the shared view.
A repeated local citation struct inside these artifacts is acceptable; a new crate is **not** required for this pass.
