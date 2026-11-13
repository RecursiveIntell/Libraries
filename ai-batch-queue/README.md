# ai-batch-queue

Model-aware batch processing queue with ETA estimation for Tauri applications.

## Example

```rust
use ai_batch_queue::{build_job, BatchQueue, OverwritePolicy, SizeBucket};

fn main() -> anyhow::Result<()> {
    let queue = BatchQueue::<String>::new();
    let job = build_job(
        "llava", "tag", OverwritePolicy::Skip,
        vec![("image-1".into(), "photo.jpg".into(), SizeBucket::Medium)],
    );
    queue.enqueue(job)?;
    if let Some(job) = queue.next_queued() {
        queue.mark_running(&job.id)?;
    }
    Ok(())
}
```

The default queue is in-memory. Implement `BatchStore` for persistence and `BatchItemHandler` for processing, then use the executor for background work. The example only enqueues and changes status; it does not process an image.

## Ecosystem

- **stack-ids**: `TraceCtx`, `AttemptId`, `TrialId` for retry lineage and trace correlation
- **Tauri-Queue** / **job-queue**: Shares queue patterns and event emission conventions

## stack-ids integration

Fully integrated. Jobs carry `AttemptId` per retry family and `TrialId` per execution.
