# P30-01 Prompt — Executable tool-call parser boundary and strict structured-output law

## Phase goal

Executable tool-call parser boundary and strict structured-output law.

## Assigned issue count

5 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-01`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-01_REPORT.md`.

## Phase-specific focus

Make executable tool-call parsing strict. Replace dropping/filtering with rejected-call receipts. Never allow permissive degraded repair to feed executable calls without blocking approval/degradation artifacts. Preserve repair reason codes.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
