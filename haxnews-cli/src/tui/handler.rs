use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyModifiers};
use crate::tui::app::{App, ActionRequest, FeedAddStage, PopupState, Screen, Theme};
use haxnews_core::models::{FeedSource, FeedStatus};
use anyhow::Result;

pub fn handle_events(app: &mut App) -> Result<()> {
    if event::poll(std::time::Duration::from_millis(50))? {
        if let Event::Key(key) = event::read()? {
            if key.kind != crossterm::event::KeyEventKind::Press {
                return Ok(());
            }
            if let PopupState::PauseFeedInput { ref mut input } = app.popup {
                match key.code {
                    KeyCode::Esc => {
                        app.popup = PopupState::None;
                    }
                    KeyCode::Enter => {
                        let minutes = input.parse::<u32>().unwrap_or(60);
                        if let Some(db) = &app.db {
                            if let Some(feed) = app.feeds.get_mut(app.selected_feed) {
                                let mut updated = feed.clone();
                                updated.status = FeedStatus::Paused(minutes);
                                if let Err(err) = db.save_feed(&updated) {
                                    app.set_error(format!("Unable to pause feed: {}", err));
                                } else {
                                    *feed = updated;
                                    app.set_status("Feed paused successfully.");
                                }
                            }
                        }
                        app.popup = PopupState::None;
                    }
                    KeyCode::Backspace => {
                        input.pop();
                    }
                    KeyCode::Char(c) if c.is_ascii_digit() => {
                        input.push(c);
                    }
                    _ => {}
                }
                return Ok(());
            }

            if let PopupState::AddFeedInput { stage, name, url, priority, category } = &mut app.popup {
                match key.code {
                    KeyCode::Esc => {
                        app.popup = PopupState::None;
                    }
                    KeyCode::Enter => {
                        match stage {
                            FeedAddStage::Name => *stage = FeedAddStage::Url,
                            FeedAddStage::Url => *stage = FeedAddStage::Priority,
                            FeedAddStage::Priority => *stage = FeedAddStage::Category,
                            FeedAddStage::Category => {
                                let priority_value = priority.trim().parse::<u32>().unwrap_or(10);
                                let category_value = if category.trim().is_empty() { "News".to_string() } else { category.trim().to_string() };
                                let feed = FeedSource::new(
                                    name.trim().to_string(),
                                    url.trim().to_string(),
                                    priority_value,
                                    60,
                                    Some(category_value),
                                    Some(FeedStatus::Active),
                                );
                                if let Some(db) = &app.db {
                                    if let Err(err) = db.save_feed(&feed) {
                                        app.set_error(format!("Unable to save feed: {}", err));
                                    } else {
                                        app.feeds.push(feed);
                                        app.set_status("Feed added successfully.");
                                    }
                                }
                                app.popup = PopupState::None;
                            }
                        }
                    }
                    KeyCode::Backspace => {
                        let field = match stage {
                            FeedAddStage::Name => name,
                            FeedAddStage::Url => url,
                            FeedAddStage::Priority => priority,
                            FeedAddStage::Category => category,
                        };
                        field.pop();
                    }
                    KeyCode::Char(c) => {
                        let field = match stage {
                            FeedAddStage::Name => name,
                            FeedAddStage::Url => url,
                            FeedAddStage::Priority => priority,
                            FeedAddStage::Category => category,
                        };
                        field.push(c);
                    }
                    _ => {}
                }
                return Ok(());
            }

            // Global Keys
            match key.code {
                KeyCode::Char('q') => {
                    app.quit();
                    return Ok(());
                }
                KeyCode::Tab => {
                    app.current_screen = match app.current_screen {
                        Screen::Dashboard => Screen::News,
                        Screen::News | Screen::Search => Screen::Feeds,
                        Screen::Feeds => Screen::Settings,
                        Screen::Settings => Screen::Dashboard,
                    };
                    return Ok(());
                }
                KeyCode::BackTab => {
                    app.current_screen = match app.current_screen {
                        Screen::Dashboard => Screen::Settings,
                        Screen::News | Screen::Search => Screen::Dashboard,
                        Screen::Feeds => Screen::News,
                        Screen::Settings => Screen::Feeds,
                    };
                    return Ok(());
                }
                KeyCode::Char('1') => { app.current_screen = Screen::Dashboard; return Ok(()); }
                KeyCode::Char('2') => { app.current_screen = Screen::News; return Ok(()); }
                KeyCode::Char('3') => { app.current_screen = Screen::Feeds; return Ok(()); }
                KeyCode::Char('4') => { app.current_screen = Screen::Settings; return Ok(()); }
                _ => {}
            }

            match app.current_screen {
                Screen::Dashboard => handle_dashboard_keys(app, key),
                Screen::News => handle_news_keys(app, key),
                Screen::Search => handle_search_keys(app, key),
                Screen::Feeds => handle_feeds_keys(app, key),
                Screen::Settings => handle_settings_keys(app, key),
            }
        }
    }
    Ok(())
}

fn handle_dashboard_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Char('s') => app.current_screen = Screen::Search,
        _ => {}
    }
}

fn handle_news_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Down | KeyCode::Char('j') => app.next_item(),
        KeyCode::Up | KeyCode::Char('k') => app.prev_item(),
        KeyCode::PageDown => {
            app.article_scroll_offset = app.article_scroll_offset.saturating_add(3);
        }
        KeyCode::PageUp => {
            app.article_scroll_offset = app.article_scroll_offset.saturating_sub(3);
        }
        KeyCode::Char('s') => app.current_screen = Screen::Search,
        _ => {}
    }
}

fn handle_search_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Esc => {
            app.clear_search();
            app.current_screen = Screen::News;
        }
        KeyCode::Enter => {
            app.search();
        }
        KeyCode::Backspace => {
            app.search_query.pop();
            app.search();
        }
        KeyCode::Char(c) => {
            if key.modifiers.contains(KeyModifiers::CONTROL) && c == 'u' {
                app.search_query.clear();
                app.search_results.clear();
            } else {
                app.search_query.push(c);
                app.search();
            }
        }
        _ => {}
    }
}

fn handle_feeds_keys(app: &mut App, key: KeyEvent) {
    if app.feeds.is_empty() {
        if let KeyCode::Char('a') = key.code {
            app.popup = PopupState::AddFeedInput {
                stage: FeedAddStage::Name,
                name: String::new(),
                url: String::new(),
                priority: String::from("10"),
                category: String::from("News"),
            };
        }
        return;
    }

    match key.code {
        KeyCode::Down | KeyCode::Char('j') => {
            app.selected_feed = (app.selected_feed + 1) % app.feeds.len();
            app.feeds_list_state.select(Some(app.selected_feed));
        }
        KeyCode::Up | KeyCode::Char('k') => {
            app.selected_feed = if app.selected_feed == 0 {
                app.feeds.len() - 1
            } else {
                app.selected_feed - 1
            };
            app.feeds_list_state.select(Some(app.selected_feed));
        }
        KeyCode::Char('p') => {
            app.popup = PopupState::PauseFeedInput { input: String::from("60") };
        }
        KeyCode::Char('e') => {
            if let Some(db) = &app.db {
                if let Some(feed) = app.feeds.get_mut(app.selected_feed) {
                    let mut updated = feed.clone();
                    updated.status = FeedStatus::Active;
                    if let Err(err) = db.save_feed(&updated) {
                        app.set_error(format!("Unable to enable feed: {}", err));
                    } else {
                        *feed = updated;
                        app.set_status("Feed enabled.");
                    }
                }
            }
        }
        KeyCode::Char('d') => {
            if let Some(db) = &app.db {
                if let Some(feed) = app.feeds.get_mut(app.selected_feed) {
                    let mut updated = feed.clone();
                    updated.status = FeedStatus::Disabled;
                    if let Err(err) = db.save_feed(&updated) {
                        app.set_error(format!("Unable to disable feed: {}", err));
                    } else {
                        *feed = updated;
                        app.set_status("Feed disabled.");
                    }
                }
            }
        }
        KeyCode::Char('x') => {
            if let Some(db) = &app.db {
                let id = app.feeds[app.selected_feed].id.to_string();
                match db.delete_feed(&id) {
                    Ok(true) => {
                        app.feeds.remove(app.selected_feed);
                        app.selected_feed = app.selected_feed.saturating_sub(1);
                        app.feeds_list_state.select(Some(app.selected_feed));
                        app.set_status("Feed removed.");
                    }
                    Ok(false) => app.set_error("Feed not found.".to_string()),
                    Err(err) => app.set_error(format!("Unable to remove feed: {}", err)),
                }
            }
        }
        KeyCode::Char('a') => {
            app.popup = PopupState::AddFeedInput {
                stage: FeedAddStage::Name,
                name: String::new(),
                url: String::new(),
                priority: String::from("10"),
                category: String::from("News"),
            };
        }
        KeyCode::Char('y') => {
            app.request_action(ActionRequest::FeedsSync);
        }
        KeyCode::Char('f') => {
            let feed_id = app.feeds.get(app.selected_feed).map(|feed| feed.id.to_string());
            app.request_action(ActionRequest::FetchSelectedFeed(feed_id));
        }
        _ => {}
    }
}

fn handle_settings_keys(app: &mut App, key: KeyEvent) {
    match key.code {
        KeyCode::Enter => {
            app.current_theme = match app.current_theme {
                Theme::Default => Theme::Cyberpunk,
                Theme::Cyberpunk => Theme::Monokai,
                Theme::Monokai => Theme::Ocean,
                Theme::Ocean => Theme::Dracula,
                Theme::Dracula => Theme::Gruvbox,
                Theme::Gruvbox => Theme::Default,
            };
        }
        KeyCode::Char('i') => {
            app.request_action(ActionRequest::Install);
        }
        KeyCode::Char('f') => {
            app.request_action(ActionRequest::FetchAll);
        }
        KeyCode::Char('y') => {
            app.request_action(ActionRequest::FeedsSync);
        }
        KeyCode::Char('a') => {
            app.popup = PopupState::AddFeedInput {
                stage: FeedAddStage::Name,
                name: String::new(),
                url: String::new(),
                priority: String::from("10"),
                category: String::from("News"),
            };
        }
        KeyCode::Char('t') => {
            app.request_action(ActionRequest::Status);
        }
        KeyCode::Char('c') => {
            app.request_action(ActionRequest::Config);
        }
        KeyCode::Char('l') => {
            app.request_action(ActionRequest::Cleanup);
        }
        KeyCode::Char('r') => {
            app.request_action(ActionRequest::StartServer);
        }
        KeyCode::Char('R') => {
            app.request_action(ActionRequest::RunForeground);
        }
        _ => {}
    }
}
