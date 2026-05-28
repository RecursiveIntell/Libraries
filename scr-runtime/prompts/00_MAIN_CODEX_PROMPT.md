# Main Codex Prompt — SCR-P0A: Deterministic Control Evaluator Reference Kernel

You are implementing **SCR-P0A**.

Your task is to create the first deterministic, fixture-driven, receipt-bearing control evaluator for Structured Correlation Regularization.

This pass is not a generic scoring library. It is a **reference control-policy evaluator over proposed actions**.

## Absolute constraints

Read and obey `AGENTS.md`.

Do not integrate with Recall, AiDENs, memory, retrieval, or tools in this pass.

Do not use LLMs, embeddings, learned weights, network calls, stochastic scoring, or runtime web lookups.

Do not introduce FEUT/EEG/P=NP/Clay/universal-constant language into production code, schemas, policies, or crate docs.

Do not create replacement canonical ID/provenance/artifact/receipt semantics if the target workspace already provides them.

Do not return naked bool decisions.

Do not use `f32` or `f64` in durable scoring artifacts.

## Target final repository shape

Create or update:

```text
crates/
  scr-reference/
  scr-kernel/
  scr-audit-adapter/
  scr-cli/

schemas/generated/

policies/
  audit_policy_v1.toml
  audit_policy_v1.canonical.json

fixtures/audit/
  cases/
  expected/

scripts/
  run_all_checks.sh
  generate_schemas.sh
  validate_schemas.py
  verify_golden_fixtures.sh
  assert_no_unexplained_golden_changes.sh
  assert_no_feut_contamination.sh
  assert_no_durable_float_scores.sh
  assert_no_naked_decision_booleans.sh
  assert_no_shadow_truth.sh
  assert_no_llm_or_network_calls.sh

docs/
  SOURCE_BASIS.md
  CANONICAL_OWNERS.md
  QUARANTINED_TERMS.md
  ARCHITECTURE.md
  EVALUATOR_REFERENCE.md
  POLICY_MODEL.md
  ACTION_RESOLUTION.md
  DECISION_RECEIPTS.md
  AUDIT_ADAPTER.md
  FAILURE_MODES.md
  INTEGRATION_SEQUENCE.md
  NON_GOALS.md
```

If the repository structure requires minor adjustment, document it in `docs/SOURCE_BASIS.md` and preserve the same semantics.

## Required phases

Execute these phases in order:

1. Phase 0 — Source basis and quarantine
2. Phase 1 — Deterministic core types
3. Phase 2 — Policy and hard-rule evaluator
4. Phase 3 — Pressure derivation
5. Phase 4 — Audit adapter
6. Phase 5 — CLI, golden fixtures, conformance
7. Phase 6 — Hostile scripts and final audit

After each phase, stop and wait for the matching automated phase gate before continuing.

## Core model

The evaluated object is:

```text
ControlEvaluationInputV1 {
  actor_ref
  permit_ref
  subject_ref
  domain
  proposed_action
  requested_effect
  evidence_refs
  environment_ref
  valid_time_basis
  recorded_time
  schema_version
}
```

The score axes are:

```text
Hazard
EvidenceConfidence
Uncertainty
Authority
Containment
IntegrityRisk
```

Derived pressures:

```text
AutonomyPressure
VerificationPressure
RepairPriority
QuarantinePressure
```

Required action resolver order:

```text
1. Schema invalidity
2. Authority/permit failure
3. Hard veto
4. Quarantine rule
5. Minimum action floor
6. Score-derived action
7. Operator override if permitted
8. Post-decision invariant validation
```

## Required audit fixture outcomes

These fixture cases must exist and pass:

```text
low_hazard_confirmed              -> low action / backlog or allow-with-receipt
high_hazard_confirmed_fixable     -> GenerateRepairPacket
high_hazard_uncertain             -> RequireVerification
source_truth_drift                -> at least RequireVerification
false_completion_missing_tests    -> GenerateRepairPacket
unknown_owner_mutation            -> RequireOwnerResolution or BlockMutation
destructive_missing_rollback      -> BlockRelease
feut_contamination                -> QuarantineArtifact and hostile script failure if in production path
```

## Final response requirements

Your final report must include:

- changed files
- exact commands run
- exact results
- fixture decisions
- seeded violation results
- unresolved risks
- assumptions
- non-goals preserved
- confirmation that P0A did not integrate into Recall/AiDENs/memory/retrieval/tools
- next pass recommendation

Use `target_files/templates/FINAL_REPORT.md` as the structure if this bundle is available.
