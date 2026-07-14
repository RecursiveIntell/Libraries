//! agent-graph demo: builder + checkpoint + router APIs
//!
//! Run with: cargo run -p agent-graph --example demo

use agent_graph::prelude::*;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<()> {
    println!("=== agent-graph Demo ===\n");

    // ── 1. Build a 3-node graph: input -> process -> output ──────────────
    println!("[1] Building 3-node graph: input -> process -> output");

    let checkpoint_store = Arc::new(InMemoryCheckpointStore::new());

    let graph = AgentGraph::builder()
        .with_name("demo-graph")
        .set_entry_point("input")
        // input node: seed the state
        .add_node(
            "input",
            node!("input", |state| async move {
                println!("    > [input] setting initial value = 42");
                state.set("value", 42_i64).await?;
                state.set("step", "input_done").await?;
                Ok(())
            }),
        )
        .add_edge("input", "process")
        // process node: transform the value
        .add_node(
            "process",
            node!("process", |state| async move {
                let value: i64 = state.get("value").await?;
                let doubled = value * 2;
                println!(
                    "    > [process] received value={}, doubling to {}",
                    value, doubled
                );
                state.set("value", doubled).await?;
                state.set("step", "process_done").await?;
                Ok(())
            }),
        )
        // conditional edge: router decides where to go from process
        .add_conditional_edge(
            "process",
            router!(|state| async move {
                let value: i64 = state.get("value").await?;
                if value > 50 {
                    println!("    > [router] value={} > 50 → routing to 'output'", value);
                    Ok(Some("output".to_string()))
                } else {
                    println!("    > [router] value={} <= 50 → routing to END", value);
                    Ok(None)
                }
            }),
        )
        // output node: emit final result
        .add_node(
            "output",
            node!("output", |state| async move {
                let value: i64 = state.get("value").await?;
                let step: String = state.get("step").await?;
                println!("    > [output] final value={}, last_step={}", value, step);
                state.set("step", "output_done").await?;
                Ok(())
            }),
        )
        .add_edge("output", END)
        // attach checkpoint store for persistence
        .with_checkpoint_store(checkpoint_store.clone())
        .build()?;

    println!("    Graph built: nodes = {:?}\n", graph.node_names());

    // ── 2. Execute with checkpoint ────────────────────────────────────────
    println!("[2] Executing graph with checkpoint store");

    let state = AgentState::new();
    let config = GraphConfig::default()
        .with_thread_id("demo-thread-001")
        .with_recursion_limit(50);

    let result = graph.execute_with_config("input", state, config).await?;

    let final_value: i64 = result.get("value").await?;
    let final_step: String = result.get("step").await?;
    println!(
        "    Final state: value={}, step={}\n",
        final_value, final_step
    );

    // ── 3. Inspect checkpoint store ───────────────────────────────────────
    println!("[3] Inspecting checkpoint store");
    let runs = checkpoint_store.list_runs().await;
    println!("    Runs recorded: {}", runs.len());
    for run in &runs {
        println!(
            "    Run: id={}, status={:?}, attempts={}",
            run.run_id,
            run.status,
            run.attempts.len()
        );
        for attempt in &run.attempts {
            println!(
                "      attempt: node={}, status={:?}, input={}",
                attempt.node_id, attempt.status, attempt.input
            );
        }
    }

    // ── 4. Show router directing flow ─────────────────────────────────────
    println!("\n[4] Router demo: low-value path (routes to END, skips output)");

    let graph2 = AgentGraph::builder()
        .with_name("router-demo")
        .set_entry_point("input")
        .add_node(
            "input",
            node!("input", |state| async move {
                state.set("value", 10_i64).await?;
                println!("    > [input] set value=10 (low, will skip output)");
                Ok(())
            }),
        )
        .add_edge("input", "process")
        .add_node(
            "process",
            node!("process", |state| async move {
                let value: i64 = state.get("value").await?;
                let doubled = value * 2;
                println!("    > [process] doubled to {}", doubled);
                state.set("value", doubled).await?;
                Ok(())
            }),
        )
        .add_conditional_edge(
            "process",
            router!(|state| async move {
                let value: i64 = state.get("value").await?;
                if value > 50 {
                    println!("    > [router] value={} > 50 → 'output'", value);
                    Ok(Some("output".to_string()))
                } else {
                    println!(
                        "    > [router] value={} <= 50 → END (skipping output)",
                        value
                    );
                    Ok(None)
                }
            }),
        )
        .add_node(
            "output",
            node!("output", |state| async move {
                let value: i64 = state.get("value").await?;
                println!("    > [output] reached! value={}", value);
                Ok(())
            }),
        )
        .add_edge("output", END)
        .build()?;

    let state2 = AgentState::new();
    let result2 = graph2.execute("input", state2).await?;
    let final_value2: i64 = result2.get("value").await?;
    println!(
        "    Final value (low path): {} (output node was skipped)\n",
        final_value2
    );

    // ── 5. Mermaid diagram ────────────────────────────────────────────────
    println!("[5] Graph Mermaid diagram:");
    println!("{}", graph.to_mermaid());

    println!("\n=== Demo complete ===");
    Ok(())
}
