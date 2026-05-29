# Phase 07 Report - Queue, Scheduler, Daemon Concurrency

Date: 2026-05-07

## Scope

- Backlog selector: `Suggested_Phase` contains `Phase 07` or category is `Queue, scheduler, daemon & concurrency`.
- Rows in scope: 70 (`AHD-0361` through `AHD-0430`).
- Initial status: 70 `open`.
- Final status: 70 `fixed`; no raw `open` rows remain for Phase 07.

## Files Changed

- `crates/aidens-queue-kit/src/lib.rs`
- `crates/aidens-receipts/src/lib.rs`
- `crates/aidens-runner/src/lib.rs`
- `crates/aidens-integration-tests/tests/phase_03_golden_vertical_slice.rs`
- `fixtures/provider_capability_expected_v0_1.json`
- `matrices/SUPER_PASS_BACKLOG_1020.csv`
- `matrices/SUPER_PASS_BACKLOG_1020.json`
- `handoffs/super-pass/PHASE_07_REPORT.md`

## Implementation

- Added exclusive queue log writer locking around enqueue, lease acquisition, transition/completion, and safe-mode mutation.
- Kept sequence assignment inside the single-writer critical section so concurrent appends cannot reuse sequence numbers.
- Made completion lease-bound:
  - completion now requires an explicit lease id;
  - the lease id must match the job's current lease;
  - the matching lease must be active;
  - completion after TTL expiry is rejected.
- Preserved expired lease stealing, but stale lease completion after a steal is rejected.
- Entering safe mode now quarantines existing risky nonterminal jobs with a durable queue-hop receipt.
- Safe-mode blocking of new risky jobs remains receipt-bearing and queryable through the log.
- Cleared broader command-bar receipt blockers discovered after Phase 07 gate:
  - Forge receipts derived from runtime receipts now use a namespaced event-log record id while preserving the canonical Forge receipt payload id.
  - Runner tool exposure records now use per-run event-log record ids while preserving the canonical `tool-exposure` artifact in the body.
  - Provider capability fixture now includes the `local` route as unavailable, not mock.

## Hostile/Semantic Tests Added

In `aidens-queue-kit`:

- `concurrent_enqueue_is_single_writer_and_idempotent`
- `concurrent_lease_acquisition_grants_one_active_lease`
- `late_completion_after_ttl_is_rejected_and_job_stays_leased`
- `stale_lease_cannot_complete_after_lease_is_stolen`
- `entering_safe_mode_quarantines_existing_risky_jobs`

These tests would fail against the previous snapshot-plus-append behavior or the previous completion path that did not validate an active unexpired lease.

## Validation

Passed:

- `cargo test -p aidens-queue-kit`
  - Log: `target/super-pass/audit/phase07-cargo-test-aidens-queue-kit.log`
- `cargo test -p aidens-daemon-kit`
  - Log: `target/super-pass/audit/phase07-cargo-test-aidens-daemon-kit.log`
- `cargo test -p aidens-schedule-kit`
  - Log: `target/super-pass/audit/phase07-cargo-test-aidens-schedule-kit.log`
- `cargo check -p aidens-queue-kit -p aidens-daemon-kit -p aidens-schedule-kit --all-targets`
  - Log: `target/super-pass/audit/phase07-cargo-check-queue-daemon-schedule.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase07-cargo-fmt-all-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase07-cargo-check-workspace-all-targets.log`
- `cargo test -p aidens-integration-tests --test phase_07_daemon_queue_schedule_wake`
  - Log: `target/super-pass/audit/phase07-cargo-test-integration-phase07-daemon-queue.log`
- `cargo test -p aidens-integration-tests --test phase_09_daemon_smoke`
  - Log: `target/super-pass/audit/phase07-cargo-test-integration-daemon-smoke.log`
- `cargo test -p aidens-integration-tests --test phase_03_golden_vertical_slice`
  - Log: `target/super-pass/audit/phase07-fixed-blocker-phase03-golden.log`
- `cargo test -p aidens-provider-kit p20_provider_capability_matrix_matches_executable_truth`
  - Log: `target/super-pass/audit/phase07-fixed-blocker-provider-matrix.log`
- `cargo test -p aidens-runner --test phase_06_agency_v02 runner_counts_repeated_nudges_across_turns_and_blocks_over_budget`
  - Log: `target/super-pass/audit/phase07-fixed-blocker-runner-agency.log`
- `cargo test --workspace --all-targets`
  - Log: `target/super-pass/audit/phase07-post-runner-cargo-test-workspace-all-targets.log`
- `cargo fmt --all --check`
  - Log: `target/super-pass/audit/phase07-final-cargo-fmt-all-check.log`
- `cargo check --workspace --all-targets`
  - Log: `target/super-pass/audit/phase07-final-cargo-check-workspace-all-targets.log`
- `cargo clippy --workspace --all-targets -- -D warnings`
  - Log: `target/super-pass/audit/phase07-final-cargo-clippy-workspace-all-targets.log`

## Matrix Updates

- `AHD-0361` through `AHD-0430`: `fixed`
- Notes updated with the queue lock, lease validation, safe-mode quarantine, and audit-log evidence.

## Exit Gate

Phase 07 gate result: `pass`

- Concurrent enqueue tests pass.
- Concurrent lease tests pass.
- Late completion after TTL is rejected.
- Stale completion after lease stealing is rejected.
- Safe-mode quarantine for existing risky queued work is receipt-bearing.
- No raw `open` rows remain in Phase 07.

## Unresolved Risk

- No Phase 07 queue/scheduler/daemon rows remain raw `open`.
- Full workspace test, check, fmt, and clippy command bar passed after clearing the duplicate receipt and provider fixture blockers.
- No final support label is claimed from this phase.

## Decision

`continue`

Phase 07 is fixed and gate-passing. Broader command-bar blockers discovered during this phase were resolved and revalidated.
