# P24 Phase 01 prompt — Verifier, script, and package self-replay hardening

Goal: Make the assertion layer bounded, non-hanging, path-pruned, and receipt-emitting. This protects the super-pass from false completion.

Execute only this phase unless explicitly running the full P24 super-pass.

## Required outputs

- `AiDENs/handoffs/p24/PHASE_01_REPORT.md`
- command transcript and artifact hashes
- changed files list
- pass/fail status for relevant acceptance gates

## Rules

- Do not weaken canonical ownership.
- Do not promote scaffold surfaces without tests.
- Do not leave a phase without either passing evidence or a blocked handoff.
- Re-run global invariant checks before moving to the next phase.

## Phase-specific checklist

See `P24_PHASE_PLAN.md`, `P24_ACCEPTANCE_GATES.md`, and `P24_ISSUE_MATRIX.md`.
