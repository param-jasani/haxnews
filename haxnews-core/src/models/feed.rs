// haxnews-core/src/models/feed.rs
use chrono::{DateTime, Utc};
use serde::{Serialize, Deserialize};
use uuid::Uuid;

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum FeedStatus {
    Active,
    Error,
    Disabled,
    Paused(u32),   // paused for X minutes
}

impl serde::Serialize for FeedStatus {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            FeedStatus::Active => "Active".to_string(),
            FeedStatus::Error => "Error".to_string(),
            FeedStatus::Disabled => "Disabled".to_string(),
            FeedStatus::Paused(mins) => format!("Paused({})", mins),
        };
        serializer.serialize_str(&s)
    }
}

impl<'de> Deserialize<'de> for FeedStatus {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        struct FeedStatusVisitor;

        impl<'de> serde::de::Visitor<'de> for FeedStatusVisitor {
            type Value = FeedStatus;

            fn expecting(&self, formatter: &mut std::fmt::Formatter) -> std::fmt::Result {
                formatter.write_str("a string representing FeedStatus")
            }

            fn visit_str<E>(self, value: &str) -> Result<FeedStatus, E>
            where
                E: serde::de::Error,
            {
                match value {
                    "Active" => Ok(FeedStatus::Active),
                    "Error" => Ok(FeedStatus::Error),
                    "Disabled" => Ok(FeedStatus::Disabled),
                    s if s.starts_with("Paused(") && s.ends_with(")") => {
                        let num_str = &s[7..s.len() - 1];
                        let mins = num_str.parse::<u32>().map_err(serde::de::Error::custom)?;
                        Ok(FeedStatus::Paused(mins))
                    }
                    _ => Err(serde::de::Error::unknown_variant(
                        value,
                        &["Active", "Error", "Disabled", "Paused(X)"],
                    )),
                }
            }
        }

        deserializer.deserialize_str(FeedStatusVisitor)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedSource {
    pub id: Uuid,
    pub name: String,           // Org name (e.g. "The Hindu")
    pub url: String,
    pub priority: u32,
    pub refresh_minutes: u32,
    pub category: Option<String>,
    pub enabled: bool,
    pub etag: Option<String>,
    pub last_modified: Option<String>,
    pub last_fetched_at: Option<DateTime<Utc>>,
    pub status: FeedStatus,
    pub error_count: u32,
}

impl FeedSource {
    pub fn new(name: String, url: String, priority: u32, refresh_minutes: u32, category: Option<String>, status: Option<FeedStatus>) -> Self {
        Self {
            id: Uuid::new_v4(),
            name,
            url,
            priority,
            refresh_minutes,
            category,
            enabled: true,
            etag: None,
            last_modified: None,
            last_fetched_at: None,
            status: status.unwrap_or(FeedStatus::Active),
            error_count: 0,
        }
    }
    pub fn update_status(&mut self, new_status: FeedStatus) {
        self.status = new_status;
    }

}