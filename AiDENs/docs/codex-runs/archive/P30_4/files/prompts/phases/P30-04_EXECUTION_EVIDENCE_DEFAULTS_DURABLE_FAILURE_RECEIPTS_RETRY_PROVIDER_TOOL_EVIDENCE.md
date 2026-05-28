# P30-04 Prompt — Execution evidence defaults, durable failure receipts, retry/provider/tool evidence

## Phase goal

Execution evidence defaults, durable failure receipts, retry/provider/tool evidence.

## Assigned issue count

37 hostile audit issues are assigned to this phase through `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`.

## Required steps

1. Load `AGENTS.md`, `P30_CODEX_SUPER_PASS_PROMPT.md`, and this phase prompt.
2. Filter the issue matrix for `phase=P30-04`.
3. Fix `must-fix` rows first; quarantine only with explicit owner/blocker/next-pass evidence.
4. Add or update tests that reproduce the hostile break tests.
5. Run targeted tests and `scripts/p30_verify.sh` when available.
6. Emit `handoffs/p30/P30-04_REPORT.md`.

## Phase-specific focus

Make execution evidence durable by default. Failure paths must never silently return empty evidence. Default receipt level must support replay/conformance unless explicitly downgraded by policy receipt.

## Stop condition

Do not proceed if this phase introduces hidden truth, silent widening, unreceipted material work, or unowned semantics.
