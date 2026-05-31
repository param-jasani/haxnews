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

pub async fn install_command() -> Result<()> {
    let data_dir = get_data_dir();
    let config_path = get_config_path();
    
    if !data_dir.exists() {
        fs::create_dir_all(&data_dir)?;
        println!("Created data directory at {:?}", data_dir);
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
            println!("Copied local feeds.toml to {:?}", config_path);
        } else {
            fs::write(&config_path, default_config)?;
            println!("Created default feeds.toml at {:?}", config_path);
        }
    } else {
        println!("Config already exists at {:?}", config_path);
    }
    
    // Initialize DB
    let _db = Repository::new(get_db_path())?;
    println!("Database initialized.");
    
    Ok(())
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

pub async fn fetch_command(feed_id: Option<String>) -> Result<()> {
    println!("Starting manual fetch...");
    let db = Repository::new(get_db_path())?;
    let fetcher = FeedFetcher::new();
    
    let feeds = db.get_all_feeds()?;
    let feeds_to_fetch = if let Some(id_str) = feed_id {
        feeds.into_iter().filter(|f| f.id.to_string() == id_str).collect()
    } else {
        feeds
    };
    
    if feeds_to_fetch.is_empty() {
        println!("No feeds to fetch. Make sure you have feeds in the database.");
        return Ok(());
    }
    
    for feed in feeds_to_fetch {
        println!("Fetching {} ({})", feed.name, feed.url);
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
                        println!("✅ Saved {} items for {}", saved, feed.name);
                    },
                    Err(e) => println!("❌ Parse error for {}: {}", feed.name, e),
                }
            },
            Err(e) => println!("❌ Fetch error for {}: {}", feed.name, e),
        }
    }
    
    Ok(())
}

pub async fn feeds_list() -> Result<()> {
    let db = Repository::new(get_db_path())?;
    let feeds = db.get_all_feeds()?;
    
    println!("{:<40} | {:<20} | {}", "ID", "Name", "URL");
    println!("{:-<40}-+-{:-<20}-+-{:-<40}", "", "", "");
    
    for feed in feeds {
        println!("{:<40} | {:<20} | {}", feed.id, feed.name, feed.url);
    }
    
    Ok(())
}

pub async fn feeds_add() -> Result<()> {
    let mut name = String::new();
    let mut url = String::new();
    let mut priority = String::new();
    let mut category = String::new();

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
    println!("Feed added to {:?}", config_path);

    // Call sync to update database
    feeds_sync().await?;
    Ok(())
}

pub async fn feeds_sync() -> Result<()> {
    println!("Syncing feeds from config to database...");
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
    
    println!("Sync complete. Added {} new feeds.", added);
    Ok(())
}

pub async fn items_command(search: Option<String>, limit: usize) -> Result<()> {
    let db = Repository::new(get_db_path())?;
    let search = search.as_deref();
    let items = db.get_items(Some(limit), None, search)?;
    if items.is_empty() {
        println!("No items found.");
        return Ok(());
    }
    
    for item in items {
        println!("- {} [{}]", item.title, item.org.unwrap_or_else(|| "Unknown".to_string()));
        println!("  {}", item.link);
        println!();
    }
    
    Ok(())
}

pub async fn status_command() -> Result<()> {
    println!("System Data Dir: {:?}", get_data_dir());
    let db = Repository::new(get_db_path());
    match db {
        Ok(db) => {
            match db.get_all_feeds() {
                Ok(feeds) => println!("DB Feeds: {}", feeds.len()),
                Err(err) => println!("Unable to read feeds: {}", err),
            }

            match db.get_items(None, None, None) {
                Ok(items) => println!("DB Items: {}", items.len()),
                Err(err) => println!("Unable to read items: {}", err),
            }
        }
        Err(_) => {
            println!("Database not initialized.");
        }
    }
    Ok(())
}

pub async fn cleanup_command() -> Result<()> {
    println!("Cleaning up old items...");
    let db = Repository::new(get_db_path())?;
    db.delete_old_items(60)?;
    println!("Cleanup complete.");
    Ok(())
}

pub async fn config_command() -> Result<()> {
    println!("Data Directory: {:?}", get_data_dir());
    println!("Config File: {:?}", get_config_path());
    println!("Database File: {:?}", get_db_path());
    Ok(())
}
