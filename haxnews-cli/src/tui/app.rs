use serde::{Deserialize, Serialize};
use haxnews_core::db::Repository;
use crate::get_db_path;

use crate::commands::{
    install_command, fetch_command, feeds_sync, cleanup_command, status_command, config_command
};
use haxnews_core::models::FeedSource;
use haxnews_core::api::{create_router, routes::AppState};
use std::sync::Arc;
use tokio::net::TcpListener;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewsItemUI {
    pub title: String,
    pub url: String,
    pub author: Option<String>,
    pub description: String,
    pub image_url: Option<String>,
    pub published_at: Option<String>,
    pub feed_name: Option<String>,
    pub category: String, // added category
}

use chrono::{DateTime, Utc};
use tokio::sync::mpsc::{self, UnboundedSender, UnboundedReceiver};

#[derive(Debug, Clone)]
pub struct OperationStatus {
    pub command: String,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub success: bool,
    pub message: String,
    pub details: Option<String>,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Screen {
    Dashboard,
    News,
    Search,
    Feeds,
    Settings,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum Theme {
    Default,
    Cyberpunk,
    Monokai,
    Ocean,
    Dracula,
    Gruvbox,
}

impl Theme {
    pub fn name(&self) -> &'static str {
        match self {
            Theme::Default => "Default",
            Theme::Cyberpunk => "Cyberpunk",
            Theme::Monokai => "Monokai",
            Theme::Ocean => "Ocean",
            Theme::Dracula => "Dracula",
            Theme::Gruvbox => "Gruvbox",
        }
    }
}

pub enum FeedAddStage {
    Name,
    Url,
    Priority,
    Category,
}

pub enum PopupState {
    None,
    PauseFeedInput { input: String },
    AddFeedInput {
        stage: FeedAddStage,
        name: String,
        url: String,
        priority: String,
        category: String,
    },
}

#[derive(Debug, Clone)]
pub enum ActionRequest {
    Install,
    FetchAll,
    FetchSelectedFeed(Option<String>),
    FeedsSync,
    Status,
    Cleanup,
    Config,
    StartServer,
    RunForeground,
}

pub struct App {
    pub running: bool,
    pub current_screen: Screen,
    pub items: Vec<NewsItemUI>,
    pub selected_item: usize,
    pub search_query: String,
    pub search_results: Vec<NewsItemUI>,
    pub is_searching: bool,
    pub feeds: Vec<FeedSource>,
    pub selected_feed: usize,
    pub current_theme: Theme,
    pub db: Option<Repository>,
    pub popup: PopupState,
    pub image_tx: UnboundedSender<Vec<u8>>,
    pub image_rx: UnboundedReceiver<Vec<u8>>,
    pub current_image: Option<ratatui_image::protocol::StatefulProtocol>,
    pub pending_action: Option<ActionRequest>,
    pub last_operation: Option<OperationStatus>,
    pub active_operation: Option<OperationStatus>,
    pub news_list_state: ratatui::widgets::ListState,
    pub feeds_list_state: ratatui::widgets::TableState,
    pub search_list_state: ratatui::widgets::ListState,
    pub article_scroll_offset: u16,
}

impl Default for App {
    fn default() -> Self {
        Self::new()
    }
}

impl App {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let mut app = Self {
            running: true,
            current_screen: Screen::Dashboard,
            items: Vec::new(),
            selected_item: 0,
            search_query: String::new(),
            search_results: Vec::new(),
            is_searching: false,
            feeds: Vec::new(),
            selected_feed: 0,
            current_theme: Theme::Default,
            db: Repository::new(get_db_path()).ok(),
            popup: PopupState::None,
            image_tx: tx,
            image_rx: rx,
            current_image: None,
            pending_action: None,
            last_operation: None,
            active_operation: None,
            news_list_state: ratatui::widgets::ListState::default(),
            feeds_list_state: ratatui::widgets::TableState::default(),
            search_list_state: ratatui::widgets::ListState::default(),
            article_scroll_offset: 0,
        };
        app.load_items();
        app
    }

    pub fn request_action(&mut self, action: ActionRequest) {
        self.pending_action = Some(action);
    }

    pub fn set_status(&mut self, msg: impl Into<String>) {
        self.last_operation = Some(OperationStatus {
            command: "System".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            success: true,
            message: msg.into(),
            details: None,
        });
    }

    pub fn set_error(&mut self, msg: impl Into<String>) {
        self.last_operation = Some(OperationStatus {
            command: "System".to_string(),
            started_at: Utc::now(),
            completed_at: Some(Utc::now()),
            success: false,
            message: msg.into(),
            details: None,
        });
    }

    pub async fn process_pending_action(&mut self) {
        if let Some(action) = self.pending_action.take() {
            let action_name = action.clone();
            
            self.active_operation = Some(OperationStatus {
                command: format!("{:?}", action_name),
                started_at: Utc::now(),
                completed_at: None,
                success: false,
                message: "Running...".to_string(),
                details: None,
            });

            // Release the local DB connection so commands can acquire the lock
            self.db = None;

            let result = match action {
                ActionRequest::Install => install_command().await.map_err(|e| e.to_string()),
                ActionRequest::FetchAll => fetch_command(None).await.map_err(|e| e.to_string()),
                ActionRequest::FetchSelectedFeed(id) => fetch_command(id).await.map_err(|e| e.to_string()),
                ActionRequest::FeedsSync => feeds_sync().await.map_err(|e| e.to_string()),
                ActionRequest::Status => status_command().await.map_err(|e| e.to_string()),
                ActionRequest::Cleanup => cleanup_command().await.map_err(|e| e.to_string()),
                ActionRequest::Config => config_command().await.map_err(|e| e.to_string()),
                ActionRequest::StartServer => {
                    let db = match Repository::new(get_db_path()) {
                        Ok(db) => db,
                        Err(err) => {
                            self.set_error(format!("Unable to start server: {}", err));
                            self.active_operation = None;
                            return;
                        }
                    };
                    let state = AppState { db: Arc::new(db) };
                    let app = create_router(state);
                    tokio::spawn(async move {
                        if let Ok(listener) = TcpListener::bind("127.0.0.1:8080").await {
                            let _ = axum::serve(listener, app).await;
                        }
                    });
                    self.set_status("Background server started on http://127.0.0.1:8080");
                    self.active_operation = None;
                    return;
                }
                ActionRequest::RunForeground => {
                    let db = match Repository::new(get_db_path()) {
                        Ok(db) => db,
                        Err(err) => {
                            self.set_error(format!("Unable to start background mode: {}", err));
                            self.active_operation = None;
                            return;
                        }
                    };
                    let db_arc = Arc::new(db);
                    let state = AppState { db: db_arc.clone() };
                    let app = create_router(state);
                    tokio::spawn(async move {
                        if let Ok(listener) = TcpListener::bind("127.0.0.1:8080").await {
                            let _ = axum::serve(listener, app).await;
                        }
                    });
                    let db2 = db_arc.clone();
                    tokio::spawn(async move {
                        let fetcher = haxnews_core::feed::fetcher::FeedFetcher::new();
                        loop {
                            if let Ok(feeds) = db2.get_all_feeds() {
                                for feed in feeds {
                                    if let Ok(content) = fetcher.fetch(&feed.url).await {
                                        if let Ok(items) = haxnews_core::feed::parser::FeedParser::parse(feed.id, &content, &feed.name) {
                                            for item in items {
                                                let _ = db2.save_item(&item);
                                            }
                                        }
                                    }
                                }
                            }
                            tokio::time::sleep(std::time::Duration::from_secs(3600)).await;
                        }
                    });
                    self.set_status("Run mode started: API + hourly fetch loop on http://127.0.0.1:8080");
                    self.active_operation = None;
                    return;
                }
            };

            // Re-acquire the DB lock after command finishes
            if let Ok(repo) = Repository::new(crate::get_db_path()) {
                self.db = Some(repo);
            }

            let mut op = self.active_operation.take().unwrap();
            op.completed_at = Some(Utc::now());

            match result {
                Ok(cmd_result) => {
                    op.success = cmd_result.success;
                    op.message = cmd_result.message;
                    op.details = cmd_result.details;
                }
                Err(err) => {
                    op.success = false;
                    op.message = format!("Action failed: {}", err);
                }
            }
            self.last_operation = Some(op);
            self.load_items();
        }
    }

    pub fn load_items(&mut self) {
        if let Some(db) = &self.db {
            if let Ok(feeds) = db.get_all_feeds() {
                self.feeds = feeds;
            }
            let mut category_map = std::collections::HashMap::new();
            for feed in &self.feeds {
                category_map.insert(feed.id.to_string(), feed.category.clone().unwrap_or_else(|| "General".to_string()));
            }

            if let Ok(db_items) = db.get_items(Some(100), None, None) {
                self.items = db_items.into_iter().map(|item| {
                    let category = category_map.get(&item.feed_id.to_string()).cloned().unwrap_or_else(|| "General".to_string());
                    NewsItemUI {
                        title: item.title,
                        url: item.link,
                        author: item.author,
                        description: item.summary.unwrap_or_default(),
                        image_url: item.image_url,
                        published_at: item.published_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
                        feed_name: item.org,
                        category,
                    }
                }).collect();
            }
            if !self.items.is_empty() && self.news_list_state.selected().is_none() {
                self.news_list_state.select(Some(0));
                self.selected_item = 0;
            }
        }
        self.trigger_image_load();
    }

    pub fn next_item(&mut self) {
        if !self.items.is_empty() {
            self.selected_item = (self.selected_item + 1) % self.items.len();
            self.news_list_state.select(Some(self.selected_item));
            self.current_image = None;
            self.article_scroll_offset = 0;
            self.trigger_image_load();
        }
    }

    pub fn prev_item(&mut self) {
        if !self.items.is_empty() {
            self.selected_item = if self.selected_item == 0 {
                self.items.len() - 1
            } else {
                self.selected_item - 1
            };
            self.news_list_state.select(Some(self.selected_item));
            self.current_image = None;
            self.article_scroll_offset = 0;
            self.trigger_image_load();
        }
    }

    pub fn trigger_image_load(&self) {
        if let Some(item) = self.items.get(self.selected_item) {
            if let Some(url) = item.image_url.clone() {
                let tx = self.image_tx.clone();
                tokio::spawn(async move {
                    if let Ok(resp) = reqwest::get(&url).await {
                        if let Ok(bytes) = resp.bytes().await {
                            let _ = tx.send(bytes.to_vec());
                        }
                    }
                });
            }
        }
    }

    pub fn search(&mut self) {
        if self.search_query.trim().is_empty() {
            self.clear_search();
            return;
        }
        
        let mut category_map = std::collections::HashMap::new();
        for feed in &self.feeds {
            category_map.insert(feed.id.to_string(), feed.category.clone().unwrap_or_else(|| "General".to_string()));
        }

        if let Some(db) = &self.db {
            if let Ok(db_items) = db.get_items(Some(50), None, Some(&self.search_query)) {
                self.search_results = db_items.into_iter().map(|item| {
                    let category = category_map.get(&item.feed_id.to_string()).cloned().unwrap_or_else(|| "General".to_string());
                    NewsItemUI {
                        title: item.title,
                        url: item.link,
                        author: item.author,
                        description: item.summary.unwrap_or_default(),
                        image_url: item.image_url,
                        published_at: item.published_at.map(|d| d.format("%Y-%m-%d %H:%M").to_string()),
                        feed_name: item.org,
                        category,
                    }
                }).collect();
            }
        } else {
            // Fallback to in-memory search if DB failed
            let query = self.search_query.to_lowercase();
            self.search_results = self
                .items
                .iter()
                .filter(|item| {
                    item.title.to_lowercase().contains(&query)
                        || item.description.to_lowercase().contains(&query)
                })
                .cloned()
                .collect();
        }
        if !self.search_results.is_empty() {
            self.search_list_state.select(Some(0));
        } else {
            self.search_list_state.select(None);
        }
        self.is_searching = true;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_results.clear();
        self.is_searching = false;
    }

    pub fn quit(&mut self) {
        self.running = false;
    }
}
