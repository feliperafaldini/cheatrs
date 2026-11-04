use cheatrs::{data, theme, ui};
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;

fn screen_text(terminal: &Terminal<TestBackend>) -> String {
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

fn render(app: &mut ui::App, width: u16, height: u16) -> String {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    screen_text(&terminal)
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
    assert!(
        library.topics.len() >= 48,
        "the library must never shrink below the original 48, got {}",
        library.topics.len()
    );
    assert!(
        library.skipped == 0,
        "nothing should be skipped: {}",
        library.skipped
    );
    let total: usize = library.topics.iter().map(|t| t.snippet_count()).sum();
    // Growth during the rewrite is expected; the floor is what matters.
    assert!(
        total >= 1100,
        "the dataset must stay substantial, got {total}"
    );
    assert!(
        library.topics.iter().all(|topic| topic.sections.len() >= 5),
        "every topic needs real sections"
    );
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

    let entry_index = app.results[0].entry;
    let expected = app.index.entries[entry_index]
        .section
        .expect("first hit is a section");
    app.on_key(key(KeyCode::Enter));

    assert_eq!(app.screen, ui::Screen::Reader);
    let start = app.sections[expected].1;
    assert_eq!(app.scroll, start, "the reader lands on the matched section");
}

#[test]
fn search_finds_a_command_inside_the_code() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);

    app.on_key(key(KeyCode::Char('/')));
    for character in "pip install -r".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }

    assert!(
        !app.results.is_empty(),
        "the flag inside the code must be findable"
    );
    let hit = &app.results[0];
    let excerpt = hit.excerpt.as_deref().expect("a code hit carries the line");
    assert!(excerpt.contains("pip install"), "excerpt: {excerpt}");

    let section = app.index.entries[hit.entry]
        .section
        .expect("code lives in a section");
    app.on_key(key(KeyCode::Enter));
    assert_eq!(app.screen, ui::Screen::Reader);
    assert_eq!(
        app.scroll, app.sections[section].1,
        "the reader lands on the section holding the line"
    );
}

#[test]
fn search_list_scrolls_with_the_cursor() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Char('/')));
    for character in "pip".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    assert!(
        app.results.len() > 14,
        "need a long list, got {}",
        app.results.len()
    );

    for _ in 0..15 {
        app.on_key(key(KeyCode::Down));
    }
    let chosen = app.index.entries[app.results[app.cursor].entry]
        .title
        .clone();
    let screen = render(&mut app, 140, 30);
    assert!(
        screen.contains(&chosen),
        "the cursor sits on {chosen} and it must be visible: {screen}"
    );
}

#[test]
fn jumping_from_search_marks_the_landing_line() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Char('/')));
    for character in "data types".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    app.on_key(key(KeyCode::Enter));

    let line = app.flash_line.expect("a jump sets the landing line");
    assert_eq!(
        Some(line),
        Some(app.scroll),
        "the flash sits on the scroll target"
    );
    assert!(app.flash_until.is_some(), "and it has an expiry");
}

#[test]
fn rust_topic_carries_the_sections_the_source_lacked() {
    let library = data::load();
    let rust = library
        .topics
        .iter()
        .find(|topic| topic.slug == "rust")
        .expect("rust topic");
    let titles: Vec<&str> = rust.sections.iter().map(|s| s.title.as_str()).collect();

    assert!(
        rust.sections.len() >= 22,
        "got {} sections",
        rust.sections.len()
    );
    for expected in [
        "Lifetimes",
        "Cargo",
        "Testing",
        "Concurrency",
        "Smart Pointers",
    ] {
        assert!(
            titles.contains(&expected),
            "missing section {expected}: {titles:?}"
        );
    }
    let errors = rust
        .sections
        .iter()
        .find(|s| s.title == "Error Handling")
        .expect("error handling");
    assert!(
        errors.snippets.len() >= 2,
        "error handling must show ? and the custom enum"
    );
}

#[test]
fn the_shipped_data_validates_clean() {
    let (checked, problems) = cheatrs::data::validate_all();
    assert!(
        checked >= 48,
        "the embedded topics must be checked, got {checked}"
    );
    assert!(problems.is_empty(), "{problems:?}");
}

#[test]
fn validation_points_at_the_broken_section() {
    let dir = std::path::Path::new(env!("CARGO_TARGET_TMPDIR"));
    let good = dir.join("ok.json");
    std::fs::write(&good, include_str!("../data/template.json")).unwrap();
    let (topic, _) = cheatrs::data::validate_file(&good).expect("the template is valid");
    assert_eq!(topic.slug, "my-topic");

    let broken = dir.join("broken.json");
    std::fs::write(
        &broken,
        r##"{"slug":"x","title":"X","label":"X","group":"Tool","color":"#fff",
            "sections":[{"id":"a","title":"A","description":"","snippets":[]}]}"##,
    )
    .unwrap();
    let problem = cheatrs::data::validate_file(&broken).expect_err("empty section is invalid");
    assert!(
        problem.contains("no section carries a snippet"),
        "the message must say what is wrong: {problem}"
    );
}

#[test]
fn aliases_land_on_the_topic_people_mean() {
    let index = cheatrs::search::Index::build(&data::load());

    let psql = index.query("psql");
    assert!(!psql.is_empty(), "psql must resolve to something");
    assert_eq!(
        index.entries[psql[0].entry].title, "PostgreSQL",
        "psql resolves to the PostgreSQL topic"
    );

    for (alias, expected) in [("ts", "TypeScript"), ("py", "Python"), ("nvim", "Vim")] {
        let hits = index.query(alias);
        assert!(!hits.is_empty(), "{alias} must resolve");
        assert_eq!(
            index.entries[hits[0].entry].title, expected,
            "{alias} should point at {expected}"
        );
    }
}

#[test]
fn unknown_words_are_searched_as_typed() {
    let index = cheatrs::search::Index::build(&data::load());
    assert!(
        index.query("zzzznotaword").is_empty(),
        "an unknown word finds nothing instead of everything"
    );
    assert!(
        !index.query("c++").is_empty(),
        "symbols survive the alias lookup"
    );
}

#[test]
fn code_search_stays_under_a_keystroke() {
    let index = cheatrs::search::Index::build(&data::load());
    let started = std::time::Instant::now();
    let mut hits = 0;
    for query in [
        "p", "pi", "pip", "pip ", "pip i", "pip in", "pip ins", "pip inst",
    ] {
        hits += index.query(query).len();
    }
    let elapsed = started.elapsed();
    assert!(hits > 0);
    eprintln!("oito consultas em {elapsed:?} ({hits} resultados)");
    assert!(
        elapsed.as_millis() < 400,
        "eight keystrokes over the whole dataset took {elapsed:?}"
    );
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
    use cheatrs::theme::Depth;
    // #264de4 measures 2.92:1 on the dark ground; the rule pulls it to 4.80:1.
    assert_eq!(
        theme::accent_at(Depth::True, "#264de4"),
        Color::Rgb(0x62, 0x7b, 0xda)
    );
    assert_eq!(
        theme::accent_at(Depth::True, "not a colour"),
        Color::Rgb(0x8b, 0x94, 0x9a)
    );
}

#[test]
fn no_colour_drops_every_hue() {
    use cheatrs::theme::{Depth, paint_with};
    assert_eq!(
        paint_with(Depth::Mono, Color::Rgb(0x62, 0x7b, 0xda)),
        Color::Reset
    );
    assert_eq!(paint_with(Depth::Mono, theme::fg()), Color::Reset);
    assert_eq!(
        theme::accent_at(Depth::Mono, "#264de4"),
        Color::Reset,
        "a monochrome terminal gets no accent at all"
    );
}

#[test]
fn sixteen_colour_maps_to_ansi_and_never_to_rgb() {
    use cheatrs::theme::Depth;
    let mapped = theme::accent_at(Depth::Ansi16, "#264de4");
    assert!(
        !matches!(mapped, Color::Rgb(..)),
        "16-colour terminals must not receive RGB, got {mapped:?}"
    );
}

#[test]
fn two_hundred_and_fifty_six_colour_uses_the_palette() {
    use cheatrs::theme::Depth;
    let mapped = theme::paint_with(Depth::Ansi256, Color::Rgb(0x62, 0x7b, 0xda));
    assert!(
        matches!(mapped, Color::Indexed(_)),
        "256-colour terminals get an indexed colour, got {mapped:?}"
    );
}

#[test]
fn reader_colours_the_code() {
    let mut app = ui::App::new(data::load());
    open_topic(&mut app, "python");

    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let buffer = terminal.backend().buffer();

    let palette = [
        theme::fg(),
        theme::fg_dim(),
        theme::fg_faint(),
        theme::bg(),
        theme::surface(),
        theme::rule(),
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
fn contents_sits_right_of_the_text() {
    let mut app = ui::App::new(data::load());
    open_topic(&mut app, "python");
    let heading = app
        .sections
        .iter()
        .map(|(title, _)| title.clone())
        .find(|title| title.len() <= 14)
        .expect("a short section title");

    let screen = render(&mut app, 140, 30);
    let column = screen
        .lines()
        .find_map(|line| line.find(&heading).filter(|col| *col > 100))
        .unwrap_or_else(|| panic!("contents column must sit on the right:\n{screen}"));
    assert!(column > 100, "index at column {column}");
}

#[test]
fn arrow_right_moves_the_focus_to_the_contents() {
    let mut app = ui::App::new(data::load());
    open_topic(&mut app, "python");
    render(&mut app, 140, 30);
    assert!(!app.toc_focus, "the text starts focused");

    app.on_key(key(KeyCode::Right));
    assert!(
        app.toc_focus,
        "the arrow that walks right reaches the index"
    );
    assert_eq!(app.screen, ui::Screen::Reader, "and it stays in the reader");

    let deep = app.scroll;
    app.on_key(key(KeyCode::Char('j')));
    assert!(
        app.scroll > deep,
        "j inside the index steps a section: {} > {}",
        app.scroll,
        deep
    );

    app.on_key(key(KeyCode::Left));
    assert!(!app.toc_focus, "left hands the focus back to the text");
    assert_eq!(
        app.screen,
        ui::Screen::Reader,
        "and does not leave the reader"
    );
}

#[test]
fn preview_draws_the_contents_right_of_its_text() {
    let mut app = ui::App::new(data::load());
    let heading = app
        .current()
        .expect("a topic is selected")
        .sections
        .iter()
        .map(|section| section.title.clone())
        .find(|title| title.len() <= 14)
        .expect("a short section title");
    let screen = render(&mut app, 140, 30);

    // The preview index owns the rightmost 16 cells of its column, so a title
    // starting past column 120 can only be that column and not the document.
    assert!(
        screen
            .lines()
            .any(|line| line.find(&heading).is_some_and(|col| col >= 120)),
        "preview index must sit on the right:\n{screen}"
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

#[test]
fn four_columns_are_reached_in_order_and_left_in_reverse() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    assert_eq!(app.focus, ui::Focus::Groups);

    app.on_key(key(KeyCode::Right));
    assert_eq!(app.focus, ui::Focus::Topics);
    app.on_key(key(KeyCode::Right));
    assert_eq!(app.focus, ui::Focus::Preview, "third column: the text");
    app.on_key(key(KeyCode::Right));
    assert_eq!(app.focus, ui::Focus::Contents, "fourth column: the index");
    app.on_key(key(KeyCode::Right));
    assert_eq!(app.focus, ui::Focus::Contents, "and it is the last one");

    app.on_key(key(KeyCode::Char('h')));
    assert_eq!(app.focus, ui::Focus::Preview);
    app.on_key(key(KeyCode::Char('h')));
    assert_eq!(app.focus, ui::Focus::Topics);
    app.on_key(key(KeyCode::Char('h')));
    assert_eq!(app.focus, ui::Focus::Groups);
}

#[test]
fn j_in_the_contents_column_jumps_a_section_in_the_preview() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    for _ in 0..3 {
        app.on_key(key(KeyCode::Right));
    }
    assert_eq!(app.focus, ui::Focus::Contents);
    // The preview opens at the top, above the first section heading.
    assert_eq!(app.preview_scroll, 0);

    let before = app.preview_scroll;
    app.on_key(key(KeyCode::Char('j')));
    assert_eq!(
        app.preview_scroll, app.preview_sections[1].1,
        "a section jump, not a one-line scroll"
    );
    assert!(
        app.preview_scroll > before,
        "and it moved forward: {} > {before}",
        app.preview_scroll
    );

    app.on_key(key(KeyCode::Char('k')));
    assert_eq!(
        app.preview_scroll, app.preview_sections[0].1,
        "k walks back a section"
    );
}

#[test]
fn enter_on_the_contents_column_opens_the_reader_at_that_section() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    for _ in 0..3 {
        app.on_key(key(KeyCode::Right));
    }
    app.on_key(key(KeyCode::Char('j')));
    app.on_key(key(KeyCode::Char('j')));

    app.on_key(key(KeyCode::Enter));
    assert_eq!(app.screen, ui::Screen::Reader);
    assert_eq!(
        app.scroll, app.sections[app.toc_index].1,
        "the reader lands on the section the preview had selected"
    );
    assert!(
        app.toc_index >= 2,
        "third section or later, got {}",
        app.toc_index
    );
}

#[test]
fn header_carries_the_version_and_the_library_size() {
    let mut app = ui::App::new(data::load());
    let screen = render(&mut app, 140, 30);
    assert!(
        screen.contains(env!("CARGO_PKG_VERSION")),
        "the header shows the version: {screen}"
    );
    let count = format!("{} topics", app.lib.topics.len());
    assert!(
        screen.contains(&count) && screen.contains("snippets"),
        "and the size of the library: {screen}"
    );
}

#[test]
fn the_two_fixed_columns_are_measured_not_assumed() {
    let app = ui::App::new(data::load());
    assert!(
        app.group_width <= 16,
        "groups column wider than the longest label needs: {}",
        app.group_width
    );
    assert!(
        app.topics_width <= 28,
        "topics column wider than the longest title needs: {}",
        app.topics_width
    );
    assert!(
        app.group_width >= 12 && app.topics_width >= 22,
        "and never cramped"
    );
}

#[test]
fn header_names_the_skipped_files_only_when_there_are_some() {
    // The shipped library is clean: the header must read exactly as it always did.
    let mut clean = ui::App::new(data::load());
    let screen = render(&mut clean, 160, 30);
    let header = screen.lines().next().expect("a header line").to_string();
    assert!(
        !header.contains("skipped"),
        "nothing was skipped, so nothing may be said: {header}"
    );
    assert!(
        header.trim_end().ends_with("snippets"),
        "the meta ends where it always ended: {header}"
    );

    // A user file the loader had to drop must not vanish silently.
    let mut library = data::load();
    library.skipped = 3;
    let mut app = ui::App::new(library);
    let screen = render(&mut app, 160, 30);
    let header = screen.lines().next().expect("a header line").to_string();
    assert!(
        header.contains("3 skipped"),
        "the count belongs in the header: {header}"
    );
    assert!(
        header.trim_end().ends_with("3 skipped"),
        "and it is a suffix at the end of the line: {header}"
    );
    assert!(
        header.contains("snippets ·"),
        "the stats stay put in front of it: {header}"
    );
}

#[test]
fn a_terminal_that_shrank_shows_the_minimum_size_warning() {
    // One terminal, two sizes: the second frame must not keep cells of the first.
    let mut terminal = Terminal::new(TestBackend::new(120, 30)).unwrap();
    let mut app = ui::App::new(data::load());
    terminal.draw(|frame| app.draw(frame)).unwrap();

    terminal.backend_mut().resize(70, 20);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let small = screen_text(&terminal);
    assert!(
        small.contains("needs at least 80x24"),
        "the warning must be readable: {small}"
    );
    assert!(
        small.contains("70x20"),
        "and it says the real size: {small}"
    );
    assert!(
        !small.contains("LANGUAGES"),
        "the layout it replaced must be gone: {small}"
    );

    let row = small
        .lines()
        .position(|line| line.contains("needs at least"))
        .expect("the warning has a row");
    assert!(
        (8..=10).contains(&row),
        "centred on a 20-row terminal, got row {row}: {small}"
    );

    // Growing back past the minimum brings the normal screen back.
    terminal.backend_mut().resize(120, 30);
    terminal.draw(|frame| app.draw(frame)).unwrap();
    let large = screen_text(&terminal);
    assert!(!large.contains("needs at least"), "{large}");
    assert!(large.contains("LANGUAGES"), "{large}");
    assert_eq!(app.screen, ui::Screen::Home, "no mode was introduced");
}

#[test]
fn a_mistyped_title_still_finds_the_topic() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Char('/')));
    for character in "pstgrs".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    assert!(!app.results.is_empty(), "a subsequence must match");
    let first = &app.index.entries[app.results[0].entry];
    let slug = &app.lib.topics[first.topic].slug;
    assert_eq!(
        slug, "postgresql",
        "letters in order must reach the topic, got {} ({slug})",
        first.title
    );
}

#[test]
fn an_exact_prefix_still_beats_a_loose_subsequence() {
    let mut app = ui::App::new(data::load());
    render(&mut app, 140, 30);
    app.on_key(key(KeyCode::Char('/')));
    for character in "post".chars() {
        app.on_key(key(KeyCode::Char(character)));
    }
    let top = &app.index.entries[app.results[0].entry];
    let slug = &app.lib.topics[top.topic].slug;
    assert_eq!(slug, "postgresql", "prefix wins, got {}", top.title);
}

#[test]
fn a_topic_from_the_future_is_refused_with_a_reason() {
    let dir = std::env::temp_dir().join(format!("cheatrs-schema-{}", std::process::id()));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("future.json");
    std::fs::write(
        &path,
        r##"{"schema":2,"slug":"future","title":"F","label":"F","group":"Tool","color":"#ffffff",
             "desc":"d","sections":[{"id":"a","title":"A","description":"d",
             "snippets":[{"language":"bash","code":"echo hi"}]}]}"##,
    )
    .unwrap();

    let error = cheatrs::data::validate_file(&path).expect_err("schema 2 must be refused");
    assert!(
        error.contains("schema 2") && error.contains("understands 1"),
        "the message must name both versions: {error}"
    );
    std::fs::remove_dir_all(&dir).ok();
}
