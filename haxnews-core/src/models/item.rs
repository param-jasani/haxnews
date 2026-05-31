use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItem {
    pub id: Uuid,
    pub feed_id: Uuid,

    pub title: String,
    pub summary: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub link: String,
    pub org: Option<String>,
    
    pub alternative_links: Vec<String>,
    pub alternative_orgs: Vec<String>,

    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,

    pub dedup_hash: String,
    pub search_text: String,

    pub is_read: bool,
}

impl NewsItem {
    pub fn new(feed_id: Uuid, title: String, summary: Option<String>, image_url: Option<String>, author: Option<String>, link: String, org: Option<String>, published_at: Option<DateTime<Utc>>, dedup_hash: String, search_text: String) -> Self {
        Self {
            id: Uuid::new_v4(),
            feed_id,
            title,
            summary,
            image_url,
            author,
            link,
            org,
            alternative_links: Vec::new(),
            alternative_orgs: Vec::new(),
            published_at,
            fetched_at: Utc::now(),
            dedup_hash,
            search_text,
            is_read: false,
        }
    }
    pub fn mark_as_read(&mut self) {
        self.is_read = true;
    }
}