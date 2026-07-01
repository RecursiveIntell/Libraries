use context_governor::{
    audit_compression_boundary, compact_context, context_diff, CompactRequest, CompactResponse,
    ContextGovernorError, FileContextStore, SearchScope,
};

use serde::Serialize;
use std::io::{self, Read};

fn main() {
    if let Err(err) = run() {
        eprintln!("{err}");
        std::process::exit(1);
    }
}

fn run() -> Result<(), ContextGovernorError> {
    let args = std::env::args().skip(1).collect::<Vec<_>>();
    let cmd = args.first().map(String::as_str).unwrap_or("compact");
    match cmd {
        "compact" => {
            let request: CompactRequest = read_json_stdin("CompactRequest")?;
            print_json(&compact_context(request)?)
        }
        "store" => {
            let dir = arg_value(&args, "--dir").unwrap_or_else(|| ".context-governor".to_string());
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            let store = FileContextStore::new(dir);
            let path = store.save(&response)?;
            print_json(&serde_json::json!({
                "receipt_id": response.receipt.receipt_id,
                "path": path,
            }))
        }
        "expand" => {
            let dir = required_arg(&args, "--dir")?;
            let receipt = required_arg(&args, "--receipt")?;
            let item = required_arg(&args, "--item")?;
            let max_chars = arg_value(&args, "--max-chars")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100_000);
            let store = FileContextStore::new(dir);
            print_json(&store.expand(&receipt, &item, max_chars)?)
        }
        "search" => {
            let dir = required_arg(&args, "--dir")?;
            let query = required_arg(&args, "--query")?;
            let top_k = arg_value(&args, "--top-k")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(10);
            let scope = match arg_value(&args, "--scope").as_deref() {
                Some("exact") => SearchScope::ExactStore,
                Some("summary") => SearchScope::Summary,
                Some("receipt") => SearchScope::Receipt,
                Some("all") | None => SearchScope::All,
                Some(other) => {
                    return Err(ContextGovernorError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("invalid --scope {other}; expected all|exact|summary|receipt"),
                    )))
                }
            };
            let store = FileContextStore::new(dir);
            print_json(&store.search(&query, top_k, scope)?)
        }
        "status" => {
            let dir = required_arg(&args, "--dir")?;
            let store = FileContextStore::new(dir);
            print_json(&store.status()?)
        }
        "diff" => {
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            print_json(&context_diff(&response))
        }
        "boundary-audit" => {
            let request: BoundaryAuditRequest = read_json_stdin("BoundaryAuditRequest")?;
            print_json(&audit_compression_boundary(
                &request.source_fragments,
                &request.compressed_summary,
            ))
        }
        "help" | "--help" | "-h" => {
            println!(
                "context-governor commands:\n  compact < request.json > response.json\n  store --dir DIR < response.json\n  expand --dir DIR --receipt RECEIPT --item ITEM [--max-chars N]\n  search --dir DIR --query TEXT [--scope all|exact|summary|receipt] [--top-k N]\n  status --dir DIR\n  diff < response.json\n  boundary-audit < request.json"
            );
            Ok(())
        }
        other => Err(ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown command: {other}"),
        ))),
    }
}

#[derive(serde::Deserialize)]
struct BoundaryAuditRequest {
    source_fragments: Vec<String>,
    compressed_summary: String,
}

fn read_json_stdin<T: serde::de::DeserializeOwned>(label: &str) -> Result<T, ContextGovernorError> {
    let mut input = String::new();
    io::stdin().read_to_string(&mut input)?;
    serde_json::from_str(&input).map_err(|err| {
        ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("failed to parse {label} JSON: {err}"),
        ))
    })
}

fn print_json(value: &impl Serialize) -> Result<(), ContextGovernorError> {
    println!("{}", serde_json::to_string_pretty(value)?);
    Ok(())
}

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.windows(2)
        .find(|window| window[0] == flag)
        .map(|window| window[1].clone())
}

fn required_arg(args: &[String], flag: &str) -> Result<String, ContextGovernorError> {
    arg_value(args, flag).ok_or_else(|| {
        ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("missing required argument {flag}"),
        ))
    })
}
