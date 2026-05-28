# job-queue

Production-grade background job queue with SQLite persistence, priority scheduling, and retry lineage tracking.

## Example

```rust
use job_queue::{QueueManager, JobHandler, JobContext};
use serde_json::json;

let manager = QueueManager::new("jobs.db")?;

// Enqueue a job
manager.enqueue("send_email", 5, &json!({"to": "user@example.com"}))?;

// Process jobs with a handler
struct EmailHandler;
impl JobHandler for EmailHandler {
    fn handle(&self, ctx: &JobContext) -> Result<()> {
        // process job...
        Ok(())
    }
}
```

## Ecosystem

- **stack-ids**: Provides `TraceCtx`, `AttemptId`, `TrialId` for trace correlation and retry lineage
- **Tauri-Queue**: Bridges job-queue events to Tauri's frontend event system

## stack-ids integration

Fully integrated. Jobs carry `TraceCtx` for end-to-end correlation, `AttemptId` per retry family, and `TrialId` per execution.
