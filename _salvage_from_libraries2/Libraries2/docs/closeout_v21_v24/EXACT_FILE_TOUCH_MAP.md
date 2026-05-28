# Exact file touch map

## New top-level crates to add
- `effect-runtime/`
- `authority-delegation/`
- `assurance-runtime/`
- `continuity-runtime/`

## Existing workspace files to modify
- root `Cargo.toml` — add four workspace members and default-members
- `stack-ids/src/*` — land v21–v24 ID wrappers
- `contract-schema-gen/src/*` — register every new schema family
- `semantic-memory/sql/*` and `semantic-memory/src/*` — additive storage/query preservation
- `knowledge-runtime/src/*` — bounded views over effect/delegation/assurance/continuity artifacts
- `verification-control/src/*` — effect, delegation, release, and continuity review cases
- `verification-policy/src/*` — policy objects for effect, delegation, release, and continuity
- `verification-adjudication/src/*` — effect and release adjudication receipts
- `llm-tool-runtime/src/*` — effect-bearing dispatch receipt linkage

## New canonical artifact directories
- `schemas/`
- `examples/`
- `contracts/schemas/v21/` through `contracts/schemas/v24/`
- `contracts/fixtures/v21/` through `contracts/fixtures/v24/`
- `conformance/v21/` through `conformance/v24/`

## New execution-lane docs
- `plans/libraries-v21-v24-final.execplan.md`
- `docs/closeout_v21_v24/*`
- `prompts/codex_finish_handoff_prompt_v21_v24.txt`

## Rule

This pass is additive.
Do not rewrite the v16–v20 owners to absorb v21–v24; add the new owners explicitly and wire them lawfully into the existing lane.
