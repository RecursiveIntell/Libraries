//! sm-bench: retrieval quality benchmark runner for semantic-memory.
//!
//! Runs JSONL fixture files against the semantic-memory HTTP server and emits
//! a JSONL receipt to the output directory. Optionally compares with a prior report.
//!
//! Usage:
//!   sm-bench [OPTIONS]
//!
//! Options:
//!   --server-url <URL>      HTTP base URL (default: http://127.0.0.1:1738)
//!   --fixtures-dir <DIR>    Directory with .jsonl fixture files
//!   --fixtures-file <FILE>  Single .jsonl fixture file
//!   --suite-name <NAME>     Label for this run (default: timestamp-derived)
//!   --commit <SHA>          Git commit hash (auto-detected if omitted)
//!   --output-dir <DIR>      Directory for JSONL receipts (default: ./reports)
//!   --compare <FILE>        Path to a prior report JSONL for before/after diff
//!   --help                  Print this help

use std::path::{Path, PathBuf};

use receipt_bench::sm_adapter::{
    compare_reports, load_fixtures_from_dir, load_fixtures_from_jsonl, run_sm_benchmark,
    SMBenchmarkConfig, SMBenchmarkReport,
};

fn main() {
    let args: Vec<String> = std::env::args().collect();

    if args.iter().any(|a| a == "--help" || a == "-h") {
        print_help();
        return;
    }

    let cfg = parse_args(&args);

    // Load fixtures.
    let fixtures = match (&cfg.fixtures_file, &cfg.fixtures_dir) {
        (Some(f), _) => {
            eprintln!("Loading fixtures from file: {}", f.display());
            load_fixtures_from_jsonl(f).unwrap_or_else(|e| {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            })
        }
        (None, Some(d)) => {
            eprintln!("Loading fixtures from dir: {}", d.display());
            load_fixtures_from_dir(d).unwrap_or_else(|e| {
                eprintln!("ERROR: {}", e);
                std::process::exit(1);
            })
        }
        (None, None) => {
            eprintln!(
                "ERROR: specify --fixtures-dir <DIR> or --fixtures-file <FILE>"
            );
            std::process::exit(1);
        }
    };

    if fixtures.is_empty() {
        eprintln!("ERROR: no fixtures found");
        std::process::exit(1);
    }

    eprintln!(
        "Running {} fixture(s) against {} ...",
        fixtures.len(),
        cfg.server_url
    );

    let bench_cfg = SMBenchmarkConfig {
        server_url: cfg.server_url.clone(),
        suite_name: cfg.suite_name.clone(),
        commit_hash: cfg.commit.clone().unwrap_or_else(auto_commit_hash),
        default_top_k: 10,
        warmup_queries: 1,
        timeout_secs: 30,
    };

    let report = run_sm_benchmark(fixtures, bench_cfg).unwrap_or_else(|e| {
        eprintln!("ERROR: benchmark failed: {}", e);
        std::process::exit(1);
    });

    // Print summary.
    println!();
    report.print_summary();

    // Save JSONL receipt.
    let output_dir = cfg.output_dir.as_deref().unwrap_or_else(|| Path::new("reports"));
    if let Err(e) = std::fs::create_dir_all(output_dir) {
        eprintln!("WARN: could not create output dir {}: {}", output_dir.display(), e);
    }

    let filename = format!(
        "sm_bench_{}_{}.jsonl",
        sanitize_label(&report.suite_name),
        &report.commit_hash[..report.commit_hash.len().min(8)],
    );
    let receipt_path = output_dir.join(&filename);
    let jsonl = report.to_jsonl();
    match std::fs::write(&receipt_path, &jsonl) {
        Ok(()) => eprintln!("\nReceipt written: {}", receipt_path.display()),
        Err(e) => eprintln!("\nWARN: could not write receipt: {}", e),
    }

    // Optional comparison.
    if let Some(compare_path) = &cfg.compare {
        eprintln!("\nComparing with: {}", compare_path.display());
        match load_report_from_jsonl(compare_path) {
            Ok(before) => {
                let comp = compare_reports(&before, &report);
                println!();
                comp.print_summary();
                let comp_json = serde_json::to_string_pretty(&comp)
                    .unwrap_or_else(|_| "{}".to_string());
                let comp_path = output_dir.join(format!(
                    "sm_bench_diff_{}_vs_{}.json",
                    sanitize_label(&before.suite_name),
                    sanitize_label(&report.suite_name),
                ));
                if let Err(e) = std::fs::write(&comp_path, &comp_json) {
                    eprintln!("WARN: could not write comparison: {}", e);
                } else {
                    eprintln!("Comparison written: {}", comp_path.display());
                }
            }
            Err(e) => eprintln!("WARN: could not load comparison file: {}", e),
        }
    }
}

// ─── CLI config ───────────────────────────────────────────────────────────────

struct CliConfig {
    server_url: String,
    fixtures_file: Option<PathBuf>,
    fixtures_dir: Option<PathBuf>,
    suite_name: String,
    commit: Option<String>,
    output_dir: Option<PathBuf>,
    compare: Option<PathBuf>,
}

fn parse_args(args: &[String]) -> CliConfig {
    let mut server_url = "http://127.0.0.1:1738".to_string();
    let mut fixtures_file: Option<PathBuf> = None;
    let mut fixtures_dir: Option<PathBuf> = None;
    let mut suite_name = default_suite_name();
    let mut commit: Option<String> = None;
    let mut output_dir: Option<PathBuf> = None;
    let mut compare: Option<PathBuf> = None;

    let mut i = 1usize;
    while i < args.len() {
        match args[i].as_str() {
            "--server-url" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    server_url = v.clone();
                }
            }
            "--fixtures-file" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    fixtures_file = Some(PathBuf::from(v));
                }
            }
            "--fixtures-dir" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    fixtures_dir = Some(PathBuf::from(v));
                }
            }
            "--suite-name" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    suite_name = v.clone();
                }
            }
            "--commit" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    commit = Some(v.clone());
                }
            }
            "--output-dir" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    output_dir = Some(PathBuf::from(v));
                }
            }
            "--compare" => {
                i += 1;
                if let Some(v) = args.get(i) {
                    compare = Some(PathBuf::from(v));
                }
            }
            _ => {}
        }
        i += 1;
    }

    CliConfig {
        server_url,
        fixtures_file,
        fixtures_dir,
        suite_name,
        commit,
        output_dir,
        compare,
    }
}

// ─── Helpers ──────────────────────────────────────────────────────────────────

fn auto_commit_hash() -> String {
    std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .filter(|o| o.status.success())
        .and_then(|o| String::from_utf8(o.stdout).ok())
        .map(|s| s.trim().to_string())
        .unwrap_or_else(|| "unknown".to_string())
}

fn default_suite_name() -> String {
    // Use seconds since epoch as a deterministic but unique label.
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| format!("run-{}", d.as_secs()))
        .unwrap_or_else(|_| "run".to_string())
}

fn sanitize_label(s: &str) -> String {
    s.chars()
        .map(|c| if c.is_alphanumeric() || c == '-' || c == '_' { c } else { '_' })
        .collect()
}

/// Load a prior `SMBenchmarkReport` from the summary line in a JSONL receipt.
fn load_report_from_jsonl(path: &Path) -> Result<SMBenchmarkReport, String> {
    let content = std::fs::read_to_string(path)
        .map_err(|e| format!("failed to read {}: {}", path.display(), e))?;
    // The full serialised report is not the JSONL output — the JSONL has per-query
    // lines plus a summary line. For comparison we need the full struct. Try
    // deserialising the whole file as a single JSON object first (from a
    // previously JSON-serialised report), then fall back to finding a summary line.
    if let Ok(report) = serde_json::from_str::<SMBenchmarkReport>(&content) {
        return Ok(report);
    }
    // Look for a `record_type: "summary"` line and use it to reconstruct a stub.
    // This is a lightweight fallback that gives enough info for compare_reports().
    Err(format!(
        "could not parse {} as an SMBenchmarkReport JSON file. \
         Pass a full JSON report (not JSONL) for --compare.",
        path.display()
    ))
}

fn print_help() {
    println!(
        "sm-bench: semantic-memory retrieval quality benchmark\n\
         \n\
         USAGE:\n\
         \tsm-bench [OPTIONS]\n\
         \n\
         OPTIONS:\n\
         \t--server-url <URL>      HTTP base URL (default: http://127.0.0.1:1738)\n\
         \t--fixtures-dir <DIR>    Directory with .jsonl fixture files\n\
         \t--fixtures-file <FILE>  Single .jsonl fixture file\n\
         \t--suite-name <NAME>     Label for this run (default: run-<epoch>)\n\
         \t--commit <SHA>          Git commit hash (auto-detected if omitted)\n\
         \t--output-dir <DIR>      Directory for receipts (default: ./reports)\n\
         \t--compare <FILE>        Prior JSON report for before/after diff\n\
         \t--help                  Print this help\n\
         \n\
         The server must be running before invoking sm-bench.\n\
         Build with: cargo build --features sm-adapter"
    );
}
