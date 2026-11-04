pub mod home;
pub mod layout;
pub mod reader;
pub mod search;

use crossterm::event::{
    DisableMouseCapture, EnableMouseCapture, KeyCode, KeyEvent, MouseButton, MouseEvent,
    MouseEventKind,
};
use crossterm::execute;
use ratatui::Frame;
use ratatui::layout::{Alignment, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Clear, Paragraph, Wrap};

use crate::data::Library;
use crate::model::Topic;
use crate::search::{Hit, Index as SearchIndex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Groups,
    Topics,
    Preview,
    /// The section index that lives inside the preview: `j`/`k` jump by section.
    Contents,
}

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Screen {
    Home,
    Reader,
}

pub struct App {
    pub lib: Library,
    pub focus: Focus,
    pub group: usize,
    pub topic: usize,
    pub screen: Screen,
    pub scroll: u16,
    pub mouse: bool,
    pub quit: bool,
    pub lines: Vec<Line<'static>>,
    /// Contents of the open topic: title and first line of each section.
    pub sections: Vec<(String, u16)>,
    pub toc_index: usize,
    pub toc_focus: bool,
    /// Preview of the selected topic: built once per selection, never per frame.
    pub preview: Vec<Line<'static>>,
    pub preview_topic: Option<usize>,
    pub preview_scroll: u16,
    pub preview_rect: Rect,
    pub preview_visible: bool,
    /// Contents of the previewed topic and the measure it was folded to.
    pub preview_sections: Vec<(String, u16)>,
    pub preview_measure: usize,
    pub groups_rect: Rect,
    pub topics_rect: Rect,
    pub groups_offset: u16,
    pub topics_offset: u16,
    /// Search: one index over topics and sections, opened as a modal.
    pub index: SearchIndex,
    pub searching: bool,
    pub query: String,
    pub results: Vec<Hit>,
    pub cursor: usize,
    /// The line to flash after jumping, and until when.
    pub flash_line: Option<u16>,
    pub flash_until: Option<std::time::Instant>,
    /// Column widths measured from the library at startup, not guessed.
    pub group_width: u16,
    pub topics_width: u16,
    /// Width of the contents list inside the preview, recomputed per frame.
    pub preview_contents: u16,
}

impl App {
    pub fn new(lib: Library) -> Self {
        let index = SearchIndex::build(&lib);
        // Measure once: the widest label and the widest title decide the two
        // fixed columns, so the text column keeps everything that is left over.
        let group_width = lib
            .groups
            .iter()
            .map(|group| crate::data::group_label(&group.name).chars().count() as u16 + 3)
            .max()
            .unwrap_or(layout::GROUP_COLUMN_MIN)
            .clamp(layout::GROUP_COLUMN_MIN, layout::GROUP_COLUMN_MAX);
        let topics_width = lib
            .topics
            .iter()
            // One bar cell, the four of `{:>3} `, then the title.
            .map(|topic| 5 + topic.title.chars().count() as u16)
            .max()
            .unwrap_or(layout::TOPICS_COLUMN_MIN)
            .clamp(layout::TOPICS_COLUMN_MIN, layout::TOPICS_COLUMN_MAX);
        Self {
            lib,
            index,
            group_width,
            topics_width,
            preview_contents: layout::PREVIEW_CONTENTS_MIN,
            searching: false,
            query: String::new(),
            results: Vec::new(),
            cursor: 0,
            flash_line: None,
            flash_until: None,
            focus: Focus::Groups,
            group: 0,
            topic: 0,
            screen: Screen::Home,
            scroll: 0,
            mouse: false,
            quit: false,
            lines: Vec::new(),
            sections: Vec::new(),
            toc_index: 0,
            toc_focus: false,
            preview: Vec::new(),
            preview_topic: None,
            preview_scroll: 0,
            preview_rect: Rect::default(),
            preview_visible: false,
            preview_sections: Vec::new(),
            preview_measure: 0,
            groups_rect: Rect::default(),
            topics_rect: Rect::default(),
            groups_offset: 0,
            topics_offset: 0,
        }
    }

    pub fn group_topics(&self) -> &[usize] {
        &self.lib.groups[self.group].topics
    }

    pub fn current(&self) -> Option<&Topic> {
        self.group_topics()
            .get(self.topic)
            .map(|index| &self.lib.topics[*index])
    }

    /// Rebuild the preview only when the selection or the measure actually moved.
    pub fn sync_preview(&mut self, measure: usize) {
        let index = self.group_topics().get(self.topic).copied();
        if index == self.preview_topic && self.preview_measure == measure {
            return;
        }
        let document = match index {
            Some(i) => reader::build_document(&self.lib.topics[i], 400, measure),
            None => reader::Document {
                lines: Vec::new(),
                sections: Vec::new(),
            },
        };
        self.preview = document.lines;
        self.preview_sections = document.sections;
        self.preview_topic = index;
        self.preview_measure = measure;
        self.preview_scroll = 0;
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        let area = frame.area();
        // A terminal that shrank mid-session gets the same sentence the boot check
        // prints, centred, instead of a layout squeezed past the point it works.
        if !crate::fits(area.width, area.height) {
            size_warning(frame, area);
            return;
        }
        match self.screen {
            Screen::Home => home::draw(frame, frame.area(), self),
            Screen::Reader => reader::draw(frame, frame.area(), self),
        }
        // The modal sits on top of whatever screen is behind it.
        search::draw(frame, frame.area(), self);
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        if self.searching {
            self.on_search_key(key);
            return;
        }
        match self.screen {
            Screen::Home => match key.code {
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.move_down(),
                KeyCode::Char('k') | KeyCode::Up => self.move_up(),
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.focus_next(),
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => self.focus_prev(),
                KeyCode::Enter => match self.focus {
                    // Enter picks the group and steps into its list, so the arrows walk topics next.
                    Focus::Groups => self.focus = Focus::Topics,
                    Focus::Topics | Focus::Preview => self.open(),
                    Focus::Contents => {
                        let section =
                            reader::section_at(&self.preview_sections, self.preview_scroll);
                        self.open();
                        if let Some((_, start)) = self.sections.get(section) {
                            self.scroll = *start;
                            self.toc_index = section;
                        }
                    }
                },
                KeyCode::Char('M') => self.toggle_mouse(),
                KeyCode::Char('/') => self.start_search(),
                _ => {}
            },
            Screen::Reader => match key.code {
                KeyCode::Char('q') => self.screen = Screen::Home,
                // The index sits to the right of the text, so the arrow that walks right
                // is the one that reaches it.
                KeyCode::Right => {
                    if !self.sections.is_empty() {
                        self.toc_focus = true;
                    }
                }
                KeyCode::Left if self.toc_focus => self.toc_focus = false,
                KeyCode::Left => self.screen = Screen::Home,
                KeyCode::Esc if self.toc_focus => self.toc_focus = false,
                KeyCode::Esc => self.screen = Screen::Home,
                KeyCode::Tab => {
                    if !self.sections.is_empty() {
                        self.toc_focus = !self.toc_focus;
                    }
                }
                KeyCode::Enter if self.toc_focus => self.toc_focus = false,
                KeyCode::Char('j') | KeyCode::Down if self.toc_focus => self.step_section(1),
                KeyCode::Char('k') | KeyCode::Up if self.toc_focus => self.step_section(-1),
                KeyCode::Char('j') | KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::Char('k') | KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Char('g') => self.scroll = 0,
                KeyCode::Char('G') => self.scroll = u16::MAX,
                KeyCode::Char('M') => self.toggle_mouse(),
                KeyCode::Char('/') => self.start_search(),
                _ => {}
            },
        }
    }

    fn start_search(&mut self) {
        self.searching = true;
        self.query.clear();
        self.results.clear();
        self.cursor = 0;
    }

    fn on_search_key(&mut self, key: KeyEvent) {
        match key.code {
            KeyCode::Esc => {
                self.searching = false;
                self.query.clear();
                self.results.clear();
            }
            KeyCode::Enter => self.commit_search(),
            KeyCode::Backspace => {
                self.query.pop();
                self.refresh_results();
            }
            KeyCode::Up => self.cursor = self.cursor.saturating_sub(1),
            KeyCode::Down => {
                if self.cursor + 1 < self.results.len() {
                    self.cursor += 1;
                }
            }
            KeyCode::Char(character) => {
                self.query.push(character);
                self.refresh_results();
            }
            _ => {}
        }
    }

    fn refresh_results(&mut self) {
        self.results = self.index.query(&self.query);
        self.cursor = 0;
    }

    /// A result is a destination: select the topic, open it, and land on the section if there is one.
    fn commit_search(&mut self) {
        let Some(hit) = self.results.get(self.cursor) else {
            self.searching = false;
            return;
        };
        let entry = &self.index.entries[hit.entry];
        let (topic_index, section) = (entry.topic, entry.section);

        if let Some((group, position)) =
            self.lib
                .groups
                .iter()
                .enumerate()
                .find_map(|(group, item)| {
                    item.topics
                        .iter()
                        .position(|topic| *topic == topic_index)
                        .map(|position| (group, position))
                })
        {
            self.group = group;
            self.topic = position;
            self.focus = Focus::Topics;
        }

        self.searching = false;
        self.query.clear();
        self.results.clear();
        self.open();

        if let Some(section) = section
            && let Some((_, start)) = self.sections.get(section)
        {
            self.scroll = *start;
            self.toc_index = section;
            // One deliberate motion: the landing line announces itself, then fades.
            self.flash_line = Some(*start);
            self.flash_until =
                Some(std::time::Instant::now() + std::time::Duration::from_millis(320));
        }
    }

    pub fn on_mouse(&mut self, event: MouseEvent) {
        let inside = |rect: Rect, x: u16, y: u16| {
            x >= rect.x && x < rect.x + rect.width && y >= rect.y && y < rect.y + rect.height
        };
        match event.kind {
            MouseEventKind::ScrollDown => {
                if self.screen == Screen::Reader {
                    self.scroll = self.scroll.saturating_add(3);
                } else {
                    self.move_down();
                }
            }
            MouseEventKind::ScrollUp => {
                if self.screen == Screen::Reader {
                    self.scroll = self.scroll.saturating_sub(3);
                } else {
                    self.move_up();
                }
            }
            MouseEventKind::Down(MouseButton::Left) if self.screen == Screen::Home => {
                if inside(self.groups_rect, event.column, event.row) {
                    let row =
                        (event.row - self.groups_rect.y) as usize + self.groups_offset as usize;
                    if row < self.lib.groups.len() {
                        self.group = row;
                        self.topic = 0;
                        self.focus = Focus::Groups;
                    }
                } else if inside(self.topics_rect, event.column, event.row) {
                    let row =
                        (event.row - self.topics_rect.y) as usize + self.topics_offset as usize;
                    if row < self.group_topics().len() {
                        self.topic = row;
                        self.focus = Focus::Topics;
                    }
                } else if self.preview_visible && inside(self.preview_rect, event.column, event.row)
                {
                    // The index owns the right edge of the preview column.
                    let index_edge =
                        self.preview_rect.x + self.preview_rect.width - self.preview_contents;
                    self.focus = if event.column >= index_edge && self.contents_visible() {
                        Focus::Contents
                    } else {
                        Focus::Preview
                    };
                }
            }
            _ => {}
        }
    }

    fn move_down(&mut self) {
        match self.focus {
            Focus::Groups => {
                if self.group + 1 < self.lib.groups.len() {
                    self.group += 1;
                    self.topic = 0;
                }
            }
            Focus::Topics => {
                if self.topic + 1 < self.group_topics().len() {
                    self.topic += 1;
                }
            }
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_add(1),
            Focus::Contents => self.step_preview_section(1),
        }
    }

    fn move_up(&mut self) {
        match self.focus {
            Focus::Groups => {
                if self.group > 0 {
                    self.group -= 1;
                    self.topic = 0;
                }
            }
            Focus::Topics => {
                self.topic = self.topic.saturating_sub(1);
            }
            Focus::Preview => self.preview_scroll = self.preview_scroll.saturating_sub(1),
            Focus::Contents => self.step_preview_section(-1),
        }
    }

    /// Columns read left to right; the preview only joins when the width pays for it.
    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Topics,
            Focus::Topics if self.preview_visible => Focus::Preview,
            Focus::Preview if self.contents_visible() => Focus::Contents,
            other => other,
        };
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Groups,
            Focus::Topics => Focus::Groups,
            Focus::Preview => Focus::Topics,
            Focus::Contents => Focus::Preview,
        };
    }

    /// The index inside the preview only exists when the column exists and the topic has sections.
    pub fn contents_visible(&self) -> bool {
        self.preview_visible && !self.preview_sections.is_empty()
    }

    /// Move the preview's contents selection and take the preview with it.
    fn step_preview_section(&mut self, delta: isize) {
        if self.preview_sections.is_empty() {
            return;
        }
        let last = self.preview_sections.len() - 1;
        let current = reader::section_at(&self.preview_sections, self.preview_scroll);
        let next = (current as isize + delta).clamp(0, last as isize) as usize;
        self.preview_scroll = self.preview_sections[next].1;
    }

    /// Move the contents selection and take the document with it.
    fn step_section(&mut self, delta: i32) {
        if self.sections.is_empty() {
            return;
        }
        let next = self.toc_index as i32 + delta;
        let last = self.sections.len() as i32 - 1;
        self.toc_index = next.clamp(0, last) as usize;
        self.scroll = self.sections[self.toc_index].1;
    }

    fn open(&mut self) {
        let doc = self.current().map(reader::build);
        if let Some(doc) = doc {
            self.lines = doc.lines;
            self.sections = doc.sections;
            self.toc_index = 0;
            self.toc_focus = false;
            self.scroll = 0;
            self.screen = Screen::Reader;
        }
    }

    fn toggle_mouse(&mut self) {
        self.mouse = !self.mouse;
        let mut out = std::io::stdout();
        let _ = if self.mouse {
            execute!(out, EnableMouseCapture)
        } else {
            execute!(out, DisableMouseCapture)
        };
    }
}

/// The minimum-size refusal, drawn in the middle of whatever room is left.
fn size_warning(frame: &mut Frame, area: Rect) {
    let text = crate::size_warning(area.width, area.height);
    // One line when it fits; otherwise three so the wrap has somewhere to land.
    let want = if text.chars().count() as u16 <= area.width {
        1
    } else {
        3
    };
    let height = want.min(area.height);
    let block = Rect {
        y: area.y + area.height.saturating_sub(height) / 2,
        height,
        ..area
    };
    // After a shrink the frame's own cells still hold the bigger layout: blank them,
    // or the diff re-emits the old screen around the warning.
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(Line::from(Span::styled(
            text,
            Style::new().fg(crate::theme::fg_dim()),
        )))
        .alignment(Alignment::Center)
        .wrap(Wrap { trim: true }),
        block,
    );
}
