# P24 Phase 00 prompt — Preflight hostile audit and source-basis freeze

Goal: Prove the package, sidecars, source tree, research basis, active docs, and verifier environment before modifying code.

Execute only this phase unless explicitly running the full P24 super-pass.

## Required outputs

- `AiDENs/handoffs/p24/PHASE_00_REPORT.md`
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
