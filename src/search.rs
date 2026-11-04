//! One index over topics, sections and the code inside them.
//! Title matches answer "where is this"; code matches answer "which flag was that".

use std::collections::BTreeMap;
use std::sync::OnceLock;

use crate::data::Library;

/// Hand-written aliases: the word people type, the words the titles use.
static ALIASES: OnceLock<BTreeMap<String, String>> = OnceLock::new();

fn aliases() -> &'static BTreeMap<String, String> {
    ALIASES.get_or_init(|| {
        serde_json::from_str(include_str!("../data/aliases.json")).unwrap_or_default()
    })
}

/// A single word that is a known alias becomes the words the data actually spells.
fn expand(query: &str) -> String {
    let needle = query.trim().to_lowercase();
    if needle.is_empty() || needle.contains(char::is_whitespace) {
        return needle;
    }
    aliases().get(&needle).cloned().unwrap_or(needle)
}

pub struct Entry {
    /// Index into `Library::topics`.
    pub topic: usize,
    /// Index into the topic's sections; `None` means the topic itself.
    pub section: Option<usize>,
    pub title: String,
    /// Where the entry lives: the group for a topic, the topic for a section.
    pub context: String,
    /// Every snippet line of the section, as written.
    pub lines: Vec<String>,
    /// The same lines lowercased once, so a keystroke does not re-fold a megabyte.
    pub haystack: String,
}

pub struct Hit {
    /// Index into `Index::entries`.
    pub entry: usize,
    /// The code line that matched, trimmed. `None` when the title matched.
    pub excerpt: Option<String>,
}

pub struct Index {
    pub entries: Vec<Entry>,
}

const EXCERPT_WIDTH: usize = 64;

impl Index {
    pub fn build(library: &Library) -> Self {
        let mut entries = Vec::new();
        for (topic_index, topic) in library.topics.iter().enumerate() {
            entries.push(Entry {
                topic: topic_index,
                section: None,
                title: topic.title.clone(),
                context: topic.group.clone(),
                lines: Vec::new(),
                haystack: String::new(),
            });
            for (section_index, section) in topic.sections.iter().enumerate() {
                let mut lines = Vec::new();
                for snippet in &section.snippets {
                    for line in snippet.code.lines() {
                        if !line.trim().is_empty() {
                            lines.push(line.to_string());
                        }
                    }
                }
                let haystack = lines.join("\n").to_lowercase();
                entries.push(Entry {
                    topic: topic_index,
                    section: Some(section_index),
                    title: section.title.clone(),
                    context: topic.title.clone(),
                    lines,
                    haystack,
                });
            }
        }
        Self { entries }
    }

    /// Title first, then code. Anything that matches both is listed once.
    pub fn query(&self, query: &str) -> Vec<Hit> {
        let needle = expand(query);
        if needle.is_empty() {
            return Vec::new();
        }
        let width = needle.chars().count();

        // rank, gap penalty, title length, entry index, excerpt
        let mut scored: Vec<(u8, u16, usize, usize, Option<String>)> = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let title = entry.title.to_lowercase();
            let rank = if title.starts_with(&needle) {
                Some((0u8, 0u16))
            } else if title.contains(&needle) {
                Some((1, 0))
            } else if entry.section.is_some() && entry.context.to_lowercase().contains(&needle) {
                Some((2, 0))
            } else if width >= 3 {
                // Typed from memory, wrong: `pstgrs` has to reach PostgreSQL.
                // Only titles get this treatment; code would produce noise.
                subsequence_gap(&title, &needle).map(|gap| (3, gap))
            } else {
                None
            };

            if let Some((rank, gap)) = rank {
                scored.push((rank, gap, title.chars().count(), index, None));
                continue;
            }

            // Code is the expensive half: only worth it from two characters on.
            if width >= 2 && entry.haystack.contains(&needle) {
                let excerpt = entry
                    .lines
                    .iter()
                    .find(|line| line.to_lowercase().contains(&needle))
                    .map(|line| truncate(line.trim(), EXCERPT_WIDTH));
                scored.push((4, 0, title.chars().count(), index, excerpt));
            }
        }

        scored.sort_unstable_by_key(|(rank, gap, length, index, _)| (*rank, *gap, *length, *index));
        scored
            .into_iter()
            .map(|(_, _, _, entry, excerpt)| Hit { entry, excerpt })
            .collect()
    }
}

/// Greedy subsequence match, scored by how many characters it had to skip.
///
/// `None` means the letters are not all there in order. A tight match (`pgsql`
/// against `postgresql`) scores low; scattered letters score high and sort last.
fn subsequence_gap(haystack: &str, needle: &str) -> Option<u16> {
    let mut chars = haystack.chars().enumerate();
    let mut gaps = 0u16;
    let mut previous: Option<usize> = None;

    for wanted in needle.chars() {
        let mut found = false;
        for (position, candidate) in chars.by_ref() {
            if candidate == wanted {
                if let Some(last) = previous {
                    // Cap a single jump so one long gap cannot poison the score.
                    gaps = gaps.saturating_add((position - last - 1).min(40) as u16);
                }
                previous = Some(position);
                found = true;
                break;
            }
        }
        if !found {
            return None;
        }
    }
    Some(gaps)
}

fn truncate(text: &str, width: usize) -> String {
    if text.chars().count() <= width {
        return text.to_string();
    }
    let mut out: String = text.chars().take(width.saturating_sub(1)).collect();
    out.push('…');
    out
}
