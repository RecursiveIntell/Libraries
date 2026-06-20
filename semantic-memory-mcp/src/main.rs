//! semantic-memory-mcp — MCP stdio server for semantic-memory.
//!
//! Wraps the semantic-memory knowledge management library as an MCP
// (Model Context Protocol) server. Works with Hermes Agent, Claude
// Desktop, Cursor, and any MCP-compatible client.
//!
//! Usage:
//!   semantic-memory-mcp --memory-dir /path/to/memory-store

mod bridge;
mod server;
mod tools;

use clap::Parser;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

/// semantic-memory MCP server configuration.
#[derive(Parser, Debug)]
#[command(name = "semantic-memory-mcp")]
#[command(about = "MCP server for semantic-memory — local-first knowledge management")]
struct Cli {
    /// Path to the memory store directory (created if it does not exist)
    #[arg(long)]
    memory_dir: String,

    /// Ollama embedding server URL (default: http://localhost:11434)
    #[arg(long)]
    embedding_url: Option<String>,

    /// Embedding model name (default: nomic-embed-text)
    #[arg(long)]
    embedding_model: Option<String>,

    /// Embedding dimensions (default: 768)
    #[arg(long)]
    embedding_dims: Option<usize>,
}

fn main() -> anyhow::Result<()> {
    // All logging goes to stderr — stdout is reserved for MCP JSON-RPC.
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .with_writer(std::io::stderr)
        .init();

    let cli = Cli::parse();

    eprintln!("semantic-memory-mcp starting...");
    eprintln!("  memory_dir: {}", cli.memory_dir);
    eprintln!(
        "  embedding: {} @ {} ({}d)",
        cli.embedding_model.as_deref().unwrap_or("nomic-embed-text"),
        cli.embedding_url.as_deref().unwrap_or("http://localhost:11434"),
        cli.embedding_dims.unwrap_or(768)
    );

    // Open the memory store
    let bridge_config = bridge::BridgeConfig::from_args(
        &cli.memory_dir,
        cli.embedding_url.as_deref(),
        cli.embedding_model.as_deref(),
        cli.embedding_dims,
    );

    let bridge = bridge::MemoryBridge::open(bridge_config)?;

    // Create the MCP server
    let server = server::SemanticMemoryServer::new(bridge);

    // Serve over stdio (MCP transport)
    // rmcp handles the JSON-RPC protocol: initialize, tools/list, tools/call
    // Multi-threaded runtime is required because tool handlers use
    // tokio::task::block_in_place to call async store methods from sync fn.
    // MCP-008: worker_threads(4) as a conservative floor to avoid thread
    // exhaustion under multi-client use.
    let rt = tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .worker_threads(4)
        .build()?;

    rt.block_on(async {
        let service = server.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}