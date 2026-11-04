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

    let measure = preview_measure(app, area.width);
    app.preview_contents = contents_width(app, area.width);
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
            Constraint::Length(app.group_width),
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
fn preview_measure(app: &App, width: u16) -> usize {
    let taken = app.group_width
        + layout::GUTTER
        + app.topics_width
        + layout::GUTTER
        + app.preview_contents
        + layout::GUTTER;
    width.saturating_sub(taken).saturating_sub(1) as usize
}

/// The contents list takes three tenths of the preview column, inside bounds.
fn contents_width(app: &App, width: u16) -> u16 {
    let preview =
        width.saturating_sub(app.group_width + layout::GUTTER + app.topics_width + layout::GUTTER);
    (preview * layout::PREVIEW_CONTENTS_SHARE / 10)
        .clamp(layout::PREVIEW_CONTENTS_MIN, layout::PREVIEW_CONTENTS_MAX)
}

/// The preview is a fourth surface, not a decoration: it appears only when the width pays for it.
/// Inside it, the same document the reader shows on the left, contents on the right.
fn with_preview(frame: &mut Frame, area: Rect, app: &mut App) {
    let cols = Layout::horizontal([
        Constraint::Length(app.group_width),
        Constraint::Length(layout::GUTTER),
        Constraint::Length(app.topics_width),
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
        Constraint::Length(app.preview_contents),
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
    // The mark is the same cell that marks a selection everywhere else in the app,
    // tinted with the colour of the topic under the cursor: the header points at
    // what you are looking at instead of labelling the window.
    let accent = app
        .current()
        .map_or(theme::fg_dim(), |topic| theme::accent(&topic.color));
    let version = env!("CARGO_PKG_VERSION");
    // The trailing suffix is a warning, not a stat: it only exists when a file lost.
    let skipped = if app.lib.skipped > 0 {
        format!(" · {} skipped", app.lib.skipped)
    } else {
        String::new()
    };
    let meta = format!(
        "{} topics · {} sections · {} snippets{skipped}",
        app.lib.topics.len(),
        app.lib.sections,
        app.lib.snippets
    );
    let short = if skipped.is_empty() {
        format!("{} topics", app.lib.topics.len())
    } else {
        format!("{} topics{skipped}", app.lib.topics.len())
    };

    // Bar, wordmark, version: 2 + 7 + 1 + len(version).
    let left = 2 + "cheatrs".len() as u16 + 1 + version.len() as u16;
    let mut spans = vec![
        Span::styled("▌ ", Style::new().fg(accent)),
        Span::styled(
            "cheatrs",
            Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD),
        ),
        Span::raw(" "),
        Span::styled(version, Style::new().fg(theme::fg_faint())),
    ];

    // The rule only exists when it can fit between the two halves without crowding them.
    let right = if area.width > left + meta.chars().count() as u16 + 6 {
        Some(meta)
    } else if area.width > left + short.len() as u16 + 4 {
        Some(short)
    } else {
        None
    };
    if let Some(right) = right {
        let rule = area
            .width
            .saturating_sub(left + right.chars().count() as u16 + 2);
        spans.push(Span::raw(" "));
        spans.push(Span::styled(
            "─".repeat(rule as usize),
            Style::new().fg(theme::rule()),
        ));
        spans.push(Span::raw(" "));
        spans.push(Span::styled(right, Style::new().fg(theme::fg_faint())));
    }

    frame.render_widget(Paragraph::new(Line::from(spans)), area);
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
            "hjkl/←→↑↓ move · ⇥/l column · ↵ open list · / search · M mouse {mouse} · ? help · q quit"
        ),
        Focus::Preview => format!(
            "hjkl/←→↑↓ scroll preview · ⇥/l sections · ↵ read · / search · M mouse {mouse} · ? help · q quit"
        ),
        Focus::Contents => {
            format!("↑↓ section · ↵ read · ⇥/h text · / search · M mouse {mouse} · ? help · q quit")
        }
        Focus::Topics => {
            format!(
                "hjkl/←→↑↓ move · ⇥/l preview · ↵ read · / search · M mouse {mouse} · ? help · q quit"
            )
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
