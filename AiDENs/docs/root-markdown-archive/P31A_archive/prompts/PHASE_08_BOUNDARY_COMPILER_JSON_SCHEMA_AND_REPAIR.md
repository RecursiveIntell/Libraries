# Codex Phase 08 Prompt — Boundary compiler, JSON, schema, and repair

Use this only after all prior phase gates pass.

## Phase objective

Strict boundary profiles; full schema validation or explicit unsupported-feature rejection; default material repair reject/quarantine; treatment-critical checks.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 08` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_08_REPORT.md`.

## Exit gate

Duplicate keys, fenced JSON, unknown fields, schema mismatch, critical-field repair fixtures pass.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
