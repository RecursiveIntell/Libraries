# Phase 06 — CLI separation, fixtures, and negative tests

## Goal

Make CLI behavior auditable and fixture discipline strict.

## Tasks

1. CLI commands must be separated:
   - `generate-schemas`
   - `eval-fixtures`
   - `verify-fixtures`
   - `explain-receipt`
   - `validate-receipt`
   - `hash-policy`
2. `verify-fixtures` must be read-only and fail on drift.
3. `eval-fixtures --write` may update expected outputs only when `docs/P32_POLICY_CHANGE_RECEIPT.md` is updated.
4. Add negative fixtures:
   - missing authority material mutation
   - insufficient evidence release
   - unknown owner mutation
   - destructive missing rollback
   - wrong policy domain
   - unknown hard rule
   - schema empty strings
   - invalid recorded time if RFC3339 chosen
   - same signal/different action
5. Existing golden fixtures must be updated only with rationale.

## Acceptance gate

- CLI help documents generation vs verification.
- Negative fixture suite fails before fix and passes after.
- Golden update has a policy-change receipt.
