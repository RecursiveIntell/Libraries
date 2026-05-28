# P30-05 Prompt — Verification semantics, proof debt, degradation honesty, and no advisory promotion

## Phase goal

Verification semantics, proof debt, degradation honesty, and no advisory promotion.

## Assigned issue count

82 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-05`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-05_REPORT.md`.

## Phase-specific focus

Separate advisory observation from verification success. Proof debt, waiver, degradation, and blocked checks must be explicit. No risk-bearing output can self-promote from advisory-only checks.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
