use std::io::IsTerminal;

use cheatrs::ui;
use crossterm::event::{self, Event, KeyEventKind};
use ratatui::DefaultTerminal;

fn main() -> std::io::Result<()> {
    let args: Vec<String> = std::env::args().skip(1).collect();

    if args.iter().any(|arg| arg == "--version" || arg == "-V") {
        println!("cheatrs {}", env!("CARGO_PKG_VERSION"));
        return Ok(());
    }

    if args.iter().any(|arg| arg == "--help" || arg == "-h") {
        println!(
            "cheatrs — terminal browser for programming cheatsheets\n\n\
             usage:\n  cheatrs                  open the browser\n  \
             cheatrs --version        print the version and exit\n  \
             cheatrs --validate       check every topic the binary can see\n  \
             cheatrs --validate FILE  check a single topic file\n\n\
             keys: hjkl/arrows move · tab or left/right change column · enter open · \
             / search · M mouse · q quit\n\n\
             topics you write belong in $XDG_DATA_HOME/cheatrs/topics/*.json; \
             data/template.json is the shape."
        );
        return Ok(());
    }

    if let Some(position) = args.iter().position(|arg| arg == "--validate") {
        validate(args.get(position + 1).map(String::as_str));
        return Ok(());
    }

    // Drawing into anything but a terminal is a crash, not a degraded mode: say so and leave.
    if !std::io::stdin().is_terminal() || !std::io::stdout().is_terminal() {
        eprintln!("cheatrs: needs a terminal, this is a pipe or a file");
        std::process::exit(1);
    }

    let library = cheatrs::data::load();
    if library.groups.is_empty() {
        eprintln!("cheatrs: no topics loaded");
        return Ok(());
    }

    let (cols, rows) = crossterm::terminal::size()?;
    if !cheatrs::fits(cols, rows) {
        eprintln!("{}", cheatrs::size_warning(cols, rows));
        return Ok(());
    }

    let mut terminal = ratatui::init();
    let result = run(&mut terminal, library);
    ratatui::restore();
    result
}

/// Says what is wrong, topic by topic, and exits non-zero when something is.
fn validate(target: Option<&str>) {
    if let Some(path) = target {
        match cheatrs::data::validate_file(std::path::Path::new(path)) {
            Ok((topic, dropped)) => {
                println!(
                    "{}: ok — {} sections, {} snippets{}",
                    topic.slug,
                    topic.sections.len(),
                    topic.snippet_count(),
                    if dropped > 0 {
                        format!(", {dropped} section(s) would be dropped as empty")
                    } else {
                        String::new()
                    }
                );
            }
            Err(problem) => {
                eprintln!("{problem}");
                std::process::exit(1);
            }
        }
        return;
    }

    let (checked, problems) = cheatrs::data::validate_all();
    if problems.is_empty() {
        println!("ok — {checked} topics validated");
        return;
    }
    for problem in &problems {
        eprintln!("{problem}");
    }
    eprintln!("{} problem(s) in {checked} topics", problems.len());
    std::process::exit(1);
}

fn run(terminal: &mut DefaultTerminal, library: cheatrs::data::Library) -> std::io::Result<()> {
    let mut app = ui::App::new(library);
    while !app.quit {
        terminal.draw(|frame| app.draw(frame))?;
        // A flash needs the clock to advance; everything else waits for a key.
        let pending = app.flash_until.is_some();
        if pending {
            if event::poll(std::time::Duration::from_millis(40))? {
                step(&mut app)?;
            }
        } else {
            step(&mut app)?;
        }
    }
    Ok(())
}

/// One event. The terminal's size is read by the next `draw`, which resizes its
/// buffers and repaints whole — so a resize only has to let the loop turn: the
/// screen is laid out again, or shows the minimum-size warning if it no longer fits.
/// Below the minimum the screen is wiped first, because ratatui only erases on a
/// horizontal shrink and the warning must not sit on top of the layout it replaced.
/// `Terminal::clear` is avoided on purpose: it asks the terminal where the cursor is
/// and blocks until answered.
fn step(app: &mut ui::App) -> std::io::Result<()> {
    match event::read()? {
        Event::Resize(cols, rows) if !cheatrs::fits(cols, rows) => {
            crossterm::execute!(
                std::io::stdout(),
                crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
            )?;
        }
        Event::Resize(..) => {}
        other => handle(app, other),
    }
    Ok(())
}

fn handle(app: &mut ui::App, event: Event) {
    match event {
        Event::Key(key) if key.kind == KeyEventKind::Press => app.on_key(key),
        Event::Mouse(mouse) if app.mouse => app.on_mouse(mouse),
        _ => {}
    }
}
