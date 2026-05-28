# Codex Prompt — P11 Queue, schedule, wake, daemon, leases, and duplicate-storm immunity

Read `AGENTS.md`, `SOURCE_BASIS.md`, `BUILD_ORDER_DAG.md`, and `passes/P11_QUEUE_SCHEDULE_WAKE_DAEMON_DUPLICATE_STORM_IMMUNITY.md`.

Implement P11 only. Do not start later passes.

## Goal

Implement the autonomous execution substrate only after receipts, permits, memory, and tool safety are real.

## Primary crates

- `aidens-queue-kit`
- `aidens-schedule-kit`
- `aidens-wake-kit`
- `aidens-daemon-kit`
- `aidens-receipts`
- `aidens-contracts`
- `aidens-cli`

## Required artifacts

- `JobV1`
- `QueueLeaseV1`
- `ScheduleOccurrenceV1`
- `WakeSignalV1`
- `DaemonNamespaceV1`
- `SafeModeReceiptV1`
- `DuplicateSuppressionReceiptV1`
- `QueueHopReceiptV1`

## Acceptance gates

- Repeated same schedule occurrence cannot create duplicate logical jobs.
- Daemon restart preserves queued jobs and does not resurrect cancelled jobs.
- Safe mode blocks new risky jobs and allows inspection/drain.

## Forbidden shortcuts

- Do not implement recurring schedules before idempotency keys.
- Do not use timestamps alone as job identity.

## Finish by producing a handoff

Include files changed, tests added, commands run, blockers, and next-pass readiness.
