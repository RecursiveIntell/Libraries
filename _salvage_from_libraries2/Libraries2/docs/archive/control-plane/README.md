# Control-Plane Archive

This directory quarantines historical control-plane generations that are no longer active repo
entrypoints.

Use the active root finish-line pack for current work:

1. `README.md`
2. `PACK_README.md`
3. `SOURCE_BASIS.md`
4. `AGENTS.md`
5. `STATUS_DASHBOARD.md`
6. `MASTER_ISSUE_MATRIX.md`
7. `CONFORMANCE_GATES.md`
8. `PHASED_EXECUTION_PLAN.md`

Preserved architecture law remains in the canonical stack specs, but those specs are not the repo
front door for implementation sequencing.

Archive layout:

- `root/`: historical root control-plane docs, prompt packs, matrices, and status snapshots
- `stack-ids/`: crate-local stale control-plane copies moved out of the active crate surface

Files in this archive may still be useful for lineage or migration history, but they must not be
used as active planning law or status truth.

Archive movement rules:

- If a historical front-door document is superseded, move it under `ARCHIVE/` instead of leaving a competing root entrypoint behind.
- Archived control-plane files must not remain in the active root read order or release checklist.
- If a root-pack document is replaced, update the active read order in this README and the root finish-pack docs in the same change.
