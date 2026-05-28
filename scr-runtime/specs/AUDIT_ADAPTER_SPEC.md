# Audit Adapter Spec

The audit adapter maps audit issue fixtures to `ControlEvaluationInputV1`.

It does not decide truth. It evaluates proposed actions such as:

```text
ImplementFix
GenerateRepairPacket
BlockRelease
QuarantineArtifact
RequireOwnerResolution
```

## Required audit components

- severity
- blast radius
- source truth drift
- false completion risk
- missing tests
- rollback availability
- owner known
- reproduction quality
- exact file anchor quality
- recurrence
- fixability

## Required fixture outcomes

```text
low_hazard_confirmed              -> low action / backlog or allow-with-receipt
high_hazard_confirmed_fixable     -> GenerateRepairPacket
high_hazard_uncertain             -> RequireVerification
source_truth_drift                -> at least RequireVerification
false_completion_missing_tests    -> GenerateRepairPacket
unknown_owner_mutation            -> RequireOwnerResolution or BlockMutation
destructive_missing_rollback      -> BlockRelease
feut_contamination                -> QuarantineArtifact
```
