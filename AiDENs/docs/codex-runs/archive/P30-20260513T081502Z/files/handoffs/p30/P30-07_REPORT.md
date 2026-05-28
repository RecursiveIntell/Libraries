# P30-07 Report

## Scope

Phase slice: async runtime blocking, megafile/code-shape debt, panic surface, dynamic JSON, and lint suppression.

Matrix inventory from `matrices/P30_HOSTILE_AUDIT_ABSORPTION_MATRIX.csv`:

- 220 total P30-07 rows.
- Priority split: 90 P1, 130 P2.
- Category split: `ASYNC-RUNTIME` 1, `CODE-SHAPE` 7, `PANIC-SURFACE` 80, `RUNTIME-PANIC` 2, `DYNAMIC-JSON` 80, `LINT-SUPPRESSION` 50.
- ID ranges: `P30-ABSORB-0149` through `P30-ABSORB-0156`, `P30-ABSORB-0218` through `P30-ABSORB-0297`, `P30-ABSORB-0299` through `P30-ABSORB-0300`, and `P30-ABSORB-0308` through `P30-ABSORB-0437`.

Issue IDs addressed:

- `P30-ABSORB-0299`: agency nudge ledger poisoning no longer panics the runner. Poison recovery is recorded in the agency policy report reason codes as `agency-nudge-ledger-poison-recovered`.
- `P30-ABSORB-0300`: run report ledger poisoning no longer panics `append`, `list`, or `len`; the ledger recovers the inner append-only vector.

Issue IDs quarantined:

- `P30-ABSORB-0149`: synchronous command polling sleep remains scheduling debt. It overlaps P30-08.
- `P30-ABSORB-0150` through `P30-ABSORB-0156`: megafile/code-shape refactors are broad ownership changes and were not attempted in this pass.
- `P30-ABSORB-0218` through `P30-ABSORB-0297`: broad panic-surface findings remain open. Many observed rows are test-only, but they were not all classified or rewritten here.
- `P30-ABSORB-0308` through `P30-ABSORB-0387`: dynamic JSON findings remain open except for prior narrow receipt-detail changes made in P30-05.
- `P30-ABSORB-0388` through `P30-ABSORB-0437`: lint-suppression findings remain open.

## Changed Files

- `crates/aidens-runner/src/lib.rs`
  - `RunReportLedger` now uses a poison-recovering lock helper instead of `expect("run report ledger poisoned")`.
  - `evaluate_agency_policy` now recovers a poisoned nudge ledger and records `agency-nudge-ledger-poison-recovered` in the agency decision receipt path.
- `crates/aidens-runner/src/tests.rs`
  - Added targeted regression tests for poisoned run report ledger recovery and poisoned agency nudge ledger recovery.

Observed code evidence:

- `crates/aidens-runner/src/lib.rs:646`: `RunReportLedger::lock_reports` recovers poisoned mutex state with `into_inner`.
- `crates/aidens-runner/src/lib.rs:1462`: agency nudge lock handles `Err(poisoned)`.
- `crates/aidens-runner/src/lib.rs:1467`: agency poison recovery adds a receipt-visible reason code.
- `crates/aidens-runner/src/tests.rs:40`: run report ledger poison recovery regression test.
- `crates/aidens-runner/src/tests.rs:56`: agency nudge ledger poison recovery regression test.

## Tests Added Or Updated

Added:

- `p30_run_report_ledger_recovers_from_poisoned_lock`
- `p30_agency_nudge_ledger_poison_recovery_is_receipted`

## Commands Run

- `cargo test --manifest-path Cargo.toml -p aidens-runner p30_`
  - Result: pass, 2 tests passed.
- `cargo check --manifest-path Cargo.toml -p aidens-runner --all-targets --locked`
  - Result: pass.
- `cargo fmt --manifest-path Cargo.toml --all -- --check`
  - Result: pass.
- `python3 scripts/p30_guard.py --repo . | tail -n 8`
  - Result: exit 0, `findings=1841 hard=0`.

## Unresolved Risks And Quarantines

- Broad panic-surface cleanup is still open. This pass fixed only the two runner runtime-panic rows.
- Megafile refactors were not attempted because they are high-conflict structural changes requiring file ownership splits and broader tests.
- Dynamic JSON remains a large typed-boundary debt area. Existing `serde_json::Value` use is still reported by `p30_guard`.
- Lint suppressions remain visible debt and need per-site justification or removal.
- Command polling sleep remains scheduling debt and is also recorded in P30-08.

## Invariant Revalidation Checklist

- Runner poisoned ledgers no longer panic on the two targeted runtime paths.
- Agency poison recovery emits receipt-visible reason evidence.
- No broad panic/dynamic-JSON/lint cleanup is overclaimed.
- No v11A/v11B compliance claim is made from this phase.

## Proceed Statement

P30-07 can proceed only for `P30-ABSORB-0299` and `P30-ABSORB-0300`. The remaining P30-07 rows are explicit quarantined debt and must block any claim that panic-surface, typed-boundary, code-shape, scheduler, or lint-suppression hardening is complete.
