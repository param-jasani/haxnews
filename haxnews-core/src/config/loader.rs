use crate::models::{FeedSource, FeedStatus};
use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
pub struct FeedSourceConfig {
    pub name: String,
    pub url: String,
    pub priority: u32,
    pub refresh_minutes: u32,
    pub category: Option<String>,
    pub status: Option<FeedStatus>,
}

impl From<FeedSourceConfig> for FeedSource {
    fn from(config: FeedSourceConfig) -> Self {
        FeedSource::new(
            config.name,
            config.url,
            config.priority,
            config.refresh_minutes,
            config.category,
            config.status
        )
    }
}

#[derive(Debug, Deserialize)]
pub struct FeedConfig {
    pub feeds: Vec<FeedSourceConfig>,
}
pub struct LoadFeeds;

impl LoadFeeds {
    pub fn execute(config_path: PathBuf) -> Result<Vec<FeedSource>> {
        let file_content = std::fs::read_to_string(&config_path)?;

        let feeds = match config_path
            .extension()
            .and_then(|ext| ext.to_str())
        {
            Some("toml") => {
                let config: FeedConfig = toml::from_str(&file_content)?;
                config.feeds
            }

            Some("json") => {
                let config: FeedConfig = serde_json::from_str(&file_content)?;
                config.feeds
            }

            _ => {
                return Err(anyhow::anyhow!(
                    "Unsupported config file format. Use .toml or .json"
                ))
            }
        };

        Ok(feeds.into_iter().map(FeedSource::from).collect())
    }
}