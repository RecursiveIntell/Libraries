//! Binary entry point for the AiDENs TUI.
//!
//! Usage:
//!   aidens-tui [--memory-dir <path>] [--queue-dir <path>] [--model-url <url>]
//!              [--chosen-model <model>] [--http-base-url <url>]
//!
//! Defaults:
//!   --memory-dir    ~/.hermes/semantic-memory.db
//!   --queue-dir     /tmp/aidens-queue
//!   --model-url    http://127.0.0.1:11434
//!   --chosen-model deepseek-v4-flash
//!   --http-base-url http://127.0.0.1:1738

use aidens_autonomous::{AutonomousLoop, LoopConfig};
use aidens_tui::app::App;
use anyhow::Result;
use crossterm::{
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use ratatui::{backend::CrosstermBackend, Terminal};
use std::io::Stdout;
use std::path::PathBuf;

/// Parse simple `--key value` style CLI args.
fn parse_args() -> LoopConfig {
    let args: Vec<String> = std::env::args().collect();
    let mut config = LoopConfig::default();

    let mut i = 1;
    while i < args.len() {
        let key = args[i].as_str();
        let value = args.get(i + 1).cloned();

        let consumed = match (key, value) {
            ("--memory-dir", Some(v)) => {
                config.memory_dir = PathBuf::from(&v);
                2
            }
            ("--queue-dir", Some(v)) => {
                config.queue_dir = PathBuf::from(&v);
                2
            }
            ("--model-url", Some(v)) => {
                config.model_url = v;
                2
            }
            ("--chosen-model", Some(v)) => {
                config.chosen_model = v;
                2
            }
            ("--http-base-url", Some(v)) => {
                config.http_base_url = v;
                2
            }
            _ => 1,
        };
        i += consumed;
    }

    config
}

fn setup_terminal() -> Result<Terminal<CrosstermBackend<Stdout>>> {
    enable_raw_mode()?;
    execute!(std::io::stdout(), EnterAlternateScreen)?;
    let backend = CrosstermBackend::new(std::io::stdout());
    let terminal = Terminal::new(backend)?;
    Ok(terminal)
}

fn restore_terminal() -> Result<()> {
    disable_raw_mode()?;
    execute!(std::io::stdout(), LeaveAlternateScreen)?;
    Ok(())
}

#[tokio::main]
async fn main() -> Result<()> {
    let config = parse_args();

    // Build the autonomous loop from config.
    let loop_instance = AutonomousLoop::from_config(config.clone())?;

    // Get the shared state handle.
    let loop_state = loop_instance.state.clone();

    // The queue root and HTTP base URL for the TUI's own polling.
    let queue_root = config.queue_dir.clone();
    let http_base_url = config.http_base_url.clone();

    // Spawn the loop in a local task (non-Send futures need LocalSet).
    // Wrap in error handling so a panic in the loop doesn't kill the TUI.
    let local_set = tokio::task::LocalSet::new();
    let loop_state_clone = loop_state.clone();
    let loop_handle = local_set.spawn_local(async move {
        // Run the loop. If it errors or panics, log to shared state
        // and let the TUI stay alive so the user can see what happened.
        let result = loop_instance.run().await;
        if let Err(e) = result {
            if let Ok(mut s) = loop_state_clone.lock() {
                s.last_error = Some(format!("loop exited: {e}"));
            }
        }
    });

    // Setup terminal.
    let mut terminal = setup_terminal()?;

    // Build and run the app INSIDE the LocalSet so both the loop task
    // and the TUI event loop share the same executor. Without this,
    // the LocalSet never gets polled and the loop never runs.
    let app_result = local_set
        .run_until(async {
            let mut app = App::new(loop_state, http_base_url, queue_root);
            app.run(&mut terminal).await
        })
        .await;

    // Cleanup terminal regardless of app result.
    let _ = restore_terminal();

    // Abort the loop task.
    loop_handle.abort();

    app_result
}
