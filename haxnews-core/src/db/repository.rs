use crate::models::{FeedSource, NewsItem};
use crate::db::tables::{FEEDS_TABLE, ITEMS_TABLE};
use anyhow::Result;
use redb::{Database, ReadableTable, ReadableDatabase};
use chrono::Utc;
use std::path::Path;

pub struct Repository {
    db: Database,
}

impl Repository {
    pub fn new(db_path: impl AsRef<Path>) -> Result<Self> {
        let db = Database::create(db_path)?;
        
        // Ensure tables exist by opening them in a write transaction
        let write_txn = db.begin_write()?;
        {
            write_txn.open_table(FEEDS_TABLE)?;
            write_txn.open_table(ITEMS_TABLE)?;
        }
        write_txn.commit()?;
        
        Ok(Repository { db })
    }

    pub fn save_feed(&self, feed: &FeedSource) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(FEEDS_TABLE)?;
            let json = serde_json::to_string(feed)?;
            table.insert(feed.id.to_string().as_str(), json.as_bytes())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_all_feeds(&self) -> Result<Vec<FeedSource>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(FEEDS_TABLE)?;
        
        let mut feeds = Vec::new();
        for result in table.iter()? {
            let (_, value) = result?;
            let json_str = std::str::from_utf8(value.value())?;
            let feed: FeedSource = serde_json::from_str(json_str)?;
            feeds.push(feed);
        }
        Ok(feeds)
    }

    pub fn save_item(&self, item: &NewsItem) -> Result<()> {
        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ITEMS_TABLE)?;
            let json = serde_json::to_string(item)?;
            table.insert(item.dedup_hash.as_str(), json.as_bytes())?;
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_all_items(&self) -> Result<Vec<NewsItem>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ITEMS_TABLE)?;
        
        let mut items = Vec::new();
        for result in table.iter()? {
            let (_, value) = result?;
            let json_str = std::str::from_utf8(value.value())?;
            let item: NewsItem = serde_json::from_str(json_str)?;
            items.push(item);
        }
        Ok(items)
    }

    pub fn delete_old_items(&self, days_old: i64) -> Result<()> {
        let cutoff = Utc::now().timestamp() - (days_old * 24 * 60 * 60);

        let write_txn = self.db.begin_write()?;
        {
            let mut table = write_txn.open_table(ITEMS_TABLE)?;
            let mut to_remove = Vec::new();

            for result in table.iter()? {
                let (key, value) = result?;
                let json_str = std::str::from_utf8(value.value())?;
                let item: NewsItem = serde_json::from_str(json_str)?;
                
                if let Some(published) = item.published_at {
                    if published.timestamp() < cutoff {
                        to_remove.push(key.value().to_string());
                    }
                }
            }

            for key in to_remove {
                table.remove(key.as_str())?;
            }
        }
        write_txn.commit()?;
        Ok(())
    }

    pub fn get_feed(&self, id: &str) -> Result<Option<FeedSource>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(FEEDS_TABLE)?;
        
        if let Some(value) = table.get(id)? {
            let json_str = std::str::from_utf8(value.value())?;
            let feed: FeedSource = serde_json::from_str(json_str)?;
            Ok(Some(feed))
        } else {
            Ok(None)
        }
    }

    pub fn delete_feed(&self, id: &str) -> Result<bool> {
        let write_txn = self.db.begin_write()?;
        let removed = {
            let mut table = write_txn.open_table(FEEDS_TABLE)?;
            table.remove(id)?.is_some()
        };
        write_txn.commit()?;
        Ok(removed)
    }

    pub fn get_item(&self, id: &str) -> Result<Option<NewsItem>> {
        // News items are keyed by dedup_hash, but we want to find by ID
        // To be correct, since items are keyed by dedup_hash, we'll iterate.
        // It's less efficient but items count is bounded.
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ITEMS_TABLE)?;
        
        for result in table.iter()? {
            let (_, value) = result?;
            let json_str = std::str::from_utf8(value.value())?;
            let item: NewsItem = serde_json::from_str(json_str)?;
            if item.id.to_string() == id {
                return Ok(Some(item));
            }
        }
        Ok(None)
    }

    pub fn get_items(
        &self, 
        limit: Option<usize>, 
        offset: Option<usize>, 
        search: Option<&str>, 
    ) -> Result<Vec<NewsItem>> {
        let read_txn = self.db.begin_read()?;
        let table = read_txn.open_table(ITEMS_TABLE)?;
        
        let mut items = Vec::new();
        for result in table.iter()? {
            let (_, value) = result?;
            let json_str = std::str::from_utf8(value.value())?;
            let item: NewsItem = serde_json::from_str(json_str)?;
            
            let mut matches = true;
            if let Some(query) = search {
                let query = query.to_lowercase();
                if !item.title.to_lowercase().contains(&query) && 
                   !item.summary.as_ref().map(|s| s.to_lowercase().contains(&query)).unwrap_or(false) &&
                   !item.search_text.to_lowercase().contains(&query) {
                    matches = false;
                }
            }
            if matches {
                // To support category filtering properly we'd need feed's category,
                // but since the model item doesn't store category directly, 
                // we'd skip it here or join. For now, let's just collect.
                items.push(item);
            }
        }

        // Sort items by published_at or fetched_at descending
        items.sort_by(|a, b| {
            let time_a = a.published_at.unwrap_or(a.fetched_at);
            let time_b = b.published_at.unwrap_or(b.fetched_at);
            time_b.cmp(&time_a)
        });

        let offset = offset.unwrap_or(0);
        let items: Vec<NewsItem> = items.into_iter().skip(offset).take(limit.unwrap_or(50)).collect();

        Ok(items)
    }
}