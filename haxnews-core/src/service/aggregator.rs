use crate::feed::fetcher::FeedFetcher;
use crate::feed::parser::FeedParser;
use crate::models::NewsItem;
use crate::dedup::DedupEngine;
use anyhow::Result;
use uuid::Uuid;

/// Feed Aggregator Service
pub struct AggregatorService {
    fetcher: FeedFetcher,
    dedup_engine: DedupEngine,
}

impl AggregatorService {
    /// Create a new aggregator service
    pub fn new(dedup_threshold: f64) -> Self {
        AggregatorService {
            fetcher: FeedFetcher::new(),
            dedup_engine: DedupEngine::new(dedup_threshold),
        }
    }

    /// Fetch and parse a single RSS feed
    pub async fn fetch_feed(
        &self,
        feed_id: Uuid,
        feed_url: &str,
        feed_name: &str,
    ) -> Result<Vec<NewsItem>> {
        // Fetch the feed content
        let content = self.fetcher.fetch(feed_url).await?;

        // Parse the RSS feed
        let items = FeedParser::parse(feed_id, &content, feed_name)?;

        Ok(items)
    }

    /// Fetch and parse multiple feeds, returning deduplicated items
    pub async fn fetch_feeds(
        &self,
        feeds: Vec<(Uuid, String, String)>, // (feed_id, url, name)
    ) -> Result<Vec<NewsItem>> {
        let mut all_items = Vec::new();

        for (feed_id, url, name) in feeds {
            match self.fetch_feed(feed_id, &url, &name).await {
                Ok(items) => all_items.extend(items),
                Err(e) => {
                    eprintln!("Failed to fetch feed {}: {}", name, e);
                }
            }
        }

        // Deduplicate items
        let deduplicated = self.dedup_engine.deduplicate(all_items);
        Ok(deduplicated)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_aggregator_creation() {
        let aggregator = AggregatorService::new(0.85);
        assert_eq!(aggregator.dedup_engine.threshold, 0.85);
    }
}
