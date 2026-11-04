use std::sync::LazyLock;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::model::Topic;
use crate::theme;

use super::{App, layout};

/// Folding happens at build time: folding at draw time would desync the scroll offset.
const INDENT: usize = 2;

static SYNTAX: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Theme> = LazyLock::new(|| {
    ThemeSet::load_defaults()
        .themes
        .get("base16-ocean.dark")
        .cloned()
        .unwrap_or_default()
});

/// A topic worth of lines, more than any terminal can show.
pub fn build(topic: &Topic) -> Vec<Line<'static>> {
    build_capped(topic, usize::MAX)
}

/// Same document, stopped early: the preview column does not need 900 lines.
pub fn build_capped(topic: &Topic, limit: usize) -> Vec<Line<'static>> {
    let accent = theme::accent(&topic.color);
    let measure = layout::READING_WIDTH as usize - INDENT;
    let mut lines: Vec<Line<'static>> = Vec::new();

    lines.push(Line::from(Span::styled(
        topic.title.clone(),
        Style::new().fg(accent).add_modifier(Modifier::BOLD),
    )));
    if !topic.desc.is_empty() {
        for piece in wrap(&topic.desc, measure) {
            lines.push(Line::from(Span::styled(
                piece,
                Style::new().fg(theme::FG_DIM),
            )));
        }
    }

    for section in &topic.sections {
        if lines.len() >= limit {
            break;
        }
        lines.push(Line::default());
        lines.push(Line::from(Span::styled(
            section.title.clone(),
            Style::new().fg(theme::FG).add_modifier(Modifier::BOLD),
        )));
        if !section.description.is_empty() {
            for piece in wrap(&section.description, measure) {
                lines.push(Line::from(Span::styled(
                    piece,
                    Style::new().fg(theme::FG_DIM),
                )));
            }
        }
        for snippet in &section.snippets {
            let mut spans = vec![Span::raw(" ".repeat(INDENT))];
            for (index, mut coloured) in highlight(&snippet.code, &snippet.language)
                .into_iter()
                .enumerate()
            {
                if index > 0 {
                    lines.extend(fold(std::mem::take(&mut spans), measure));
                    spans = vec![Span::raw(" ".repeat(INDENT))];
                }
                spans.append(&mut coloured);
            }
            lines.extend(fold(spans, measure));
        }
    }

    lines
}

/// One screen line per code line, folded to the reading measure, colour kept.
fn highlight(code: &str, language: &str) -> Vec<Vec<Span<'static>>> {
    let syntax = SYNTAX
        .find_syntax_by_token(language)
        .unwrap_or_else(|| SYNTAX.find_syntax_plain_text());
    let mut highlighter = HighlightLines::new(syntax, &THEME);

    LinesWithEndings::from(code)
        .map(|line| {
            let ranges = highlighter
                .highlight_line(line, &SYNTAX)
                .unwrap_or_default();
            ranges
                .into_iter()
                .map(|(style, text)| {
                    let colour = style.foreground;
                    Span::styled(
                        text.trim_end_matches('\n').to_string(),
                        Style::new().fg(Color::Rgb(colour.r, colour.g, colour.b)),
                    )
                })
                .collect()
        })
        .collect()
}

/// Hard fold at `width` cells, character by character, styles preserved.
fn fold(spans: Vec<Span<'static>>, width: usize) -> Vec<Line<'static>> {
    let mut out: Vec<Line<'static>> = Vec::new();
    let mut current: Vec<Span<'static>> = Vec::new();
    let mut used = 0usize;

    for span in spans {
        let style = span.style;
        let text = span.content.into_owned();
        let mut rest: Vec<char> = text.chars().collect();
        while !rest.is_empty() {
            if used == width {
                out.push(Line::from(std::mem::take(&mut current)));
                used = 0;
            }
            let room = width - used;
            let take = room.min(rest.len());
            let piece: String = rest.drain(..take).collect();
            used += piece.chars().count();
            current.push(Span::styled(piece, style));
        }
    }
    if !current.is_empty() {
        out.push(Line::from(current));
    }
    if out.is_empty() {
        out.push(Line::default());
    }
    out
}

pub fn draw(frame: &mut Frame, area: Rect, app: &mut App) {
    let rows = Layout::vertical([
        Constraint::Length(1),
        Constraint::Min(1),
        Constraint::Length(1),
    ])
    .split(area);

    // The reading measure: text never runs wider than this, the margins take the rest.
    let width = area.width.min(layout::READING_WIDTH);
    let x = area.x + area.width.saturating_sub(width) / 2;
    let centred = |rect: Rect| Rect { x, width, ..rect };
    let header = centred(rows[0]);
    let body = centred(rows[1]);
    let footer = centred(rows[2]);

    let accent = app
        .current()
        .map_or(theme::FG_DIM, |t| theme::accent(&t.color));
    let (crumb, badge) = app.current().map_or(("‹".to_string(), String::new()), |t| {
        (
            format!("‹ {} / {}", t.group.to_uppercase(), t.title),
            t.group.to_uppercase(),
        )
    });
    let header_line = Line::from(vec![
        Span::styled(crumb, Style::new().fg(accent)),
        Span::raw("  "),
        Span::styled(badge, Style::new().fg(theme::FG_FAINT)),
    ]);
    frame.render_widget(Paragraph::new(header_line), header);

    let page = body.height as usize;
    let max = app.lines.len().saturating_sub(page) as u16;
    if app.scroll > max {
        app.scroll = max;
    }
    frame.render_widget(
        Paragraph::new(app.lines.clone()).scroll((app.scroll, 0)),
        body,
    );

    let footer_line = Line::from(vec![
        Span::styled(
            "↑↓ scroll · g/G top/bottom · q back",
            Style::new().fg(theme::FG_FAINT),
        ),
        Span::raw("  "),
        Span::styled(
            format!("{}%", percent(app.scroll, max)),
            Style::new().fg(theme::FG_FAINT),
        ),
    ]);
    frame.render_widget(Paragraph::new(footer_line), footer);
}

fn percent(scroll: u16, max: u16) -> u16 {
    if max == 0 {
        return 100;
    }
    ((scroll as f32 / max as f32) * 100.0).round() as u16
}

fn wrap(text: &str, width: usize) -> Vec<String> {
    if text.chars().count() <= width {
        return vec![text.to_string()];
    }
    let mut out = Vec::new();
    let mut current = String::new();
    for word in text.split(' ') {
        if !current.is_empty() && current.chars().count() + 1 + word.chars().count() > width {
            out.push(std::mem::take(&mut current));
        }
        if !current.is_empty() {
            current.push(' ');
        }
        current.push_str(word);
    }
    if !current.is_empty() {
        out.push(current);
    }
    out
}
