use cheatrs::ui;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    let library = cheatrs::data::load();
    if library.groups.is_empty() {
        eprintln!("cheatrs: no topics loaded");
        return Ok(());
    }

    let (cols, rows) = crossterm::terminal::size()?;
    if !cheatrs::fits(cols, rows) {
        eprintln!(
            "cheatrs needs at least {}x{} (terminal is {cols}x{rows})",
            cheatrs::MIN_WIDTH,
            cheatrs::MIN_HEIGHT
        );
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, library);
    ratatui::restore();
    result
}

fn run(terminal: &mut DefaultTerminal, library: cheatrs::data::Library) -> std::io::Result<()> {
    let mut app = ui::App::new(library);
    while !app.quit {
        terminal.draw(|frame| app.draw(frame))?;
        match event::read()? {
            Event::Key(key) if key.kind == KeyEventKind::Press => app.on_key(key),
            Event::Mouse(mouse) if app.mouse => app.on_mouse(mouse),
            _ => {}
        }
    }
    Ok(())
}
