#[cfg(feature = "checkpointing")]
use agent_graph::prelude::*;

#[cfg(feature = "checkpointing")]
#[tokio::main]
async fn main() -> Result<()> {
    println!("=== Durable Checkpoint Store Example ===\n");

    let db_path = "/tmp/agent_graph_checkpoints.db";
    let store = std::sync::Arc::new(SqliteCheckpointStore::new(db_path)?);

    // The graph's durable execution state is owned by the granular store.
    let graph = AgentGraph::builder()
        .with_name("checkpointing-example")
        .with_checkpoint_store(store.clone())
        .add_node(
            "step1",
            node!("step1", |state| async move {
                println!("Step 1: Initializing...");
                state.set("progress", 1u32).await?;
                state.set("data", "initial data").await?;
                Ok(())
            }),
        )
        .add_node(
            "step2",
            node!("step2", |state| async move {
                let progress: u32 = state.get("progress").await?;
                println!("Step 2: Processing (progress={})...", progress);
                state.set("progress", progress + 1).await?;
                state.set("data", "processed data").await?;
                Ok(())
            }),
        )
        .add_edge("step1", "step2")
        .build()?;

    let (result, summary) = graph
        .execute_with_summary("step1", AgentState::new(), GraphConfig::default())
        .await;
    result?;
    println!("Run ID: {}\n", summary.run_id);

    let persisted = store
        .load_run(&summary.run_id)
        .await?
        .ok_or_else(|| AgentGraphError::RunNotFound(summary.run_id.clone()))?;
    println!("Persisted status: {:?}", persisted.status);
    println!("Persisted attempts: {}", persisted.attempts.len());

    drop(store);
    std::fs::remove_file(db_path).ok();
    std::fs::remove_file(format!("{db_path}-wal")).ok();
    std::fs::remove_file(format!("{db_path}-shm")).ok();
    Ok(())
}

#[cfg(not(feature = "checkpointing"))]
fn main() {
    println!("This example requires the 'checkpointing' feature.");
    println!("Run with: cargo run --example checkpointing --features checkpointing");
}
