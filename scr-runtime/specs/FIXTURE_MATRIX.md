# Fixture Matrix

| Case | Required outcome | Required reason codes |
|---|---|---|
| low_hazard_confirmed | Backlog/AllowWithReceipt | LOW_HAZARD, SUFFICIENT_EVIDENCE |
| high_hazard_confirmed_fixable | GenerateRepairPacket | HIGH_HAZARD, HIGH_CONFIDENCE, FIXABLE |
| high_hazard_uncertain | RequireVerification | HIGH_HAZARD, LOW_CONFIDENCE_OR_HIGH_UNCERTAINTY |
| source_truth_drift | RequireVerification or stronger | SOURCE_TRUTH_DRIFT |
| false_completion_missing_tests | GenerateRepairPacket | FALSE_COMPLETION, MISSING_TESTS |
| unknown_owner_mutation | RequireOwnerResolution or BlockMutation | UNKNOWN_OWNER, MUTATION_REQUESTED |
| destructive_missing_rollback | BlockRelease | DESTRUCTIVE_CHANGE, MISSING_ROLLBACK |
| feut_contamination | QuarantineArtifact | FORBIDDEN_PRODUCTION_TERM |
