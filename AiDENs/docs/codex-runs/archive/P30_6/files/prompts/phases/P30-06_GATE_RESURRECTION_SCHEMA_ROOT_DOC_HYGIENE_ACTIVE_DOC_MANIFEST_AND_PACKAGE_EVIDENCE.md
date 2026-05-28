# P30-06 Prompt — Gate resurrection, schema/root-doc hygiene, active-doc manifest, and package evidence

## Phase goal

Gate resurrection, schema/root-doc hygiene, active-doc manifest, and package evidence.

## Assigned issue count

11 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-06`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-06_REPORT.md`.

## Phase-specific focus

Restore gate truth. Add missing/superseded gate scripts/schemas or a gate-supersession manifest. Normalize active docs and root Markdown classification so stale runs cannot steer current instructions.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
