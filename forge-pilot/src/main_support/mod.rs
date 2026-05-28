// LIB-MED-001: Decomposed into focused modules.
mod cli;
mod explain;
mod provider;
mod storage;
mod tui;

use std::env;
use std::io::{self, Write};

pub async fn run() -> Result<(), String> {
    let raw_args = env::args().skip(1).collect::<Vec<_>>();
    if raw_args.is_empty() {
        return tui::run_tui().await;
    }

    if raw_args.len() == 1 && matches!(raw_args[0].as_str(), "-h" | "--help") {
        print!("{}", cli::CommandArgs::help());
        return Ok(());
    }

    if matches!(
        raw_args.first().map(|s| s.as_str()),
        Some("tui" | "interactive")
    ) {
        return tui::run_tui().await;
    }

    cli::run_command_cli(raw_args).await
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

#[cfg(test)]
#[path = "tests.rs"]
mod tests;
