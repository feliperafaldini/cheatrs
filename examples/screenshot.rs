//! Prints the two screens as plain text. For checking layout without a terminal.
//!
//!   cargo run --example screenshot

use cheatrs::{data, ui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;

fn show(app: &mut ui::App, label: &str, width: u16, height: u16) {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    println!("── {label} ──");
    for y in 0..buffer.area.height {
        let mut row = String::new();
        for x in 0..buffer.area.width {
            row.push_str(buffer[(x, y)].symbol());
        }
        println!("{}", row.trim_end());
    }
    println!();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let width: u16 = args.get(1).and_then(|v| v.parse().ok()).unwrap_or(120);
    let height: u16 = args.get(2).and_then(|v| v.parse().ok()).unwrap_or(30);

    let mut app = ui::App::new(data::load());
    show(&mut app, "home", width, height);
    app.on_key(KeyEvent::new(KeyCode::Tab, KeyModifiers::NONE));
    app.on_key(KeyEvent::new(KeyCode::Enter, KeyModifiers::NONE));
    show(&mut app, "reader", width, height);
}
