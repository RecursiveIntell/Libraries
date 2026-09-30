use claim_ledger_mcp::server;
use claim_ledger_mcp::trusted_head;
use clap::Parser;
use rmcp::ServiceExt;
use std::path::PathBuf;
use tracing_subscriber::EnvFilter;

#[derive(Parser, Debug)]
#[command(name = "claim-ledger-mcp", about = "MCP server for claim-ledger")]
struct Cli {
    #[arg(long)]
    ledger_dir: PathBuf,
    /// Optional operator trust-root file (ClaimLedgerTrustRootV1, mode 0600).
    /// Requires --expected-head; together they enable anchored projection.
    #[arg(long)]
    trust_root: Option<PathBuf>,
    /// Optional independent trusted-head file (ClaimLedgerMcpTrustedHeadV1).
    /// Requires --trust-root; start fails closed without it.
    #[arg(long)]
    expected_head: Option<PathBuf>,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_writer(std::io::stderr)
        .with_env_filter(EnvFilter::from_default_env())
        .init();
    let cli = Cli::parse();
    // Fail closed at start: if only one of the pair is given, refuse. Both
    // files are read and validated NOW; a bad root/head aborts startup.
    let (trust_root, expected_head) = match (cli.trust_root, cli.expected_head) {
        (None, None) => (None, None),
        (Some(root), Some(head)) => {
            let root_text = std::fs::read_to_string(&root)
                .map_err(|e| anyhow::anyhow!("--trust-root unreadable: {}: {e}", root.display()))?;
            let head_text = std::fs::read_to_string(&head).map_err(|e| {
                anyhow::anyhow!("--expected-head unreadable: {}: {e}", head.display())
            })?;
            // Validate both up front (typed, before serving anything).
            claim_ledger::trust_root::load_trust_root_from_str(&root_text)
                .map_err(|e| anyhow::anyhow!("--trust-root invalid: {e}"))?;
            trusted_head::parse_expected_head(&head_text)
                .map_err(|e| anyhow::anyhow!("--expected-head invalid: {e}"))?;
            (Some(root_text), Some(head_text))
        }
        (Some(_), None) => {
            return Err(anyhow::anyhow!(
                "--trust-root requires --expected-head (fail closed)"
            ));
        }
        (None, Some(_)) => {
            return Err(anyhow::anyhow!(
                "--expected-head requires --trust-root (fail closed)"
            ));
        }
    };
    let service = server::ClaimLedgerServer::with_trust(cli.ledger_dir, trust_root, expected_head)
        .serve(rmcp::transport::stdio())
        .await?;
    service.waiting().await?;
    Ok(())
}
