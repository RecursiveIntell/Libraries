use forge_engine::ForgeStore;
use forge_pilot::{
    answer_repo_question, bootstrap_source_workspace, import_recent_forge_bundles, observe_scope,
    provider_fallback_allowed, render_candidate_report, render_loop_report,
    render_observation_report, render_terminal_answer, repo_chat_provider_grounding_message,
    score_targets, ExternalHaltFlag, HaltReason, LoopConfig, LoopReport, LoopRunner,
    LoopRunnerResources, PilotHistory,
};
use knowledge_runtime::{RuntimeConfig, Scope};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use semantic_memory::{MemoryConfig, MemoryStore, MockEmbedder};
use serde::{Deserialize, Serialize};
use serde_json::json;
use std::env;
use std::fs;
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

pub async fn run() -> Result<(), String> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if raw_args.is_empty() {
        return run_tui().await;
    }

    if raw_args.len() == 1 && matches!(raw_args[0].as_str(), "-h" | "--help") {
        print!("{}", CommandArgs::help());
        return Ok(());
    }

    if matches!(
        raw_args.first().map(|s| s.as_str()),
        Some("tui" | "interactive")
    ) {
        return run_tui().await;
    }

    run_command_cli(raw_args).await
}

async fn run_command_cli(argv: Vec<String>) -> Result<(), String> {
    let args = CommandArgs::parse(argv)?;
    if args.help {
        print!("{}", CommandArgs::help());
        return Ok(());
    }

    let scope = Scope::new(args.namespace.clone());
    let config = build_loop_config(
        &scope,
        args.workspace_path.clone(),
        &args.memory_dir,
        &args.forge_db,
        args.loop_interval_secs,
        args.max_iterations,
    );
    let resources = open_resources(&args.memory_dir, &args.forge_db, &config)?;

    match args.command.as_str() {
        "observe" => {
            let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            if args.json {
                println!("{}", render_observation_report(&observation, true));
            } else {
                println!("{}", explain_observation(&observation));
            }
        }
        "candidates" => {
            let observation = observe_scope(&resources.runtime, &resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            let candidates = score_targets(&observation, &PilotHistory::default(), &config);
            if args.json {
                println!("{}", render_candidate_report(&candidates, true));
            } else {
                println!("{}", explain_candidates(&candidates));
            }
        }
        "bootstrap-source" => {
            let report = bootstrap_source_workspace(&resources.memory_store, &config)
                .await
                .map_err(|error| error.to_string())?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into())
                );
            } else {
                println!("{}", explain_bootstrap_report(&report));
            }
        }
        "import" => {
            let report = import_recent_forge_bundles(
                &args.namespace,
                &resources.forge_store,
                &resources.memory_store,
                64,
            )
            .await
            .map_err(|error| error.to_string())?;
            if args.json {
                println!(
                    "{}",
                    serde_json::to_string_pretty(&report).unwrap_or_else(|_| "{}".into())
                );
            } else {
                println!("{}", explain_import_report(&report));
            }
        }
        "run" => {
            let mut runner = LoopRunner::new(config, resources);
            let report = runner.run().await.map_err(|error| error.to_string())?;
            if args.json {
                println!("{}", render_loop_report(&report, true));
            } else {
                println!("{}", explain_loop_report(&report));
            }
        }
        other => {
            return Err(format!(
                "unknown command `{other}`\n\n{}",
                CommandArgs::help()
            ))
        }
    }

    Ok(())
}

async fn run_tui() -> Result<(), String> {
    let mut state = AppState::load()?;
    state.prepare_runtime_layout()?;
    let mut active_loop: Option<ActiveLoop> = None;

    loop {
        println!();
        println!("Forge Pilot");
        println!("{}", state.plain_english_status());
        if let Some(loop_state) = &active_loop {
            let summary = loop_state
                .status
                .lock()
                .map_err(|_| "failed to read loop status".to_string())?
                .headline();
            println!("Closed-loop status: {summary}");
        } else {
            println!("Closed-loop status: stopped");
        }
        println!();
        println!("1. Set workspace folder");
        println!("2. Set namespace");
        println!("3. Provider setup");
        println!("4. Start closed-loop");
        println!("5. Stop closed-loop");
        println!("6. Status screen");
        println!("7. Observe current state");
        println!("8. Bootstrap source memory");
        println!("9. Import Forge bundles");
        println!("10. Show candidates");
        println!("11. Chat");
        println!("12. Save settings");
        println!("0. Quit");

        match prompt("Choose an option")?.trim() {
            "1" => {
                let folder = prompt("Workspace folder")?;
                if !folder.trim().is_empty() {
                    state.set_workspace_folder(PathBuf::from(folder.trim()));
                    state.prepare_runtime_layout()?;
                    println!("Workspace updated.");
                }
            }
            "2" => {
                let namespace = prompt("Namespace")?;
                if !namespace.trim().is_empty() {
                    state.namespace = namespace.trim().to_string();
                    println!("Namespace updated.");
                }
            }
            "3" => {
                configure_provider(&mut state).await?;
            }
            "4" => {
                if active_loop.is_some() {
                    println!("The closed loop is already running.");
                } else {
                    active_loop = Some(start_closed_loop(state.clone())?);
                    println!("The closed loop has started.");
                }
            }
            "5" => {
                if let Some(loop_state) = active_loop.take() {
                    stop_closed_loop(loop_state)?;
                    println!("The closed loop has stopped.");
                } else {
                    println!("The closed loop is not running.");
                }
            }
            "6" => {
                show_status_screen(&state, active_loop.as_ref())?;
            }
            "7" => {
                let config = state.to_loop_config();
                let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
                let observation =
                    observe_scope(&resources.runtime, &resources.memory_store, &config)
                        .await
                        .map_err(|error| error.to_string())?;
                println!("{}", explain_observation(&observation));
            }
            "8" => {
                let report = run_bootstrap_from_tui(&state).await?;
                println!("{}", explain_bootstrap_report(&report));
            }
            "9" => {
                let report = run_import_from_tui(&state).await?;
                println!("{}", explain_import_report(&report));
            }
            "10" => {
                let config = state.to_loop_config();
                let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
                let observation =
                    observe_scope(&resources.runtime, &resources.memory_store, &config)
                        .await
                        .map_err(|error| error.to_string())?;
                let candidates = score_targets(&observation, &PilotHistory::default(), &config);
                println!("{}", explain_candidates(&candidates));
            }
            "11" => {
                chat_loop(&state).await?;
            }
            "12" => {
                state.prepare_runtime_layout()?;
                state.save()?;
                println!("Settings saved.");
            }
            "0" => {
                if let Some(loop_state) = active_loop.take() {
                    stop_closed_loop(loop_state)?;
                }
                state.prepare_runtime_layout()?;
                state.save()?;
                println!("Goodbye.");
                break;
            }
            _ => {
                println!("I didn't understand that choice.");
            }
        }
    }

    Ok(())
}

#[derive(Debug)]
struct ActiveLoop {
    stop_flag: Arc<AtomicBool>,
    halt_flag: ExternalHaltFlag,
    status: Arc<Mutex<LoopStatusSnapshot>>,
    join_handle: Option<thread::JoinHandle<()>>,
}

fn start_closed_loop(state: AppState) -> Result<ActiveLoop, String> {
    let mut state = state;
    state.prepare_runtime_layout()?;
    let stop_flag = Arc::new(AtomicBool::new(false));
    let status = Arc::new(Mutex::new(LoopStatusSnapshot::starting()));
    let stop_flag_thread = stop_flag.clone();
    let status_thread = status.clone();

    let config = state.to_loop_config();
    let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
    let runner = LoopRunner::new(config, resources);
    let halt_flag = runner.halt_flag();
    let halt_flag_thread = halt_flag.clone();
    let interval_secs = state.loop_interval_secs;

    let join_handle = thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build();

        let Ok(runtime) = runtime else {
            let mut slot = status_thread.lock().ok();
            if let Some(ref mut slot) = slot {
                slot.push_event("The closed loop failed to build its runtime.".into());
                slot.last_error = Some("runtime build failed".into());
            }
            return;
        };

        runtime.block_on(async move {
            let mut runner = runner;
            loop {
                if stop_flag_thread.load(Ordering::SeqCst) {
                    break;
                }

                match runner.run().await {
                    Ok(report) => {
                        if let Ok(mut slot) = status_thread.lock() {
                            slot.record_report(&report);
                        }
                    }
                    Err(error) => {
                        if let Ok(mut slot) = status_thread.lock() {
                            slot.last_error = Some(error.to_string());
                            slot.push_event(format!("The closed loop failed: {error}"));
                        }
                    }
                }

                if stop_flag_thread.load(Ordering::SeqCst) {
                    break;
                }
                tokio::time::sleep(tokio::time::Duration::from_secs(interval_secs)).await;
            }
        });

        halt_flag_thread.halt();
    });

    Ok(ActiveLoop {
        stop_flag,
        halt_flag,
        status,
        join_handle: Some(join_handle),
    })
}

fn stop_closed_loop(mut active_loop: ActiveLoop) -> Result<(), String> {
    active_loop.stop_flag.store(true, Ordering::SeqCst);
    active_loop.halt_flag.halt();
    if let Some(join_handle) = active_loop.join_handle.take() {
        join_handle
            .join()
            .map_err(|_| "closed-loop worker panicked".to_string())?;
    }
    Ok(())
}

async fn configure_provider(state: &mut AppState) -> Result<(), String> {
    println!("Provider setup");
    println!("Provider fallback is used only for clearly non-repo questions.");
    println!("Provider responses stream in the chat view when supported.");
    println!("1. Ollama");
    println!("2. OpenAI");
    println!("3. Disable chat provider");

    match prompt("Choose a provider")?.trim() {
        "1" => {
            let base_url = prompt_default("Ollama base URL", state.ollama_base_url())?;
            let model = prompt_default("Ollama model", state.ollama_model())?;
            state.provider = Some(ChatProviderConfig::Ollama { base_url, model });
            println!("Ollama is configured.");
        }
        "2" => {
            let base_url = prompt_default("OpenAI base URL", state.openai_base_url())?;
            let model = prompt_default("OpenAI model", state.openai_model())?;
            let api_key =
                prompt_default("OpenAI API key", state.openai_api_key().unwrap_or_default())?;
            state.provider = Some(ChatProviderConfig::OpenAi {
                base_url,
                model,
                api_key,
            });
            println!("OpenAI is configured.");
        }
        "3" => {
            state.provider = None;
            println!("Chat provider disabled.");
        }
        _ => println!("No provider change applied."),
    }

    Ok(())
}

async fn chat_loop(state: &AppState) -> Result<(), String> {
    println!("Plain-English chat");
    println!("Press enter on an empty line to leave chat.");
    println!(
        "Repo-grounded answers render locally; provider fallback streams for non-repo questions."
    );
    let config = state.to_loop_config();
    let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
    let mut history = vec![ChatMessage {
        role: "system".into(),
        content: "You are Forge Pilot's plain-English terminal assistant. Use grounded workspace-source citations when available and abstain when the source memory cannot support a claim.".into(),
    }];

    loop {
        let input = prompt("You")?;
        if input.trim().is_empty() {
            break;
        }
        history.push(ChatMessage {
            role: "user".into(),
            content: input.trim().into(),
        });
        let grounded = answer_repo_question(
            &resources.runtime,
            &resources.memory_store,
            &config,
            input.trim(),
        )
        .await
        .map_err(|error| error.to_string())?;
        let allow_fallback =
            !grounded.grounded && state.provider.is_some() && provider_fallback_allowed(&grounded);
        let reply = if allow_fallback {
            let provider = match state.provider.as_ref() {
                Some(provider) => provider,
                None => unreachable!("provider presence checked"),
            };
            let augmented_history = augment_history_with_grounding(&history, &grounded);
            print!("Pilot: ");
            io::stdout().flush().map_err(|error| error.to_string())?;
            chat_with_provider(provider, &augmented_history).await?
        } else {
            render_terminal_answer(&grounded)
        };
        if !allow_fallback {
            println!("Pilot: {reply}");
        } else {
            println!();
        }
        history.push(ChatMessage {
            role: "assistant".into(),
            content: reply,
        });
    }

    Ok(())
}

fn augment_history_with_grounding(
    history: &[ChatMessage],
    grounded: &forge_pilot::RepoChatAnswer,
) -> Vec<ChatMessage> {
    let mut augmented = history.to_vec();
    augmented.push(ChatMessage {
        role: "system".into(),
        content: format!(
            "Answer from grounded imported repo memory first. Do not contradict the grounding. If grounding is unavailable, say so plainly.\n{}",
            repo_chat_provider_grounding_message(grounded)
        ),
    });
    augmented
}

async fn chat_with_provider(
    provider: &ChatProviderConfig,
    history: &[ChatMessage],
) -> Result<String, String> {
    match provider {
        ChatProviderConfig::Ollama { base_url, model } => {
            chat_with_ollama(base_url, model, history).await
        }
        ChatProviderConfig::OpenAi {
            base_url,
            model,
            api_key,
        } => chat_with_openai(base_url, model, api_key, history).await,
    }
}

fn show_status_screen(state: &AppState, active_loop: Option<&ActiveLoop>) -> Result<(), String> {
    loop {
        print!("\x1B[2J\x1B[H");
        println!("Forge Pilot Status");
        println!();
        println!("{}", state.plain_english_status());
        println!();
        if let Some(active_loop) = active_loop {
            let snapshot = active_loop
                .status
                .lock()
                .map_err(|_| "failed to read live status".to_string())?
                .clone();
            println!("{}", snapshot.render());
        } else {
            println!("The closed loop is currently stopped.");
        }
        println!();
        let input = prompt("Press Enter to refresh, or type q to return")?;
        if input.trim().eq_ignore_ascii_case("q") {
            break;
        }
    }
    Ok(())
}

async fn chat_with_ollama(
    base_url: &str,
    model: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    let client = reqwest::Client::new();
    let mut response = client
        .post(format!("{}/api/chat", trim_trailing_slash(base_url)))
        .json(&json!({
            "model": model,
            "messages": history,
            "stream": true
        }))
        .send()
        .await
        .map_err(|error| format!("ollama request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("ollama response decode failed: {error}"))?;
        return Err(format!("ollama returned {status}: {value}"));
    }

    stream_json_lines(&mut response, "ollama response stream", |value| {
        value
            .get("message")
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(ToString::to_string)
    })
    .await
}

async fn chat_with_openai(
    base_url: &str,
    model: &str,
    api_key: &str,
    history: &[ChatMessage],
) -> Result<String, String> {
    if api_key.trim().is_empty() {
        return Err("OpenAI API key is not configured.".into());
    }

    let client = reqwest::Client::new();
    let mut response = client
        .post(format!(
            "{}/v1/chat/completions",
            trim_trailing_slash(base_url)
        ))
        .header(AUTHORIZATION, format!("Bearer {api_key}"))
        .header(CONTENT_TYPE, "application/json")
        .json(&json!({
            "model": model,
            "messages": history,
            "stream": true
        }))
        .send()
        .await
        .map_err(|error| format!("openai request failed: {error}"))?;

    let status = response.status();
    if !status.is_success() {
        let value: serde_json::Value = response
            .json()
            .await
            .map_err(|error| format!("openai response decode failed: {error}"))?;
        return Err(format!("openai returned {status}: {value}"));
    }

    stream_sse_lines(&mut response, "openai response stream", |value| {
        value
            .get("choices")
            .and_then(|choices| choices.as_array())
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
            .and_then(|delta| delta.get("content"))
            .and_then(|content| content.as_str())
            .map(ToString::to_string)
    })
    .await
}

async fn run_bootstrap_from_tui(
    state: &AppState,
) -> Result<forge_pilot::BootstrapSourceReport, String> {
    let config = state.to_loop_config();
    let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
    bootstrap_source_workspace(&resources.memory_store, &config)
        .await
        .map_err(|error| error.to_string())
}

async fn run_import_from_tui(
    state: &AppState,
) -> Result<forge_pilot::ImportBootstrapReport, String> {
    let config = state.to_loop_config();
    let resources = open_resources(&state.memory_dir, &state.forge_db, &config)?;
    import_recent_forge_bundles(
        &state.namespace,
        &resources.forge_store,
        &resources.memory_store,
        64,
    )
    .await
    .map_err(|error| error.to_string())
}

async fn stream_json_lines<F>(
    response: &mut reqwest::Response,
    stream_name: &str,
    mut extract: F,
) -> Result<String, String>
where
    F: FnMut(&serde_json::Value) -> Option<String>,
{
    let mut buffer = String::new();
    let mut reply = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("{stream_name} read failed: {error}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer.drain(..=index);
            if line.is_empty() {
                continue;
            }
            let value: serde_json::Value = serde_json::from_str(&line)
                .map_err(|error| format!("{stream_name} decode failed: {error}; line={line}"))?;
            if let Some(content) = extract(&value) {
                print!("{content}");
                io::stdout().flush().map_err(|error| error.to_string())?;
                reply.push_str(&content);
            }
        }
    }
    if !buffer.trim().is_empty() {
        let value: serde_json::Value = serde_json::from_str(buffer.trim())
            .map_err(|error| format!("{stream_name} decode failed: {error}; trailing={buffer}"))?;
        if let Some(content) = extract(&value) {
            print!("{content}");
            io::stdout().flush().map_err(|error| error.to_string())?;
            reply.push_str(&content);
        }
    }
    let reply = reply.trim().to_string();
    if reply.is_empty() {
        Err(format!(
            "{stream_name} did not include any assistant content"
        ))
    } else {
        Ok(reply)
    }
}

async fn stream_sse_lines<F>(
    response: &mut reqwest::Response,
    stream_name: &str,
    mut extract: F,
) -> Result<String, String>
where
    F: FnMut(&serde_json::Value) -> Option<String>,
{
    let mut buffer = String::new();
    let mut reply = String::new();
    while let Some(chunk) = response
        .chunk()
        .await
        .map_err(|error| format!("{stream_name} read failed: {error}"))?
    {
        buffer.push_str(&String::from_utf8_lossy(&chunk));
        while let Some(index) = buffer.find('\n') {
            let line = buffer[..index].trim().to_string();
            buffer.drain(..=index);
            if line.is_empty() || !line.starts_with("data:") {
                continue;
            }
            let payload = line.trim_start_matches("data:").trim();
            if payload == "[DONE]" {
                let reply = reply.trim().to_string();
                return if reply.is_empty() {
                    Err(format!("{stream_name} ended without assistant content"))
                } else {
                    Ok(reply)
                };
            }
            let value: serde_json::Value = serde_json::from_str(payload).map_err(|error| {
                format!("{stream_name} decode failed: {error}; payload={payload}")
            })?;
            if let Some(content) = extract(&value) {
                print!("{content}");
                io::stdout().flush().map_err(|error| error.to_string())?;
                reply.push_str(&content);
            }
        }
    }
    let reply = reply.trim().to_string();
    if reply.is_empty() {
        Err(format!(
            "{stream_name} did not include any assistant content"
        ))
    } else {
        Ok(reply)
    }
}

fn explain_observation(observation: &forge_pilot::Observation) -> String {
    let mut lines = vec![format!(
        "I checked namespace {}.",
        observation.scope_key.namespace
    )];
    lines.push(format!(
        "Resolved workspace path: {}.",
        observation.paths.workspace_path
    ));
    lines.push(format!(
        "Memory folder: {} (state: {:?}).",
        observation.paths.memory_dir, observation.paths.memory_dir_state
    ));
    lines.push(format!(
        "Forge DB: {} (state: {:?}).",
        observation.paths.forge_db_path, observation.paths.forge_db_state
    ));
    lines.push(format!(
        "Supported source files: {}. Import records found: {}. Import state: {:?}.",
        observation.status.supported_file_count,
        if observation.status.import_records_found {
            "yes"
        } else {
            "no"
        },
        observation.status.import_record_disposition
    ));
    lines.push(format!(
        "Forge evidence bundles available: {}.",
        observation.status.available_forge_bundle_count
    ));
    lines.push(format!(
        "I found {} claim versions, {} relation versions, and {} episodes.",
        observation.scope_health.total_claim_versions,
        observation.scope_health.total_relation_versions,
        observation.scope_health.total_episodes
    ));
    if observation.status.imported_file_count > 0 {
        lines.push(format!(
            "Imported current state from the latest manifest: {} file(s), {} chunk(s), {} symbol(s). Bootstrap richness: {:?}.",
            observation.status.imported_file_count,
            observation.status.imported_chunk_count,
            observation.status.imported_symbol_count,
            observation.status.bootstrap_richness
        ));
        if observation
            .source_inventory
            .imported_current_state
            .large_file_skip_count
            > 0
        {
            lines.push(format!(
                "Large files skipped and still visible in manifest truth: {}.",
                observation
                    .source_inventory
                    .imported_current_state
                    .large_file_skip_count
            ));
        }
    }
    lines.push(format!(
        "Disposition: {:?}. Next step: {}",
        observation.status.disposition, observation.status.exact_next_step
    ));
    if !observation.status.other_import_namespaces.is_empty() {
        lines.push(format!(
            "Recent imports for other namespaces: {}.",
            observation.status.other_import_namespaces.join(", ")
        ));
    }
    if observation.degradations.is_empty() {
        lines.push("There are no active degradation markers.".into());
    } else {
        lines.push(format!(
            "Active degradations: {}.",
            observation
                .degradations
                .iter()
                .map(|degradation| format!("{} ({})", degradation.kind, degradation.detail))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    lines.join("\n")
}

fn explain_candidates(candidates: &[forge_pilot::TargetCandidate]) -> String {
    if candidates.is_empty() {
        return "I do not see any candidates worth investigating right now.".into();
    }

    let mut lines = vec![format!(
        "I found {} candidate(s). Here are the top ones:",
        candidates.len()
    )];
    for candidate in candidates.iter().take(5) {
        lines.push(format!(
            "- {} with urgency {:.2}. {}",
            candidate.stable_key, candidate.urgency, candidate.rationale
        ));
    }
    lines.join("\n")
}

fn explain_import_report(report: &forge_pilot::ImportBootstrapReport) -> String {
    if report.forge_bundle_count == 0 {
        return format!(
            "No Forge evidence bundles are available yet for namespace {}. The pilot cannot bootstrap canonical semantic-memory state until Forge has evidence to export.",
            report.namespace
        );
    }

    format!(
        "Imported {} Forge evidence bundle(s) into canonical semantic-memory state for namespace {}.",
        report.imported_bundle_ids.len(),
        report.namespace
    )
}

fn explain_bootstrap_report(report: &forge_pilot::BootstrapSourceReport) -> String {
    if report.scanned_file_count == 0 {
        return format!(
            "Bootstrap source scan found no supported files for namespace {} under {}.",
            report.namespace, report.workspace_path
        );
    }

    let mut summary = format!(
        "Bootstrap source imported {} file-backed source claim(s), {} chunk claim(s), and {} symbol claim(s) for namespace {}. Import status: {}.",
        report.imported_file_count,
        report.imported_chunk_count,
        report.imported_symbol_count,
        report.namespace,
        report
            .import_status
            .as_deref()
            .unwrap_or("not_imported")
    );
    if report.was_duplicate {
        summary.push_str(" The snapshot was already imported.");
    }
    if report.skipped_file_count > 0 {
        summary.push_str(&format!(" Skipped {} file(s).", report.skipped_file_count));
    }
    if report.manifest_id.is_some() {
        summary.push_str(&format!(
            " Current manifest: {} file(s), {} chunk(s), {} symbol(s), richness {:?}.",
            report.current_manifest_file_count,
            report.current_manifest_chunk_count,
            report.current_manifest_symbol_count,
            report.richness
        ));
        let source_delta = &report.source_delta;
        summary.push_str(&format!(
            " Source delta: new {}, changed {}, unchanged {}, deleted {}.",
            source_delta.new_file_count,
            source_delta.changed_file_count,
            source_delta.unchanged_file_count,
            source_delta.deleted_file_count
        ));
        if report.derived_delta.file_count > 0 {
            summary.push_str(&format!(
                " Derived-only changes affected {} source-unchanged file(s), {} chunk(s), and {} symbol(s).",
                report.derived_delta.source_unchanged_file_count,
                report.derived_delta.chunk_count,
                report.derived_delta.symbol_count
            ));
        }
    }
    summary
}

fn explain_loop_report(report: &LoopReport) -> String {
    if report.iterations.is_empty() {
        if let Some(status) = &report.receipt.observation_status {
            return format!(
                "The closed loop stopped immediately for namespace {}. Status: {:?}. Supported source files: {}. Next step: {}",
                status.namespace_queried,
                status.disposition,
                status.supported_file_count,
                status.exact_next_step
            );
        }
        if report.halt_reason == HaltReason::ImportRequired {
            return "The closed loop stopped immediately because source files were found but this namespace has no usable canonical import yet.".into();
        }
        return format!(
            "The closed loop finished immediately. Halt reason: {:?}.",
            report.halt_reason
        );
    }

    if report.halt_reason == HaltReason::AdvisoryOnlyFallback {
        if let Some(status) = &report.receipt.observation_status {
            return format!(
                "The closed loop ran {} advisory bootstrap iteration(s) for namespace {}. Supported source files: {}. Next step: {}",
                report.iterations_completed,
                status.namespace_queried,
                status.supported_file_count,
                status.exact_next_step
            );
        }
    }

    let first = &report.iterations[0];
    let target = first
        .selected_target_key
        .clone()
        .unwrap_or_else(|| "none".into());
    let action = first
        .action_family
        .map(|family| format!("{family:?}"))
        .unwrap_or_else(|| "none".into());
    format!(
        "The closed loop ran {} iteration(s). It investigated {} using {}. Halt reason: {:?}. Actions executed: {}. Imports completed: {}.",
        report.iterations_completed,
        target,
        action,
        report.halt_reason,
        report.actions_executed,
        report.imports_completed
    )
}

fn explain_loop_report_detailed(report: &LoopReport) -> String {
    let mut lines = vec![format!(
        "The loop finished after {} iteration(s). Halt reason: {:?}.",
        report.iterations_completed, report.halt_reason
    )];
    lines.push(format!(
        "It executed {} real action(s), completed {} export(s), and completed {} import(s).",
        report.actions_executed, report.exports_completed, report.imports_completed
    ));
    lines.push(format!(
        "It investigated these targets: {}.",
        if report.targets_investigated.is_empty() {
            "none".into()
        } else {
            report.targets_investigated.join(", ")
        }
    ));
    lines.push(format!(
        "Advisory-only steps were {} and degraded iterations totaled {}.",
        if report.advisory_only_steps {
            "present"
        } else {
            "not needed"
        },
        report.degraded_iterations
    ));

    for (index, iteration) in report.iterations.iter().enumerate() {
        lines.push(String::new());
        lines.push(format!("Iteration {}.", index + 1));
        lines.push(format!(
            "The pilot selected target {}.",
            iteration
                .selected_target_key
                .clone()
                .unwrap_or_else(|| "none".into())
        ));
        lines.push(format!(
            "It used action family {} and outcome {}.",
            iteration
                .action_family
                .map(|family| format!("{family:?}"))
                .unwrap_or_else(|| "none".into()),
            iteration
                .outcome_signature
                .clone()
                .unwrap_or_else(|| "no outcome signature".into())
        ));
        lines.push(format!(
            "This iteration was {} and {}.",
            if iteration.degraded {
                "degraded"
            } else {
                "not degraded"
            },
            if iteration.advisory_only {
                "advisory only"
            } else {
                "allowed to execute normally"
            }
        ));
        if let Some(normalization) = &iteration.target_normalization {
            lines.push(format!(
                "The canonical case class was {:?} with budget class {:?}.",
                normalization.canonical_case_class, normalization.budget_class
            ));
        }
        if let Some(audit) = &iteration.decision_audit {
            lines.push(format!(
                "The cheapest admissible step was {}.",
                audit
                    .cheapest_admissible
                    .map(|step| format!("{step:?}"))
                    .unwrap_or_else(|| "none".into())
            ));
            if !audit.blocked_steps.is_empty() {
                lines.push(format!(
                    "Blocked steps: {}.",
                    audit
                        .blocked_steps
                        .iter()
                        .map(|step| format!("{:?} because {}", step.step_kind, step.reason))
                        .collect::<Vec<_>>()
                        .join("; ")
                ));
            }
        }
        if let Some(stop) = &iteration.stop_rule_evaluation {
            lines.push(format!(
                "Stop-rule view: halt reason {}, retry cap reached {}, cooldown applied {}, damping applied {}.",
                stop.halt_reason,
                stop.retry_cap_reached,
                stop.cooldown_applied,
                stop.damping_applied
            ));
        }
        if let Some(lineage) = &iteration.lineage_receipt {
            lines.push(format!(
                "Lineage: case {}, plan {}, attempt {}.",
                lineage.case_id, lineage.plan_id, lineage.attempt_id
            ));
            if !lineage.degradation_markers.is_empty() {
                lines.push(format!(
                    "Degradation markers: {}.",
                    lineage.degradation_markers.join(", ")
                ));
            }
        }
    }

    lines.join("\n")
}

fn open_resources(
    memory_dir: &Path,
    forge_db: &Path,
    config: &LoopConfig,
) -> Result<LoopRunnerResources, String> {
    let memory_store = open_memory_store(memory_dir)?;
    let forge_store = open_forge_store(forge_db)?;
    LoopRunnerResources::from_memory_store(memory_store, forge_store, config.runtime_config.clone())
        .map_err(|error| error.to_string())
}

fn open_memory_store(base_dir: &Path) -> Result<MemoryStore, String> {
    fs::create_dir_all(base_dir).map_err(|error| {
        format!(
            "memory folder {} could not be created: {}. Next step: point --memory-dir at a writable semantic-memory base directory or repair the parent path.",
            base_dir.display(),
            error
        )
    })?;
    let config = MemoryConfig {
        base_dir: base_dir.to_path_buf(),
        ..Default::default()
    };
    let embedder = Box::new(MockEmbedder::new(config.embedding.dimensions));
    MemoryStore::open_with_embedder(config, embedder).map_err(|error| {
        format!(
            "memory folder {} could not be opened: {}. Next step: point --memory-dir at a valid semantic-memory base directory or repair the existing store.",
            base_dir.display(),
            error
        )
    })
}

fn open_forge_store(path: &Path) -> Result<ForgeStore, String> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent).map_err(|error| error.to_string())?;
    }
    ForgeStore::open(path).map_err(|error| {
        format!(
            "forge db {} could not be opened: {}. Next step: repair or replace the sqlite file, or point --forge-db at a valid Forge database.",
            path.display(),
            error
        )
    })
}

fn build_loop_config(
    scope: &Scope,
    workspace_path: String,
    memory_dir: &Path,
    forge_db: &Path,
    loop_interval_secs: u64,
    max_iterations: u32,
) -> LoopConfig {
    let mut config = LoopConfig::default_for_scope(scope.clone(), workspace_path);
    config.memory_dir = memory_dir.to_string_lossy().to_string();
    config.forge_db_path = forge_db.to_string_lossy().to_string();
    config.max_iterations = max_iterations;
    config.cooldown_secs = loop_interval_secs.min(60);
    config.runtime_config = RuntimeConfig {
        default_scope: scope.clone(),
        ..config.runtime_config.clone()
    };
    config
}

fn normalize_user_path(path: &Path) -> PathBuf {
    let raw = path.to_string_lossy();
    let expanded = if raw == "~" {
        env::var_os("HOME")
            .map(PathBuf::from)
            .unwrap_or_else(|| path.to_path_buf())
    } else if let Some(rest) = raw.strip_prefix("~/") {
        env::var_os("HOME")
            .map(|home| PathBuf::from(home).join(rest))
            .unwrap_or_else(|| path.to_path_buf())
    } else {
        path.to_path_buf()
    };

    if expanded.is_absolute() {
        expanded
    } else {
        env::current_dir()
            .unwrap_or_else(|_| PathBuf::from("."))
            .join(expanded)
    }
}

fn prompt(label: &str) -> Result<String, String> {
    print!("{label}: ");
    io::stdout().flush().map_err(|error| error.to_string())?;
    let mut input = String::new();
    io::stdin()
        .read_line(&mut input)
        .map_err(|error| error.to_string())?;
    Ok(input.trim_end().to_string())
}

fn prompt_default(label: &str, current: impl AsRef<str>) -> Result<String, String> {
    let current = current.as_ref();
    let input = prompt(&format!("{label} [{current}]"))?;
    if input.trim().is_empty() {
        Ok(current.to_string())
    } else {
        Ok(input.trim().to_string())
    }
}

#[derive(Debug, Clone)]
struct LoopStatusSnapshot {
    cycles_completed: u64,
    last_headline: String,
    last_detail: String,
    last_halt_reason: Option<String>,
    last_target: Option<String>,
    last_error: Option<String>,
    recent_events: Vec<String>,
}

impl LoopStatusSnapshot {
    fn starting() -> Self {
        Self {
            cycles_completed: 0,
            last_headline: "The closed loop is starting its first bounded run.".into(),
            last_detail: "No completed loop cycle has been recorded yet.".into(),
            last_halt_reason: None,
            last_target: None,
            last_error: None,
            recent_events: vec!["The closed loop was started.".into()],
        }
    }

    fn record_report(&mut self, report: &LoopReport) {
        self.cycles_completed += 1;
        self.last_headline = explain_loop_report(report);
        self.last_detail = explain_loop_report_detailed(report);
        self.last_halt_reason = Some(format!("{:?}", report.halt_reason));
        self.last_target = report.targets_investigated.last().cloned();
        self.last_error = None;
        self.push_event(format!(
            "Cycle {} finished with halt reason {:?}.",
            self.cycles_completed, report.halt_reason
        ));
    }

    fn push_event(&mut self, event: String) {
        self.recent_events.push(event);
        if self.recent_events.len() > 12 {
            let overflow = self.recent_events.len() - 12;
            self.recent_events.drain(0..overflow);
        }
    }

    fn headline(&self) -> String {
        if let Some(error) = &self.last_error {
            format!("running with last error: {error}")
        } else {
            self.last_headline.clone()
        }
    }

    fn render(&self) -> String {
        let mut lines = vec![
            format!("Cycles completed: {}", self.cycles_completed),
            format!(
                "Latest target: {}",
                self.last_target.clone().unwrap_or_else(|| "none".into())
            ),
            format!(
                "Latest halt reason: {}",
                self.last_halt_reason
                    .clone()
                    .unwrap_or_else(|| "none yet".into())
            ),
        ];
        if let Some(error) = &self.last_error {
            lines.push(format!("Latest error: {error}"));
        }
        lines.push(String::new());
        lines.push("Latest detailed explanation:".into());
        lines.push(self.last_detail.clone());
        if !self.recent_events.is_empty() {
            lines.push(String::new());
            lines.push("Recent events:".into());
            for event in &self.recent_events {
                lines.push(format!("- {event}"));
            }
        }
        lines.join("\n")
    }
}

fn trim_trailing_slash(value: &str) -> &str {
    value.trim_end_matches('/')
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ChatMessage {
    role: String,
    content: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(tag = "provider", rename_all = "snake_case")]
enum ChatProviderConfig {
    Ollama {
        base_url: String,
        model: String,
    },
    OpenAi {
        base_url: String,
        model: String,
        api_key: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct AppState {
    workspace_path: PathBuf,
    namespace: String,
    memory_dir: PathBuf,
    forge_db: PathBuf,
    loop_interval_secs: u64,
    max_iterations: u32,
    provider: Option<ChatProviderConfig>,
}

impl Default for AppState {
    fn default() -> Self {
        let workspace_path = normalize_user_path(Path::new("."));
        let managed_root = managed_storage_root_for_workspace(&workspace_path);
        Self {
            memory_dir: managed_root.join("memory"),
            forge_db: managed_root.join("forge.db"),
            workspace_path,
            namespace: "default".into(),
            loop_interval_secs: 5,
            max_iterations: 4,
            provider: None,
        }
    }
}

impl AppState {
    fn config_path() -> PathBuf {
        let home = env::var("HOME").unwrap_or_else(|_| ".".into());
        PathBuf::from(home)
            .join(".config")
            .join("forge-pilot")
            .join("tui-config.json")
    }

    fn load() -> Result<Self, String> {
        let path = Self::config_path();
        if !path.exists() {
            return Ok(Self::default());
        }
        let contents = fs::read_to_string(&path).map_err(|error| error.to_string())?;
        let mut state =
            serde_json::from_str::<Self>(&contents).map_err(|error| error.to_string())?;
        state.normalize_paths();
        Ok(state)
    }

    fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let body = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(path, body).map_err(|error| error.to_string())
    }

    fn set_workspace_folder(&mut self, folder: PathBuf) {
        let folder = normalize_user_path(&folder);
        let migrate_storage = uses_managed_or_legacy_storage_defaults(
            &self.workspace_path,
            &self.memory_dir,
            &self.forge_db,
        );
        self.workspace_path = folder.clone();
        if migrate_storage {
            let managed_root = managed_storage_root_for_workspace(&folder);
            self.memory_dir = managed_root.join("memory");
            self.forge_db = managed_root.join("forge.db");
        }
    }

    fn normalize_paths(&mut self) {
        self.workspace_path = normalize_user_path(&self.workspace_path);
        self.memory_dir = normalize_user_path(&self.memory_dir);
        self.forge_db = normalize_user_path(&self.forge_db);
    }

    fn prepare_runtime_layout(&mut self) -> Result<(), String> {
        self.normalize_paths();
        if uses_managed_or_legacy_storage_defaults(
            &self.workspace_path,
            &self.memory_dir,
            &self.forge_db,
        ) {
            let managed_root = managed_storage_root_for_workspace(&self.workspace_path);
            self.memory_dir = managed_root.join("memory");
            self.forge_db = managed_root.join("forge.db");
        }
        fs::create_dir_all(&self.memory_dir).map_err(|error| {
            format!(
                "failed to prepare memory folder {}: {}",
                self.memory_dir.display(),
                error
            )
        })?;
        if let Some(parent) = self.forge_db.parent() {
            fs::create_dir_all(parent).map_err(|error| {
                format!(
                    "failed to prepare forge db parent {}: {}",
                    parent.display(),
                    error
                )
            })?;
        }
        Ok(())
    }

    fn to_loop_config(&self) -> LoopConfig {
        let scope = Scope::new(self.namespace.clone());
        build_loop_config(
            &scope,
            self.workspace_path.to_string_lossy().to_string(),
            &self.memory_dir,
            &self.forge_db,
            self.loop_interval_secs,
            self.max_iterations,
        )
    }

    fn plain_english_status(&self) -> String {
        format!(
            "Workspace: {}\nNamespace: {}\nMemory folder: {}\nForge DB: {}\nProvider: {}",
            self.workspace_path.display(),
            self.namespace,
            self.memory_dir.display(),
            self.forge_db.display(),
            self.provider_name()
        )
    }

    fn provider_name(&self) -> &'static str {
        match self.provider {
            Some(ChatProviderConfig::Ollama { .. }) => "Ollama",
            Some(ChatProviderConfig::OpenAi { .. }) => "OpenAI",
            None => "not configured",
        }
    }

    fn ollama_base_url(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::Ollama { base_url, .. }) => base_url.clone(),
            _ => "http://localhost:11434".into(),
        }
    }

    fn ollama_model(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::Ollama { model, .. }) => model.clone(),
            _ => "llama3.1".into(),
        }
    }

    fn openai_base_url(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { base_url, .. }) => base_url.clone(),
            _ => "https://api.openai.com".into(),
        }
    }

    fn openai_model(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { model, .. }) => model.clone(),
            _ => "gpt-4.1-mini".into(),
        }
    }

    fn openai_api_key(&self) -> Option<String> {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { api_key, .. }) => Some(api_key.clone()),
            _ => None,
        }
    }
}

fn managed_storage_root_for_workspace(workspace_path: &Path) -> PathBuf {
    detect_project_root(workspace_path).join(".forge-pilot")
}

fn uses_managed_or_legacy_storage_defaults(
    workspace_path: &Path,
    memory_dir: &Path,
    forge_db: &Path,
) -> bool {
    let legacy_memory = workspace_path.join("memory");
    let legacy_forge_db = workspace_path.join("forge.db");
    let managed_root = managed_storage_root_for_workspace(workspace_path);
    let managed_memory = managed_root.join("memory");
    let managed_forge_db = managed_root.join("forge.db");

    (memory_dir == legacy_memory && forge_db == legacy_forge_db)
        || (memory_dir == managed_memory && forge_db == managed_forge_db)
}

fn detect_project_root(workspace_path: &Path) -> PathBuf {
    let mut current = if workspace_path.is_file() {
        workspace_path
            .parent()
            .map(Path::to_path_buf)
            .unwrap_or_else(|| workspace_path.to_path_buf())
    } else {
        workspace_path.to_path_buf()
    };
    current = normalize_user_path(&current);

    let mut candidate = Some(current.as_path());
    while let Some(path) = candidate {
        if looks_like_project_root(path) {
            return path.to_path_buf();
        }
        candidate = path.parent();
    }

    current
}

fn looks_like_project_root(path: &Path) -> bool {
    [
        "Cargo.toml",
        "package.json",
        "pnpm-workspace.yaml",
        "pyproject.toml",
        ".git",
    ]
    .iter()
    .any(|marker| path.join(marker).exists())
}

struct CommandArgs {
    command: String,
    namespace: String,
    memory_dir: PathBuf,
    forge_db: PathBuf,
    workspace_path: String,
    loop_interval_secs: u64,
    max_iterations: u32,
    json: bool,
    help: bool,
}

impl CommandArgs {
    fn parse(argv: Vec<String>) -> Result<Self, String> {
        let mut args = Self::default();
        let mut iter = argv.into_iter();
        while let Some(arg) = iter.next() {
            match arg.as_str() {
                "-h" | "--help" => args.help = true,
                "--json" => args.json = true,
                "--namespace" => {
                    args.namespace = iter
                        .next()
                        .ok_or_else(|| "--namespace requires a value".to_string())?;
                }
                "--memory-dir" => {
                    args.memory_dir = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--memory-dir requires a value".to_string())?,
                    );
                }
                "--forge-db" => {
                    args.forge_db = PathBuf::from(
                        iter.next()
                            .ok_or_else(|| "--forge-db requires a value".to_string())?,
                    );
                }
                "--workspace-path" => {
                    args.workspace_path = iter
                        .next()
                        .ok_or_else(|| "--workspace-path requires a value".to_string())?;
                }
                "--workspace" => {
                    args.workspace_path = iter
                        .next()
                        .ok_or_else(|| "--workspace requires a value".to_string())?;
                }
                "--loop-interval" => {
                    args.loop_interval_secs = iter
                        .next()
                        .ok_or_else(|| "--loop-interval requires a value".to_string())?
                        .parse()
                        .map_err(|_| "--loop-interval must be an integer".to_string())?;
                }
                "--max-iterations" => {
                    args.max_iterations = iter
                        .next()
                        .ok_or_else(|| "--max-iterations requires a value".to_string())?
                        .parse()
                        .map_err(|_| "--max-iterations must be an integer".to_string())?;
                }
                value if value.starts_with('-') => {
                    return Err(format!("unknown flag `{value}`\n\n{}", Self::help()));
                }
                value => args.command = value.into(),
            }
        }
        args.memory_dir = normalize_user_path(&args.memory_dir);
        args.forge_db = normalize_user_path(&args.forge_db);
        args.workspace_path = normalize_user_path(Path::new(&args.workspace_path))
            .to_string_lossy()
            .to_string();
        Ok(args)
    }

    fn help() -> &'static str {
        "forge-pilot\n\nUSAGE:\n  cargo run -p forge-pilot --                 # interactive plain-English wrapper\n  cargo run -p forge-pilot -- tui             # same interactive wrapper\n  cargo run -p forge-pilot -- <command> [options]\n\nCOMMANDS:\n  observe           Reconstruct and print the current observation\n  candidates        Score and print current target candidates\n  bootstrap-source  Scan the workspace and bootstrap thin imported source state\n  import            Import recent Forge evidence bundles through the canonical export -> bridge -> memory lane\n  run               Execute one bounded pilot loop\n\nOPTIONS:\n  --namespace <name>        Namespace to inspect (default: default)\n  --memory-dir <path>       Semantic memory base dir (default: ./memory)\n  --forge-db <path>         Forge DB path (default: ./forge.db)\n  --workspace-path <path>   Workspace path recorded in reports (default: .)\n  --workspace <path>        Alias for --workspace-path\n  --loop-interval <secs>    Background loop pause for the TUI worker (default: 5)\n  --max-iterations <n>      Max iterations per bounded run (default: 4)\n  --json                    Print raw JSON for command mode\n  -h, --help                Show this help\n"
    }
}

impl Default for CommandArgs {
    fn default() -> Self {
        Self {
            command: "run".into(),
            namespace: "default".into(),
            memory_dir: PathBuf::from("memory"),
            forge_db: PathBuf::from("forge.db"),
            workspace_path: ".".into(),
            loop_interval_secs: 5,
            max_iterations: 4,
            json: false,
            help: false,
        }
    }
}

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
