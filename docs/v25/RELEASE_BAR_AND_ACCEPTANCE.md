# Release bar and acceptance — v25

## Release bar

v25 is considered repo-ready only if all of the following are true:

- the repo carries the v25 canonical spec in the root,
- current entry points point to v25 rather than the historical no-v25 terminal pack,
- `profile-runtime` is in the workspace,
- all nine v25 artifact families have canonical schemas, examples, and manifest entries,
- the fixture corpus covers block, exception, conflict, diff, delegation, release, continuity, and vendor cases,
- v25 repo-truth and JSON-surface checks pass,
- and the `libraries-source/` mirror can be synced from the active repo root.

## Acceptance checklist

- [ ] `24_V25_SUPERSESSION_AND_CONSTITUTIONAL_CHANGE_NOTE_20260317.md` exists
- [ ] root v25 and v26 canonical spec files exist
- [ ] `docs/v25/` pack exists
- [ ] `plans/v25-effective-constitution.execplan.md` exists
- [ ] `scripts/check_v25_repo_truth.sh` passes
- [ ] `scripts/check_v25_json_surface.py` passes
- [ ] `apply/v25/SYNC_LIBRARIES_SOURCE_MIRROR.sh` uses whole-tree sync rather than a stale path list
- [ ] Rust tooling is later used to run the required cargo commands

## Forbidden shortcuts

The following remain disqualifying:

- teaching v25 only through one external zip and not through the repo itself,
- silently leaving the older no-v25 entry points as the current taught surface,
- adding more v25 schemas/examples while leaving the fixture manifest stale,
- relying on hand-maintained mirror lists after the repo file set grew,
- or claiming v25 is fully landed while effect/control/adjudication consumer work remains unproven.
