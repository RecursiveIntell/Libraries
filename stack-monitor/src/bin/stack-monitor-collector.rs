//! Launch-managed local collector daemon for stack-monitor.

use stack_monitor::{start_unix_collector_with_live, start_unix_live_server, LiveHub};
use std::env;
use std::error::Error;
use std::fs;
use std::os::unix::fs::PermissionsExt;
use std::path::{Path, PathBuf};
use tokio::signal::unix::{signal, SignalKind};

fn option(args: &[String], name: &str) -> Option<String> {
    args.windows(2)
        .find(|pair| pair[0] == name)
        .map(|pair| pair[1].clone())
}

fn private_dir(path: &Path) -> Result<(), Box<dyn Error>> {
    if !path.exists() {
        fs::create_dir_all(path)?;
        fs::set_permissions(path, fs::Permissions::from_mode(0o700))?;
    }
    Ok(())
}

fn default_socket() -> Result<PathBuf, Box<dyn Error>> {
    let runtime = env::var_os("XDG_RUNTIME_DIR")
        .ok_or("XDG_RUNTIME_DIR is required for the default collector socket")?;
    Ok(PathBuf::from(runtime)
        .join("ares-observatory")
        .join("collector.sock"))
}

fn default_database() -> Result<PathBuf, Box<dyn Error>> {
    let state = env::var_os("XDG_STATE_HOME")
        .map(PathBuf::from)
        .or_else(|| env::var_os("HOME").map(|home| PathBuf::from(home).join(".local/state")))
        .ok_or("XDG_STATE_HOME or HOME is required for the default database path")?;
    Ok(state.join("ares-observatory").join("activity.db"))
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args: Vec<String> = env::args().skip(1).collect();
    let socket = option(&args, "--socket")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(default_socket)?;
    let live_socket = option(&args, "--live-socket")
        .map(PathBuf::from)
        .unwrap_or_else(|| socket.with_file_name("live.sock"));
    let database = option(&args, "--database")
        .map(PathBuf::from)
        .map(Ok)
        .unwrap_or_else(default_database)?;
    let capacity = option(&args, "--capacity")
        .map(|value| value.parse::<usize>())
        .transpose()?
        .unwrap_or(4096);

    if let Some(parent) = socket.parent() {
        private_dir(parent)?;
    }
    if let Some(parent) = live_socket.parent() {
        private_dir(parent)?;
    }
    if let Some(parent) = database.parent() {
        private_dir(parent)?;
    }

    let store = stack_monitor::ActivityStore::open(&database)?;
    let live = std::sync::Arc::new(LiveHub::new(capacity));
    let collector = start_unix_collector_with_live(
        &socket,
        store,
        capacity,
        Some(std::sync::Arc::clone(&live)),
    )?;
    let live_server = start_unix_live_server(&live_socket, live)?;
    eprintln!(
        "stack-monitor-collector listening at {} (live {}) with database {}",
        socket.display(),
        live_socket.display(),
        database.display()
    );

    let mut sigint = signal(SignalKind::interrupt())?;
    let mut sigterm = signal(SignalKind::terminate())?;
    tokio::select! {
        _ = sigint.recv() => {},
        _ = sigterm.recv() => {},
    }

    live_server.shutdown();
    let stats = collector.shutdown();
    eprintln!(
        "stack-monitor-collector stopped: persisted={}, dropped={}, storage_failures={}",
        stats.persisted, stats.dropped, stats.storage_failures
    );
    Ok(())
}
