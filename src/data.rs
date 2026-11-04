use std::collections::BTreeMap;
use std::path::PathBuf;

use include_dir::{Dir, include_dir};

use crate::model::Topic;

static EMBEDDED: Dir<'_> = include_dir!("$CARGO_MANIFEST_DIR/data/topics");

/// Display order of the groups; anything unknown lands at the end.
pub const GROUP_ORDER: [&str; 6] = [
    "Language",
    "Framework",
    "Library",
    "Tool",
    "Database",
    "Runtime",
];

pub struct Library {
    pub topics: Vec<Topic>,
    pub groups: Vec<Group>,
    pub skipped: usize,
    /// Counted once at load so the header never walks the library per frame.
    pub sections: usize,
    pub snippets: usize,
}

pub struct Group {
    pub name: String,
    pub topics: Vec<usize>,
}

/// Embedded base, then the user directory on top: same slug, the user wins.
pub fn load() -> Library {
    let mut by_slug: BTreeMap<String, Topic> = BTreeMap::new();
    let mut skipped = 0usize;

    for file in EMBEDDED.files() {
        if file.path().extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        match parse(file.contents()) {
            Some((topic, dropped)) => {
                skipped += dropped;
                by_slug.insert(topic.slug.clone(), topic);
            }
            None => skipped += 1,
        }
    }

    if let Some(dir) = user_dir().filter(|d| d.is_dir())
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            match std::fs::read(&path) {
                Ok(bytes) => match parse(&bytes) {
                    Some((topic, dropped)) => {
                        skipped += dropped;
                        by_slug.insert(topic.slug.clone(), topic);
                    }
                    None => skipped += 1,
                },
                Err(_) => skipped += 1,
            }
        }
    }

    let mut topics: Vec<Topic> = by_slug.into_values().collect();
    topics.sort_by_key(|topic| topic.title.to_lowercase());

    let mut groups: Vec<Group> = GROUP_ORDER
        .iter()
        .map(|name| Group {
            name: (*name).to_string(),
            topics: Vec::new(),
        })
        .collect();

    for (index, topic) in topics.iter().enumerate() {
        match groups.iter_mut().find(|g| g.name == topic.group) {
            Some(group) => group.topics.push(index),
            None => {
                groups.push(Group {
                    name: topic.group.clone(),
                    topics: vec![index],
                });
            }
        }
    }
    groups.retain(|g| !g.topics.is_empty());

    let sections = topics.iter().map(|topic| topic.sections.len()).sum();
    let snippets = topics
        .iter()
        .map(|topic| {
            topic
                .sections
                .iter()
                .map(|s| s.snippets.len())
                .sum::<usize>()
        })
        .sum();

    Library {
        topics,
        groups,
        skipped,
        sections,
        snippets,
    }
}

fn parse(bytes: &[u8]) -> Option<(Topic, usize)> {
    let mut topic: Topic = serde_json::from_slice(bytes).ok()?;
    let dropped = topic.normalize();
    if topic.sections.is_empty() {
        return None;
    }
    Some((topic, dropped))
}

/// Same rules the loader applies, but it says what is wrong instead of skipping.
pub fn validate_file(path: &std::path::Path) -> Result<(Topic, usize), String> {
    let bytes = std::fs::read(path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut topic: Topic =
        serde_json::from_slice(&bytes).map_err(|error| format!("{}: {error}", path.display()))?;
    if topic.slug.trim().is_empty() {
        return Err(format!("{}: slug is empty", path.display()));
    }
    let dropped = topic.normalize();
    if topic.sections.is_empty() {
        return Err(format!(
            "{}: no section carries a snippet, the topic would never show",
            path.display()
        ));
    }
    Ok((topic, dropped))
}

/// Everything the binary can see, with one line per problem.
pub fn validate_all() -> (usize, Vec<String>) {
    let mut checked = 0;
    let mut problems = Vec::new();

    for file in EMBEDDED.files() {
        if file.path().extension().and_then(|e| e.to_str()) != Some("json") {
            continue;
        }
        checked += 1;
        let name = file.path().display().to_string();
        match serde_json::from_slice::<Topic>(file.contents()) {
            Ok(mut topic) => {
                let dropped = topic.normalize();
                if topic.sections.is_empty() {
                    problems.push(format!("{name}: no section carries a snippet"));
                }
                if dropped > 0 {
                    problems.push(format!("{name}: {dropped} section(s) dropped as empty"));
                }
            }
            Err(error) => problems.push(format!("{name}: {error}")),
        }
    }

    if let Some(dir) = user_dir().filter(|d| d.is_dir())
        && let Ok(entries) = std::fs::read_dir(&dir)
    {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.extension().and_then(|e| e.to_str()) != Some("json") {
                continue;
            }
            checked += 1;
            if let Err(problem) = validate_file(&path) {
                problems.push(problem);
            }
        }
    }

    (checked, problems)
}

fn user_dir() -> Option<PathBuf> {
    let base = std::env::var_os("XDG_DATA_HOME")
        .map(PathBuf::from)
        .or_else(|| std::env::var_os("HOME").map(|h| PathBuf::from(h).join(".local/share")))?;
    Some(base.join("cheatrs/topics"))
}

/// Display label: the data says `Language`, the header reads `LANGUAGES`.
pub fn group_label(name: &str) -> String {
    match name {
        "Language" => "LANGUAGES".into(),
        "Framework" => "FRAMEWORKS".into(),
        "Library" => "LIBRARIES".into(),
        "Tool" => "TOOLS".into(),
        "Database" => "DATABASES".into(),
        "Runtime" => "RUNTIMES".into(),
        other => other.to_uppercase(),
    }
}

/// Two or three letters per topic. Explicit table: language acronyms are not guessable.
pub fn monogram(slug: &str, title: &str) -> String {
    const TABLE: [(&str, &str); 56] = [
        ("angular", "NG"),
        ("bash", "SH"),
        ("bootstrap", "BS"),
        ("c", "C"),
        ("cpp", "C++"),
        ("csharp", "C#"),
        ("css", "CSS"),
        ("dart", "DT"),
        ("django", "DJ"),
        ("docker", "DK"),
        ("express", "EX"),
        ("fastapi", "FA"),
        ("flask", "FL"),
        ("flutter", "FL"),
        ("git", "GIT"),
        ("github-actions", "GHA"),
        ("go", "GO"),
        ("html", "HT"),
        ("htmx", "HTX"),
        ("java", "JV"),
        ("js", "JS"),
        ("jquery", "JQ"),
        ("kotlin", "KT"),
        ("kubernetes", "K8S"),
        ("laravel", "LV"),
        ("linux-commands", "LX"),
        ("lua", "LUA"),
        ("mongodb", "MG"),
        ("motion", "MO"),
        ("mysql", "MY"),
        ("next", "NX"),
        ("nginx", "NGX"),
        ("nodejs", "ND"),
        ("numpy", "NP"),
        ("pandas", "PD"),
        ("php", "PHP"),
        ("postgresql", "PG"),
        ("python", "PY"),
        ("react", "RX"),
        ("react-native", "RN"),
        ("redis", "RD"),
        ("ruby", "RB"),
        ("rust", "RS"),
        ("r", "R"),
        ("springboot", "SB"),
        ("swift", "SW"),
        ("shell-tools", "CLI"),
        ("sql", "SQL"),
        ("tailwind", "TW"),
        ("turbo", "TB"),
        ("typescript", "TS"),
        ("testing-js", "TST"),
        ("vim", "VIM"),
        ("vue", "VUE"),
        ("windows-commands", "WIN"),
        ("yaml", "YML"),
    ];
    if let Some((_, mark)) = TABLE.iter().find(|(s, _)| *s == slug) {
        return (*mark).to_string();
    }
    let mut out = String::new();
    for word in title.split_whitespace().take(3) {
        if let Some(ch) = word.chars().next() {
            out.extend(ch.to_uppercase());
        }
    }
    out
}
