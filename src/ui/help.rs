use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Paragraph};

use crate::theme;

use super::App;

/// The four columns, then the keys that are not obvious from the footer.
const ROWS: [(&str, &str); 14] = [
    ("columns", "groups · topics · text · sections"),
    ("→ ←", "walk the four columns; h and l do the same"),
    (
        "j k",
        "move in a column: a line in the text, a section in the index",
    ),
    (
        "enter",
        "step into a group, open the topic, or open at a section",
    ),
    ("g G", "first and last line, inside the reader"),
    ("/", "search titles and the code inside snippets"),
    ("tab", "in the reader, jump to the section index"),
    ("esc", "close a dialog, leave a column, go back"),
    ("M", "mouse capture, off by default"),
    ("q", "back one level, or quit from the home screen"),
    ("topics", "your own: $XDG_DATA_HOME/cheatrs/topics/*.json"),
    ("validate", "cheatrs --validate FILE explains what is wrong"),
    ("colour", "NO_COLOR or TERM=dumb turns it off"),
    ("", ""),
];

/// `?` explains the app without leaving it: one panel, no pages to page through.
pub fn draw(frame: &mut Frame, area: Rect, app: &App) {
    if !app.help {
        return;
    }

    let width = area.width.saturating_sub(6).clamp(44, 78);
    let height = (ROWS.len() as u16 + 4).min(area.height);
    let panel = Rect {
        x: area.x + area.width.saturating_sub(width) / 2,
        y: area.y + area.height.saturating_sub(height) / 2,
        width,
        height,
    };
    frame.render_widget(Clear, panel);
    frame.render_widget(Block::new().style(Style::new().bg(theme::surface())), panel);

    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(panel);

    let title = Line::from(vec![
        Span::styled("▌ ", Style::new().fg(theme::fg())),
        Span::styled(
            "cheatrs",
            Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  keys", Style::new().fg(theme::fg_faint())),
    ]);
    frame.render_widget(Paragraph::new(title), shift(rows[0]));

    let body = Rect {
        x: rows[1].x + 2,
        width: rows[1].width.saturating_sub(4),
        ..rows[1]
    };
    let lines: Vec<Line> = ROWS
        .iter()
        .map(|(key, text)| {
            // The label rows ("columns", "topics") are headings, not key bindings.
            let head = !text.is_empty() && key.chars().all(|c| c.is_ascii_lowercase());
            let key_style = if head {
                Style::new().fg(theme::fg_dim())
            } else {
                Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD)
            };
            Line::from(vec![
                Span::styled(format!("{key:<9}"), key_style),
                Span::styled(
                    text.to_string(),
                    Style::new().fg(if head { theme::fg() } else { theme::fg_dim() }),
                ),
            ])
        })
        .collect();
    frame.render_widget(Paragraph::new(lines), body);

    let foot = Line::from(vec![Span::styled(
        "esc or ? closes",
        Style::new().fg(theme::fg_faint()),
    )]);
    frame.render_widget(Paragraph::new(foot), shift(rows[2]));
}

fn shift(area: Rect) -> Rect {
    Rect {
        x: area.x + 2,
        width: area.width.saturating_sub(4),
        ..area
    }
}
