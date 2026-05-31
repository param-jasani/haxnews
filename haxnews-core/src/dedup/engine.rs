use crate::models::NewsItem;
use crate::dedup::scorer::SimilarityScorer;

/// Deduplication Engine for news items
pub struct DedupEngine {
    pub threshold: f64,
}

impl DedupEngine {
    /// Create a new dedup engine with a similarity threshold (0.0 - 1.0)
    pub fn new(threshold: f64) -> Self {
        DedupEngine {
            threshold: threshold.clamp(0.0, 1.0),
        }
    }

    /// Find duplicate items in a collection
    pub fn find_duplicates(&self, items: &[NewsItem]) -> Vec<Vec<usize>> {
        let mut duplicates: Vec<Vec<usize>> = Vec::new();
        let mut processed = vec![false; items.len()];

        for i in 0..items.len() {
            if processed[i] {
                continue;
            }

            let mut group = vec![i];
            processed[i] = true;

            for j in (i + 1)..items.len() {
                if processed[j] {
                    continue;
                }

                if SimilarityScorer::are_items_similar(
                    &items[i],
                    &items[j],
                    self.threshold,
                ) {
                    group.push(j);
                    processed[j] = true;
                }
            }

            if group.len() > 1 {
                duplicates.push(group);
            }
        }

        duplicates
    }

    /// Filter items to keep only unique ones (first occurrence of duplicates)
    pub fn deduplicate(&self, items: Vec<NewsItem>) -> Vec<NewsItem> {
        let duplicates = self.find_duplicates(&items);
        let mut keep_indices = (0..items.len()).collect::<std::collections::HashSet<_>>();

        // Keep only the first item in each duplicate group
        for group in duplicates {
            for &idx in &group[1..] {
                keep_indices.remove(&idx);
            }
        }

        items
            .into_iter()
            .enumerate()
            .filter(|(idx, _)| keep_indices.contains(idx))
            .map(|(_, item)| item)
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;
    use chrono::Utc;

    fn create_test_item(title: &str, content: &str) -> NewsItem {
        NewsItem::new(
            Uuid::new_v4(),
            title.to_string(),
            Some(content.to_string()),
            None,
            None,
            "http://example.com".to_string(),
            None,
            Some(Utc::now()),
            "hash".to_string(),
            content.to_string(),
        )
    }

    #[test]
    fn test_find_duplicates() {
        let engine = DedupEngine::new(0.8);
        let items = vec![
            create_test_item("Hello World", "This is a test"),
            create_test_item("Hello World", "This is a test"),
            create_test_item("Different", "Completely different content"),
        ];

        let duplicates = engine.find_duplicates(&items);
        assert_eq!(duplicates.len(), 1);
        assert_eq!(duplicates[0].len(), 2);
    }
}
