# Codex Prompt — P15 Regional decoder kernel, right-graph law, and local repair geometry

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P15_REGIONAL_DECODER_KERNEL_AND_LOCAL_REPAIR_GEOMETRY.md`.

Implement P15 only. Do not start later passes.

## Goal

Implement the first bounded hypergraph/factor-region kernel after the evidence/memory/control substrate is lawful.

## Primary crates

- `aidens-kernel-kit`
- `aidens-repair-kit`
- `aidens-memory-kit`
- `aidens-contracts`
- `aidens-receipts`

## Required artifacts

- `CompiledRegionGraphV1`
- `RegionContractV1`
- `SyndromeV1`
- `ResidualV1`
- `OracleSliceRequestV1`
- `KernelRunDisplayReportV1`
- `ConvergenceReportV1`

## Acceptance gates

- A synthetic contradiction emits SyndromeV1 and local repair candidate instead of global recompute.
- Loopy propagation with non-convergence emits ConvergenceReportV1 and degraded state.
- Oracle slice agrees with approximate path on small gold fixtures or records bounded disagreement.

## Forbidden shortcuts

- Do not treat one giant graph as the runtime.
- Do not report convergence without explicit stop rule evidence.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
