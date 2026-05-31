use crate::models::NewsItem;

pub struct SimilarityScorer;

impl SimilarityScorer {
    /// Calculate similarity between two texts using Jaro-Winkler distance
    pub fn calculate_similarity(text1: &str, text2: &str) -> f64 {
        strsim::jaro_winkler(text1, text2)
    }

    /// Check if two items are similar based on title and content
    pub fn are_items_similar(item1: &NewsItem, item2: &NewsItem, threshold: f64) -> bool {
        let title_similarity = Self::calculate_similarity(
            &item1.title.to_lowercase(),
            &item2.title.to_lowercase(),
        );

        let content_similarity = Self::calculate_similarity(
            &item1.search_text,
            &item2.search_text,
        );

        // Use weighted average: 70% title, 30% content
        let combined_score = (title_similarity * 0.7) + (content_similarity * 0.3);
        combined_score >= threshold
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_similarity_score() {
        let score = SimilarityScorer::calculate_similarity("hello", "hello");
        assert!(score > 0.99, "Identical strings should have very high similarity");

        let score = SimilarityScorer::calculate_similarity("hello", "world");
        assert!(score < 0.5, "Completely different strings should have low similarity");
    }
}
