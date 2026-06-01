use anyhow::Result;
use std::fs;
use crate::{get_config_path, get_data_dir, get_db_path};
use haxnews_core::db::Repository;
use haxnews_core::config::LoadFeeds;
use haxnews_core::api::{create_router, routes::AppState};

use haxnews_core::feed::fetcher::FeedFetcher;
use haxnews_core::feed::parser::FeedParser;
use std::sync::Arc;
use tokio::net::TcpListener;

use std::io::{self, Write};

#[derive(Debug, Clone)]
pub struct CommandResult {
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

impl CommandResult {
    pub fn success(msg: impl Into<String>) -> Self {
        Self { success: true, message: msg.into(), details: None }
    }
    
    pub fn error(msg: impl Into<String>) -> Self {
        Self { success: false, message: msg.into(), details: None }
    }
}

pub async fn install_command() -> Result<CommandResult> {
    let data_dir = get_data_dir();
    let config_path = get_config_path();
    
    let mut details = String::new();
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
        details.push_str(&format!("Created data directory at {:?}\n", data_dir));
    }

    if !config_path.exists() {
        let default_config = r#"
[[feeds]]
name = "The Hacker News"
url = "https://feeds.feedburner.com/TheHackersNews"
priority = 1
refresh_minutes = 60
category = "Cybersecurity"

[[feeds]]
name = "Bleeping Computer"
url = "https://www.bleepingcomputer.com/feed/"
priority = 2
refresh_minutes = 60
category = "Cybersecurity"
"#;
        // Try reading from local project "config/feeds.toml" first
        let local_config = std::path::Path::new("config/feeds.toml");
        if local_config.exists() {
            fs::copy(local_config, &config_path)?;
            details.push_str(&format!("Copied local feeds.toml to {:?}\n", config_path));
        } else {
            fs::write(&config_path, default_config)?;
            details.push_str(&format!("Created default feeds.toml at {:?}\n", config_path));
        }
    } else {
        details.push_str(&format!("Config already exists at {:?}\n", config_path));
    }
    
    // Initialize DB
    let _db = Repository::new(get_db_path())?;
    details.push_str("Database initialized.\n");
    
    Ok(CommandResult {
        success: true,
        message: "Installation completed successfully".to_string(),
        details: Some(details),
    })
}

pub async fn run_command_fg() -> Result<()> {
    println!("Starting HaxNews in foreground...");
    
    let db = Arc::new(Repository::new(get_db_path())?);
    let state = AppState { db: db.clone() };
    let app = create_router(state);
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("API listening on http://127.0.0.1:8080");
    
    // Fetcher daemon loop could be implemented here
    
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn server_start() -> Result<()> {
    println!("Starting server mode (API only)...");
    let db = Arc::new(Repository::new(get_db_path())?);
    let state = AppState { db: db.clone() };
    let app = create_router(state);
    
    let listener = TcpListener::bind("127.0.0.1:8080").await?;
    println!("API listening on http://127.0.0.1:8080");
    
    axum::serve(listener, app).await?;
    Ok(())
}

pub async fn fetch_command(feed_id: Option<String>) -> Result<CommandResult> {
    let db = Repository::new(get_db_path())?;
    let fetcher = FeedFetcher::new();
    
    let feeds = db.get_all_feeds()?;
    let feeds_to_fetch = if let Some(id_str) = feed_id {
        feeds.into_iter().filter(|f| f.id.to_string() == id_str).collect()
    } else {
        feeds
    };
    
    if feeds_to_fetch.is_empty() {
        return Ok(CommandResult::error("No feeds to fetch. Make sure you have feeds in the database."));
    }
    
    let mut details = String::new();
    let mut total_saved = 0;

    for feed in feeds_to_fetch {
        match fetcher.fetch(&feed.url).await {
            Ok(content) => {
                match FeedParser::parse(feed.id, &content, &feed.name) {
                    Ok(items) => {
                        let mut saved = 0;
                        for item in items {
                            if db.save_item(&item).is_ok() {
                                saved += 1;
                            }
                        }
                        details.push_str(&format!("✅ Saved {} items for {}\n", saved, feed.name));
                        total_saved += saved;
                    },
                    Err(e) => details.push_str(&format!("❌ Parse error for {}: {}\n", feed.name, e)),
                }
            },
            Err(e) => details.push_str(&format!("❌ Fetch error for {}: {}\n", feed.name, e)),
        }
    }
    
    Ok(CommandResult {
        success: true,
        message: format!("Fetch complete. {} items saved.", total_saved),
        details: Some(details),
    })
}

pub async fn feeds_list() -> Result<CommandResult> {
    let db = Repository::new(get_db_path())?;
    let feeds = db.get_all_feeds()?;
    
    let mut details = format!("{:<40} | {:<20} | {}\n", "ID", "Name", "URL");
    details.push_str(&format!("{:-<40}-+-{:-<20}-+-{:-<40}\n", "", "", ""));
    
    for feed in feeds {
        details.push_str(&format!("{:<40} | {:<20} | {}\n", feed.id, feed.name, feed.url));
    }
    
    Ok(CommandResult {
        success: true,
        message: "Feeds listed successfully".to_string(),
        details: Some(details),
    })
}

pub async fn feeds_add() -> Result<CommandResult> {
    let mut name = String::new();
    let mut url = String::new();
    let mut priority = String::new();
    let mut category = String::new();

    // Since this is interactive via stdin, we'll leave prints here but it shouldn't be called from TUI.
    // The TUI has its own popup logic.
    print!("Enter Feed Name: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut name)?;

    print!("Enter Feed URL: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut url)?;

    print!("Enter Priority (default 10): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut priority)?;
    let priority = priority.trim();
    let priority = if priority.is_empty() { "10" } else { priority };

    print!("Enter Category (default 'News'): ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut category)?;
    let category = category.trim();
    let category = if category.is_empty() { "News" } else { category };

    let new_feed_entry = format!(
        "\n[[feeds]]\nname = \"{}\"\nurl = \"{}\"\npriority = {}\nrefresh_minutes = 60\ncategory = \"{}\"\nstatus = \"Active\"\n",
        name.trim(),
        url.trim(),
        priority,
        category
    );

    let config_path = get_config_path();
    let mut file = fs::OpenOptions::new()
        .append(true)
        .open(&config_path)?;
    
    file.write_all(new_feed_entry.as_bytes())?;

    // Call sync to update database
    let _ = feeds_sync().await?;
    Ok(CommandResult::success("Feed added successfully"))
}

pub async fn feeds_sync() -> Result<CommandResult> {
    let config_path = get_config_path();
    let feeds_config = LoadFeeds::execute(config_path)?;
    
    let db = Repository::new(get_db_path())?;
    let mut added = 0;
    
    // Simple sync: just add them if they don't match exactly by URL (in reality we'd update)
    let existing_feeds = db.get_all_feeds()?;
    for feed in feeds_config {
        if !existing_feeds.iter().any(|f| f.url == feed.url) {
            db.save_feed(&feed)?;
            added += 1;
        }
    }
    
    Ok(CommandResult::success(format!("Sync complete. Added {} new feeds.", added)))
}

pub async fn items_command(search: Option<String>, limit: usize) -> Result<CommandResult> {
    let db = Repository::new(get_db_path())?;
    let search = search.as_deref();
    let items = db.get_items(Some(limit), None, search)?;
    if items.is_empty() {
        return Ok(CommandResult::error("No items found."));
    }
    
    let mut details = String::new();
    for item in items {
        details.push_str(&format!("- {} [{}]\n", item.title, item.org.unwrap_or_else(|| "Unknown".to_string())));
        details.push_str(&format!("  {}\n\n", item.link));
    }
    
    Ok(CommandResult {
        success: true,
        message: "Items retrieved.".to_string(),
        details: Some(details),
    })
}

pub async fn status_command() -> Result<CommandResult> {
    let mut details = format!("System Data Dir: {:?}\n", get_data_dir());
    let db = Repository::new(get_db_path());
    match db {
        Ok(db) => {
            match db.get_all_feeds() {
                Ok(feeds) => details.push_str(&format!("DB Feeds: {}\n", feeds.len())),
                Err(err) => details.push_str(&format!("Unable to read feeds: {}\n", err)),
            }

            match db.get_items(None, None, None) {
                Ok(items) => details.push_str(&format!("DB Items: {}\n", items.len())),
                Err(err) => details.push_str(&format!("Unable to read items: {}\n", err)),
            }
        }
        Err(_) => {
            details.push_str("Database not initialized.\n");
        }
    }
    Ok(CommandResult {
        success: true,
        message: "Status check complete.".to_string(),
        details: Some(details),
    })
}

pub async fn cleanup_command() -> Result<CommandResult> {
    let db = Repository::new(get_db_path())?;
    db.delete_old_items(60)?;
    Ok(CommandResult::success("Cleanup complete. Deleted items older than 60 days."))
}

pub async fn config_command() -> Result<CommandResult> {
    let details = format!(
        "Data Directory: {:?}\nConfig File: {:?}\nDatabase File: {:?}",
        get_data_dir(),
        get_config_path(),
        get_db_path()
    );
    Ok(CommandResult {
        success: true,
        message: "Config paths loaded.".to_string(),
        details: Some(details),
    })
}
