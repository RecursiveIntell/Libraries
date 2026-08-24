//! Ares Observatory Tauri v2 read-side shell.

mod commands;

use commands::{coverage, export_observations, health, timeline};
use stack_monitor::{ActivityStore, LiveHub, ProjectionService, TransportStats};
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use tauri::Emitter;

/// Application state owned by the read-side shell.
#[derive(Clone)]
pub struct AppState {
    pub(crate) projections: Arc<ProjectionService>,
    pub(crate) stats: Arc<Mutex<TransportStats>>,
    pub(crate) live_cursor: Arc<AtomicU64>,
}

/// Run the read-side desktop shell against a local observation database.
pub fn run(database: PathBuf) -> Result<(), Box<dyn std::error::Error>> {
    let store = ActivityStore::open(database)?;
    let live = Arc::new(LiveHub::new(1024));
    let state = AppState {
        projections: Arc::new(ProjectionService::new(store, live)),
        stats: Arc::new(Mutex::new(TransportStats::default())),
        live_cursor: Arc::new(AtomicU64::new(0)),
    };
    let live_stats = Arc::clone(&state.stats);
    let live_cursor = Arc::clone(&state.live_cursor);

    tauri::Builder::default()
        .manage(state)
        .invoke_handler(tauri::generate_handler![
            timeline,
            health,
            coverage,
            export_observations
        ])
        .setup(move |app| {
            #[cfg(unix)]
            attach_live_events(app.handle().clone(), live_stats, live_cursor);
            Ok(())
        })
        .run(tauri::generate_context!())?;
    Ok(())
}

#[cfg(unix)]
fn attach_live_events(
    app: tauri::AppHandle,
    stats: Arc<Mutex<TransportStats>>,
    live_cursor: Arc<AtomicU64>,
) {
    let path = std::env::var_os("ARES_OBSERVATORY_LIVE_SOCKET")
        .map(PathBuf::from)
        .or_else(|| {
            std::env::var_os("XDG_RUNTIME_DIR").map(|runtime| {
                PathBuf::from(runtime)
                    .join("ares-observatory")
                    .join("live.sock")
            })
        });
    let Some(path) = path else {
        eprintln!("Ares Observatory: live socket path unavailable");
        return;
    };
    let Ok((subscription, client)) = stack_monitor::start_unix_live_client(path, 256) else {
        eprintln!("Ares Observatory: live socket client could not start");
        return;
    };
    std::thread::spawn(move || {
        loop {
            match subscription.recv_timeout(Duration::from_millis(250)) {
                Ok(event) => {
                    live_cursor.store(event.cursor, Ordering::Release);
                    if let Ok(mut current) = stats.lock() {
                        current.attempted = current.attempted.saturating_add(1);
                        current.accepted = current.accepted.saturating_add(1);
                        current.persisted = current.persisted.saturating_add(1);
                    }
                    if app.emit("observation-live", event).is_err() {
                        break;
                    }
                }
                Err(std::sync::mpsc::RecvTimeoutError::Timeout) => {}
                Err(std::sync::mpsc::RecvTimeoutError::Disconnected) => break,
            }
        }
        client.shutdown();
    });
}
