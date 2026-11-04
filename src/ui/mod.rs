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
use ratatui::layout::Rect;
use ratatui::text::Line;

use crate::data::Library;
use crate::model::Topic;
use crate::search::{Hit, Index as SearchIndex};

#[derive(Clone, Copy, PartialEq, Eq, Debug)]
pub enum Focus {
    Groups,
    Topics,
    Preview,
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
}

impl App {
    pub fn new(lib: Library) -> Self {
        let index = SearchIndex::build(&lib);
        Self {
            lib,
            index,
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
                },
                KeyCode::Char('M') => self.toggle_mouse(),
                KeyCode::Char('/') => self.start_search(),
                _ => {}
            },
            Screen::Reader => match key.code {
                KeyCode::Char('q') | KeyCode::Left => self.screen = Screen::Home,
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
                    self.focus = Focus::Preview;
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
        }
    }

    /// Columns read left to right; the preview only joins when the width pays for it.
    fn focus_next(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Topics,
            Focus::Topics if self.preview_visible => Focus::Preview,
            Focus::Topics | Focus::Preview => self.focus,
        };
    }

    fn focus_prev(&mut self) {
        self.focus = match self.focus {
            Focus::Groups => Focus::Groups,
            Focus::Topics => Focus::Groups,
            Focus::Preview => Focus::Topics,
        };
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
