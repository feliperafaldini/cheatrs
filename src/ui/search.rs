use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::theme;

use super::App;

const MAX_RESULTS: usize = 12;

/// Search is a modal over whatever is on screen: it answers "where", then gets out of the way.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if !app.searching {
        return;
    }

    let width = area.width.saturating_sub(4).clamp(30, 76);
    let height = area
        .height
        .saturating_sub(4)
        .clamp(5, MAX_RESULTS as u16 + 3);
    let panel = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + 2,
        width,
        height,
    };
    frame.render_widget(Clear, panel);
    frame.render_widget(Block::new().style(Style::new().bg(theme::SURFACE)), panel);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(1),
    ])
    .split(panel);

    let input = Line::from(vec![
        Span::styled("› ", Style::new().fg(theme::FG_DIM)),
        Span::styled(
            app.query.clone(),
            Style::new().fg(theme::FG).add_modifier(Modifier::BOLD),
        ),
        Span::styled("▌", Style::new().fg(theme::FG_DIM)),
    ]);
    frame.render_widget(Paragraph::new(input), shift(rows[0]));

    let count = Line::from(vec![Span::styled(
        format!("{} results", app.results.len()),
        Style::new().fg(theme::FG_FAINT),
    )]);
    frame.render_widget(Paragraph::new(count), shift(rows[1]));

    let lines: Vec<Line> = app
        .results
        .iter()
        .take(MAX_RESULTS)
        .enumerate()
        .map(|(position, hit)| {
            let entry = &app.index.entries[hit.entry];
            let selected = position == app.cursor;
            let bar = if selected { "▌" } else { " " };
            let title_style = if selected {
                Style::new().fg(theme::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::FG)
            };
            let context_style = Style::new().fg(theme::FG_FAINT);
            let kind = if entry.section.is_some() {
                format!("{} ›", entry.context)
            } else {
                entry.context.clone()
            };
            let mut spans = vec![
                Span::styled(bar, Style::new().fg(theme::FG)),
                Span::raw(" "),
                Span::styled(truncate(&entry.title, 30), title_style),
                Span::raw("  "),
                Span::styled(kind, context_style),
            ];
            if let Some(excerpt) = &hit.excerpt {
                spans.push(Span::raw("  "));
                spans.push(Span::styled(
                    format!("⌁ {}", truncate(excerpt, 40)),
                    Style::new().fg(theme::FG_DIM),
                ));
            }
            Line::from(spans)
        })
        .collect();

    if lines.is_empty() {
        let empty = Line::from(Span::styled(
            "nothing matches",
            Style::new().fg(theme::FG_FAINT),
        ));
        frame.render_widget(Paragraph::new(empty), shift(rows[2]));
    } else {
        frame.render_widget(Paragraph::new(lines), shift(rows[2]));
    }
}

/// One column of inset on each side, inside the panel.
fn shift(area: Rect) -> Rect {
    Rect {
        x: area.x + 1,
        width: area.width.saturating_sub(2),
        ..area
    }
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}
