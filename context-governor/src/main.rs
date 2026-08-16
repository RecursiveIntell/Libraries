use context_governor::{
    audit_compression_boundary, audit_mcp_tool_surface, compact_context, context_diff,
    evaluate_governed_memory, evaluate_leakage_free_rag, finalize_compacted_response,
    finalize_compacted_response_v2, parse_summary_output, receipt_index, render_summary_prompt,
    screen_knowledge_conflicts, select_retrieval_route, v2_projection, CertifiedCompactRequest,
    CompactRequest, CompactResponse, CompactResponseV2, ContextGovernorError, EvidenceClaim,
    FileContextStore, GovernanceCase, GovernanceFailureMode, PromptConfigV1, RagEvalInput,
    ReceiptActivationRequestV2, SearchScope, ToolManifestEntry,
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
        "capabilities" => print_json(&serde_json::json!({
            "schema": "ContextGovernorCapabilitiesV1",
            "engine": "ri-context-governor",
            "engine_version": env!("CARGO_PKG_VERSION"),
            "receipt_schema": "ContextCompactionReceiptV2",
            "integrity": "hmac-sha256-canonical-json-v1",
            "exactness_scope": "canonical_utf8_text_v1",
            "supports_recursive_lineage": true,
            "supports_certified_receipt_store": true,
            "supports_evidence_hmac": true,
            "supports_pending_activation": true,
        })),
        "compact-v2" => {
            let dir = required_arg(&args, "--dir")?;
            let parent = arg_value(&args, "--parent-receipt");
            reject_forbidden_certified_key_args(&args)?;
            let request: CertifiedCompactRequest = read_json_stdin("CertifiedCompactRequest")?;
            let authority = governed_authority_from_args(&args)?;
            let store = FileContextStore::with_key_ring(&dir, authority.key_ring().clone());
            print_json(&store.compact_next_v2(request.into(), parent.as_deref())?)
        }
        "finalize" => {
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            require_v1_schema(&response)?;
            let messages = response.compacted_messages.clone();
            print_json(&finalize_compacted_response(response, messages)?)
        }
        "finalize-v2" => {
            reject_forbidden_certified_key_args(&args)?;
            let request: CertifiedFinalizeRequestV2 =
                read_json_stdin("CertifiedFinalizeRequestV2")?;
            let authority = governed_authority_from_args(&args)?;
            print_json(&finalize_compacted_response_v2(
                request.candidate,
                request.compacted_messages,
                authority.key_ring(),
            )?)
        }
        "store" => {
            let dir = arg_value(&args, "--dir").unwrap_or_else(|| ".context-governor".to_string());
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            require_v1_schema(&response)?;
            let store = verified_store_or_legacy(&dir, arg_value(&args, "--hmac-key").as_deref(), arg_value(&args, "--keyring").as_deref())?;
            if let Some(key_path) = arg_value(&args, "--hmac-key") {
                let key = receipt_index::load_hmac_key(Path::new(&key_path))?;
                print_json(&store.save_with_status_with_hmac_key(&response, &key)?)
            } else {
                print_json(&store.save_with_status(&response)?)
            }
        }
        "store-v2" => {
            let dir = arg_value(&args, "--dir").unwrap_or_else(|| ".context-governor".to_string());
            let response: CompactResponseV2 = read_json_stdin("CompactResponseV2")?;
            reject_forbidden_certified_key_args(&args)?;
            let authority = governed_authority_from_args(&args)?;
            let store = FileContextStore::with_key_ring(&dir, authority.key_ring().clone());
            print_json(&store.save_v2_with_hmac_key(&response, &authority.key_ring().active)?)
        }
        "prepare-v2" => {
            let dir = required_arg(&args, "--dir")?;
            let response: CompactResponseV2 = read_json_stdin("CompactResponseV2")?;
            let store = governed_store_from_args(&args, &dir)?;
            print_json(&store.prepare_v2(&response)?)
        }
        "pending-v2" => {
            let dir = required_arg(&args, "--dir")?;
            let receipt = arg_value(&args, "--receipt");
            let store = governed_store_from_args(&args, &dir)?;
            print_json(&store.list_pending_v2(receipt.as_deref())?)
        }
        "activate-v2" => {
            let dir = required_arg(&args, "--dir")?;
            let request: ReceiptActivationRequestV2 =
                read_json_stdin("ReceiptActivationRequestV2")?;
            let store = governed_store_from_args(&args, &dir)?;
            print_json(&store.activate_v2(request)?)
        }
        "discard-v2" => {
            let dir = required_arg(&args, "--dir")?;
            let receipt = required_arg(&args, "--receipt")?;
            let store = governed_store_from_args(&args, &dir)?;
            print_json(&store.discard_pending_v2(&receipt)?)
        }
        "expand" => {
            let dir = required_arg(&args, "--dir")?;
            let receipt = required_arg(&args, "--receipt")?;
            let item = required_arg(&args, "--item")?;
            let max_chars = arg_value(&args, "--max-chars")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100_000);
            let store = store_for_read_or_retention(&args, &dir)?;
            print_json(&store.expand_lineage(&receipt, &item, max_chars)?)
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
            let store = store_for_read_or_retention(&args, &dir)?;
            print_json(&store.search(&query, top_k, scope)?)
        }
        "status" => {
            let dir = required_arg(&args, "--dir")?;
            let store = store_for_read_or_retention(&args, &dir)?;
            print_json(&store.status()?)
        }
        "prune" => {
            let dir = required_arg(&args, "--dir")?;
            let keep_last = arg_value(&args, "--keep-last")
                .and_then(|value| value.parse::<usize>().ok())
                .unwrap_or(100);
            let store = store_for_read_or_retention(&args, &dir)?;
            print_json(&store.prune_receipts_keep_last(keep_last)?)
        }
        "diff" => {
            let response: CompactResponse = read_json_stdin("CompactResponse")?;
            require_v1_schema(&response)?;
            print_json(&context_diff(&response))
        }
        "diff-v2" => {
            let response: CompactResponseV2 = read_json_stdin("CompactResponseV2")?;
            print_json(&context_diff(&v2_projection(&response)))
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
            require_v1_schema(&response)?;
            let config = PromptConfigV1::default();
            let prompt = render_summary_prompt(&response, &[], &config);
            print_json(&RenderPromptOutput {
                system: prompt.system,
                user: prompt.user,
            })
        }
        "render-prompt-v2" => {
            let response: CompactResponseV2 = read_json_stdin("CompactResponseV2")?;
            let projection = v2_projection(&response);
            let config = PromptConfigV1::default();
            let mut prompt = render_summary_prompt(&projection, &[], &config);
            if !response.receipt.covered_original_sources.is_empty() {
                prompt
                    .user
                    .push_str("\n\n=== TRANSITIVE EXACT SOURCE IDS ===\n");
                // Prompt-visible provenance is a bounded projection. The full
                // manifest remains in the receipt store for verified traversal;
                // inserting it all into the next compaction prompt would make
                // recursive metadata consume the very budget compaction saves.
                // Four opaque source IDs plus the overflow marker remain well
                // below the 512-byte prompt-provenance budget even when an ID
                // carries a long session-qualified prefix. Exact traversal
                // never depends on this display projection.
                const PROMPT_SOURCE_ID_LIMIT: usize = 4;
                for source in response
                    .receipt
                    .covered_original_sources
                    .iter()
                    .take(PROMPT_SOURCE_ID_LIMIT)
                {
                    prompt.user.push_str(&source.source_id);
                    prompt.user.push('\n');
                }
                let omitted = response
                    .receipt
                    .covered_original_sources
                    .len()
                    .saturating_sub(PROMPT_SOURCE_ID_LIMIT);
                if omitted > 0 {
                    prompt.user.push_str(&format!("… {omitted} additional source IDs retained in the verified receipt store\n"));
                }
                prompt.user.push_str("=== END TRANSITIVE EXACT SOURCE IDS ===\n");
            }
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
            let ring = load_key_ring_or_fail(&hmac_key_path, arg_value(&args, "--keyring").as_deref())?;
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
            let ring = load_key_ring_or_fail(&hmac_key_path, arg_value(&args, "--keyring").as_deref())?;
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
                "key_id": ring.active_key_id()?,
                "retired_key_count": ring.retired.len(),
                "receipt_count_in_dir": receipt_count,
            });
            print_json(&key_info)
        }
        "key-id-fd" => {
            let fd = required_arg(&args, "--governed-key-fd")?
                .parse::<i32>()
                .map_err(|_| ContextGovernorError::KeyUnreadable { path: "invalid governed key descriptor".to_string() })?;
            let path = format!("/proc/self/fd/{fd}");
            let key = receipt_index::load_hmac_key(Path::new(&path))?;
            print_json(&serde_json::json!({"key_id": receipt_index::key_id(&key)?}))
        }
        "key-init" => {
            Err(ContextGovernorError::ConfigurationPathOutsideCanonicalState {
                path: "key-init is Ares lifecycle-owned; Context Governor never creates canonical keys".to_string(),
            })
        }
        "key-rotate" => {
            Err(ContextGovernorError::ConfigurationPathOutsideCanonicalState {
                path: "key-rotate is Ares lifecycle-owned; Context Governor never rotates canonical keys".to_string(),
            })
        }
        "help" | "--help" | "-h" => {
            println!(
                "context-governor commands:\n  capabilities\n  compact < request.json > response.json\n  compact-v2 --dir DIR [--parent-receipt ID] GOVERNED_AUTH < request.json\n  finalize < response.json > finalized-response.json\n  finalize-v2 GOVERNED_AUTH < {{candidate,compacted_messages}}.json\n  store --dir DIR [--hmac-key PATH] < V1-response.json\n  prepare-v2 --dir DIR GOVERNED_AUTH < finalized-V2.json\n  pending-v2 --dir DIR [--receipt ID] GOVERNED_AUTH\n  activate-v2 --dir DIR GOVERNED_AUTH < {{receipt_id,committed_messages}}.json\n  discard-v2 --dir DIR --receipt ID GOVERNED_AUTH\n  store-v2 --dir DIR GOVERNED_AUTH < finalized-V2.json  (compatibility immediate prepare+activate)\n  expand --dir DIR --receipt RECEIPT --item ITEM [--max-chars N] GOVERNED_AUTH  (omit auth only for V1 inspection)\n  search --dir DIR --query TEXT [--scope all|exact|summary|receipt] [--top-k N] GOVERNED_AUTH  (omit auth only for V1 inspection)\n  status --dir DIR [GOVERNED_AUTH]\n  prune --dir DIR [--keep-last N] GOVERNED_AUTH  (omit auth only for V1-only stores)\n  GOVERNED_AUTH := --governed-key-fd FD --governed-snapshot-fd FD [--governed-retired-key-fd KEY_ID:FD]...\n  diff < response.json\n  diff-v2 < response.json\n  boundary-audit < request.json\n  audit-tool-surface --tools-json JSON\n  eval-governed-memory --harness-id ID --cases-json JSON\n  eval-rag-leakage --query Q --retrieved R --model-answer A\n  screen-conflicts --claims-json JSON\n  select-route --query Q\n  render-prompt < response.json  (renders LLM summary prompt)\n  render-prompt-v2 < response.json\n  verify --dir DIR [--hmac-key PATH] [--receipt ID]  (legacy/offline V1 inspection)\n  key-init\n  key-rotate\n  key-status --dir DIR [--hmac-key PATH]
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

/// Certified operations receive authority solely via inherited Ares-held file
/// descriptors.  Paths, key IDs, and raw request fields are not selectors.
fn governed_authority_from_args(
    args: &[String],
) -> Result<context_governor::GovernedKeyAuthority, ContextGovernorError> {
    let active = required_arg(args, "--governed-key-fd")?
        .parse::<i32>()
        .map_err(|_| ContextGovernorError::KeyUnreadable {
            path: "invalid governed key descriptor".to_string(),
        })?;
    let snapshot = required_arg(args, "--governed-snapshot-fd")?
        .parse::<i32>()
        .map_err(|_| ContextGovernorError::KeyUnreadable {
            path: "invalid governed snapshot descriptor".to_string(),
        })?;
    let mut retired = Vec::new();
    for value in args
        .windows(2)
        .filter(|pair| pair[0] == "--governed-retired-key-fd")
        .map(|pair| pair[1].as_str())
    {
        let Some((id, fd)) = value.split_once(':') else {
            return Err(ContextGovernorError::InvalidKeyEncoding {
                path: "invalid retired governed descriptor".to_string(),
            });
        };
        let fd = fd
            .parse::<i32>()
            .map_err(|_| ContextGovernorError::KeyUnreadable {
                path: "invalid retired governed descriptor".to_string(),
            })?;
        retired.push((id.to_string(), fd));
    }
    context_governor::GovernedKeyAuthority::from_fds(active, snapshot, &retired)
}

fn reject_forbidden_certified_key_args(args: &[String]) -> Result<(), ContextGovernorError> {
    for forbidden in [
        "--hmac-key",
        "--keyring",
        "--signing-key-id",
        "--verification-key",
        "--key-path",
    ] {
        if args.iter().any(|arg| arg == forbidden) {
            return Err(
                ContextGovernorError::ConfigurationPathOutsideCanonicalState {
                    path: format!("ForbiddenCallerKeyMaterial: {forbidden}"),
                },
            );
        }
    }
    Ok(())
}

fn governed_store_from_args(
    args: &[String],
    dir: &str,
) -> Result<FileContextStore, ContextGovernorError> {
    reject_forbidden_certified_key_args(args)?;
    let authority = governed_authority_from_args(args)?;
    Ok(FileContextStore::with_key_ring(
        dir,
        authority.key_ring().clone(),
    ))
}

/// V1 remains readable without authority.  A V2 signed receipt is still
/// rejected by the store unless inherited governed descriptors are supplied.
/// Caller-owned legacy key paths are never a certified fallback.
fn store_for_read_or_retention(
    args: &[String],
    dir: &str,
) -> Result<FileContextStore, ContextGovernorError> {
    if args.iter().any(|arg| arg == "--governed-key-fd") {
        governed_store_from_args(args, dir)
    } else {
        reject_forbidden_certified_key_args(args)?;
        Ok(FileContextStore::new(dir))
    }
}

fn load_key_ring_or_fail(
    configured_key_path: &Option<String>,
    keyring_path: Option<&str>,
) -> Result<receipt_index::KeyRing, ContextGovernorError> {
    let Some(path) = configured_key_path else {
        return Err(ContextGovernorError::CanonicalActiveKeyMissing {
            path: "Ares profile context-governor/keys/active.key".to_string(),
        });
    };
    match keyring_path {
        Some(keyring) => receipt_index::load_governed_key_ring(Path::new(path), Path::new(keyring)),
        None => receipt_index::load_hmac_key_ring(Path::new(path)),
    }
}

/// Certified runtime commands construct the HMAC-enforcing store here rather
/// than relying on each command to remember a separate verification step.
/// Omitting `--hmac-key` is retained only for explicit legacy inspection of
/// existing V1 receipts; the Hermes adapter always supplies it for V2 use.
fn verified_store_or_legacy(
    dir: impl AsRef<Path>,
    hmac_key_path: Option<&str>,
    keyring_path: Option<&str>,
) -> Result<FileContextStore, ContextGovernorError> {
    match hmac_key_path {
        Some(path) => {
            let ring =
                load_key_ring_or_fail(&Some(path.to_string()), keyring_path).map_err(|error| {
                    ContextGovernorError::ReceiptIntegrityUnavailable {
                        operation: "constructing certified receipt store".to_string(),
                        reason: error.to_string(),
                    }
                })?;
            Ok(FileContextStore::with_key_ring(dir, ring))
        }
        None => Ok(FileContextStore::new(dir)),
    }
}

#[derive(serde::Serialize)]
struct RenderPromptOutput {
    system: String,
    user: String,
}

#[derive(serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct CertifiedFinalizeRequestV2 {
    candidate: CompactResponseV2,
    compacted_messages: Vec<context_governor::Message>,
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

fn require_v1_schema(response: &CompactResponse) -> Result<(), ContextGovernorError> {
    if response.receipt.schema == "ContextCompactionReceiptV1" {
        Ok(())
    } else {
        Err(ContextGovernorError::UnsupportedReceiptSchema(
            response.receipt.schema.clone(),
        ))
    }
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
