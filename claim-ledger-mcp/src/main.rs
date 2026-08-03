mod server;
mod tools;

use clap::Parser;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "claim-ledger-mcp", about = "MCP server for claim-ledger")]
struct Cli {
    #[arg(long)]
    ledger_dir: PathBuf,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    let service = server::ClaimLedgerServer::new(cli.ledger_dir)
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
