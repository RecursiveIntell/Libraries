# P30-02 Prompt — Patch safety, rollback truth, command sandbox, and permit fail-closed behavior

## Phase goal

Patch safety, rollback truth, command sandbox, and permit fail-closed behavior.

## Assigned issue count

5 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-02`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-02_REPORT.md`.

## Phase-specific focus

Make patch and command execution fail closed. Missing/unreadable files must not become empty input. Rollback must return receipts. Command sandbox must use trusted absolute toolchain paths or explicit quarantine; kill process trees, not only direct children.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
