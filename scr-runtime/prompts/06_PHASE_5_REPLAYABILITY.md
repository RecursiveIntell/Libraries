# Phase 5 — CLI, Golden Fixtures, and Conformance

Implement `scr-cli`.

Required commands:

```text
scr-cli evaluate-audit <case.json> --policy <policy.toml>
scr-cli explain-receipt <receipt.json>
scr-cli verify-fixtures
```

## Replayability requirements

- Same input + same policy + same algorithm -> identical receipt.
- Changing input changes input hash.
- Changing policy changes policy hash.
- `explain-receipt` explains from receipt contents, not current policy.
- Golden expected outputs cannot be changed without `POLICY_CHANGE.md`.

## Required docs

```text
docs/EVALUATOR_REFERENCE.md
docs/DECISION_RECEIPTS.md
```

## Tests

- CLI output matches expected golden files
- explain-receipt does not re-evaluate
- fixture receipts include input hash, policy hash, algorithm ID, axes, pressures, chosen action, reason codes
