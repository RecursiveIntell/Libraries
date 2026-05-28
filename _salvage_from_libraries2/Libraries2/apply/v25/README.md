# v25 apply pack

This directory is the engineer-facing landing kit for **v25 — Effective Constitution, Profile Composition, and Obligation Folding Runtime**.

## What is in scope now

- root-local v25 and v26 spec files,
- repo-facing docs/v25 execution pack,
- canonical v25 core code and schemas,
- broadened v25 fixture corpus plus manifest,
- repo-truth and JSON-surface checks,
- whole-tree mirror sync for `libraries-source/`.

## What remains explicitly downstream

- direct `effect-runtime` consumption of compiled obligations,
- direct `verification-control` and `verification-adjudication` citation of composite constitutional refs,
- cargo-backed schema regeneration and test execution,
- and CI enforcement of the no-local-recomposition rule.

## Best companion files

1. `../../24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md`
2. `../../docs/v25/README.md`
3. `../../plans/v25-effective-constitution.execplan.md`
4. `../../scripts/check_v25_repo_truth.sh`
5. `../../scripts/run_v25_local_checks.sh`

## Production closure continuation

The production-ready closure lane now starts with:

1. `../../docs/v25/PRODUCTION_GAP_AUDIT_20260318.md`
2. `../../docs/v25/PRODUCTION_MASTER_ISSUE_MATRIX_20260318.md`
3. `../../plans/v25-production-closure.execplan.md`
4. `PRODUCTION_APPLY_SEQUENCE_20260318.md`
5. `../../docs/v25/PRODUCTION_ACCEPTANCE_AND_COMMANDS_20260318.md`

Use `bash scripts/run_v25_production_pack_checks.sh` before coding and `bash scripts/run_v25_production_pack_checks.sh --final` only after the code, schema, and CI work lands.
