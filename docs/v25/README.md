# V25 repo pack — effective constitution and obligation folding

This directory is the repo-facing implementation and governance pack for:

- `CANONICAL_STACK_SPEC_V25_EFFECTIVE_CONSTITUTION_PROFILE_COMPOSITION_AND_OBLIGATION_FOLDING_RUNTIME.md`

It turns the v25 seam into something landable in the March 16 code snapshot instead of leaving it as a standalone spec pack.

## What this repo pack contains

- the canonical v25 spec inside the repo root,
- repo-truth notes that supersede the older March 15 no-v25 terminal position,
- exact file-touch and per-crate apply maps,
- schema registry and compatibility guidance,
- conformance and release-bar material,
- a broadened fixture corpus and fixture manifest,
- apply scripts and repo-truth checks,
- and mirror-sync instructions for `libraries-source/`.

## What is already landed in code

- `profile-runtime` exists as the new v25 owner crate,
- `stack-ids` carries the v25 identity types,
- `contract-schema-gen` publishes the v25 schema family,
- `knowledge-runtime` exposes v25 runtime views,
- canonical schemas, examples, fixtures, and conformance directories exist.

## What is still downstream after this pass

- direct effect-path consumption of `CompiledObligationSetV1`,
- direct control/adjudication citation of `effective_constitution_id` and `compiled_obligation_set_id`,
- full no-local-recomposition enforcement in CI,
- and schema regeneration with the Rust toolchain present.

## Best starting points

1. `../../24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md`
2. `../../CANONICAL_STACK_SPEC_V25_EFFECTIVE_CONSTITUTION_PROFILE_COMPOSITION_AND_OBLIGATION_FOLDING_RUNTIME.md`
3. `MASTER_ISSUE_MATRIX.md`
4. `EXACT_FILE_TOUCH_MAP.md`
5. `PER_CRATE_APPLY_PLAN.md`
6. `RELEASE_BAR_AND_ACCEPTANCE.md`
7. `../../plans/v25-effective-constitution.execplan.md`
8. `../../apply/v25/README.md`

## Production closure pack

For the current terminal closure pass, continue from:

1. `PRODUCTION_GAP_AUDIT_20260318.md`
2. `PRODUCTION_MASTER_ISSUE_MATRIX_20260318.md`
3. `../../plans/v25-production-closure.execplan.md`
4. `PRODUCTION_EXACT_FILE_TOUCH_MAP_20260318.md`
5. `PRODUCTION_ACCEPTANCE_AND_COMMANDS_20260318.md`
6. `../../prompts/codex_finish_handoff_prompt_v25_production.txt`
