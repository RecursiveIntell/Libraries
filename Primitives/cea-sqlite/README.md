# cea-sqlite

SQLite implementation of the `cea-store` observational-graph contract.

## Behavior

- schema versioning and legacy v1-to-v2 migration;
- WAL mode, foreign keys, and a bounded busy timeout;
- transactional node/edge/run-log updates;
- exact persistence of normalized attribution weights;
- persisted positive and negative Beta evidence; and
- version-filtered graph loading.

The adapter owns persistence mechanics, not causal policy. `cea-store` rejects
all non-observational evidence before this adapter mutates the graph. Legacy
rows are migrated conservatively and are not reclassified as intervention
evidence.

## Verification

```bash
cargo test -p cea-sqlite --all-targets
```
