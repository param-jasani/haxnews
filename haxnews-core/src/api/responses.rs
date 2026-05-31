use serde::{Serialize, Deserialize};
use crate::models::NewsItem;
use chrono::{DateTime, Utc};

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemResponse {
    pub id: String,
    pub feed_id: String,
    pub title: String,
    pub summary: Option<String>,
    pub image_url: Option<String>,
    pub author: Option<String>,
    pub link: String,
    pub org: Option<String>,
    pub published_at: Option<DateTime<Utc>>,
    pub fetched_at: DateTime<Utc>,
}

impl From<NewsItem> for ItemResponse {
    fn from(item: NewsItem) -> Self {
        ItemResponse {
            id: item.id.to_string(),
            feed_id: item.feed_id.to_string(),
            title: item.title,
            summary: item.summary,
            image_url: item.image_url,
            author: item.author,
            link: item.link,
            org: item.org,
            published_at: item.published_at,
            fetched_at: item.fetched_at,
        }
    }
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ItemsListResponse {
    pub items: Vec<ItemResponse>,
    pub total: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrorResponse {
    pub error: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct HealthResponse {
    pub status: String,
    pub version: String,
}
