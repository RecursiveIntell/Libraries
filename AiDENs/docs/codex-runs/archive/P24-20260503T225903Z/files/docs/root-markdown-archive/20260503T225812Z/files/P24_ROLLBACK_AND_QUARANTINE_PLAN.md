# P24 rollback and quarantine plan

## Non-destructive change law

Do not delete historical pass material unless the repo already has an archival policy and the action emits an archive/quarantine record. Prefer:

- move to `docs/codex-runs/archive/P23/...`, or
- mark as historical/stale in classification, or
- leave in place but exclude from active package profiles.

## Quarantine cases

Quarantine rather than promote:

- local DTOs that resemble canonical owner types but lack aliases/backpointers;
- schema sketches that are not generated from type owners;
- provider/cloud paths without runnable local evidence;
- profile crates without fixture or integration tests;
- repairs that alter treatment/outcome/episode identity without verification plan;
- package/verifier scripts that can hang or scan generated/archive paths.

## Rollback triggers

Rollback the phase if:

- `cargo check` fails due to the phase and cannot be fixed locally;
- canonical type ownership tests fail;
- run-bundle V2 loses P23 fixture capability;
- coding-agent lane allows unapproved writes;
- memory seam uses an AiDENs-local truth store;
- package strict mode emits validation findings.

## Required rollback artifact

Each rollback must create `AiDENs/handoffs/p24/ROLLBACK_<phase>_<slug>.md` containing:

- trigger
- changed files reverted
- changed files preserved
- commands run
- exact failure
- quarantine decision
- next attempt recommendation
