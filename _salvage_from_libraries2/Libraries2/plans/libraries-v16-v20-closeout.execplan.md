# Libraries v16–v20 closeout ExecPlan

## Purpose

Finish the current libraries workspace from its **actual checked-out state**.

This pass is a closeout pass for the v16–v20 horizon crates already present in the repo. The goal is not to pretend those waves are fully operational. The goal is to finish the lawful typed surface, deepen the current evaluators, wire schema publication completely, expand conformance coverage, and leave the repo in a truthful release-ready state.

## Current reality

### Present in this checkout
- `stack-ids/` contains the extended identifier/types substrate.
- `contract-schema-gen/` exists and is the correct schema publication seam.
- `federated-settlement/`, `mechanism-runtime/`, `discovery-portfolio/`, `constitutional-memory/`, and `spec-execution/` already exist as bounded crates.
- Each new crate already has a `src/lib.rs` and at least one slice test.

### Still incomplete
- some v16 artifact families are still shallow or missing richer downgrade/suspension/replay semantics
- v17 fit/refuter linkage is still thin
- v18 portfolio choice is still weakly value-aware
- v19 archive/deprecation/rollback flows are still narrow
- v20 self-hosting outputs need a richer build receipt and stronger generated-artifact / veto / proof-obligation linkage
- crate READMEs/AGENTS and closeout docs need to exist in-repo so coding agents stop reaching for stale plans

## Active target files

Primary code paths for this pass:

- `stack-ids/src/ids.rs`
- `stack-ids/src/lib.rs`
- `contract-schema-gen/src/lib.rs`
- `contract-schema-gen/src/main.rs`
- `federated-settlement/src/lib.rs`
- `federated-settlement/tests/settlement_slice.rs`
- `mechanism-runtime/src/lib.rs`
- `mechanism-runtime/tests/mechanism_fit_slice.rs`
- `discovery-portfolio/src/lib.rs`
- `discovery-portfolio/tests/portfolio_slice.rs`
- `constitutional-memory/src/lib.rs`
- `constitutional-memory/tests/amendment_slice.rs`
- `spec-execution/src/lib.rs`
- `spec-execution/tests/spec_to_schema_slice.rs`

Primary supporting files for this pass:

- `docs/closeout_v16_v20/*`
- `prompts/codex_finish_operating_prompt_v16_v20.md`
- `prompts/codex_finish_handoff_prompt_v16_v20.txt`
- `scripts/run_v16_v20_closeout_checks.sh`

## Milestones

### M0 — source-of-truth repair
- replace stale execution entrypoints with current-repo guidance
- archive the Forge Workbench perfection plan in place
- land active closeout docs into repo-root-relative directories

### M1 — artifact family completion
- fill the missing typed artifacts called out in `docs/closeout_v16_v20/MASTER_ISSUE_MATRIX_CLOSEOUT.md`
- ensure every wire-visible artifact has schema generation coverage where applicable
- expand fixtures/examples if the generator requires them

### M2 — evaluator deepening
- extend the bounded evaluators in each v16–v20 crate without over-claiming maturity
- prefer explicit downgrade / advisory / veto / refuter semantics over opaque scoring

### M3 — conformance expansion
- extend slice tests into stronger scenario tests
- add degraded-path and receipt assertions
- ensure the closeout check script passes cleanly

### M4 — release-bar truthfulness
- ensure crate READMEs and AGENTS reflect actual support level
- ensure no doc in the active lane over-claims operational completeness

## Hard rules

- do not invent Forge Workbench paths
- do not build app/UI code that this checkout does not contain
- do not mark any horizon crate as production-complete unless the release bar explicitly says so
- do not add schemas without tests/fixtures/receipts where the crate semantics require them
- do not bypass proof-obligation, downgrade, dissent, or veto semantics for convenience

## First execution slice

Start with the active code surface only:

1. `stack-ids/`
2. `contract-schema-gen/`
3. `federated-settlement/`
4. `mechanism-runtime/`
5. `discovery-portfolio/`
6. `constitutional-memory/`
7. `spec-execution/`

Then run the closeout checks and tighten any failing assertions before widening scope.
