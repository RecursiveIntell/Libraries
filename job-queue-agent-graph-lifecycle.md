# Job Queue ↔ Agent Graph Lifecycle Integration Protocol

**Status:** Design specification (no implementation changes)  
**Scope:** `job-queue` and `agent-graph` remain independent crates and communicate only through this protocol.  
**Recommended protocol crate:** a tiny new workspace crate named `run-lifecycle` (or an equivalent shared-types crate). It must depend only on `serde`, `chrono`, and `stack-ids`; neither runtime crate is a dependency.

## 1. Design goals

1. Correlate a queue job and graph execution with one stable `run_id`.
2. Let `job-queue` publish durable lifecycle notifications without knowing about checkpoints.
3. Let `agent-graph` consume notifications and translate them into checkpoint operations.
4. Make delivery at-least-once safe through stable event IDs and idempotent handling.
5. Preserve attempt/trial identity: `run_id` identifies the graph run; queue `AttemptId` and `TrialId` identify retry and concrete execution instances.

## 2. Identity and ownership

- **`run_id`** is the protocol correlation key. It is the `agent_graph::checkpoint_store::RunId` value and should be carried in the queue payload or queue job metadata. The queue must not generate a different correlation key on retry.
- **`job_id`** is the queue record identity. It may be re-enqueued or reclaimed, but all re-enqueues for the same logical graph run carry the same `run_id`.
- **`attempt_id`** is owned by `job-queue`: a new one is created per re-enqueue.
- **`trial_id`** is owned by `job-queue`: a new one is created per concrete worker execution.
- **`trace_ctx`** is optional end-to-end tracing metadata. It is not a substitute for `run_id`.
- `worker_id` is present when the event has an owner; it may be absent for startup recovery or administrative cancellation.

## 3. Canonical protocol types

The following definitions are normative. They are intentionally independent of either crate's internal error or database types.

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use stack_ids::{AttemptId, TraceCtx, TrialId};

pub type RunId = String;
pub type LifecycleEventId = String;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "kind")]
pub enum RunLifecycleEvent {
    Claimed {
        event_id: LifecycleEventId,
        run_id: RunId,
        job_id: String,
        worker_id: String,
        attempt_id: Option<AttemptId>,
        trial_id: Option<TrialId>,
        trace_ctx: Option<TraceCtx>,
        occurred_at: DateTime<Utc>,
    },
    Stale {
        event_id: LifecycleEventId,
        run_id: RunId,
        job_id: String,
        worker_id: Option<String>,
        attempt_id: Option<AttemptId>,
        trial_id: Option<TrialId>,
        detected_at: DateTime<Utc>,
        reason: StaleReason,
    },
    Requeued {
        event_id: LifecycleEventId,
        run_id: RunId,
        job_id: String,
        previous_worker_id: Option<String>,
        next_attempt_id: Option<AttemptId>,
        requeued_at: DateTime<Utc>,
        available_at: DateTime<Utc>,
    },
    Cancelled {
        event_id: LifecycleEventId,
        run_id: RunId,
        job_id: String,
        worker_id: Option<String>,
        attempt_id: Option<AttemptId>,
        cancelled_at: DateTime<Utc>,
        reason: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum StaleReason {
    HeartbeatTimeout,
    VisibilityTimeout,
    WorkerLost,
    RecoveredOnStartup,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case", tag = "operation")]
pub enum CheckpointOperation {
    Persist { run_id: RunId },
    Restore { run_id: RunId },
    Prune { run_id: RunId, policy: PrunePolicy },
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PrunePolicy {
    /// Keep the latest resumable state and history needed for audit.
    RetainLatest,
    /// Remove checkpoint history after terminal completion/cancellation.
    TerminalOnly,
    /// Implementation-defined retention; must be documented by the adapter.
    Custom { retain: u32 },
}
```

`event_id` is generated once by the queue transition and remains stable across redelivery. Timestamps are informational and must not be used as the ordering key; consumers use their durable event offset or idempotency table.

## 4. Emitter and consumer traits

The shared crate should expose transport-neutral traits. Implementations may use an in-process channel, SQLite outbox, message broker, or HTTP, without changing either runtime crate.

```rust
use std::future::Future;
use std::pin::Pin;

type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = Result<T, LifecycleError>> + Send + 'a>>;

pub trait JobQueueLifecycle: Send + Sync {
    /// Publish after the queue state transition commits.
    /// Delivery is at-least-once; `event_id` makes retries safe.
    fn publish(&self, event: RunLifecycleEvent) -> BoxFuture<'_, ()>;
}

pub trait CheckpointLifecycle: Send + Sync {
    /// Apply an operation for a correlated graph run. Implementations must be
    /// idempotent for the same (event_id, operation) pair.
    fn apply(&self, event_id: &str, operation: CheckpointOperation) -> BoxFuture<'_, ()>;
}

#[derive(Debug, thiserror::Error)]
pub enum LifecycleError {
    #[error("transport error: {0}")]
    Transport(String),
    #[error("checkpoint operation rejected: {0}")]
    Rejected(String),
}
```

If adding a shared crate is not acceptable, these exact definitions may live as a versioned, copied wire contract in documentation. They must not be defined in `job-queue` and imported by `agent-graph`, or vice versa, because that creates ownership coupling.

## 5. Normative event → checkpoint mapping

| Queue event | Required graph operation | Rationale |
|---|---|---|
| `Claimed` | `Restore { run_id }` | A claim may be a first execution or a retry. Restore is a no-op when no checkpoint exists. The worker/graph executor then continues from the restored state. |
| `Stale` | `Persist { run_id }` (best effort), then mark the active attempt interrupted | Persist only data known to be coherent. If the worker is unreachable, the checkpoint store's startup recovery remains authoritative. |
| `Requeued` | No mandatory operation; optionally `Restore { run_id }` on the next claim | Requeue changes queue ownership, not graph state. Avoid duplicate restore on a requeue notification. |
| `Cancelled` | `Prune { run_id, policy: TerminalOnly }` after cancellation is durably recorded | Cancellation is terminal for this queue run. The adapter must not prune before the cancellation transition commits. |

A queue event never directly mutates agent-graph storage. It is a signal; the graph-side adapter chooses the concrete `CheckpointStore`/`CheckpointSaver` calls. `Persist` means saving the current resumable state, not forcing a synthetic successful checkpoint. `Restore` means loading the latest state and treating absent state as a clean start. `Prune` is retention/cleanup and must not delete records required by configured audit policy.

## 6. Ordering, delivery, and failure rules

- Queue transition first, event publication second. Use a transactional outbox if publication cannot be performed in the same process; never publish `Claimed` before ownership is committed.
- Delivery is **at least once**. Consumers persist `(event_id, operation)` before acknowledging, or use an equivalent atomic deduplication mechanism.
- Events for one `run_id` should be emitted in transition order. A consumer must tolerate duplicates and delayed delivery.
- `Stale` and `Requeued` are not cancellation. A stale worker may be reclaimed and the same `run_id` retried.
- A late `Persist` from a stale worker must not overwrite a newer checkpoint. The graph adapter compares a monotonic queue attempt/trial or checkpoint revision and rejects older writes.
- `Cancelled` wins over later `Claimed`/`Requeued` events for the same terminal queue generation. Such late events are acknowledged as no-ops and retained for audit.
- Checkpoint operation failures must be observable and retryable; they must not cause the queue to claim a second unrelated `run_id`.

## 7. Reference adapter pattern

This is an integration-layer example, not code to add to either crate:

```rust
struct GraphLifecycleAdapter<S> {
    store: std::sync::Arc<S>,
    dedupe: DedupeStore,
}

impl<S> GraphLifecycleAdapter<S>
where
    S: agent_graph::checkpoint_store::CheckpointStore + 'static,
{
    async fn on_event(&self, event: RunLifecycleEvent) -> Result<(), LifecycleError> {
        let (event_id, op) = match &event {
            RunLifecycleEvent::Claimed { event_id, run_id, .. } =>
                (event_id, CheckpointOperation::Restore { run_id: run_id.clone() }),
            RunLifecycleEvent::Stale { event_id, run_id, .. } =>
                (event_id, CheckpointOperation::Persist { run_id: run_id.clone() }),
            RunLifecycleEvent::Requeued { .. } => return Ok(()),
            RunLifecycleEvent::Cancelled { event_id, run_id, .. } =>
                (event_id, CheckpointOperation::Prune {
                    run_id: run_id.clone(),
                    policy: PrunePolicy::TerminalOnly,
                }),
        };

        if self.dedupe.already_applied(event_id, &op).await? { return Ok(()); }
        // Map Restore/Persist/Prune to the selected CheckpointStore or
        // CheckpointSaver implementation, then record successful application.
        apply_to_checkpoint_store(&self.store, &op).await?;
        self.dedupe.record(event_id, &op).await?;
        Ok(())
    }
}
```

For `agent-graph`'s current APIs, `Restore` maps to `CheckpointStore::load_run(run_id)` or `CheckpointSaver::load(run_id)`. `Persist` maps to `save_state_snapshot` plus any active-attempt update available to the executor. `Prune` maps to the appropriate saver cleanup (`clear`) only when the selected retention policy permits it. The protocol intentionally does not require either checkpoint trait to change in the MVP.

## 8. MVP acceptance criteria

- A shared, versioned definition exists for `RunLifecycleEvent` with exactly `Claimed`, `Stale`, `Requeued`, and `Cancelled` variants.
- A shared definition exists for `CheckpointOperation` with exactly `Persist`, `Restore`, and `Prune` operations.
- Every event carries a stable `run_id` and `event_id`.
- `job-queue` emits only; `agent-graph` consumes only; neither crate depends on the other.
- Duplicate delivery, stale-worker races, restart recovery, and cancellation terminality are specified and tested in the integration layer.
- Queue and checkpoint SQLite databases remain independently owned; this protocol does not require a shared schema or transaction.

## 9. Explicit non-goals

This protocol does not merge queues and checkpoints, define a broker, change retry/backoff policy, make checkpoint persistence synchronous with every heartbeat, or guarantee exactly-once execution. It coordinates durable state transitions while preserving each crate's ownership and crash-recovery behavior.
