# P30 Rollback and Quarantine Plan

## Rollback standard

Every risky change must be small enough to revert independently. For each phase, record:

- files changed;
- tests added;
- issue IDs targeted;
- rollback command or manual revert path;
- expected state after rollback.

## Quarantine standard

Quarantine is allowed only when fixing an issue would require owner ambiguity resolution or a larger architectural pass. Quarantine must produce an entry with:

- issue ID;
- reason;
- owner;
- risk if left unresolved;
- temporary guard preventing worse behavior;
- next-pass trigger.

## Required quarantine files

- `handoffs/p30/UNRESOLVED_RISK_LEDGER.md`
- `handoffs/p30/OWNERSHIP_AMBIGUITY_LEDGER.md`
- `handoffs/p30/QUARANTINE_LEDGER.md`

## Stop conditions

Stop instead of fixing forward blindly when:

- a sibling crate owns the semantics;
- a fix requires changing artifact law but no reference behavior exists;
- a repair path would silently reinterpret data;
- a test reveals a larger invariant break than the phase scope.
