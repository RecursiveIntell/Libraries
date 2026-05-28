# P30-08 Report

## Scope

Phase slice: scheduling and sleep/backoff proof debt.

Matrix inventory from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- 6 total P30-08 rows.
- All are P2 `SCHEDULING` rows.

Issue IDs quarantined:

- `P30-ABSORB-0438`: `aidens-tool-kit` synchronous command polling uses `std::thread::sleep` while waiting for command completion or timeout.
- `P30-ABSORB-0439`: `Primitives/check-runner` test waits briefly after process-group timeout before asserting child termination.
- `P30-ABSORB-0440`: `semantic-memory` pool timeout test holds a read connection with `std::thread::sleep`.
- `P30-ABSORB-0552`: `forge-pilot` TUI loop uses `tokio::time::sleep` for interval polling.
- `P30-ABSORB-0553`: `llm-tool-runtime` cancellation wait loop uses `tokio::time::sleep`.
- `P30-ABSORB-0554`: `llm-tool-runtime` slow-tool test fixture uses `tokio::time::sleep`.

## Changed Files

No P30-08 code changes were made.

Reason: these rows require scheduler-policy proof or structural async/runtime changes. Replacing `sleep` tokens mechanically would reduce static visibility without proving non-blocking scheduling, fairness, cancellation latency, or receipt semantics.

## Tests Added Or Updated

No tests were added in this slice.

Relevant observations:

- `P30-ABSORB-0438` is in production command polling, but timeout already emits `check-command-timeout` and `command-output-partial-after-timeout` in the command receipt path.
- `P30-ABSORB-0439`, `P30-ABSORB-0440`, and `P30-ABSORB-0554` are test-only sleeps.
- `P30-ABSORB-0552` and `P30-ABSORB-0553` remain runtime scheduling-policy debt.

## Commands Run

Previously in this session after the latest code changes:

- `cargo check --manifest-path Cargo.toml -p aidens-cli -p aidens-contracts -p aidens-provider-kit --all-targets --locked`
  - Result: pass.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo . | tail -n 8`
  - Result: exit 0, `findings=1841 hard=0`.

No P30-08-specific code was changed, so no P30-08-specific unit test was run.

## Unresolved Risks And Quarantines

- Command polling can occupy a host thread until command completion or timeout.
- Async interval sleeps lack an explicit scheduler-policy artifact proving fairness, cancellation deadline, or backoff semantics.
- Test sleeps remain visible static debt but are lower operational risk than production runtime sleeps.

## Invariant Revalidation Checklist

- No scheduling behavior was silently relabeled as fixed.
- Existing timeout receipt semantics in `aidens-tool-kit` remain visible and unchanged.
- Runtime scheduler-policy debt is explicitly quarantined.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-08 can proceed only as explicit quarantined P2 scheduling debt. A later pass should either introduce typed scheduler-policy receipts or refactor the affected runtime loops with tests that prove cancellation and fairness behavior.
