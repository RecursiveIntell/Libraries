//! Terminal UI for observing the AiDENs autonomous loop.
//!
//! This crate provides a ratatui-based TUI that renders four panels:
//! - Memory stats (HTTP polled from semantic-memory)
//! - Loop state (shared `Arc<Mutex<LoopState>>`)
//! - Queue snapshot (read-only `DaemonControllerV1`)
//! - Activity log (scrollable)
//!
//! Keyboard controls: `q` = quit, `p` = pause polling, `s` = set safe mode.

pub mod app;
pub mod ui;

pub use app::{App, MemoryStats};