use context_governor::{
    audit_compression_boundary, audit_mcp_tool_surface, compact_context, context_diff,
    evaluate_governed_memory, evaluate_leakage_free_rag, screen_knowledge_conflicts,
    select_retrieval_route, CompactRequest, CompactResponse, ContextGovernorError, EvidenceClaim,
    FileContextStore, GovernanceCase, GovernanceFailureMode, RagEvalInput, SearchScope,
    ToolManifestEntry,
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
        "prune" => {
            let dir = required_arg(&args, "--dir")?;
            let keep_last = arg_value(&args, "--keep-last")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100);
            let store = FileContextStore::new(dir);
            print_json(&store.prune_receipts_keep_last(keep_last)?)
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
        "audit-tool-surface" => {
            let tools_json = arg_value(&args, "--tools-json").unwrap_or_else(|| "[]".to_string());
            let tools: Vec<ToolManifestEntry> =
                serde_json::from_str(&tools_json).map_err(|err| {
                    ContextGovernorError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("failed to parse tools JSON: {err}"),
                    ))
                })?;
            print_json(&audit_mcp_tool_surface(&tools))
        }
        "eval-governed-memory" => {
            let harness_id =
                arg_value(&args, "--harness-id").unwrap_or_else(|| "default".to_string());
            let cases_json = arg_value(&args, "--cases-json").unwrap_or_else(|| "[]".to_string());
            let cases: Vec<GovernanceCaseJson> =
                serde_json::from_str(&cases_json).map_err(|err| {
                    ContextGovernorError::Io(std::io::Error::new(
                        std::io::ErrorKind::InvalidInput,
                        format!("failed to parse cases JSON: {err}"),
                    ))
                })?;
            let cases: Vec<GovernanceCase> = cases
                .into_iter()
                .map(|c| GovernanceCase::new(c.case_id, c.mode, c.passed))
                .collect();
            print_json(&evaluate_governed_memory(&harness_id, &cases))
        }
        "eval-rag-leakage" => {
            let task_id = arg_value(&args, "--task-id").unwrap_or_else(|| "default".to_string());
            let closed_book = arg_value(&args, "--closed-book-correct")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            let retrieved_correct = arg_value(&args, "--retrieved-correct")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(false);
            let retrieval_used = arg_value(&args, "--retrieval-used")
                .map(|v| v == "true" || v == "1")
                .unwrap_or(true);
            let input = RagEvalInput {
                task_id,
                closed_book_correct: closed_book,
                retrieved_answer_correct: retrieved_correct,
                retrieval_used,
            };
            print_json(&evaluate_leakage_free_rag(input))
        }
        "screen-conflicts" => {
            let claims_json = arg_value(&args, "--claims-json").unwrap_or_else(|| "[]".to_string());
            let claims_raw: Vec<ClaimJson> = serde_json::from_str(&claims_json).map_err(|err| {
                ContextGovernorError::Io(std::io::Error::new(
                    std::io::ErrorKind::InvalidInput,
                    format!("failed to parse claims JSON: {err}"),
                ))
            })?;
            let claims: Vec<EvidenceClaim> = claims_raw
                .into_iter()
                .map(|c| EvidenceClaim::new(c.id, c.text))
                .collect();
            print_json(&screen_knowledge_conflicts(&claims))
        }
        "select-route" => {
            let query = required_arg(&args, "--query")?;
            print_json(&select_retrieval_route(&query))
        }
        "help" | "--help" | "-h" => {
            println!(
                "context-governor commands:\n  compact < request.json > response.json\n  store --dir DIR < response.json\n  expand --dir DIR --receipt RECEIPT --item ITEM [--max-chars N]\n  search --dir DIR --query TEXT [--scope all|exact|summary|receipt] [--top-k N]\n  status --dir DIR\n  prune --dir DIR [--keep-last N]\n  diff < response.json\n  boundary-audit < request.json\n  audit-tool-surface --tools-json JSON\n  eval-governed-memory --harness-id ID --cases-json JSON\n  eval-rag-leakage --query Q --retrieved R --model-answer A\n  screen-conflicts --claims-json JSON\n  select-route --query Q"
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

#[derive(serde::Deserialize)]
struct GovernanceCaseJson {
    case_id: String,
    mode: GovernanceFailureMode,
    passed: bool,
}

#[derive(serde::Deserialize)]
struct ClaimJson {
    id: String,
    text: String,
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
