use std::sync::LazyLock;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Scrollbar, ScrollbarOrientation, ScrollbarState};
use syntect::easy::HighlightLines;
use syntect::highlighting::{Theme, ThemeSet};
use syntect::parsing::SyntaxSet;
use syntect::util::LinesWithEndings;

use crate::model::Topic;
use crate::theme;

use super::{App, layout};

/// Folding happens at build time: folding at draw time would desync the scroll offset.
const INDENT: usize = 2;
/// The contents column only earns its place when the terminal can pay for both.
pub const TOC_COLUMN: u16 = 20;
pub const TOC_THRESHOLD: u16 = 120;

static SYNTAX: LazyLock<SyntaxSet> = LazyLock::new(SyntaxSet::load_defaults_newlines);
static THEME: LazyLock<Theme> = LazyLock::new(|| {
    ThemeSet::load_defaults()
        .themes
        .get("base16-ocean.dark")
        .cloned()
        .unwrap_or_default()
});

pub struct Document {
    pub lines: Vec<Line<'static>>,
    /// Title and first line of every section: what the table of contents jumps to.
    pub sections: Vec<(String, u16)>,
}

pub fn build(topic: &Topic) -> Document {
    build_document(topic, usize::MAX, layout::READING_WIDTH as usize - INDENT)
}

/// Same document, stopped early and folded to an explicit measure — the preview needs both.
pub fn build_document(topic: &Topic, limit: usize, measure: usize) -> Document {
    let accent = theme::accent(&topic.color);
    let measure = measure.max(20);
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut sections: Vec<(String, u16)> = Vec::new();

    lines.push(Line::from(Span::styled(
        topic.title.clone(),
        Style::new().fg(accent).add_modifier(Modifier::BOLD),
    )));
    if !topic.desc.is_empty() {
        for piece in wrap(&topic.desc, measure) {
            lines.push(Line::from(Span::styled(
                piece,
                Style::new().fg(theme::fg_dim()),
            )));
        }
    }

    for section in &topic.sections {
        if lines.len() >= limit {
            break;
        }
        // One blank line separates sections; the description sits tight under its title.
        lines.push(Line::default());
        sections.push((section.title.clone(), clamp_len(lines.len())));

        lines.push(Line::from(Span::styled(
            section.title.clone(),
            Style::new().fg(theme::fg()).add_modifier(Modifier::BOLD),
        )));
        if !section.description.is_empty() {
            for piece in wrap(&section.description, measure) {
                lines.push(Line::from(Span::styled(
                    piece,
                    Style::new().fg(theme::fg_dim()),
                )));
            }
            // Breathing room between the prose and the code it introduces.
            lines.push(Line::default());
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

    Document { lines, sections }
}

fn clamp_len(value: usize) -> u16 {
    u16::try_from(value).unwrap_or(u16::MAX)
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
                        Style::new().fg(theme::paint_rgb(colour.r, colour.g, colour.b)),
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

    let show_toc = area.width >= TOC_THRESHOLD && !app.sections.is_empty();
    if !show_toc {
        app.toc_focus = false;
    }
    let toc_width = if show_toc { TOC_COLUMN } else { 0 };
    let gap = u16::from(show_toc);

    // Reading measure on the left, contents on the right of it, centred as one block.
    let block = (layout::READING_WIDTH + toc_width + gap).min(area.width);
    let x = area.x + area.width.saturating_sub(block) / 2;
    let text_width = block.saturating_sub(toc_width + gap);

    let header = Rect {
        x,
        y: rows[0].y,
        width: block,
        height: 1,
    };
    let body = Rect {
        x,
        y: rows[1].y,
        width: text_width,
        height: rows[1].height,
    };
    let footer = Rect {
        x,
        y: rows[2].y,
        width: block,
        height: 1,
    };

    let accent = app
        .current()
        .map_or(theme::fg_dim(), |t| theme::accent(&t.color));
    let crumb = app.current().map_or_else(
        || "‹".to_string(),
        |t| format!("‹ {} / {}", t.group.to_uppercase(), t.title),
    );
    let header_line = Line::from(vec![Span::styled(crumb, Style::new().fg(accent))]);
    frame.render_widget(Paragraph::new(header_line), header);

    let page = body.height as usize;
    let max = app.lines.len().saturating_sub(page) as u16;
    if app.scroll > max {
        app.scroll = max;
    }
    // Scrolling the document keeps the contents in step; the contents drive it when focused.
    if !app.toc_focus {
        app.toc_index = section_at(&app.sections, app.scroll);
    }

    let mut page_lines = app.lines.clone();
    if let (Some(line), Some(until)) = (app.flash_line, app.flash_until) {
        if std::time::Instant::now() < until {
            if let Some(target) = page_lines.get_mut(line as usize) {
                let mut patched = Line::default();
                for span in target.spans.clone() {
                    patched.push_span(Span::styled(
                        span.content.clone(),
                        span.style.bg(theme::surface()),
                    ));
                }
                page_lines[line as usize] = patched;
            }
        } else {
            app.flash_line = None;
            app.flash_until = None;
        }
    }
    frame.render_widget(Paragraph::new(page_lines).scroll((app.scroll, 0)), body);

    if show_toc {
        let toc = Rect {
            x: body.x + body.width + gap,
            y: rows[1].y,
            width: toc_width,
            height: rows[1].height,
        };
        contents(
            frame,
            toc,
            &app.sections,
            app.toc_index,
            app.toc_focus,
            accent,
        );
    }

    if max > 0 {
        let mut state = ScrollbarState::new(max as usize).position(app.scroll as usize);
        let rail = Rect {
            x: body.x + body.width - 1,
            y: body.y,
            width: 1,
            height: body.height,
        };
        frame.render_stateful_widget(
            Scrollbar::new(ScrollbarOrientation::VerticalRight)
                .begin_symbol(None)
                .end_symbol(None)
                .track_symbol(Some("▏"))
                .thumb_symbol("█")
                .track_style(Style::new().fg(theme::rule()))
                .thumb_style(Style::new().fg(theme::fg_faint())),
            rail,
            &mut state,
        );
    }

    let keys = if show_toc {
        "↑↓ scroll · ⇥/→ sections · g/G top/bottom · ? help · q back"
    } else {
        "↑↓ scroll · g/G top/bottom · ? help · q back"
    };
    let footer_line = Line::from(vec![Span::styled(keys, Style::new().fg(theme::fg_faint()))]);
    frame.render_widget(Paragraph::new(footer_line), footer);
}

/// One contents list, shared by the reader and the preview so both read the same way.
pub fn contents(
    frame: &mut Frame,
    area: Rect,
    sections: &[(String, u16)],
    selected: usize,
    focused: bool,
    accent: Color,
) {
    let height = area.height as usize;
    let offset = selected.saturating_sub(height.saturating_sub(1));

    let lines: Vec<Line> = sections
        .iter()
        .enumerate()
        .map(|(index, (title, _))| {
            let is_selected = index == selected;
            let active = is_selected && focused;
            let bar = if is_selected { "▌" } else { " " };
            let bar_style = Style::new().fg(if active { accent } else { theme::fg_faint() });
            let text_style = if is_selected {
                Style::new().fg(theme::fg())
            } else {
                Style::new().fg(theme::fg_faint())
            };
            Line::from(vec![
                Span::styled(bar, bar_style),
                Span::raw(" "),
                Span::styled(
                    truncate(title, (area.width as usize).saturating_sub(2)),
                    text_style,
                ),
            ])
        })
        .collect();

    frame.render_widget(Paragraph::new(lines).scroll((offset as u16, 0)), area);
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}

/// Which section contains this line.
pub fn section_at(sections: &[(String, u16)], scroll: u16) -> usize {
    sections
        .iter()
        .rposition(|(_, start)| *start <= scroll)
        .unwrap_or(0)
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
