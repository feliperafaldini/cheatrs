//! One index over topics and sections. Search answers "where is this", not "what does it say".

use crate::data::Library;

pub struct Entry {
    /// Index into `Library::topics`.
    pub topic: usize,
    /// Index into the topic's sections; `None` means the topic itself.
    pub section: Option<usize>,
    pub title: String,
    /// Where the entry lives: the group for a topic, the topic for a section.
    pub context: String,
}

pub struct Index {
    pub entries: Vec<Entry>,
}

impl Index {
    pub fn build(library: &Library) -> Self {
        let mut entries = Vec::new();
        for (topic_index, topic) in library.topics.iter().enumerate() {
            entries.push(Entry {
                topic: topic_index,
                section: None,
                title: topic.title.clone(),
                context: topic.group.clone(),
            });
            for (section_index, section) in topic.sections.iter().enumerate() {
                entries.push(Entry {
                    topic: topic_index,
                    section: Some(section_index),
                    title: section.title.clone(),
                    context: topic.title.clone(),
                });
            }
        }
        Self { entries }
    }

    /// Case-insensitive, ranked: prefix beats contains beats "the section's parent matched".
    pub fn query(&self, query: &str) -> Vec<usize> {
        let needle = query.trim().to_lowercase();
        if needle.is_empty() {
            return Vec::new();
        }

        let mut scored: Vec<(u8, usize, usize)> = Vec::new();
        for (index, entry) in self.entries.iter().enumerate() {
            let title = entry.title.to_lowercase();
            let context = entry.context.to_lowercase();
            let rank = if title.starts_with(&needle) {
                0
            } else if title.contains(&needle) {
                1
            } else if entry.section.is_some() && context.contains(&needle) {
                2
            } else {
                continue;
            };
            scored.push((rank, title.chars().count(), index));
        }

        scored.sort_unstable();
        scored.into_iter().map(|(_, _, index)| index).collect()
    }
}
