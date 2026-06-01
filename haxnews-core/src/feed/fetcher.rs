use anyhow::Result;
use reqwest::Client;
use std::time::Duration;

pub struct FeedFetcher {
    client: Client,
}

impl Default for FeedFetcher {
    fn default() -> Self {
        Self::new()
    }
}

impl FeedFetcher {
    pub fn new() -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(15))
            .user_agent("HaxNews/0.1")
            .build()
            .unwrap();

        Self { client }
    }

    pub async fn fetch(&self, url: &str) -> Result<String> {
        let response = self.client.get(url).send().await?;
        response.error_for_status_ref()?;
        
        let content = response.text().await?;
        Ok(content)
    }
}