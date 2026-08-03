use clap::{Parser, Subcommand};
use std::path::PathBuf;
#[derive(Debug, Parser)]
#[command(name = "aew", about = "Agent Evidence Workbench")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}
#[derive(Debug, Subcommand)]
pub enum Commands {
    Init {
        path: Option<PathBuf>,
    },
    Run {
        #[arg(long)]
        name: Option<String>,
        #[arg(trailing_var_arg = true)]
        cmd: Vec<String>,
    },
    Verify {
        run_id: String,
    },
    Report {
        run_id: String,
        #[arg(long)]
        format: Option<String>,
    },
    Claims {
        run_id: Option<String>,
    },
    Evidence {
        run_id: Option<String>,
    },
    Adjudicate {
        run_id: String,
    },
    ImportTranscript {
        #[arg(long)]
        name: String,
        #[arg(long)]
        transcript: PathBuf,
    },
    ImportGraphResult {
        #[arg(long)]
        run_id: String,
        #[arg(long)]
        result: PathBuf,
    },
    Sign {
        run_id: String,
        #[arg(long)]
        key_hex: String,
    },
    VerifyReceipt {
        run_id: String,
        #[arg(long)]
        key_hex: String,
    },
    Promote {
        run_id: String,
        #[arg(long)]
        memory_dir: PathBuf,
        #[arg(long)]
        key_hex: String,
    },
}
