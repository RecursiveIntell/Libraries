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
    VerifyLibrariesRelease {
        #[arg(long)]
        repo: PathBuf,
    },
    InspectLibrariesRelease {
        #[arg(long)]
        repo: PathBuf,
    },
    SnapshotV2,
    CaptureV2 {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        evidence_id: String,
        #[arg(trailing_var_arg = true, required = true)]
        cmd: Vec<String>,
    },
    EvaluateV2 {
        #[arg(long)]
        input: PathBuf,
        #[arg(long)]
        record: bool,
    },
    Sign {
        run_id: String,
        #[arg(long)]
        key_file: PathBuf,
    },
    VerifyReceipt {
        run_id: String,
        #[arg(long)]
        key_file: PathBuf,
    },
    Promote {
        run_id: String,
        #[arg(long)]
        memory_dir: PathBuf,
        #[arg(long)]
        key_file: PathBuf,
    },
}
