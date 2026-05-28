# P30-00 Prompt — Preflight, source-basis lock, workspace portability, build-certification split

## Phase goal

Preflight, source-basis lock, workspace portability, build-certification split.

## Assigned issue count

5 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-00`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-00_REPORT.md`.

## Phase-specific focus

Establish source-basis truth. Separate package validation, build validation, conformance validation, and release validation. Add workspace-layout preflight and cargo/build command capture. Make standalone-vs-archive-root assumptions explicit.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
