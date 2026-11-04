use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph};

use crate::data::{group_label, monogram};
use crate::theme;

use super::{App, Focus, layout};

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);

    app.sync_preview();
    header(frame, rows[0], app);
    if area.width >= layout::PREVIEW_THRESHOLD {
        with_preview(frame, rows[1], app);
    } else {
        let cols = Layout::horizontal([
            Constraint::Length(layout::GROUP_COLUMN),
            Constraint::Length(layout::GUTTER),
            Constraint::Min(layout::MIN_TOPICS),
        ])
        .split(rows[1]);

        app.groups_rect = cols[0];
        app.topics_rect = cols[2];

        groups(frame, cols[0], app);
        rule(frame, cols[1]);
        topics(frame, cols[2], app);
    }
    footer(frame, rows[2], app);
}

/// The preview is a fourth surface, not a decoration: it appears only when the width pays for it.
fn with_preview(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::horizontal([
        Constraint::Length(layout::GROUP_COLUMN),
        Constraint::Length(layout::GUTTER),
        Constraint::Length(layout::TOPICS_COLUMN),
        Constraint::Length(layout::GUTTER),
        Constraint::Min(layout::PREVIEW_COLUMN),
    ])
    .split(area);

    app.groups_rect = cols[0];
    app.topics_rect = cols[2];

    groups(frame, cols[0], app);
    rule(frame, cols[1]);
    topics(frame, cols[2], app);
    rule(frame, cols[3]);
    // One column of breathing room, same as everywhere else.
    let preview = Rect {
        x: cols[4].x + 1,
        width: cols[4].width.saturating_sub(1),
        ..cols[4]
    };
    frame.render_widget(Paragraph::new(app.preview.clone()), preview);
}

fn rule(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::new()
            .borders(Borders::LEFT)
            .border_style(Style::new().fg(theme::RULE)),
        area,
    );
}

fn header(frame: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::styled(
            "cheatrs",
            Style::new().fg(theme::FG).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{} topics · {} skipped",
                app.lib.topics.len(),
                app.lib.skipped
            ),
            Style::new().fg(theme::FG_FAINT),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}

fn groups(frame: &mut Frame, area: Rect, app: &mut App) {
    let height = area.height as usize;
    let offset = app.group.saturating_sub(height.saturating_sub(1));
    app.groups_offset = offset as u16;

    let lines: Vec<Line> = app
        .lib
        .groups
        .iter()
        .enumerate()
        .map(|(index, group)| {
            let selected = index == app.group;
            let active = selected && app.focus == Focus::Groups;
            let bar = if active { "▌" } else { " " };
            let bar_style = Style::new().fg(theme::FG);
            let text_style = if selected {
                Style::new().fg(theme::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::FG_DIM)
            };
            Line::from(vec![
                Span::styled(bar, bar_style),
                Span::raw(" "),
                Span::styled(group_label(&group.name), text_style),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), area);
}

fn topics(frame: &mut Frame, area: Rect, app: &mut App) {
    let height = area.height as usize;
    let offset = app.topic.saturating_sub(height.saturating_sub(1));
    app.topics_offset = offset as u16;

    let lines: Vec<Line> = app
        .group_topics()
        .iter()
        .enumerate()
        .map(|(index, topic_index)| {
            let topic = &app.lib.topics[*topic_index];
            let selected = index == app.topic;
            let active = selected && app.focus == Focus::Topics;
            let accent = theme::accent(&topic.color);
            let bar = if selected { "▌" } else { " " };
            let bar_style = if active {
                Style::new().fg(accent)
            } else {
                Style::new().fg(theme::FG_FAINT)
            };
            let mut name_style = if selected {
                Style::new().fg(theme::FG).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::FG)
            };
            if selected {
                name_style = name_style.bg(theme::SURFACE);
            }
            Line::from(vec![
                Span::styled(bar, bar_style),
                Span::styled(
                    format!("{:>3} ", monogram(&topic.slug, &topic.title)),
                    Style::new().fg(accent),
                ),
                Span::styled(topic.title.clone(), name_style),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), area);
}

fn footer(frame: &mut Frame, area: Rect, app: &App) {
    let keys = if app.mouse {
        "↑↓ move · ⇥ column · ↵ open · M mouse on · q quit"
    } else {
        "↑↓ move · ⇥ column · ↵ open · M mouse off · q quit"
    };
    let line = Line::from(vec![
        Span::styled(keys, Style::new().fg(theme::FG_FAINT)),
        Span::raw("  "),
        Span::styled(
            format!("{}/{}", app.group + 1, app.lib.groups.len()),
            Style::new().fg(theme::FG_FAINT),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
