# Codex Phase 16 Prompt — Config, environment, secrets, and redaction

Use this only after all prior phase gates pass.

## Phase objective

Harden config validation; redact provider/tool/log secrets; record environment fingerprints.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 16` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_16_REPORT.md`.

## Exit gate

Secret values never appear in receipts/logs; config mismatch degrades with receipt.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
