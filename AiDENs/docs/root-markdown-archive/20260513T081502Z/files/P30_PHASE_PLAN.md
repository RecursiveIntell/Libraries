# P30 Phase Plan — v11B-centered

Each phase has a bundle-level prompt and a human manual injection. Do not continue past a phase without a phase report and revalidation.

## P30-00 — Preflight, source-basis lock, workspace portability, v11B target lock

Required output:
- `handoffs/p30/P30-00_REPORT.md`
- current build/check status
- source-basis lock
- v11A prerequisite status
- v11B target surfaces selected
- go/no-go decision

## P30-01 — v11A blocker closure: material receipts, execution context, deterministic IDs, proof/degradation honesty

Required output:
- `handoffs/p30/P30-01_REPORT.md`
- fixed/quarantined P0 blockers
- tests proving no material done-state without receipts
- deterministic ID policy proof
- proof/degradation debt ledger update

## P30-02 — Strict boundary hardening: tool-call parser, structured-output law, patch/rollback/command safety

Required output:
- `handoffs/p30/P30-02_REPORT.md`
- parser hostile fixtures
- rollback failure tests
- command sandbox tests
- treatment-integrity repair receipts

## P30-03 — Execution evidence and durable failure receipts across provider/tool/retry/queue paths

Required output:
- `handoffs/p30/P30-03_REPORT.md`
- tool/retry/provider receipt fixtures
- serialization failure cannot become empty success
- failure receipts durable by default or explicitly quarantined

## P30-04 — v11B right-graph spine: graph surface declarations and graph-misuse gates

Required output:
- `handoffs/p30/P30-04_REPORT.md`
- `GraphSurfaceDeclarationV1` or canonical equivalent
- tests rejecting storage-as-inference shortcuts
- tests rejecting retrieval-as-causal-evidence shortcuts
- graph owner/source-of-truth map

## P30-05 — v11B region protocol: region contracts, boundary messages/receipts, state snapshots, replay slices

Required output:
- `handoffs/p30/P30-05_REPORT.md`
- `RegionContractV1` or canonical equivalent
- `RegionBoundaryMessageV1` / `RegionBoundaryReceiptV1` or equivalents
- `RegionStateSnapshotV1` and replay slice fixture
- boundary reject/quarantine fixture

## P30-06 — v11B convergence, residual, syndrome, local repair law

Required output:
- `handoffs/p30/P30-06_REPORT.md`
- `ResidualEnvelopeV1` / `SyndromeEnvelopeV1`
- `ConvergenceReportV1`
- oscillation/non-convergence degradation fixture
- `RepairCandidateBundleV1` / `RepairExecutionReceiptV1` fixture

## P30-07 — v11B lawful subtraction, delta/invalidation, and causal/interventional packages

Required output:
- `handoffs/p30/P30-07_REPORT.md`
- `SupportCoreV1`
- `RemovalFrontierV1`
- `InvariantPreservationReceiptV1`
- `HistoricalLossBudgetV1`
- delta/invalidation cone fixture
- causal attribution degrade-or-bundle fixture

## P30-08 — Hostile sweep: panic, dynamic JSON, silent degradation, lint suppression, doc/root/gate hygiene

Required output:
- `handoffs/p30/P30-08_REPORT.md`
- hostile pattern sweep results
- root-doc current/archival classification
- gate supersession manifest
- unresolved risk ledger

## P30-09 — Full conformance, replay, package, v11B draft-runtime handoff

Required output:
- `handoffs/p30/P30-09_REPORT.md`
- `handoffs/p30/FINAL_AUDITOR_HANDOFF.md`
- `handoffs/p30/V11B_RUNTIME_SPINE_REPORT.md`
- `handoffs/p30/V11B_CONFORMANCE_DEBT_LEDGER.md`
- `handoffs/p30/P30_RELEASE_CLAIMS.md`
- command output / receipt paths for every final claim
