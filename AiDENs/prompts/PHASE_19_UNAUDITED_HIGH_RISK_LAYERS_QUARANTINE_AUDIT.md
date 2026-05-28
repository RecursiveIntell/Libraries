# Codex Phase 19 Prompt — Unaudited high-risk layers quarantine/audit

Use this only after all prior phase gates pass.

## Phase objective

Audit or quarantine forge-pilot, effect-runtime, verification pipeline, federation, attestation, authority-delegation, recursive-kernel-core.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 19` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_19_REPORT.md`.

## Exit gate

Each high-risk layer is fixed, quarantined, or explicitly out-of-scope with guard tests.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
