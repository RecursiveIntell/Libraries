//! Memory stats panel — renders HTTP-polled memory statistics.

use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the memory stats panel.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let stats = app.memory_stats_ref();

    let block = Block::default()
        .borders(Borders::ALL)
        .title(" Memory Stats ");

    let lines = vec![
        Line::from(vec![Span::styled(
            "Facts",
            Style::default().fg(Color::Cyan),
        )]),
        Line::from(format!("  Facts:     {}", stats.facts)),
        Line::from(format!("  Documents: {}", stats.documents)),
        Line::from(format!("  Chunks:    {}", stats.chunks)),
        Line::from(format!(
            "  Edges:     {}",
            stats
                .graph_edges
                .map(|e| e.to_string())
                .unwrap_or_else(|| "—".to_string())
        )),
        Line::from(format!("  DB Size:   {:.2} MB", stats.db_size_mb)),
        Line::from(""),
        Line::from(Span::styled(
            format!("HTTP: {}", app.http_base_url_ref()),
            Style::default().fg(Color::DarkGray),
        )),
    ];

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
