# P29 v11B Executable Seed Spec

P29 may implement v11B seed surfaces only after v11A local release gates pass.

## Required seed surfaces

### Right-graph declarations

- storage graph;
- retrieval graph;
- inference graph;
- repair graph;
- subtraction graph;
- control/receipt graph;
- causal/intervention graph.

### Region contract

- `RegionContractV1`;
- `BoundaryMessageV1`;
- `BoundaryReceiptV1`;
- `RegionReplaySliceV1`.

### Convergence/residual/syndrome

- `ConvergenceReportV1`;
- `ResidualEnvelopeV1`;
- `SyndromeEnvelopeV1`;
- convergence budget;
- failed convergence degradation.

### Lawful subtraction

- `SupportCoreV1`;
- `RemovalFrontierV1`;
- `InvariantPreservationReceiptV1`;
- `HistoricalLossBudgetV1`.

## Required non-claim

These are executable seeds, not full v11B completion.
