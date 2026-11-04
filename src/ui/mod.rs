pub mod home;
pub mod layout;
pub mod reader;

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

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum Focus {
    Groups,
    Topics,
}

#[derive(Clone, Copy, PartialEq, Eq)]
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
    /// Preview of the selected topic: built once per selection, never per frame.
    pub preview: Vec<Line<'static>>,
    pub preview_topic: Option<usize>,
    pub groups_rect: Rect,
    pub topics_rect: Rect,
    pub groups_offset: u16,
    pub topics_offset: u16,
}

impl App {
    pub fn new(lib: Library) -> Self {
        Self {
            lib,
            focus: Focus::Groups,
            group: 0,
            topic: 0,
            screen: Screen::Home,
            scroll: 0,
            mouse: false,
            quit: false,
            lines: Vec::new(),
            preview: Vec::new(),
            preview_topic: None,
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

    /// Rebuild the preview only when the selection actually moved.
    pub fn sync_preview(&mut self) {
        let index = self.group_topics().get(self.topic).copied();
        if index == self.preview_topic {
            return;
        }
        let lines = match index {
            Some(i) => reader::build_capped(&self.lib.topics[i], 240),
            None => Vec::new(),
        };
        self.preview = lines;
        self.preview_topic = index;
    }

    pub fn draw(&mut self, frame: &mut Frame) {
        match self.screen {
            Screen::Home => home::draw(frame, frame.area(), self),
            Screen::Reader => reader::draw(frame, frame.area(), self),
        }
    }

    pub fn on_key(&mut self, key: KeyEvent) {
        match self.screen {
            Screen::Home => match key.code {
                KeyCode::Char('q') => self.quit = true,
                KeyCode::Char('j') | KeyCode::Down => self.move_down(),
                KeyCode::Char('k') | KeyCode::Up => self.move_up(),
                KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => self.focus = Focus::Topics,
                KeyCode::Left | KeyCode::Char('h') | KeyCode::Esc => self.focus = Focus::Groups,
                KeyCode::Enter => self.open(),
                KeyCode::Char('M') => self.toggle_mouse(),
                _ => {}
            },
            Screen::Reader => match key.code {
                KeyCode::Char('q') | KeyCode::Esc | KeyCode::Left => self.screen = Screen::Home,
                KeyCode::Char('j') | KeyCode::Down => self.scroll = self.scroll.saturating_add(1),
                KeyCode::Char('k') | KeyCode::Up => self.scroll = self.scroll.saturating_sub(1),
                KeyCode::Char('g') => self.scroll = 0,
                KeyCode::Char('G') => self.scroll = u16::MAX,
                KeyCode::Char('M') => self.toggle_mouse(),
                _ => {}
            },
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
        }
    }

    fn open(&mut self) {
        let lines = self.current().map(reader::build);
        if let Some(lines) = lines {
            self.lines = lines;
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
