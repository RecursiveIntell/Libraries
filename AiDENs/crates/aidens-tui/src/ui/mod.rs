//! UI rendering coordinator.
//!
//! Splits the terminal into 4 panels:
//! - Top row: memory stats | loop state | queue (3 columns)
//! - Bottom: activity log (full width)

pub mod log_panel;
pub mod loop_panel;
pub mod queue_panel;
pub mod stats_panel;

pub(crate) fn truncate_for_panel(value: &str, limit: usize) -> String {
    let mut chars = value.chars();
    let prefix: String = chars.by_ref().take(limit).collect();
    if chars.next().is_some() {
        format!("{prefix}…")
    } else {
        prefix
    }
}

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

#[cfg(test)]
mod tests {
    use super::truncate_for_panel;

    #[test]
    fn truncation_respects_unicode_scalar_boundaries() {
        assert_eq!(
            truncate_for_panel("1234567890123456789💥tail", 20),
            "1234567890123456789💥…"
        );
        assert_eq!(truncate_for_panel("short", 20), "short");
    }
}
