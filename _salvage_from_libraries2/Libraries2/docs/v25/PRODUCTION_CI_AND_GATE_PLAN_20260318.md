# Production CI and gate plan — 2026-03-18

## Goal

Make the repo enforce the anti-fork rule:
consumers may cite and consume the composite constitutional lane, but they may not privately recompute profile interaction.

## New local targets to add

Add these Makefile targets:

- `v25-local-checks` -> runs the existing `scripts/run_v25_local_checks.sh`
- `no-local-recomposition-check` -> runs `scripts/check_no_local_recomposition.sh`
- `v25-production-pack-check` -> runs `scripts/run_v25_production_pack_checks.sh`
- `v25-production-closure` -> runs `scripts/run_v25_production_pack_checks.sh --final`

## New scripts in this pack

- `scripts/check_v25_production_pack_truth.sh` — validates the Codex closure pack itself
- `scripts/check_no_local_recomposition.sh` — fails if target consumers touch raw profile fields or raw profile types
- `scripts/check_v25_production_closure.py` — end-state gate for the production closure
- `scripts/run_v25_production_pack_checks.sh` — wrapper for pack-only or final checks

## CI update to land after code changes

Once the source changes and example/schema backfill are complete, update `.github/workflows/ci.yml` so that a pre-workspace lane runs:

1. `bash scripts/check_v25_repo_truth.sh`
2. `python3 scripts/check_v25_json_surface.py`
3. `bash scripts/check_no_local_recomposition.sh`
4. `python3 scripts/check_v25_production_closure.py`
5. `bash scripts/run_v25_local_checks.sh`

Only after those pass should the broader workspace lane run.

## Important detail

Do **not** wire `check_v25_production_closure.py` into active CI before the code, schema, example, and test work lands.
It is intentionally strict and should be green only at the end of the closure pass.
