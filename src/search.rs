use crate::indexer::ProgramEntry;
use fuzzy_matcher::skim::SkimMatcherV2;
use fuzzy_matcher::FuzzyMatcher;

/// Search result with score
#[derive(Clone, Debug)]
pub struct SearchResult {
    pub entry: ProgramEntry,
    pub score: i64,
}

/// Fast fuzzy search engine for programs
pub struct SearchEngine {
    matcher: SkimMatcherV2,
}

impl Default for SearchEngine {
    fn default() -> Self {
        Self::new()
    }
}

impl SearchEngine {
    pub fn new() -> Self {
        Self {
            matcher: SkimMatcherV2::default(),
        }
    }

    /// Search through program entries
    pub fn search(&self, query: &str, entries: &[ProgramEntry]) -> Vec<SearchResult> {
        if query.is_empty() {
            // Return first 20 programs when no query
            return entries
                .iter()
                .take(20)
                .map(|e| SearchResult {
                    entry: e.clone(),
                    score: 0,
                })
                .collect();
        }

        let query_lower = query.to_lowercase();

        let mut results: Vec<SearchResult> = entries
            .iter()
            .filter_map(|entry| {
                // Try matching against display name
                let display_score = self.matcher.fuzzy_match(&entry.display_name.to_lowercase(), &query_lower);
                
                // Try matching against file name
                let name_score = self.matcher.fuzzy_match(&entry.name, &query_lower);

                // Take the best score
                let base_score = display_score.max(name_score)?;

                // Boost Start Menu items
                let source_boost = match entry.source {
                    crate::indexer::ProgramSource::StartMenu => 50,
                    crate::indexer::ProgramSource::ProgramFiles => 0,
                };

                // Boost exact prefix matches
                let prefix_boost = if entry.display_name.to_lowercase().starts_with(&query_lower) {
                    100
                } else {
                    0
                };

                Some(SearchResult {
                    entry: entry.clone(),
                    score: base_score + source_boost + prefix_boost,
                })
            })
            .collect();

        // Sort by score descending
        results.sort_by(|a, b| b.score.cmp(&a.score));

        // Limit results
        results.truncate(50);

        results
    }
}
