# P20 Rollback, Repair, Quarantine, and Contradiction Handling

## Failure handling ladder

1. Halt.
2. Identify invariant violated.
3. Determine canonical owner.
4. Repair by delegating to canonical owner if possible.
5. If owner unclear, quarantine and report ambiguity.
6. If repair breaks build, rollback specific change and mark blocker.
7. If feature cannot be made truthful, demote docs/status.
8. If core acceptance gate cannot pass, mark `P20 FAILED`.

## Contradiction handling

Contradictions are not score drops. They must be represented as typed records or final audit findings.

Minimum contradiction record fields:

- identifier;
- files/types involved;
- invariant violated;
- canonical owner expected;
- observed behavior;
- repair/quarantine action;
- residual risk;
- verification command.

## Rollback policy

Do not rollback broad swaths of P00-P19 work unless the blast radius is bounded and documented. Prefer targeted repair or quarantine.

## Compatibility layer policy

Compatibility layers are forbidden when they silently widen semantics. A compatibility adapter is allowed only if:

- explicitly named as legacy/compat;
- non-authoritative;
- emits a repair/degradation receipt;
- has tests proving it does not reinterpret canonical meaning.
