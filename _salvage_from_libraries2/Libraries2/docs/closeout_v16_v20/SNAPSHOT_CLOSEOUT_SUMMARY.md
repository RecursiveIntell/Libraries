# Snapshot Closeout Summary

## Repo baseline

The workspace is already past the “specs only” phase.

The snapshot contains:

- five new horizon-owner crates:
  - `federated-settlement`
  - `mechanism-runtime`
  - `discovery-portfolio`
  - `constitutional-memory`
  - `spec-execution`
- generated schema registration for v16–v20 in `contract-schema-gen`
- fixture sets under `contracts/fixtures/v16` through `v20`
- schema manifests under `contracts/schemas/v16` through `v20`
- reference-interpreter wrappers in `kernel-conformance/src/reference_interpreters.rs`
- at least one direct slice test per new crate

## What is already good

1. Canonical nouns now have owners.
2. IDs and schemas exist for the current artifact families.
3. Advisory-only / degraded / non-admitted posture is preserved.
4. The repo has begun proving slices rather than only naming them.

## What is still unfinished

The current gap is not “missing architecture.”
The current gap is **thin runtime depth**.

Across the new crates, the main unfinished seams are:

- missing artifact families that the specs call out but the code does not yet own,
- evaluators that are still narrow booleans or straight-line selectors,
- limited cross-crate use of the richer artifacts already defined,
- missing crate-local READMEs / AGENTS for the new owners,
- missing closeout fixtures and tests for replay, challenge, suspension, veto, rollback, and budget-pressure paths,
- one explicit spec drift: `SelfHostingBuildReceiptV1` is called out in the v20 build order but is not yet represented in `stack-ids`, `spec-execution`, or schemas.

## Closeout thesis

The next Codex pass should therefore do four things:

1. **Complete the missing artifact vocabulary** for the current bounded slices.
2. **Deepen each evaluator one notch**, enough to make the slice materially more real.
3. **Expand conformance and fixtures** so the repo can prove the new behavior.
4. **Tighten docs and crate ownership language** so the code and the operating model stop drifting.

## Definition of success for the next pass

Success is reached when:

- the new crates have crate-local README/AGENTS ownership docs,
- the main missing artifact families are present and schema-owned,
- v16–v20 each have one richer bounded vertical slice than the current snapshot,
- `kernel-conformance` proves those slices with explicit degraded/advisory behavior,
- `contract-schema-gen` and manifest files are in sync,
- and the repo still tells the truth that these are bounded slices, not mature end-state runtimes.
