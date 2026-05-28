# Repo gap report — v25 (2026-03-17)

## What the code snapshot already had after the previous pass

- `profile-runtime` existed and was wired into the workspace.
- v25 IDs were already present in `stack-ids`.
- `contract-schema-gen` already published the v25 family set.
- `knowledge-runtime` already exposed v25 runtime views.
- a first baseline fixture set already existed.

## What still made the repo operationally incomplete

- the repo root still taught an older March 15 no-v25 terminal position,
- there was no repo-local v25 canonical spec file,
- there was no repo-facing docs/v25 execution pack,
- there was no fixture manifest,
- the mirror sync script still relied on a stale enumerated path list,
- and there was no non-Rust repo-truth / JSON-surface validation path.

## What this pass closes

- current taught-surface truth,
- v25 docs pack in the repo,
- v25 and v26 canonical spec files in the repo root,
- broader fixture corpus and fixture manifest,
- whole-tree mirror sync,
- and repo-truth / JSON-surface scripts.

## What still remains outside this environment

- Rust compilation and test execution,
- schema regeneration from Rust types,
- direct effect/control/adjudication consumer adoption,
- and CI enforcement of the no-local-recomposition rule.
