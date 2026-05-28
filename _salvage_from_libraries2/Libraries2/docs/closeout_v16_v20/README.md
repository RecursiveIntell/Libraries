# Codex Finish-Closeout Pack — v16 through v20

This pack is the **repo-grounded closeout bundle** for the next Codex pass.

It is based on the actual March 14, 2026 workspace snapshot, where the repo already contains:

- `federated-settlement`
- `mechanism-runtime`
- `discovery-portfolio`
- `constitutional-memory`
- `spec-execution`
- v16–v20 fixtures and schema manifests
- schema registration in `contract-schema-gen`
- reference-interpreter wrappers in `kernel-conformance`

The next pass should **finish the bounded closeout**, not widen the horizon again.

## Current landed status

The bounded closeout pass now lands the high-value gaps this pack called out:

- v16 owns shared replay, divergence, and treaty suspension artifacts.
- v17 fit evaluation consumes refuter-suite and stability-report inputs.
- v18 portfolio traces carry information-value, budget-pressure, and hypothesis context.
- v19 amendment/archive paths bind rollback, semantic-diff linkage, and historical-query guarantees.
- v20 emits generated companion bundles plus a self-hosting build receipt with veto/challenge baseline handling.

The repo still tells the truth that these are bounded advisory slices rather than mature autonomous runtimes.

## What “finish” means here

“Finish” does **not** mean fully completing v16–v20.

It means the repo should leave the next pass with:

- the current new crates telling the truth about their maturity,
- the missing high-value artifact families added,
- shallow evaluators upgraded into bounded lawful slices,
- schema/fixture/test coverage expanded to match the new artifact families,
- reference-interpreter and conformance surfaces widened enough to prove the slices,
- and root docs updated so the repo is no longer ahead of its own operating instructions.

## What this pack includes

- `SNAPSHOT_CLOSEOUT_SUMMARY.md`
- `CLOSEOUT_SCOPE_AND_NON_GOALS.md`
- `MASTER_ISSUE_MATRIX_CLOSEOUT.md`
- `MASTER_ISSUE_MATRIX_CLOSEOUT.json`
- `EXACT_FILE_TOUCH_MAP.md`
- `PER_CRATE_CLOSEOUT_PLAN.md`
- `CONFORMANCE_AND_FIXTURE_EXPANSION_PLAN.md`
- `RELEASE_BAR_AND_ACCEPTANCE.md`
- `COMMANDS_AND_GREPS.md`
- `CODEX_FINISH_OPERATING_PROMPT.md`
- `CODEX_FINISH_HANDOFF_PROMPT.txt`
- `AGENTS.md`
- `TASK_GRAPH.json`
- `run_closeout_checks.sh`
- `templates/...` crate-level README/AGENTS starter files

## Recommended use

1. Give Codex `CODEX_FINISH_HANDOFF_PROMPT.txt`.
2. Mount this pack alongside the repo.
3. Tell Codex to work in issue order from `MASTER_ISSUE_MATRIX_CLOSEOUT.md`.
4. Require it to run `run_closeout_checks.sh` before claiming completion.
