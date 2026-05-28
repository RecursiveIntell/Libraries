# P30-03 Prompt — Replay identity, deterministic material IDs, and exposure/attempt identity law

## Phase goal

Replay identity, deterministic material IDs, and exposure/attempt identity law.

## Assigned issue count

183 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-03`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-03_REPORT.md`.

## Phase-specific focus

Eliminate nondeterministic material IDs. Rename or quarantine process-local ID helpers. Derive tool exposure, operator invocation, receipt, manifest, and attempt IDs from material inputs or explicit replay handles.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
