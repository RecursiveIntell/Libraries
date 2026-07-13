//! Queue panel — renders the daemon queue snapshot.

use aidens_contracts::JobStateV1;
use ratatui::layout::Rect;
use ratatui::style::{Color, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::{app::App, ui::truncate_for_panel};

/// Render the queue panel.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let block = Block::default().borders(Borders::ALL).title(" Queue ");

    let gray = Style::default().fg(Color::DarkGray);
    let green = Style::default().fg(Color::Green);
    let red = Style::default().fg(Color::Red);
    let yellow = Style::default().fg(Color::Yellow);

    let mut lines = Vec::new();

    match app.queue_snapshot_ref() {
        Some(snapshot) => {
            let active = snapshot
                .jobs
                .iter()
                .filter(|j| !j.state.is_terminal())
                .count();
            let queued = snapshot
                .jobs
                .iter()
                .filter(|j| j.state == JobStateV1::Queued)
                .count();
            let total = snapshot.jobs.len();

            lines.push(Line::from(vec![
                Span::raw("  Active:  "),
                Span::styled(active.to_string(), if active > 0 { yellow } else { gray }),
            ]));
            lines.push(Line::from(vec![
                Span::raw("  Queued:  "),
                Span::styled(queued.to_string(), Style::default().fg(Color::Cyan)),
            ]));
            lines.push(Line::from(format!("  Total:   {}", total)));
            lines.push(Line::from(vec![
                Span::raw("  Safe:    "),
                Span::styled(
                    if snapshot.safe_mode_enabled {
                        "ON"
                    } else {
                        "off"
                    },
                    if snapshot.safe_mode_enabled {
                        red
                    } else {
                        gray
                    },
                ),
            ]));
            lines.push(Line::from(""));

            // List up to 10 most recent jobs.
            let display_count = snapshot.jobs.len().min(10);
            if display_count > 0 {
                lines.push(Line::from(Span::styled("  Jobs:", gray)));
                for job in snapshot.jobs.iter().rev().take(display_count) {
                    let state_color = match job.state {
                        JobStateV1::Completed => green,
                        JobStateV1::Cancelled | JobStateV1::Poisoned => red,
                        JobStateV1::Running | JobStateV1::Leased => yellow,
                        _ => gray,
                    };
                    let job_display = truncate_for_panel(&job.idempotency_key, 20);
                    lines.push(Line::from(vec![
                        Span::raw("  "),
                        Span::styled(format!("{:?}", job.state), state_color),
                        Span::raw(" "),
                        Span::raw(job_display),
                    ]));
                }
            } else {
                lines.push(Line::from(Span::styled("  (empty)", gray)));
            }
        }
        None => {
            lines.push(Line::from(Span::styled("  No snapshot available", gray)));
            lines.push(Line::from(Span::styled(
                format!("  Queue: {}", app.queue_root_display()),
                gray,
            )));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
