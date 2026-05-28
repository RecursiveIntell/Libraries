# v25 production apply sequence — 2026-03-18

## Before coding

1. Run `bash scripts/check_v25_production_pack_truth.sh`.
2. Read the production gap audit and issue matrix.
3. Create a working branch specifically for the production closure pass.

## During coding

1. Land `effect-runtime` typed IDs and v25 citation fields.
2. Land the control/policy/adjudication citations.
3. Land remote admission and settlement citations.
4. Regenerate schemas and backfill all example JSON files.
5. Add tests and fixture corpus updates.
6. Add CI/local gate wiring.

## Before opening a PR

1. Run all commands from `docs/v25/PRODUCTION_ACCEPTANCE_AND_COMMANDS_20260318.md`.
2. Sync `libraries-source/`.
3. Update any status docs that still mention downstream gaps as unresolved.
4. Re-run `scripts/audit_v25_production_gap.py` and confirm the generated report now reflects the closed state.
