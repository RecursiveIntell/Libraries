//! Loop state panel — renders the autonomous loop's shared state.

use ratatui::layout::Rect;
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};
use ratatui::Frame;

use crate::app::App;

/// Render the loop state panel.
pub fn render(frame: &mut Frame, area: Rect, app: &App) {
    let state = app.loop_state_snapshot();

    let title = if app.is_paused() {
        " Loop State [PAUSED] "
    } else {
        " Loop State "
    };

    let block = Block::default().borders(Borders::ALL).title(Span::styled(
        title,
        Style::default().add_modifier(Modifier::BOLD),
    ));

    let green = Style::default().fg(Color::Green);
    let red = Style::default().fg(Color::Red);
    let yellow = Style::default().fg(Color::Yellow);
    let gray = Style::default().fg(Color::DarkGray);

    let mut lines = vec![
        Line::from(format!("  Iteration:   {}", state.iteration)),
        Line::from(format!("  Gaps:        {}", state.gaps_detected)),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Tasks Gen:   "),
            Span::styled(
                state.tasks_generated.to_string(),
                Style::default().fg(Color::Cyan),
            ),
        ]),
        Line::from(vec![
            Span::raw("  Tasks Done:  "),
            Span::styled(state.tasks_completed.to_string(), green),
        ]),
        Line::from(vec![
            Span::raw("  Tasks Fail:  "),
            Span::styled(state.tasks_failed.to_string(), red),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Facts Cap:   "),
            Span::styled(state.facts_captured.to_string(), green),
        ]),
        Line::from(vec![
            Span::raw("  Facts Rej:   "),
            Span::styled(state.facts_rejected.to_string(), red),
        ]),
        Line::from(""),
        Line::from(vec![
            Span::raw("  Consec Fail: "),
            Span::styled(
                state.consecutive_failures.to_string(),
                if state.consecutive_failures > 0 {
                    yellow
                } else {
                    gray
                },
            ),
        ]),
        Line::from(vec![
            Span::raw("  Safe Mode:   "),
            Span::styled(
                if state.safe_mode { "ON" } else { "off" },
                if state.safe_mode { red } else { gray },
            ),
        ]),
        Line::from(""),
    ];

    // Current job.
    match &state.current_job {
        Some(job) => {
            lines.push(Line::from(vec![
                Span::raw("  Current:     "),
                Span::styled(job, Style::default().fg(Color::Blue)),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::raw("  Current:     "),
                Span::styled("idle", gray),
            ]));
        }
    }

    // Last error.
    match &state.last_error {
        Some(err) => {
            let err_display: String = if err.len() > 40 {
                format!("{}…", &err[..40])
            } else {
                err.clone()
            };
            lines.push(Line::from(vec![
                Span::raw("  Last Error:  "),
                Span::styled(err_display, red),
            ]));
        }
        None => {
            lines.push(Line::from(vec![
                Span::raw("  Last Error:  "),
                Span::styled("none", gray),
            ]));
        }
    }

    let paragraph = Paragraph::new(lines).block(block);
    frame.render_widget(paragraph, area);
}
