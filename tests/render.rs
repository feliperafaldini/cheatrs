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

/// Put the selection on a known topic, whatever the alphabetical order does with it.
fn select_topic(app: &mut ui::App, slug: &str) {
    let index = app
        .lib
        .topics
        .iter()
        .position(|topic| topic.slug == slug)
        .unwrap_or_else(|| panic!("topic {slug} must exist"));
    for (group_index, group) in app.lib.groups.iter().enumerate() {
        if let Some(position) = group.topics.iter().position(|topic| *topic == index) {
            app.group = group_index;
            app.topic = position;
            return;
        }
    }
    panic!("topic {slug} is not listed under any group");
}

fn open_topic(app: &mut ui::App, slug: &str) {
    select_topic(app, slug);
    app.on_key(key(KeyCode::Tab)); // into the topic list
    app.on_key(key(KeyCode::Enter)); // open
}

#[test]
fn loads_the_whole_dataset() {
    let library = data::load();
    assert_eq!(
        library.topics.len(),
        48,
        "every topic of the source must load"
    );
    assert!(
        library.skipped == 0,
        "nothing should be skipped: {}",
        library.skipped
    );
    let total: usize = library.topics.iter().map(|t| t.snippet_count()).sum();
    assert_eq!(total, 1158, "snippet count of the converted data");
    assert_eq!(library.groups.len(), 6, "six groups carry topics");
}

#[test]
fn home_shows_groups_and_topics() {
    let mut app = ui::App::new(data::load());
    let first = app
        .current()
        .expect("a topic under the first group")
        .title
        .clone();
    let screen = render(&mut app, 120, 30);
    assert!(screen.contains("LANGUAGES"), "{screen}");
    assert!(
        screen.contains("DATABASES") || screen.contains("DATABASE"),
        "{screen}"
    );
    assert!(
        screen.contains(&first),
        "the topic list shows {first}: {screen}"
    );
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
    assert_eq!(
        app.current().map(|topic| topic.slug.as_str()),
        Some("docker")
    );
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

    // FRAMEWORKS carries several topics; step into its list.
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
    open_topic(&mut app, "python");
    let screen = render(&mut app, 120, 30);
    assert!(
        screen.contains("‹ LANGUAGE / Python"),
        "breadcrumb: {screen}"
    );
    assert!(app.sections.len() > 10, "sections must be indexed");
    assert!(app.lines.len() > 100, "the document must have a body");
}

#[test]
fn scroll_moves_and_clamps_at_the_end() {
    let mut app = ui::App::new(data::load());
    open_topic(&mut app, "python");
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
    open_topic(&mut app, "python");
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
    open_topic(&mut app, "python");

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
            if !palette.contains(&buffer[(x, y)].fg) {
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
    select_topic(&mut app, "docker");
    let wide = render(&mut app, 140, 30);
    assert!(
        wide.contains("docker --version"),
        "preview must show the selected topic: {wide}"
    );

    let mut narrow = ui::App::new(data::load());
    select_topic(&mut narrow, "docker");
    let narrow_screen = render(&mut narrow, 100, 30);
    assert!(
        !narrow_screen.contains("docker --version"),
        "no preview column under the threshold: {narrow_screen}"
    );
    assert!(
        narrow_screen.contains("Docker"),
        "the list stays: {narrow_screen}"
    );
}

#[test]
fn preview_follows_the_selection_without_the_reader() {
    let mut app = ui::App::new(data::load());
    select_topic(&mut app, "docker");
    let before = render(&mut app, 140, 30);
    assert!(before.contains("docker --version"), "{before}");

    select_topic(&mut app, "python");
    let after = render(&mut app, 140, 30);
    assert!(after.contains("pip install"), "preview follows: {after}");
    assert!(
        !after.contains("docker --version"),
        "the previous preview must be gone: {after}"
    );
}

#[test]
fn reading_measure_stays_under_a_hundred_columns() {
    let mut app = ui::App::new(data::load());
    open_topic(&mut app, "python");

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
    open_topic(&mut app, "python");
    let heading = app.sections[0].0.clone();

    let screen = render(&mut app, 140, 30);
    let first = screen.find(&heading).expect("contents entry");
    let body = screen.rfind(&heading).expect("document heading");
    assert!(first < body, "contents must come before the text");

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
    open_topic(&mut app, "python");

    let screen = render(&mut app, 140, 30);
    let first_section = app.sections[0].0.clone();
    assert!(
        screen.contains(&first_section),
        "contents lists sections: {screen}"
    );
    assert!(
        app.sections.len() > 10,
        "python has many sections, got {}",
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
