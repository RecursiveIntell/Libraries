# P21 Deletion / Quarantine Rules

## Delete only when safe

Codex may delete code only when all are true:

1. the code is dead or scaffold-only;
2. no supported command/test depends on it;
3. deletion does not remove canonical backpointers/receipts/evidence;
4. the final report lists the deletion.

## Quarantine instead of guessing

Quarantine when:

- canonical ownership is unclear;
- a feature is useful but unsupported;
- a provider path is partial;
- a Recall/Recall-Coding pattern is app-specific;
- a local type might be shadow truth.

Suggested quarantine path:

```text
docs/p21/quarantine/<topic>.md
```

## Do not delete

- tests/evals/fixtures to make scanners pass;
- scripts referenced by code;
- canonical adapter code;
- receipt/backpointer preservation;
- agency policy surfaces;
- provider honesty reports;
- generated audit artifacts.
