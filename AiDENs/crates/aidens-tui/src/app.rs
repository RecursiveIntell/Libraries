//! App struct and main event loop for the TUI.

use aidens_autonomous::LoopState;
use aidens_daemon_kit::DaemonControllerV1;
use aidens_queue_kit::QueueSnapshotV1;
use anyhow::Result;
use crossterm::event::{Event, KeyCode};
use ratatui::{backend::CrosstermBackend, layout::Rect, Terminal};
use serde::Deserialize;
use std::collections::VecDeque;
use std::io::Stdout;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::time::Duration;

use crate::ui;

/// Memory stats polled from the semantic-memory HTTP server.
#[derive(Debug, Clone, Default, Deserialize)]
pub struct MemoryStats {
    pub facts: usize,
    pub documents: usize,
    pub chunks: usize,
    pub graph_edges: Option<usize>,
    pub db_size_mb: f64,
}

/// The main TUI application state.
pub struct App {
    /// Shared loop state from the autonomous loop.
    pub loop_state: Arc<Mutex<LoopState>>,
    /// Latest memory stats (from HTTP polling).
    pub memory_stats: MemoryStats,
    /// Latest queue snapshot (from read-only daemon controller).
    pub queue_snapshot: Option<QueueSnapshotV1>,
    /// Activity log entries (max 100).
    pub logs: VecDeque<String>,
    /// Whether polling is paused.
    pub paused: bool,
    /// Whether the app should quit.
    pub should_quit: bool,
    /// Semantic-memory HTTP base URL.
    pub http_base_url: String,
    /// Queue root directory for read-only daemon access.
    pub queue_root: PathBuf,
    /// HTTP client for polling memory stats.
    http_client: reqwest::Client,
}

impl App {
    /// Create a new App instance.
    pub fn new(loop_state: Arc<Mutex<LoopState>>, http_base_url: String, queue_root: PathBuf) -> Self {
        Self {
            loop_state,
            memory_stats: MemoryStats::default(),
            queue_snapshot: None,
            logs: VecDeque::with_capacity(100),
            paused: false,
            should_quit: false,
            http_base_url,
            queue_root,
            http_client: reqwest::Client::new(),
        }
    }

    /// Push a log entry, maintaining a max of 100 entries.
    fn log(&mut self, msg: impl Into<String>) {
        if self.logs.len() >= 100 {
            self.logs.pop_front();
        }
        self.logs.push_back(msg.into());
    }

    /// Main event loop at ~10fps (100ms tick).
    ///
    /// Uses crossterm's async EventStream with tokio::select! so the loop
    /// task runs concurrently with keyboard polling -- no blocking.
    pub async fn run(
        &mut self,
        terminal: &mut Terminal<CrosstermBackend<Stdout>>,
    ) -> Result<()> {
        use crossterm::event::EventStream;
        use futures::StreamExt;

        self.log("TUI started — press 'q' to quit, 'p' to pause, 's' for safe mode");

        let mut event_stream = EventStream::new();
        let mut render_interval = tokio::time::interval(Duration::from_millis(250));

        loop {
            // Render the current state.
            terminal.draw(|frame| {
                let area = frame.area();
                ui::render(frame, area, self);
            })?;

            // Use select! to poll keyboard events AND yield to the runtime
            // concurrently. The loop task (on the LocalSet) gets polled
            // during every await point, so it runs freely.
            tokio::select! {
                // Keyboard event (async, non-blocking)
                maybe_event = event_stream.next() => {
                    if let Some(Ok(Event::Key(key))) = maybe_event {
                        match key.code {
                            KeyCode::Char('q') => {
                                self.should_quit = true;
                                self.log("Quit requested");
                            }
                            KeyCode::Char('p') => {
                                self.paused = !self.paused;
                                if self.paused {
                                    self.log("Polling paused");
                                } else {
                                    self.log("Polling resumed");
                                }
                            }
                            KeyCode::Char('s') => {
                                self.log("Attempting to set safe mode on queue...");
                                if let Err(e) = self.set_queue_safe_mode() {
                                    self.log(format!("Safe mode error: {e}"));
                                } else {
                                    self.log("Safe mode set on queue");
                                }
                            }
                            _ => {}
                        }
                    }
                }
                // Render/poll tick every 250ms
                _ = render_interval.tick() => {}
            }

            if self.should_quit {
                break;
            }

            // If paused, skip polling — just re-render last state.
            if !self.paused {
                // Poll memory stats from HTTP.
                if let Err(e) = self.poll_memory_stats().await {
                    // Don't spam the log on every failed poll.
                    // Only log if we had stats before (transition to error).
                    if self.memory_stats.facts > 0 {
                        self.log(format!("Stats poll error: {e}"));
                    }
                }

                // Poll queue snapshot (read-only).
                if let Err(e) = self.poll_queue() {
                    self.log(format!("Queue poll error: {e}"));
                }
            }
        }

        Ok(())
    }

    /// Poll memory stats from the semantic-memory HTTP server.
    async fn poll_memory_stats(&mut self) -> Result<()> {
        let url = format!("{}/stats", self.http_base_url);
        let response = self
            .http_client
            .post(&url)
            .header("Content-Type", "application/json")
            .body("{}")
            .send()
            .await?;

        if !response.status().is_success() {
            anyhow::bail!("HTTP {}", response.status());
        }

        let stats: MemoryStats = response.json().await?;
        self.memory_stats = stats;
        Ok(())
    }

    /// Poll the queue snapshot using a read-only daemon controller.
    fn poll_queue(&mut self) -> Result<()> {
        let namespace = DaemonControllerV1::namespace(
            &self.queue_root,
            "autonomous-loop",
            "aidens-autonomous",
        );
        let controller = DaemonControllerV1::open_read_only(&self.queue_root, namespace)?;
        let snapshot = controller.snapshot()?;
        self.queue_snapshot = Some(snapshot);
        Ok(())
    }

    /// Set safe mode on the queue (requires a writable open).
    fn set_queue_safe_mode(&mut self) -> Result<()> {
        let namespace = DaemonControllerV1::namespace(
            &self.queue_root,
            "autonomous-loop",
            "aidens-autonomous",
        );
        let controller = DaemonControllerV1::open(&self.queue_root, namespace, "aidens-tui")?;
        controller.set_safe_mode(true, "tui-manual-override")?;
        Ok(())
    }

    /// Get a snapshot of the loop state for rendering.
    pub(crate) fn loop_state_snapshot(&self) -> LoopState {
        self.loop_state
            .lock()
            .map(|s| s.clone())
            .unwrap_or_default()
    }

    /// Get a reference to the logs for rendering.
    pub(crate) fn logs_ref(&self) -> &VecDeque<String> {
        &self.logs
    }

    /// Get a reference to memory stats for rendering.
    pub(crate) fn memory_stats_ref(&self) -> &MemoryStats {
        &self.memory_stats
    }

    /// Get a reference to the queue snapshot for rendering.
    pub(crate) fn queue_snapshot_ref(&self) -> Option<&QueueSnapshotV1> {
        self.queue_snapshot.as_ref()
    }

    /// Whether the app is paused.
    pub(crate) fn is_paused(&self) -> bool {
        self.paused
    }

    /// Get the HTTP base URL as a string reference (for rendering).
    pub(crate) fn http_base_url_ref(&self) -> &str {
        &self.http_base_url
    }

    /// Get the queue root as a display string (for rendering).
    pub(crate) fn queue_root_display(&self) -> String {
        self.queue_root.display().to_string()
    }

    /// Get the terminal area split into panel regions.
    /// Returns (top_row_rects, bottom_rect) where top_row_rects has 3 columns.
    pub(crate) fn split_layout(area: Rect) -> ([Rect; 3], Rect) {
        use ratatui::layout::{Constraint, Direction, Layout};

        // Split into top (60%) and bottom (40%) — the bottom is the log.
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Percentage(55), Constraint::Percentage(45)])
            .split(area);

        let top_row = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage(33),
                Constraint::Percentage(34),
                Constraint::Percentage(33),
            ])
            .split(chunks[0]);

        ([top_row[0], top_row[1], top_row[2]], chunks[1])
    }
}