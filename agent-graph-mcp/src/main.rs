use std::path::PathBuf;

use agent_graph_mcp::AgentGraphServer;
use rmcp::ServiceExt;
use tracing_subscriber::EnvFilter;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(
            EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info")),
        )
        .init();

    let mut base_url = "http://127.0.0.1:11434".to_string();
    let mut default_model = "glm-5.2:cloud".to_string();
    let mut data_dir: Option<PathBuf> = None;
    let args: Vec<String> = std::env::args().collect();
    let mut iter = args.iter().skip(1);
    while let Some(arg) = iter.next() {
        match arg.as_str() {
            "--base-url" => {
                if let Some(value) = iter.next() {
                    base_url = value.clone();
                }
            }
            "--model" => {
                if let Some(value) = iter.next() {
                    default_model = value.clone();
                }
            }
            "--data-dir" => {
                if let Some(value) = iter.next() {
                    data_dir = Some(PathBuf::from(value));
                }
            }
            "--help" => {
                eprintln!("agent-graph-mcp [--base-url <ollama-url>] [--model <model-name>] [--data-dir <path>]");
                eprintln!("  --base-url  Ollama server URL (default: http://127.0.0.1:11434)");
                eprintln!("  --model     Default model for LLM nodes (default: glm-5.2:cloud)");
                eprintln!("  --data-dir  Persistent storage directory (default: in-memory only)");
                return Ok(());
            }
            _ => {}
        }
    }

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = AgentGraphServer::new(base_url, default_model, data_dir)
            .map_err(|e| anyhow::anyhow!(e))?;
        let service = server.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
