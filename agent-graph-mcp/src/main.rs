use agent_graph_mcp::cli;
use agent_graph_mcp::proxy;
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
        Ok(config) => config,
        Err(error) => {
            if error.exit_code != 0 {
                eprintln!("agent-graph-mcp: {}", error.message);
            }
            std::process::exit(error.exit_code);
        }
    };

    let runtime = tokio::runtime::Runtime::new()?;
    if !config.ephemeral {
        let runtime_dir = match cli::resolve_runtime_dir(&config) {
            Ok(path) => path,
            Err(error) => {
                eprintln!("agent-graph-mcp: {}", error.message);
                std::process::exit(error.exit_code);
            }
        };
        let socket = agent_graph_mcp::daemon::socket_path(&runtime_dir, &config.instance);
        if let Err(error) = runtime.block_on(proxy::run_stdio_proxy(&socket)) {
            eprintln!("agent-graph-mcp: {error}");
            std::process::exit(1);
        }
        return Ok(());
    }

    runtime.block_on(async {
        let integrity_key_path = config.integrity_key_path.or_else(|| {
            std::env::var("AGENT_GRAPH_INTEGRITY_KEY_PATH")
                .ok()
                .map(std::path::PathBuf::from)
        });
        let checkpoint_db_path = config.checkpoint_db_path.or_else(|| {
            std::env::var("AGENT_GRAPH_CHECKPOINT_DB_PATH")
                .ok()
                .map(std::path::PathBuf::from)
        });
        let server = AgentGraphServer::new_with_checkpoint_db(
            config.base_url,
            config.default_model,
            config.api_key,
            None,
            integrity_key_path,
            checkpoint_db_path,
        )
        .map_err(|error| anyhow::anyhow!(error))?;
        let service = server.serve(rmcp::transport::stdio()).await?;
        service.waiting().await?;
        Ok::<(), anyhow::Error>(())
    })?;

    Ok(())
}
