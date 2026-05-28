# ai-batch-queue

Model-aware batch processing queue with ETA estimation for Tauri applications.

## Example

```rust
use ai_batch_queue::{BatchQueue, BatchJob};

let queue = BatchQueue::new();
let job = BatchJob::new("generate-images", items, "stable-diffusion");
queue.enqueue(job)?;

// Mark next job as running (model-aware scheduling)
if let Some(job_id) = queue.next_queued()? {
    queue.mark_running(&job_id)?;
}
```

## Ecosystem

- **stack-ids**: `TraceCtx`, `AttemptId`, `TrialId` for retry lineage and trace correlation
- **Tauri-Queue** / **job-queue**: Shares queue patterns and event emission conventions

## stack-ids integration

Fully integrated. Jobs carry `AttemptId` per retry family and `TrialId` per execution.
