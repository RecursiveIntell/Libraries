# v25 conformance

This directory tracks the conformance target for v25 — Effective Constitution, Profile Composition, and Obligation Folding Runtime.

## Required checks

- schema and example publication is complete,
- deterministic `ProfileSetV1` normalization remains stable,
- blocked, exception, conflict, diff, delegation, release, continuity, and vendor fixtures all exist,
- `profile-runtime` remains the canonical composition owner,
- current repo entry points teach v25 rather than the historical no-v25 terminal position,
- and `libraries-source/` can be synced from the active repo root.

## Canonical owner

- `profile-runtime`

## Fixture directory

- `contracts/fixtures/v25/`

## Local validation

```bash
bash scripts/check_v25_repo_truth.sh
python3 scripts/check_v25_json_surface.py
```
