# Codex implementation prompt — final v21/v24 pass

You are landing the post-v20 final closeout pack in the libraries workspace.

## Goal
Implement the additive owner-crate, schema, fixture, and integration surfaces for v21–v24.

## Read first
- `00_START_HERE.md`
- `plans/libraries-v21-v24-final.execplan.md`
- `docs/closeout_v21_v24/MASTER_ISSUE_MATRIX_CLOSEOUT.md`
- `docs/closeout_v21_v24/EXACT_FILE_TOUCH_MAP.md`
- `docs/closeout_v21_v24/PER_CRATE_CLOSEOUT_PLAN.md`
- `docs/closeout_v21_v24/RELEASE_BAR_AND_ACCEPTANCE.md`

## Requirements
- keep names exactly aligned with the pack,
- add the four new owners explicitly,
- publish all schemas/examples/manifests,
- keep the existing canonical lane authoritative,
- preserve advisory/non-admitted labels honestly,
- refuse any move that silently creates a v25.

## Forbidden shortcuts
- no score-only release gates,
- no credential-only delegation,
- no emergency logic hidden in operator docs,
- no live effects emitted only as logs,
- no folding the new families into generic catch-all events.
