use agent_graph_mcp::cli;
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

    let args: Vec<String> = std::env::args().skip(1).collect();
    let config = match cli::parse_args(&args) {
        Ok(c) => c,
        Err(e) => {
            if e.exit_code != 0 {
                eprintln!("agent-graph-mcp: {}", e.message);
            }
            std::process::exit(e.exit_code);
        }
    };

    // Validate integrity key if required
    if config.require_integrity_key {
        if let Some(ref key_path) = config.integrity_key_path {
            let metadata = match std::fs::metadata(key_path) {
                Ok(m) => m,
                Err(e) => {
                    eprintln!(
                        "agent-graph-mcp: --require-integrity-key: key file not readable: {e}"
                    );
                    std::process::exit(2);
                }
            };
            if metadata.len() < 32 {
                eprintln!(
                    "agent-graph-mcp: --require-integrity-key: key file is less than 32 bytes"
                );
                std::process::exit(2);
            }
        } else {
            // Try environment variable as fallback
            if std::env::var("AGENT_GRAPH_INTEGRITY_KEY_PATH").is_err() {
                eprintln!(
                    "agent-graph-mcp: --require-integrity-key requires --integrity-key or AGENT_GRAPH_INTEGRITY_KEY_PATH env"
                );
                std::process::exit(2);
            }
        }
    }

    // Resolve integrity key from env if not on CLI
    let integrity_key_path = config.integrity_key_path.or_else(|| {
        std::env::var("AGENT_GRAPH_INTEGRITY_KEY_PATH")
            .ok()
            .map(std::path::PathBuf::from)
    });

    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let server = AgentGraphServer::new(
            config.base_url,
            config.default_model,
            config.data_dir,
            integrity_key_path,
        )
        .map_err(|e| anyhow::anyhow!(e))?;
        let service = server.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
