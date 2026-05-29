# P30 v11B Execution Spine

## Thesis

P30 must stop treating v11B as a late optional seed. The runtime now needs a thin but executable v11B spine:

```text
GraphSurfaceDeclarationV1
+ RegionContractV1
+ RegionBoundaryMessageV1 / RegionBoundaryReceiptV1
+ RegionStateSnapshotV1 / RegionReplaySliceV1
+ ResidualEnvelopeV1 / SyndromeEnvelopeV1
+ ConvergenceReportV1
+ RepairCandidateBundleV1 / RepairExecutionReceiptV1
+ SupportCoreV1 / RemovalFrontierV1 / InvariantPreservationReceiptV1 / HistoricalLossBudgetV1
+ CausalAttributionBundleV1 where attribution exists
```

## Required implementation stance

- v11A contracts remain prerequisite law.
- v11B surfaces must be declared as runtime surfaces, not documentation ornaments.
- If a surface cannot be fully implemented, it must have a fixture stub, explicit release debt, and a no-claim policy.

## Minimum viable v11B production path

By the end of P30, at least one bounded path should exercise:

1. graph surface declaration;
2. region contract;
3. typed boundary message;
4. boundary receipt;
5. execution context reference;
6. region state snapshot;
7. replay slice or explicit non-replayability reason;
8. residual or syndrome emission;
9. convergence report or explicit non-iterative proof;
10. degradation/proof debt if incomplete.

## Minimum viable lawful subtraction path

At least one test fixture should attempt a subtraction/summarization/compaction scenario and prove:

1. operator declares `SUBTRACTS_STRUCTURE`;
2. support core is identified or degraded;
3. removal frontier is emitted;
4. protected query/invariant set is declared;
5. invariant-preservation receipt is emitted for applied subtraction;
6. historical-loss budget is emitted when replay/query fidelity weakens;
7. challenge/rollback path exists or debt is explicit.

## Minimum viable causal/attribution path

Any code-change attribution, blame, or fix-causality claim must either:

- use `CausalAttributionBundleV1` with treatment/outcome/confounder/nuisance/refuter fields; or
- degrade/refuse causal language and record why.

## Release claim policy

P30 may produce `v11B-draft-runtime` only. It must not claim `v11B-conformant-runtime` unless all v11B release-bar gates pass with receipts.
