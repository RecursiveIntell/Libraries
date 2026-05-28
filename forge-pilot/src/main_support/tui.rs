use forge_pilot::{
    bootstrap_source_workspace, import_recent_forge_bundles, observe_scope, score_targets,
    ExternalHaltFlag, LoopReport, LoopRunner, PilotHistory,
};
use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{
    atomic::{AtomicBool, Ordering},
    Arc, Mutex,
};
use std::thread;

use super::explain::{
    explain_bootstrap_report, explain_candidates, explain_import_report, explain_loop_report,
    explain_loop_report_detailed, explain_observation,
};
use super::prompt;
use super::provider::{chat_loop, configure_provider, ChatProviderConfig};
use super::storage::{
    build_loop_config, managed_storage_root_for_workspace, normalize_user_path, open_resources,
    uses_managed_or_legacy_storage_defaults,
};

pub(super) async fn run_tui() -> Result<(), String> {
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

// ── Types ──────────────────────────────────────────────────────

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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub(super) struct AppState {
    pub workspace_path: PathBuf,
    pub namespace: String,
    pub memory_dir: PathBuf,
    pub forge_db: PathBuf,
    pub loop_interval_secs: u64,
    pub max_iterations: u32,
    pub provider: Option<ChatProviderConfig>,
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

    pub(super) fn load() -> Result<Self, String> {
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

    pub(super) fn save(&self) -> Result<(), String> {
        let path = Self::config_path();
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|error| error.to_string())?;
        }
        let body = serde_json::to_string_pretty(self).map_err(|error| error.to_string())?;
        fs::write(path, body).map_err(|error| error.to_string())
    }

    pub(super) fn set_workspace_folder(&mut self, folder: PathBuf) {
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

    pub(super) fn prepare_runtime_layout(&mut self) -> Result<(), String> {
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

    pub(super) fn to_loop_config(&self) -> forge_pilot::LoopConfig {
        let scope = knowledge_runtime::Scope::new(self.namespace.clone());
        build_loop_config(
            &scope,
            self.workspace_path.to_string_lossy().to_string(),
            &self.memory_dir,
            &self.forge_db,
            self.loop_interval_secs,
            self.max_iterations,
        )
    }

    pub(super) fn plain_english_status(&self) -> String {
        format!(
            "Workspace: {}\nNamespace: {}\nMemory folder: {}\nForge DB: {}\nProvider: {}",
            self.workspace_path.display(),
            self.namespace,
            self.memory_dir.display(),
            self.forge_db.display(),
            self.provider_name()
        )
    }

    pub(super) fn provider_name(&self) -> &'static str {
        match self.provider {
            Some(ChatProviderConfig::Ollama { .. }) => "Ollama",
            Some(ChatProviderConfig::OpenAi { .. }) => "OpenAI",
            None => "not configured",
        }
    }

    pub(super) fn ollama_base_url(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::Ollama { base_url, .. }) => base_url.clone(),
            _ => "http://localhost:11434".into(),
        }
    }

    pub(super) fn ollama_model(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::Ollama { model, .. }) => model.clone(),
            _ => "llama3.1".into(),
        }
    }

    pub(super) fn openai_base_url(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { base_url, .. }) => base_url.clone(),
            _ => "https://api.openai.com".into(),
        }
    }

    pub(super) fn openai_model(&self) -> String {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { model, .. }) => model.clone(),
            _ => "gpt-4.1-mini".into(),
        }
    }

    pub(super) fn openai_api_key(&self) -> Option<String> {
        match &self.provider {
            Some(ChatProviderConfig::OpenAi { api_key, .. }) => Some(api_key.clone()),
            _ => None,
        }
    }
}
