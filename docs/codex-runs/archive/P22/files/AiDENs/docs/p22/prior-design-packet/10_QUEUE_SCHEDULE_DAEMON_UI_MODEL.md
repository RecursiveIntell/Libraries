# 10 — Queue, Schedule, Daemon, and UI Model

## Separation

Recall currently contains scheduler, queue, daemon, and UI behavior. AiDENs should split them:

```text
aidens-schedule-kit = canonical trigger and recurrence law
aidens-queue-kit    = durable job, lease, attempt, cancellation law
aidens-wake-kit     = host wake adapter law
aidens-daemon-kit   = process and IPC lifecycle
aidens-tauri-kit    = UI command/event adapter
aidens-cli          = scaffold/doctor/check/run
```

## Schedule law

`aidens-schedule-kit` owns:

```text
TriggerSpecV1
TriggerKind
MisfirePolicy
CatchUpPolicy
OverlapPolicy
TimezonePolicy
NextFireCalculator
TriggerFireReceiptV1
```

Host wake systems are projections/adapters, not schedule truth.

## Queue law

`aidens-queue-kit` owns:

```text
job_id
lease_id
attempt_family_id
attempt_id
trace_id
queue_hop_receipt
cancellation token
retry policy
cooldown policy
```

Every queued action must have a lease. Stale jobs must fail safely.

## Future action law

A future action should include:

```text
action_id
action_kind
state
plan_id optional
plan_revision_id optional
trigger_id optional
permit_id optional
dedupe_key
lease_id optional
leased_until optional
attempt_family_id
last_attempt_id
created_at
not_before
expires_at
```

## Daemon law

When daemon mode is enabled:

```text
daemon owns runtime state
GUI/CLI are clients
config changes go through daemon
approval decisions go to permit-kit via daemon
local fallback is explicit
```

IPC endpoint must be namespaced by:

```text
vendor_id
app_id
profile_id
instance_id
```

Never use a global generic socket name.

## UI law

`aidens-tauri-kit` should not construct providers, mutate queue state, or own approvals.

It owns:

```text
commands/events
status projection
approval prompt display
stream rendering
daemon client bridge
```

Stream events should be idempotent:

```text
run_id
stream_id
sequence_no
event_kind
payload_digest
receipt_ref
```

## Host wake law

`aidens-wake-kit` wraps:

```text
systemd
cron
launchd
Windows Task Scheduler
```

It reports:

```text
backend support
install mode
one-shot support
recurring support
drift status
last binding id
```

But it does not define canonical recurrence semantics.

## Required receipts

Scheduled and daemon actions must emit:

```text
ScheduleReceiptV1
QueueHopReceiptV1
LeaseReceiptV1
ProviderRouteReceiptV1
ToolAttemptReceiptV1 when tools run
ApprovalReceiptV1 when approval is needed
RunClosureV1
```

## Startup recovery

Daemon startup must:

1. load scheduler state,
2. reclaim expired leases,
3. evaluate unfinished plans,
4. emit recovery receipts,
5. not duplicate already leased actions,
6. not fire host wake actions without canonical trigger confirmation.

## Failure states

Use explicit states:

```text
draft
pending
host_armed
leased
running
cooldown
waiting
blocked
succeeded
failed
exhausted
cancelled
superseded
needs_review
expired
```

No implicit retry loops.
