//! Activity log panel — renders scrollable log entries with an input line.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the activity log panel.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Activity Log ");

    let gray = Style::default().fg(Color::DarkGray);
    let green = Style::default().fg(Color::Green);

    let mut lines: Vec<Line> = Vec::new();

    // Render log entries (newest at bottom, which is natural for a log).
    let logs = app.logs_ref();
    for entry in logs.iter() {
        // Color-code: errors in red, safe mode in yellow, normal in white.
        let style = if entry.contains("error") || entry.contains("Error") {
            Style::default().fg(Color::Red)
        } else if entry.contains("safe mode") || entry.contains("Safe") {
            Style::default().fg(Color::Yellow)
        } else if entry.contains("paused") || entry.contains("resumed") {
            Style::default().fg(Color::Cyan)
        } else {
            Style::default()
        };
        lines.push(Line::from(Span::styled(entry.clone(), style)));
    }

    // Add a separator and input line at the bottom.
    lines.push(Line::from(Span::styled("─".repeat(60), gray)));
    let paused_indicator = if app.is_paused() {
        Span::styled(" [PAUSED] ", Style::default().fg(Color::Yellow))
    } else {
        Span::raw("")
    };
    lines.push(Line::from(vec![
        Span::styled("> ", green),
        paused_indicator,
        Span::styled("(q=quit, p=pause, s=safe mode)", gray),
    ]));

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
