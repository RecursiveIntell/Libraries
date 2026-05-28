# Codex Prompt — P18 Mechanism/theory search, experiment runtime, and falsifiable model library

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P18_MECHANISM_THEORY_SEARCH_AND_EXPERIMENT_RUNTIME.md`.

Implement P18 only. Do not start later passes.

## Goal

Represent candidate mechanisms, simulators, fit runs, stability reports, and theory refuter suites as lawful artifacts.

## Primary crates

- `aidens-kernel-kit`
- `aidens-governance-kit`
- `aidens-memory-kit`
- `aidens-contracts`
- `aidens-receipts`

## Required artifacts

- `MechanismBundleV1`
- `TheoryVersionV1`
- `HypothesisLibraryV1`
- `SimulatorContractV1`
- `FitRunReportV1`
- `InvarianceReportV1`
- `TheoryRefuterSuiteV1`

## Acceptance gates

- A candidate mechanism can be fit, refuted, versioned, superseded, and replayed from artifacts.
- High score alone cannot promote theory without refuter/gov artifacts.
- Equivalent mechanisms remain distinct unless an explicit equivalence/alias decision is admitted.

## Forbidden shortcuts

- Do not store theory updates as notebook comments or raw model weights only.
- Do not confuse observational equivalence with causal identification.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
