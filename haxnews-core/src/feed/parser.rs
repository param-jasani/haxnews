use anyhow::Result;
use uuid::Uuid;
use crate::models::item::NewsItem;
use crate::utils::hashing::compute_hash;

/// RSS Feed Parser
pub struct FeedParser;

impl FeedParser {
    /// Parse RSS feed content and convert to NewsItem objects
    pub fn parse(feed_id: Uuid, feed_content: &str, feed_name: &str) -> Result<Vec<NewsItem>> {
        // Parse the feed using feed-rs
        let feed = feed_rs::parser::parse(feed_content.as_bytes())?;

        let mut items = Vec::new();

        // Extract items from the feed
        for entry in feed.entries {
            let title = entry.title.map(|t| t.content).unwrap_or_default();
            
            // Get the summary from description or content
            let summary = entry
                .summary
                .as_ref()
                .map(|s| s.content.clone())
                .or_else(|| {
                    entry.content.as_ref().and_then(|c| {
                        c.body.as_ref().map(|b| b.clone())
                    })
                });

            // Get the link
            let link = entry
                .links
                .first()
                .map(|l| l.href.clone())
                .unwrap_or_default();

            // Get the author
            let author = entry
                .authors
                .first()
                .map(|a| a.name.clone());

            // Get the image URL from media content or enclosures
            let image_url = entry
                .media
                .first()
                .and_then(|m| {
                    m.content
                        .first()
                        .and_then(|c| c.url.as_ref().map(|u| u.to_string()))
                })
                .or_else(|| {
                    entry
                        .links
                        .iter()
                        .find(|l| l.rel.as_deref() == Some("enclosure"))
                        .map(|l| l.href.clone())
                });

            // Get publication date
            let published_at = entry.published.map(|dt| dt);

            // Generate dedup hash based on title and link
            let dedup_hash = compute_hash(&format!("{}{}", title, link));

            // Create search text from title and summary
            let search_text = format!(
                "{} {}",
                title,
                summary.as_deref().unwrap_or("")
            )
            .to_lowercase();

            // Create the NewsItem
            let item = NewsItem::new(
                feed_id,
                title,
                summary,
                image_url,
                author,
                link,
                Some(feed_name.to_string()),
                published_at,
                dedup_hash,
                search_text,
            );

            items.push(item);
        }

        Ok(items)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_rss_feed() {
        let feed_xml = r#"<?xml version="1.0" encoding="UTF-8"?>
<rss version="2.0">
  <channel>
    <title>Test Feed</title>
    <link>https://example.com</link>
    <description>Test Description</description>
    <item>
      <title>Test Article</title>
      <description>Test Summary</description>
      <link>https://example.com/article1</link>
      <pubDate>Fri, 29 May 2026 23:37:12 +0530</pubDate>
      <author>test@example.com</author>
    </item>
  </channel>
</rss>"#;

        let feed_id = Uuid::new_v4();
        let result = FeedParser::parse(feed_id, feed_xml, "Test Feed");
        
        assert!(result.is_ok());
        let items = result.unwrap();
        assert_eq!(items.len(), 1);
    }
}
