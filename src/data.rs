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

    Library {
        topics,
        groups,
        skipped,
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
    const TABLE: [(&str, &str); 8] = [
        ("python", "PY"),
        ("git", "GIT"),
        ("docker", "DK"),
        ("postgresql", "PG"),
        ("mysql", "MY"),
        ("bash", "SH"),
        ("javascript", "JS"),
        ("typescript", "TS"),
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
