# P30-09 Prompt — Full conformance, replay, final packaging, hostile auditor handoff, and unresolved risks

## Phase goal

Full conformance, replay, final packaging, hostile auditor handoff, and unresolved risks.

## Assigned issue count

0 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-09`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-09_REPORT.md`.

## Phase-specific focus

Run full command bar, package validation, replay checks, final issue absorption report, unresolved risk ledger, final auditor handoff, and precise release claims.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
