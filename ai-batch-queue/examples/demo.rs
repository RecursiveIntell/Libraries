//! # AI Batch Queue — End-to-End Demo
//!
//! Demonstrates the full lifecycle:
//! 1. Create a `BatchQueue` with a simple `BatchItemHandler`
//! 2. Submit 5 items as a batch job
//! 3. Show ETA estimation as items complete
//! 4. Execute the batch (simulating the Tauri executor loop)
//!
//! Run with:
//! ```sh
//! cargo run -p ai-batch-queue --example demo
//! ```

use std::time::Instant;

use ai_batch_queue::{
    build_job, BatchItemHandler, BatchItemStatus, BatchQueue, ItemResult, OverwritePolicy,
    SizeBucket,
};

/// A simple handler that "processes" image files by simulating work.
struct ImageTagger;

impl BatchItemHandler<String> for ImageTagger {
    async fn process(
        &self,
        data: &String,
        resource_key: &str,
        operation: &str,
    ) -> anyhow::Result<ItemResult> {
        // Simulate model inference latency (50–200 ms depending on size)
        let delay = match data.len() % 3 {
            0 => 200,
            1 => 120,
            _ => 50,
        };
        tokio::time::sleep(tokio::time::Duration::from_millis(delay)).await;

        Ok(ItemResult::success_with_output(format!(
            "[{}:{}] tagged {}",
            resource_key, operation, data
        )))
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== AI Batch Queue Demo ===\n");

    // ── 1. Create the queue ──────────────────────────────────────────
    let queue: BatchQueue<String> = BatchQueue::new();
    println!("Created BatchQueue<String>");

    // ── 2. Submit 5 items ────────────────────────────────────────────
    let items = vec![
        ("img-1".into(), "/photos/cat.jpg".into(), SizeBucket::Small),
        ("img-2".into(), "/photos/dog.jpg".into(), SizeBucket::Medium),
        (
            "img-3".into(),
            "/photos/sunset.jpg".into(),
            SizeBucket::Large,
        ),
        ("img-4".into(), "/photos/park.jpg".into(), SizeBucket::Small),
        (
            "img-5".into(),
            "/photos/river.jpg".into(),
            SizeBucket::Medium,
        ),
    ];

    let job = build_job(
        "llava:13b",           // resource key (model name)
        "tag",                 // operation
        OverwritePolicy::Skip, // skip already-processed items
        items,
    );

    let job_id = queue.enqueue(job)?;
    println!("Enqueued batch job: {} (5 items)\n", &job_id[..8]);

    // Show initial state — no ETA data yet
    match queue.estimate_remaining(&job_id) {
        Some(eta) => println!(
            "  Initial ETA: {}ms remaining, {} items, confidence: {:?}",
            eta.remaining_ms, eta.items_remaining, eta.confidence
        ),
        None => println!("  Initial ETA: no historical data yet"),
    }

    // ── 3 & 4. Execute the batch and show ETA estimation ─────────────
    // In a real Tauri app you'd call `executor::spawn()`. Here we
    // simulate the executor loop directly since there's no AppHandle.
    println!("\nExecuting batch...\n");

    queue.mark_running(&job_id)?;
    let handler = ImageTagger;

    let job_snapshot = queue.get_job(&job_id).unwrap();
    let total = job_snapshot.items.len();
    let mut completed = 0usize;

    for item in &job_snapshot.items {
        // Mark item as running
        queue.update_item(&job_id, &item.id, BatchItemStatus::Running, None, None)?;

        // Process
        let start = Instant::now();
        let result = handler
            .process(
                &item.data,
                &job_snapshot.resource_key,
                &job_snapshot.operation,
            )
            .await;
        let duration_ms = start.elapsed().as_millis() as u64;

        let (status, error) = match result {
            Ok(r) if r.success => (BatchItemStatus::Completed, None),
            Ok(r) => (
                BatchItemStatus::Failed,
                r.error.or(Some("Unknown error".into())),
            ),
            Err(e) => (BatchItemStatus::Failed, Some(format!("{:#}", e))),
        };

        let status_icon = if status == BatchItemStatus::Completed {
            "✓"
        } else {
            "✗"
        };
        queue.update_item(&job_id, &item.id, status, error, Some(duration_ms))?;
        completed += 1;

        // Show ETA after each item
        let eta = queue.estimate_remaining(&job_id);
        let eta_str = match &eta {
            Some(e) => format!(
                "{}ms ({} left, avg {}ms, {:?}, n={})",
                e.remaining_ms, e.items_remaining, e.avg_item_ms, e.confidence, e.sample_count
            ),
            None => "no data".to_string(),
        };

        println!(
            "  [{}] {}/{} id={}  {}ms  ETA: {}",
            status_icon, completed, total, item.id, duration_ms, eta_str
        );
    }

    // ── Completion summary ───────────────────────────────────────────
    let summary = queue.mark_completed(&job_id)?;

    println!("\n=== Batch Complete ===");
    if let Some(s) = summary {
        println!("  Job:          {}", &s.job_id[..8]);
        println!("  Operation:    {}", s.operation);
        println!("  Resource:     {}", s.resource_key);
        println!("  Total:        {}", s.total);
        println!("  Succeeded:    {}", s.succeeded);
        println!("  Failed:       {}", s.failed);
        println!("  Skipped:      {}", s.skipped);
        println!("  Total time:   {}ms", s.total_duration_ms);
        println!("  Avg per item: {}ms", s.avg_duration_ms);
    }

    // ── Final ETA state ──────────────────────────────────────────────
    println!("\n=== ETA Sample Store ===");
    for bucket in [SizeBucket::Small, SizeBucket::Medium, SizeBucket::Large] {
        let count = queue.eta_sample_count("llava:13b", "tag", bucket);
        println!("  {:?}: {} samples", bucket, count);
    }

    // Show final confidence
    if let Some(eta) = queue.estimate_remaining(&job_id) {
        println!(
            "\nFinal ETA: {}ms remaining, confidence: {:?}",
            eta.remaining_ms, eta.confidence
        );
    }

    Ok(())
}
