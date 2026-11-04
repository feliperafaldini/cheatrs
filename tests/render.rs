use cheatrs::{data, theme, ui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

fn render(app: &mut ui::App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();
    let mut out = String::new();
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            out.push_str(buffer[(x, y)].symbol());
        }
        out.push('\n');
    }
    out
}

fn key(code: KeyCode) -> KeyEvent {
    KeyEvent::new(code, KeyModifiers::NONE)
}

#[test]
fn loads_the_four_real_topics() {
    let library = data::load();
    assert_eq!(library.topics.len(), 4, "four converted topics must load");
    assert!(
        library.skipped == 0,
        "nothing should be skipped: {}",
        library.skipped
    );
    let total: usize = library.topics.iter().map(|t| t.snippet_count()).sum();
    assert_eq!(
        total, 111,
        "110 snippets of the 4 topics plus one flat-shape section"
    );
}

#[test]
fn home_shows_groups_and_topics() {
    let mut app = ui::App::new(data::load());
    let screen = render(&mut app, 120, 30);
    assert!(screen.contains("LANGUAGES"), "{screen}");
    assert!(
        screen.contains("DATABASES") || screen.contains("DATABASE"),
        "{screen}"
    );
    assert!(screen.contains("Python"), "{screen}");
    assert!(
        screen.contains("move"),
        "footer must carry the keys: {screen}"
    );
}

#[test]
fn enter_opens_the_reader_with_code() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Tab));
    app.on_key(key(KeyCode::Enter));
    let screen = render(&mut app, 120, 30);
    assert!(
        screen.contains("‹ LANGUAGE / Python"),
        "breadcrumb: {screen}"
    );
    assert!(screen.contains("Basic Syntax"), "first section: {screen}");
    assert!(
        screen.contains("print("),
        "code block must render: {screen}"
    );
}

#[test]
fn scroll_moves_and_clamps_at_the_end() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Enter));
    app.on_key(key(KeyCode::Char('G')));
    render(&mut app, 120, 30);
    let max = app.scroll;
    assert!(max > 0, "G must land past the first page");
    app.on_key(key(KeyCode::Char('k')));
    assert_eq!(app.scroll, max - 1);
    app.on_key(key(KeyCode::Char('g')));
    assert_eq!(app.scroll, 0);
}

#[test]
fn q_goes_back_to_home() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Enter));
    app.on_key(key(KeyCode::Char('q')));
    assert!(!app.quit, "q inside the reader must not quit the app");
    let screen = render(&mut app, 120, 30);
    assert!(screen.contains("LANGUAGES"), "{screen}");
}

#[test]
fn refused_below_eighty_by_twentyfour() {
    assert!(cheatrs::fits(80, 24));
    assert!(!cheatrs::fits(79, 24));
    assert!(!cheatrs::fits(80, 23));
}

#[test]
fn accent_normalizes_the_unreadable_blue() {
    // #264de4 measures 2.92:1 on the dark ground; the rule pulls it to 4.80:1.
    assert_eq!(theme::accent("#264de4"), Color::Rgb(0x62, 0x7b, 0xda));
    assert_eq!(theme::accent("not a colour"), theme::FG_DIM);
}

#[test]
fn reader_colours_the_code() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Tab));
    app.on_key(key(KeyCode::Enter));

    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let palette = [
        theme::FG,
        theme::FG_DIM,
        theme::FG_FAINT,
        theme::BG,
        theme::SURFACE,
        theme::RULE,
        Color::Reset,
    ];
    let mut coloured = 0;
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            let fg = buffer[(x, y)].fg;
            if !palette.contains(&fg) {
                coloured += 1;
            }
        }
    }
    assert!(
        coloured > 50,
        "syntax highlight must paint the code, foreign-coloured cells: {coloured}"
    );
}

#[test]
fn preview_appears_only_when_the_width_pays_for_it() {
    let mut app = ui::App::new(data::load());
    let wide = render(&mut app, 140, 30);
    assert!(
        wide.contains("pip install"),
        "preview must show the topic: {wide}"
    );

    let mut narrow = ui::App::new(data::load());
    let narrow_screen = render(&mut narrow, 100, 30);
    assert!(
        !narrow_screen.contains("pip install"),
        "no preview column under the threshold: {narrow_screen}"
    );
    assert!(
        narrow_screen.contains("Python"),
        "the list stays: {narrow_screen}"
    );
}

#[test]
fn preview_follows_the_selection_without_the_reader() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Down)); // TOOLS
    app.on_key(key(KeyCode::Tab));
    let screen = render(&mut app, 140, 30);
    assert!(
        screen.contains("Docker"),
        "preview follows the topic: {screen}"
    );
    assert!(
        !screen.contains("pip install"),
        "previous preview must be gone: {screen}"
    );
}

#[test]
fn reading_measure_stays_under_a_hundred_columns() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Tab));
    app.on_key(key(KeyCode::Enter));

    // 200 columns wide: text must still stop at the reading measure.
    let mut terminal = Terminal::new(TestBackend::new(200, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let mut widest = 0;
    for y in 0..buffer.area.height {
        let mut last = 0;
        for x in 0..buffer.area.width {
            if buffer[(x, y)].symbol() != " " {
                last = x;
            }
        }
        widest = widest.max(last);
    }
    assert!(
        widest > 0 && widest - 50 <= cheatrs::ui::layout::READING_WIDTH,
        "content must stay inside the measure, widest column: {widest}"
    );
}
