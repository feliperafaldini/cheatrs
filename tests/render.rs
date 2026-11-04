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
fn search_finds_a_topic_and_opens_it() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);

    app.on_key(key(KeyCode::Char('/')));
    assert!(app.searching, "/ opens the modal");
    for character in "docker".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    let screen = render(&mut app, 140, 30);
    assert!(screen.contains("Docker"), "results must show: {screen}");

    app.on_key(key(KeyCode::Enter));
    assert!(!app.searching, "committing closes the modal");
    assert_eq!(
        app.screen,
        ui::Screen::Reader,
        "a topic result opens the reader"
    );
    assert!(app.sections.len() > 5);
}

#[test]
fn search_finds_a_section_and_lands_on_it() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);

    app.on_key(key(KeyCode::Char('/')));
    for character in "data types".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    assert!(!app.results.is_empty(), "a section title must be findable");

    let entry_index = app.results[0];
    let expected = app.index.entries[entry_index]
        .section
        .expect("first hit is a section");
    app.on_key(key(KeyCode::Enter));

    assert_eq!(app.screen, ui::Screen::Reader);
    let start = app.sections[expected].1;
    assert_eq!(app.scroll, start, "the reader lands on the matched section");
}

#[test]
fn search_escapes_without_navigating() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    let group_before = app.group;

    app.on_key(key(KeyCode::Char('/')));
    app.on_key(key(KeyCode::Char('z')));
    app.on_key(key(KeyCode::Char('z')));
    app.on_key(key(KeyCode::Esc));

    assert!(!app.searching);
    assert_eq!(app.screen, ui::Screen::Home);
    assert_eq!(app.group, group_before);
    assert_eq!(app.query, "");
}

#[test]
fn enter_on_a_group_steps_into_its_topics() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    assert_eq!(app.focus, ui::Focus::Groups);

    // Move to TOOLS, which carries more than one topic, then step into its list.
    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Enter));
    assert_eq!(
        app.screen,
        ui::Screen::Home,
        "picking a group must not open a topic"
    );
    assert_eq!(app.focus, ui::Focus::Topics, "focus steps into the list");

    // Arrows now walk the topics of that group.
    app.on_key(key(KeyCode::Down));
    assert_eq!(app.topic, 1, "j/k move inside the topic column");
    app.on_key(key(KeyCode::Char('k')));
    assert_eq!(app.topic, 0);
    assert_eq!(app.group, 1, "the group stays put");
}

#[test]
fn arrow_right_from_the_topics_reaches_the_document() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Enter)); // into topics
    app.on_key(key(KeyCode::Right)); // into the preview
    assert_eq!(app.focus, ui::Focus::Preview);
    app.on_key(key(KeyCode::Enter));
    assert_eq!(
        app.screen,
        ui::Screen::Reader,
        "Enter in the preview reads the topic"
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

    // 200 columns wide: text stops at the measure and the contents column fits beside it.
    let mut terminal = Terminal::new(TestBackend::new(200, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let block = cheatrs::ui::layout::READING_WIDTH + cheatrs::ui::reader::TOC_COLUMN + 1;
    let left = (200 - block) / 2;
    let mut widest = 0;
    for y in 0..buffer.area.height {
        for x in 0..buffer.area.width {
            if buffer[(x, y)].symbol() != " " {
                widest = widest.max(x);
            }
        }
    }
    assert!(
        widest < left + block,
        "content must stay inside the centred block, widest column: {widest}, block ends at {}",
        left + block
    );
    assert!(widest > left, "something has to be drawn");
}

#[test]
fn contents_sits_left_of_the_text() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Tab));
    app.on_key(key(KeyCode::Enter));

    let screen = render(&mut app, 140, 30);
    let first = screen.find("Package Management").expect("contents entry");
    let heading = screen
        .rfind("Package Management")
        .expect("document heading");
    assert!(first < heading, "contents must come before the text");

    let line_start = screen[..first].rfind('\n').map(|i| i + 1).unwrap_or(0);
    assert!(
        first - line_start < 40,
        "contents column must sit on the left, at column {}",
        first - line_start
    );
}

#[test]
fn arrow_right_reaches_the_preview_and_scrolls_it() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);

    app.on_key(key(KeyCode::Right)); // groups -> topics
    app.on_key(key(KeyCode::Right)); // topics -> preview
    assert_eq!(
        app.focus,
        ui::Focus::Preview,
        "right arrow must reach the preview"
    );

    app.on_key(key(KeyCode::Down));
    app.on_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.preview_scroll, 2,
        "arrows and j both scroll the preview"
    );

    let screen = render(&mut app, 140, 30);
    assert!(
        screen.contains("scroll preview"),
        "footer follows the focus: {screen}"
    );

    app.on_key(key(KeyCode::Char('h')));
    assert_eq!(app.focus, ui::Focus::Topics, "h goes back one column");
}

#[test]
fn preview_focus_is_skipped_when_the_column_is_hidden() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 100, 30);
    app.on_key(key(KeyCode::Right));
    app.on_key(key(KeyCode::Right));
    assert_eq!(
        app.focus,
        ui::Focus::Topics,
        "no preview column, nowhere to go"
    );
    assert_eq!(app.preview_scroll, 0);
}

#[test]
fn contents_column_lists_sections_and_jumps() {
    let mut app = ui::App::new(data::load());
    app.on_key(key(KeyCode::Tab));
    app.on_key(key(KeyCode::Enter));

    let screen = render(&mut app, 140, 30);
    assert!(
        screen.contains("Basic Syntax"),
        "contents lists sections: {screen}"
    );
    assert!(
        app.sections.len() > 10,
        "python has 23 sections, got {}",
        app.sections.len()
    );

    app.on_key(key(KeyCode::Char('G')));
    render(&mut app, 140, 30);
    let deep = app.scroll;
    app.on_key(key(KeyCode::Tab)); // focus the contents
    app.on_key(key(KeyCode::Char('k'))); // one section up
    assert!(
        app.scroll < deep,
        "jumping must move the document: {} < {}",
        app.scroll,
        deep
    );
    assert_eq!(
        app.scroll, app.sections[app.toc_index].1,
        "scroll lands on the section start"
    );
}
