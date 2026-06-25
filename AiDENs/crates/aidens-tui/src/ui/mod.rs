//! UI rendering coordinator.
//!
//! Splits the terminal into 4 panels:
//! - Top row: memory stats | loop state | queue (3 columns)
//! - Bottom: activity log (full width)

pub mod log_panel;
pub mod loop_panel;
pub mod queue_panel;
pub mod stats_panel;

use ratatui::layout::Rect;
use ratatui::Frame;

use crate::app::App;

/// Render the full TUI layout.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let (top, bottom) = App::split_layout(area);

    stats_panel::render(frame, top[0], app);
    loop_panel::render(frame, top[1], app);
    queue_panel::render(frame, top[2], app);
    log_panel::render(frame, bottom, app);
}