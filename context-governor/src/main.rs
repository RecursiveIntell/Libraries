use context_governor::{
    audit_compression_boundary, audit_mcp_tool_surface, compact_context, context_diff,
    evaluate_governed_memory, evaluate_leakage_free_rag, finalize_compacted_response,
    parse_summary_output, receipt_index, render_summary_prompt, screen_knowledge_conflicts,
    select_retrieval_route, CompactRequest, CompactResponse, ContextGovernorError, EvidenceClaim,
    FileContextStore, GovernanceCase, GovernanceFailureMode, PromptConfigV1, RagEvalInput,
    SearchScope, ToolManifestEntry,
};

use serde::Serialize;
use std::io::{self, Read};
use std::path::Path;

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
        "finalize" => {
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            let messages = response.compacted_messages.clone();
            print_json(&finalize_compacted_response(response, messages)?)
        }
        "store" => {
            let dir = arg_value(&args, "--dir").unwrap_or_else(|| ".context-governor".to_string());
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            let store = FileContextStore::new(dir);
            if let Some(key_path) = arg_value(&args, "--hmac-key") {
                let key = receipt_index::load_hmac_key(Path::new(&key_path))?;
                print_json(&store.save_with_status_with_hmac_key(&response, &key)?)
            } else {
                print_json(&store.save_with_status(&response)?)
            }
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
        "render-prompt" => {
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            let config = PromptConfigV1::default();
            let prompt = render_summary_prompt(&response, &[], &config);
            print_json(&RenderPromptOutput {
                system: prompt.system,
                user: prompt.user,
            })
        }
        "parse-summary" => {
            let mut input = String::new();
            io::stdin().read_to_string(&mut input)?;
            let parsed = parse_summary_output(&input);
            print_json(&parsed)
        }
        "verify" => {
            let dir = required_arg(&args, "--dir")?;
            let hmac_key_path = arg_value(&args, "--hmac-key");
            let receipt = arg_value(&args, "--receipt");
            let ring = load_key_ring_or_fail(&hmac_key_path)?;
            let ids = receipt.as_ref().map(|id| vec![id.clone()]);
            let (total, passed, failures) =
                receipt_index::verify_all_receipts(Path::new(&dir), &ring, ids.as_deref());
            println!("total={total} passed={passed} failed={}", failures.len());
            if !failures.is_empty() {
                for f in &failures {
                    eprintln!("FAIL: {f}");
                }
                return Err(ContextGovernorError::Io(std::io::Error::other(
                    "receipt HMAC verification failed",
                )));
            }
            Ok(())
        }
        "key-status" => {
            let dir = arg_value(&args, "--dir").unwrap_or_else(|| ".".to_string());
            let hmac_key_path = arg_value(&args, "--hmac-key");
            let ring = load_key_ring_or_fail(&hmac_key_path)?;
            let key_hex = hex::encode(&ring.active);
            let receipt_count = std::fs::read_dir(Path::new(&dir))
                .map(|entries| {
                    entries
                        .filter_map(|e| e.ok())
                        .filter(|e| {
                            e.path()
                                .file_stem()
                                .and_then(|s| s.to_str())
                                .map(|s| s.starts_with("ctxr_") || s.starts_with("rehydrated-"))
                                .unwrap_or(false)
                        })
                        .count()
                })
                .unwrap_or(0);
            let key_info = serde_json::json!({
                "hex_prefix": &key_hex[..8],
                "len": ring.active.len(),
                "retired_key_count": ring.retired.len(),
                "receipt_count_in_dir": receipt_count,
            });
            print_json(&key_info)
        }
        "key-init" => {
            let path = hmac_key_path(&arg_value(&args, "--hmac-key"));
            if path.exists() {
                return Err(ContextGovernorError::Io(std::io::Error::new(
                    std::io::ErrorKind::AlreadyExists,
                    format!(
                        "HMAC key already exists at {}; refusing to overwrite",
                        path.display()
                    ),
                )));
            }
            let key = receipt_index::generate_hmac_key();
            receipt_index::save_hmac_key(&path, &key)?;
            print_json(&serde_json::json!({
                "path": path,
                "fingerprint": receipt_index::key_fingerprint(&key),
                "len": key.len(),
            }))
        }
        "key-rotate" => {
            let path = hmac_key_path(&arg_value(&args, "--hmac-key"));
            let (old, new) = receipt_index::rotate_hmac_key(&path)?;
            print_json(&serde_json::json!({
                "path": path,
                "old_fingerprint": receipt_index::key_fingerprint(&old),
                "new_fingerprint": receipt_index::key_fingerprint(&new),
                "retired_key_path": receipt_index::retired_hmac_key_path(&path),
            }))
        }
        "help" | "--help" | "-h" => {
            println!(
                "context-governor commands:\n  compact < request.json > response.json\n  finalize < response.json > finalized-response.json\n  store --dir DIR [--hmac-key PATH] < response.json\n  expand --dir DIR --receipt RECEIPT --item ITEM [--max-chars N]\n  search --dir DIR --query TEXT [--scope all|exact|summary|receipt] [--top-k N]\n  status --dir DIR\n  prune --dir DIR [--keep-last N]\n  diff < response.json\n  boundary-audit < request.json\n  audit-tool-surface --tools-json JSON\n  eval-governed-memory --harness-id ID --cases-json JSON\n  eval-rag-leakage --query Q --retrieved R --model-answer A\n  screen-conflicts --claims-json JSON\n  select-route --query Q\n  render-prompt < response.json  (renders LLM summary prompt)\n  verify --dir DIR [--hmac-key PATH] [--receipt ID]\n  key-init [--hmac-key PATH]\n  key-rotate [--hmac-key PATH]\n+  key-status --dir DIR [--hmac-key PATH]
  parse-summary < summary.txt    (parses LLM output into structured fields)"
            );
            Ok(())
        }
        other => Err(ContextGovernorError::Io(std::io::Error::new(
            std::io::ErrorKind::InvalidInput,
            format!("unknown command: {other}"),
        ))),
    }
}

fn hmac_key_path(hmac_key_path: &Option<String>) -> std::path::PathBuf {
    hmac_key_path.as_ref().map_or_else(
        || {
            let home = std::env::var("HOME").unwrap_or_else(|_| "/tmp".to_string());
            std::path::PathBuf::from(format!("{home}/.hermes/context-governor/hmac.key"))
        },
        std::path::PathBuf::from,
    )
}

fn load_key_ring_or_fail(
    configured_key_path: &Option<String>,
) -> Result<receipt_index::KeyRing, ContextGovernorError> {
    receipt_index::load_hmac_key_ring(&hmac_key_path(configured_key_path))
}

#[derive(serde::Serialize)]
struct RenderPromptOutput {
    system: String,
    user: String,
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
