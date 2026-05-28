# P29 Phase 00 — Source basis and run identity lock

## Objective

Source basis and run identity lock.

## Scope

Follow the phase plan and master packet for this phase.

## Required actions

1. Read the relevant matrix rows for this phase.
2. Make focused changes only.
3. Add or update tests/assertions for every bug class fixed.
4. Update or create `handoffs/p29/PHASE_00_REPORT.md`.
5. Update `P29_STATUS_EVIDENCE_MANIFEST.json` if this phase produces final evidence.
6. Quarantine any issue that cannot be safely fixed.

## Evidence requirements

The phase report must include:

- changed files;
- issue IDs fixed;
- issue IDs quarantined;
- tests/checks run;
- evidence files;
- support-label impact;
- remaining risk.

## Stop conditions

Stop and repair if:

- current run identity drifts from P29;
- P29 files are archived as stale;
- package/evidence paths become missing;
- a v11A/v11B/v11C claim is made without evidence;
- cargo/clippy/test failures are ignored;
- canonical sibling ownership is violated.

