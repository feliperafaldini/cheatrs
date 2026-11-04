use serde::Deserialize;

#[derive(Debug, Clone, Deserialize)]
pub struct Topic {
    pub slug: String,
    pub title: String,
    pub label: String,
    pub group: String,
    pub color: String,
    #[serde(default)]
    pub desc: String,
    #[serde(default)]
    pub sections: Vec<Section>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Section {
    pub id: String,
    pub title: String,
    #[serde(default)]
    pub description: String,
    #[serde(default)]
    pub snippets: Vec<Snippet>,
    // The site ships two shapes: a `snippets` array, or one flat `language`/`code` pair.
    #[serde(default)]
    pub language: Option<String>,
    #[serde(default)]
    pub code: Option<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct Snippet {
    #[serde(default)]
    pub language: String,
    pub code: String,
    #[serde(default)]
    pub description: Option<String>,
}

impl Section {
    /// Collapse the flat shape into `snippets` so the rest of the program sees one shape only.
    fn normalize(&mut self) {
        if self.snippets.is_empty()
            && let Some(code) = self.code.take()
        {
            self.snippets.push(Snippet {
                language: self.language.take().unwrap_or_else(|| "text".into()),
                code,
                description: None,
            });
        }
    }
}

impl Topic {
    /// Normalize shapes and drop what cannot be shown. Returns how many sections were dropped.
    pub fn normalize(&mut self) -> usize {
        self.sections.retain(|_| true);
        for section in &mut self.sections {
            section.normalize();
        }
        let before = self.sections.len();
        self.sections.retain(|s| !s.snippets.is_empty());
        before - self.sections.len()
    }

    pub fn snippet_count(&self) -> usize {
        self.sections.iter().map(|s| s.snippets.len()).sum()
    }
}
