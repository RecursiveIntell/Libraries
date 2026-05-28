# P29 Rollback and Quarantine Plan

## Rule

If a fix cannot be completed safely, quarantine it. Do not pretend it is done.

## Quarantine record format

Each quarantine must include:

- issue ID;
- affected file/module;
- current behavior;
- attempted fix;
- reason blocked;
- support label impact;
- required next pass;
- evidence.

## Required quarantine categories

- `docs/p29/quarantine/hnsw.md`
- `docs/p29/quarantine/sqlite_migrations.md`
- `docs/p29/quarantine/search_ranking.md`
- `docs/p29/quarantine/v11a_contracts.md`
- `docs/p29/quarantine/v11b_seed.md`
- `docs/p29/quarantine/unaudited_high_risk_surfaces.md`

## Rollback triggers

Rollback or quarantine if:

- a fix introduces nondeterminism;
- cargo test fails and failure is not understood;
- clippy requires suppressing warnings instead of fixing root cause;
- a v11A/v11B claim cannot be evidenced;
- package self-replay fails;
- manifest path validator fails.
