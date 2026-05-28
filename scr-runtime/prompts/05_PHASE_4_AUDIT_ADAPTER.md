# Phase 4 — Audit Adapter

Implement `scr-audit-adapter`.

Create audit input mapping from fixture issue fields to core evaluation axes.

Required components:

```text
source_truth_drift
false_completion
rollback_status
owner_resolution
missing_tests
reproduction_quality
exact_file_anchor
blast_radius
fixability
recurrence
```

Create docs:

```text
docs/AUDIT_ADAPTER.md
```

## Required fixture cases

Create under `fixtures/audit/cases/`:

```text
low_hazard_confirmed.json
high_hazard_confirmed_fixable.json
high_hazard_uncertain.json
source_truth_drift.json
false_completion_missing_tests.json
unknown_owner_mutation.json
destructive_missing_rollback.json
feut_contamination.json
```

Create expected outputs or expected minimums under `fixtures/audit/expected/`.

## Required outcomes

```text
source_truth_drift              -> at least RequireVerification
false_completion_missing_tests  -> GenerateRepairPacket
unknown_owner_mutation          -> RequireOwnerResolution or BlockMutation
destructive_missing_rollback    -> BlockRelease
feut_contamination              -> QuarantineArtifact
```
