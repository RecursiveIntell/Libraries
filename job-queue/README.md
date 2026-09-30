# job-queue

Background job queue with SQLite persistence, priority scheduling, and retry lineage tracking.

## Example

```rust
use job_queue::{JobContext, JobHandler, JobResult, QueueConfig, QueueError, QueueJob, QueueManager};
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
struct ExampleJob { text: String }

impl JobHandler for ExampleJob {
    async fn execute(&self, ctx: &JobContext) -> Result<JobResult, QueueError> {
        println!("{}", self.text);
        ctx.emit_progress(1, 1);
        Ok(JobResult::success())
    }
}

fn main() -> Result<(), QueueError> {
    let config = QueueConfig::builder().with_db_path("jobs.db".into()).build();
    let manager = QueueManager::new(config)?;
    let id = manager.add(QueueJob::new(ExampleJob { text: "hello".into() }))?;
    println!("queued {id}");
    Ok(())
}
```

This example persists a queued job. Start `manager.spawn::<ExampleJob>(emitter)` inside a Tokio runtime to process jobs. `QueueConfig::default()` uses an in-memory database; configure `db_path` for restart persistence.

## Ecosystem

- **stack-ids**: Provides `TraceCtx`, `AttemptId`, `TrialId` for trace correlation and retry lineage
- **Tauri-Queue**: Bridges job-queue events to Tauri's frontend event system

## stack-ids integration

Fully integrated. Jobs carry `TraceCtx` for end-to-end correlation, `AttemptId` per retry family, and `TrialId` per execution.
