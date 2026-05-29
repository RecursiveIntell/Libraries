# Codex Phase 06 Prompt — Provider honesty and local route discipline

Use this only after all prior phase gates pass.

## Phase objective

Local must not mean mock; provider/tool results routed honestly; native vs fallback exactness disclosed; network permits enforced.

## Backlog selection

Load `matrices/SUPER_PASS_BACKLOG_1020.csv` and select rows where `Suggested_Phase` contains `Phase 06` or whose category clearly belongs to this phase.

## Required work

1. Inspect relevant crates/files.
2. Implement fixes or explicit quarantines.
3. Add semantic/hostile tests that fail without the fix.
4. Run targeted tests, then broader command bar if feasible.
5. Update matrix statuses.
6. Write `PHASE_06_REPORT.md`.

## Exit gate

Local unavailable does not silently mock; Ollama sees tool results or is explicitly no-tools/degraded.

## Completion rule

Do not move to the next phase with raw `open` rows in this phase. Use `fixed`, `quarantined`, `deferred`, `superseded`, or `open-blocking`.
