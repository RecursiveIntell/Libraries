# P30-08 Prompt — v11A/B seed: artifact runtime, operator receipts, right-graph/region/convergence hooks

## Phase goal

v11A/B seed: artifact runtime, operator receipts, right-graph/region/convergence hooks.

## Assigned issue count

6 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-08`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-08_REPORT.md`.

## Phase-specific focus

Seed v11A/B executable law only where testable: artifact envelopes, operator receipts, boundary compiler profiles, right-graph declarations, minimal region/convergence report hooks. Do not overclaim full compliance.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
