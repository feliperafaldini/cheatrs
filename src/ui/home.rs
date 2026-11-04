use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{
    Block, Borders, Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState,
};

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

    let measure = preview_measure(area.width);
    app.preview_visible = area.width >= layout::PREVIEW_THRESHOLD;
    app.sync_preview(measure);
    app.preview_visible = app.preview_visible && !app.preview.is_empty();
    if !app.preview_visible && app.focus == Focus::Preview {
        app.focus = Focus::Topics;
    }
    if !app.contents_visible() && app.focus == Focus::Contents {
        app.focus = Focus::Preview;
    }
    header(frame, rows[0], app);
    if app.preview_visible {
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

/// Text measure available to the preview, once the other columns and its contents have taken theirs.
fn preview_measure(width: u16) -> usize {
    let taken = layout::GROUP_COLUMN
        + layout::GUTTER
        + layout::TOPICS_COLUMN
        + layout::GUTTER
        + layout::PREVIEW_CONTENTS
        + layout::GUTTER;
    width.saturating_sub(taken).saturating_sub(1) as usize
}

/// The preview is a fourth surface, not a decoration: it appears only when the width pays for it.
/// Inside it, the same document the reader shows on the left, contents on the right.
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

    let inner = Layout::horizontal([
        Constraint::Min(20),
        Constraint::Length(layout::GUTTER),
        Constraint::Length(layout::PREVIEW_CONTENTS),
    ])
    .split(cols[4]);

    let preview = inner[0];
    let contents_rect = inner[2];
    app.preview_rect = Rect {
        x: cols[4].x,
        width: cols[4].width,
        ..preview
    };

    let accent = app
        .current()
        .map_or(theme::fg_dim(), |t| theme::accent(&t.color));
    let selected = crate::ui::reader::section_at(&app.preview_sections, app.preview_scroll);
    crate::ui::reader::contents(
        frame,
        contents_rect,
        &app.preview_sections,
        selected,
        app.focus == Focus::Contents,
        accent,
    );

    let max = app.preview.len().saturating_sub(preview.height as usize) as u16;
    if app.preview_scroll > max {
        app.preview_scroll = max;
    }
    frame.render_widget(
        Paragraph::new(app.preview.clone()).scroll((app.preview_scroll, 0)),
        preview,
    );

    // The column only advertises its scroll when it has the focus.
    if app.focus == Focus::Preview && max > 0 {
        let mut state = ScrollbarState::new(max as usize).position(app.preview_scroll as usize);
        let rail = Rect {
            x: preview.x + preview.width - 1,
            y: preview.y,
            width: 1,
            height: preview.height,
        };
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("▏"))
                .thumb_symbol("█")
                .track_style(Style::new().fg(theme::rule()))
                .thumb_style(Style::new().fg(theme::fg_dim())),
            rail,
            &mut state,
        );
    }
}

fn rule(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Block::new()
            .borders(Borders::LEFT)
            .border_style(Style::new().fg(theme::rule())),
        area,
    );
}

fn header(frame: &mut Frame, area: Rect, app: &App) {
    let line = Line::from(vec![
        Span::styled(
            "cheatrs",
            Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD),
        ),
        Span::raw("  "),
        Span::styled(
            format!(
                "{} topics · {} skipped",
                app.lib.topics.len(),
                app.lib.skipped
            ),
            Style::new().fg(theme::fg_faint()),
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
            let bar_style = Style::new().fg(theme::fg());
            let text_style = if selected {
                Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::fg_dim())
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
                Style::new().fg(theme::fg_faint())
            };
            let mut name_style = if selected {
                Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD)
            } else {
                Style::new().fg(theme::fg())
            };
            if selected {
                name_style = name_style.bg(theme::surface());
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
    let mouse = if app.mouse { "on" } else { "off" };
    let keys = match app.focus {
        Focus::Groups => format!(
            "hjkl/←→↑↓ move · ⇥/l column · ↵ open list · / search · M mouse {mouse} · q quit"
        ),
        Focus::Preview => format!(
            "hjkl/←→↑↓ scroll preview · ⇥/l sections · ↵ read · / search · M mouse {mouse} · q quit"
        ),
        Focus::Contents => {
            format!("↑↓ section · ↵ read · ⇥/h text · / search · M mouse {mouse} · q quit")
        }
        Focus::Topics => {
            format!("hjkl/←→↑↓ move · ⇥/l preview · ↵ read · / search · M mouse {mouse} · q quit")
        }
    };
    let line = Line::from(vec![
        Span::styled(keys, Style::new().fg(theme::fg_faint())),
        Span::raw("  "),
        Span::styled(
            format!("{}/{}", app.group + 1, app.lib.groups.len()),
            Style::new().fg(theme::fg_faint()),
        ),
    ]);
    frame.render_widget(Paragraph::new(line), area);
}
