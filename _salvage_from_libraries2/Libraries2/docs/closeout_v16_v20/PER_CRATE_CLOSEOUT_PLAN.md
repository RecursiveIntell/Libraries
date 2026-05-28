# Per-Crate Closeout Plan

## `federated-settlement`

### Already present
- treaty
- runtime identity set
- cross-runtime equivalence bundle
- settlement case
- shared disposition
- local dissent
- downgrade
- settlement receipt
- bounded settlement evaluator

### Add next
- `SharedReplaySliceV1`
- `SharedDivergenceReportV1`
- `TreatySuspensionV1`

### Evaluator upgrades
Keep `evaluate_settlement(...)`, but add companion functions such as:
- `evaluate_shared_replay(...)`
- `evaluate_divergence_or_suspension(...)`

Do not try to simulate real network federation.
Stay in typed bounded-case law.

## `mechanism-runtime`

### Already present
- mechanism bundle
- theory version
- theory library
- hypothesis library
- simulation contract
- fit run
- theory refuter suite
- rollout stability report

### Add next
Do not add more nouns first.
Make the current nouns do more work.

### Evaluator upgrades
Replace the boolean-heavy path with one that consumes:
- `TheoryRefuterSuiteV1`
- `RolloutStabilityReportV1`

Suggested shape:
- keep `evaluate_fit_run(...)` as the top-level public helper
- add an input struct or overload that avoids argument explosion
- block local-review eligibility when refuters are missing or stability is not advisory-clear

## `discovery-portfolio`

### Already present
- discovery program
- program hypothesis set
- information value estimate
- experiment campaign
- portfolio plan
- verification load budget
- campaign decision trace

### Add next
Do not add a scheduler.
Deepen selection quality.

### Evaluator upgrades
Current behavior is basically “walk campaigns in order until budget says no.”

Upgrade it so:
- selection logic consumes `InformationValueEstimateV1`,
- reasons mention information gain versus review cost,
- `ProgramHypothesisSetV1` is referenced in decision traces or selection context,
- budget exhaustion emits explicit pause/defer behavior.

## `constitutional-memory`

### Already present
- charter bundle
- doctrine snapshot
- amendment proposal
- amendment decision
- archive manifest
- compaction receipt
- historical query guarantee
- deprecation bundle
- retirement bundle

### Add next
Prefer connecting existing nouns over inventing many new ones.

### Evaluator upgrades
Add a bounded archive/compaction path and bind amendment decisions more tightly to:
- semantic diff,
- migration obligations,
- archive consequences,
- rollback readiness.

If possible, reuse an existing semantic-diff artifact already present elsewhere in the repo rather than inventing a duplicate v19-only one.

## `spec-execution`

### Already present
- spec bundle
- normative AST
- generated schema bundle
- generated interpreter bundle
- generated conformance corpus
- generated migration plan
- proof obligation set
- proof evaluation receipt
- human veto bundle
- meta challenge bundle
- schema-bundle generation helper

### Add next
- `SelfHostingBuildReceiptV1`

### Evaluator upgrades
Add a higher-level generation helper that emits:
- schema bundle
- interpreter bundle
- conformance corpus
- migration plan
- self-hosting build receipt
- proof evaluation receipt

Then add a bounded veto/challenge/rollback helper that keeps generated surfaces advisory-only.
