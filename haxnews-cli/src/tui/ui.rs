use ratatui::{
    layout::{Alignment, Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, List, ListItem, Paragraph, Wrap, Tabs, Cell, Row, Table, BorderType},
    Frame,
};
use crate::tui::app::{App, FeedAddStage, Screen, Theme, PopupState};

struct ThemeColors {
    primary: Color,
    secondary: Color,
    accent: Color,
    text: Color,
    highlight: Color,
}

impl ThemeColors {
    fn from(theme: Theme) -> Self {
        match theme {
            Theme::Default => Self {
                primary: Color::Cyan,
                secondary: Color::Gray,
                accent: Color::Yellow,
                text: Color::White,
                highlight: Color::DarkGray,
            },
            Theme::Cyberpunk => Self {
                primary: Color::Magenta,
                secondary: Color::Cyan,
                accent: Color::Yellow,
                text: Color::LightGreen,
                highlight: Color::Rgb(50, 0, 50),
            },
            Theme::Monokai => Self {
                primary: Color::Rgb(249, 38, 114),
                secondary: Color::Rgb(102, 217, 239),
                accent: Color::Rgb(166, 226, 46),
                text: Color::Rgb(248, 248, 242),
                highlight: Color::Rgb(73, 72, 62),
            },
            Theme::Ocean => Self {
                primary: Color::LightBlue,
                secondary: Color::Cyan,
                accent: Color::White,
                text: Color::Rgb(200, 200, 220),
                highlight: Color::Rgb(23, 42, 69),
            },
            Theme::Dracula => Self {
                primary: Color::Rgb(189, 147, 249),
                secondary: Color::Rgb(98, 114, 164),
                accent: Color::Rgb(255, 121, 198),
                text: Color::Rgb(248, 248, 242),
                highlight: Color::Rgb(68, 71, 90),
            },
            Theme::Gruvbox => Self {
                primary: Color::Rgb(250, 189, 47),
                secondary: Color::Rgb(146, 131, 116),
                accent: Color::Rgb(254, 128, 25),
                text: Color::Rgb(235, 219, 178),
                highlight: Color::Rgb(60, 56, 54),
            },
        }
    }
}

pub fn draw(f: &mut Frame, app: &mut App) {
    let colors = ThemeColors::from(app.current_theme);
    
    let size = f.area();
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3), // Tabs
            Constraint::Min(5),    // Main content
            Constraint::Length(3), // Footer / status
        ])
        .split(size);

    // Render Tabs
    let titles: Vec<Line> = vec!["Dashboard", "News", "Feeds", "Settings"]
        .iter()
        .map(|t| Line::from(*t))
        .collect();

    let tab_index = match app.current_screen {
        Screen::Dashboard => 0,
        Screen::News | Screen::Search => 1,
        Screen::Feeds => 2,
        Screen::Settings => 3,
    };

    let tabs = Tabs::new(titles)
        .block(Block::default().borders(Borders::ALL).title(" ⬡ HΛX NEWS ").border_style(Style::default().fg(colors.primary)))
        .highlight_style(Style::default().fg(colors.accent).add_modifier(Modifier::BOLD | Modifier::REVERSED))
        .select(tab_index)
        .style(Style::default().fg(colors.text))
        .divider(" | ");

    f.render_widget(tabs, chunks[0]);

    // Render Main Content
    match app.current_screen {
        Screen::Dashboard => draw_dashboard(f, app, chunks[1], &colors),
        Screen::News => draw_news(f, app, chunks[1], &colors),
        Screen::Search => draw_search(f, app, chunks[1], &colors),
        Screen::Feeds => draw_feeds(f, app, chunks[1], &colors),
        Screen::Settings => draw_settings(f, app, chunks[1], &colors),
    }

    draw_footer(f, app, chunks[2], &colors);

    if !matches!(app.popup, PopupState::None) {
        draw_popup(f, &app.popup, &colors);
    }
}

fn draw_dashboard(f: &mut Frame, app: &mut App, area: Rect, colors: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(12), // Logo
            Constraint::Length(8),  // Welcome/Help
            Constraint::Min(5)      // Top 5 Cards
        ])
        .split(area);

    let logo_lines = vec![
        Line::from(vec![
            Span::styled("██╗  ██╗  █████╗  ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("██╗  ", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("██╗", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("    ███╗   ██╗", Style::default().fg(colors.secondary)),
        ]),
        Line::from(vec![
            Span::styled("██║  ██║ ██╔══██╗ ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("╚██╗", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("██╔╝", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("    ████╗  ██║", Style::default().fg(colors.secondary)),
        ]),
        Line::from(vec![
            Span::styled("███████║ ███████║  ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("╚███╔╝ ", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("    ██╔██╗ ██║", Style::default().fg(colors.secondary)),
        ]),
        Line::from(vec![
            Span::styled("██╔══██║ ██╔══██║  ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("██╔", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("██╗ ", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("    ██║╚██╗██║", Style::default().fg(colors.secondary)),
        ]),
        Line::from(vec![
            Span::styled("██║  ██║ ██║  ██║ ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("██╔╝ ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("██╗", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("    ██║ ╚████║", Style::default().fg(colors.secondary)),
        ]),
        Line::from(vec![
            Span::styled("╚═╝  ╚═╝ ╚═╝  ╚═╝ ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("╚═╝  ", Style::default().fg(colors.text).add_modifier(Modifier::BOLD)),
            Span::styled("╚═╝", Style::default().fg(Color::Rgb(50, 200, 200))),
            Span::styled("    ╚═╝  ╚═══╝", Style::default().fg(colors.secondary)),
        ]),
        Line::from(""),
        Line::from(Span::styled("Made with love by team Haxnation", Style::default().fg(Color::Red).add_modifier(Modifier::ITALIC))),
    ];
    
    let logo = Paragraph::new(logo_lines)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.secondary)));
    
    f.render_widget(logo, chunks[0]);

    // Welcome and Help Text
    let welcome_text = vec![
        Line::from(Span::styled("Welcome to HaxNews!!", Style::default().fg(colors.text).add_modifier(Modifier::BOLD))),
        Line::from("A high-performance RSS feed parser and aggregator."),
        Line::from(""),
        Line::from(vec![
            Span::styled("Quick Navigation: ", Style::default().fg(colors.secondary)),
            Span::styled("[Tab] ", Style::default().fg(colors.accent)),
            Span::raw("or "),
            Span::styled("[1-4] ", Style::default().fg(colors.accent)),
            Span::raw("to switch tabs. "),
            Span::styled("[q] ", Style::default().fg(colors.accent)),
            Span::raw("to quit.")
        ]),
        Line::from(vec![
            Span::styled("In News/Search: ", Style::default().fg(colors.secondary)),
            Span::styled("[j/k] ", Style::default().fg(colors.accent)),
            Span::raw("to navigate items. "),
            Span::styled("[s] ", Style::default().fg(colors.accent)),
            Span::raw("to search.")
        ]),
    ];

    let welcome = Paragraph::new(welcome_text)
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::BOTTOM).border_style(Style::default().fg(colors.highlight)));
    f.render_widget(welcome, chunks[1]);

    // Top 5 news cards
    let top_items: Vec<_> = app.items.iter().take(5).collect();
    
    if top_items.is_empty() {
        let empty = Paragraph::new("No news items available. Sync feeds first.")
            .alignment(Alignment::Center)
            .style(Style::default().fg(colors.secondary));
        f.render_widget(empty, chunks[2]);
        return;
    }

    let card_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints(vec![Constraint::Percentage(20); 5])
        .split(chunks[2]);

    for (i, item) in top_items.iter().enumerate() {
        if i >= 5 { break; }
        
        let mut lines = vec![
            Line::from(Span::styled(&item.title, Style::default().fg(colors.text).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(Span::styled(item.feed_name.as_deref().unwrap_or("Unknown Feed"), Style::default().fg(colors.secondary))),
            Line::from(Span::styled(item.published_at.as_deref().unwrap_or(""), Style::default().fg(colors.secondary))),
            Line::from(""),
        ];
        
        let clean_desc = html2text::from_read(item.description.as_bytes(), 30);
        let desc_lines: Vec<&str> = clean_desc.lines().take(6).collect();
        for line in desc_lines {
            lines.push(Line::from(Span::styled(line, Style::default().fg(Color::DarkGray))));
        }

        let card = Paragraph::new(lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.primary)))
            .wrap(Wrap { trim: true });
        
        f.render_widget(card, card_chunks[i]);
    }
}

fn draw_news(f: &mut Frame, app: &mut App, area: Rect, colors: &ThemeColors) {
    let main_chunks = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(area);

    // --- Left Pane: News Stream ---
    let left_chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(3)])
        .split(main_chunks[0]);

    let item_count = app.items.len();
    let pos = if item_count > 0 { app.selected_item + 1 } else { 0 };
    let header_text = format!(" News Stream [{}/{}] ", pos, item_count);
    let stream_header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("↑↓/jk", Style::default().fg(colors.accent)),
            Span::raw(" navigate  "),
            Span::styled("s", Style::default().fg(colors.accent)),
            Span::raw(" search"),
        ]),
    ])
    .alignment(Alignment::Center)
    .block(Block::default().title(header_text).borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.primary)));
    f.render_widget(stream_header, left_chunks[0]);

    let items: Vec<ListItem> = app.items.iter().enumerate().map(|(i, item)| {
        let is_selected = i == app.selected_item;
        let indicator = if is_selected { "▶ " } else { "  " };
        let title_style = if is_selected {
            Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.text)
        };

        let time_str = item.published_at.as_deref().unwrap_or("");

        ListItem::new(vec![
            Line::from(vec![
                Span::styled(indicator, Style::default().fg(colors.accent)),
                Span::styled(&item.title, title_style),
            ]),
            Line::from(vec![
                Span::raw("    "),
                Span::styled(format!("[{}]", item.category), Style::default().fg(colors.primary)),
                Span::raw(" "),
                Span::styled(item.feed_name.as_deref().unwrap_or(""), Style::default().fg(colors.secondary)),
                Span::raw("  "),
                Span::styled(time_str, Style::default().fg(Color::DarkGray)),
            ]),
            Line::from(""),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)));
    f.render_widget(list, left_chunks[1]);

    // --- Right Pane: Reading Pane ---
    if let Some(selected) = app.items.get(app.selected_item) {
        let has_img = app.current_image.is_some();
        let right_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(5),   // Header: title + meta
                Constraint::Length(if has_img { 16 } else { 0 }), // Image
                Constraint::Min(5),      // Body: description
                Constraint::Length(3),   // Footer: link
            ])
            .split(main_chunks[1]);

        // -- Article Header --
        let author_str = selected.author.as_deref().unwrap_or("Unknown");
        let date_str = selected.published_at.as_deref().unwrap_or("Unknown Date");
        let feed_str = selected.feed_name.as_deref().unwrap_or("Unknown Feed");

        let header_lines = vec![
            Line::from(Span::styled(&selected.title, Style::default().fg(colors.primary).add_modifier(Modifier::BOLD))),
            Line::from(""),
            Line::from(vec![
                Span::styled("  by ", Style::default().fg(Color::DarkGray)),
                Span::styled(author_str, Style::default().fg(colors.accent)),
                Span::styled("  •  ", Style::default().fg(Color::DarkGray)),
                Span::styled(date_str, Style::default().fg(colors.text)),
                Span::styled("  •  ", Style::default().fg(Color::DarkGray)),
                Span::styled(feed_str, Style::default().fg(colors.secondary)),
            ]),
        ];
        let article_header = Paragraph::new(header_lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.primary)).title(" Article "))
            .wrap(Wrap { trim: true });
        f.render_widget(article_header, right_chunks[0]);

        // -- Image --
        if has_img {
            let img_block = Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)).title(" Preview ");
            let inner = img_block.inner(right_chunks[1]);
            f.render_widget(img_block, right_chunks[1]);
            let image_widget = ratatui_image::StatefulImage::default();
            if let Some(proto) = app.current_image.as_mut() {
                f.render_stateful_widget(image_widget, inner, proto);
            }
        }

        // -- Body --
        let clean_desc = html2text::from_read(selected.description.as_bytes(), 100);
        let mut body_lines: Vec<Line> = Vec::new();
        for line in clean_desc.lines() {
            body_lines.push(Line::from(Span::styled(line, Style::default().fg(colors.text))));
        }

        let body = Paragraph::new(body_lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)).title(" Summary "))
            .wrap(Wrap { trim: true });
        f.render_widget(body, right_chunks[2]);

        // -- Footer: Link --
        let footer_lines = vec![
            Line::from(vec![
                Span::styled("  🔗 ", Style::default().fg(colors.secondary)),
                Span::styled(&selected.url, Style::default().fg(Color::LightBlue).add_modifier(Modifier::UNDERLINED)),
            ]),
        ];
        let link_footer = Paragraph::new(footer_lines)
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)));
        f.render_widget(link_footer, right_chunks[3]);

    } else {
        let empty = Paragraph::new("  No articles available. Run `haxnews fetch` to get started.")
            .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)))
            .style(Style::default().fg(Color::DarkGray))
            .alignment(Alignment::Center);
        f.render_widget(empty, main_chunks[1]);
    }
}

fn draw_search(f: &mut Frame, app: &mut App, area: Rect, colors: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5)])
        .split(area);
        
    let search_input = Paragraph::new(vec![
        Line::from(vec![Span::styled(format!("❯ {}", app.search_query), Style::default().fg(colors.accent).add_modifier(Modifier::BOLD))]),
    ])
    .block(Block::default().title(" Search ").borders(Borders::ALL).border_style(Style::default().fg(colors.primary)));
    f.render_widget(search_input, chunks[0]);

    let results: Vec<ListItem> = app.search_results.iter().map(|item| {
        ListItem::new(vec![
            Line::from(Span::styled(&item.title, Style::default().fg(colors.text))),
            Line::from(Span::styled(&item.url, Style::default().fg(colors.secondary))),
            Line::from(""),
        ])
    }).collect();

    let list = List::new(results)
        .block(Block::default().title(format!(" Results ({}) ", app.search_results.len())).borders(Borders::ALL).border_style(Style::default().fg(colors.secondary)));
    f.render_widget(list, chunks[1]);
}

fn draw_feeds(f: &mut Frame, app: &mut App, area: Rect, colors: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(5), Constraint::Length(3)])
        .split(area);

    let header_cells = ["Feed Name", "URL", "Category", "Status"]
        .iter()
        .map(|h| Cell::from(*h).style(Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)));
    let header = Row::new(header_cells).style(Style::default().bg(colors.highlight)).height(1).bottom_margin(1);

    if app.feeds.is_empty() {
        let empty = Paragraph::new("No feeds configured yet. Press [a] to add a new feed or sync existing config.")
            .style(Style::default().fg(colors.secondary))
            .alignment(Alignment::Center)
            .block(Block::default().borders(Borders::ALL).title(" Managed Feeds ").border_style(Style::default().fg(colors.primary)));
        f.render_widget(empty, chunks[0]);
    } else {
        let rows: Vec<Row> = app.feeds.iter().enumerate().map(|(i, feed)| {
            let style = if i == app.selected_feed {
                Style::default().fg(colors.accent).add_modifier(Modifier::BOLD | Modifier::REVERSED)
            } else {
                Style::default().fg(colors.text)
            };
            
            let status_str = match &feed.status {
                haxnews_core::models::FeedStatus::Active => "Active",
                haxnews_core::models::FeedStatus::Error => "Error",
                haxnews_core::models::FeedStatus::Paused(_) => "Paused",
                haxnews_core::models::FeedStatus::Disabled => "Disabled",
            };

            let cat = feed.category.as_deref().unwrap_or("General");

            Row::new(vec![
                Cell::from(feed.name.clone()),
                Cell::from(feed.url.clone()),
                Cell::from(cat),
                Cell::from(status_str),
            ]).style(style)
        }).collect();

        let table = Table::new(rows, [
            Constraint::Percentage(25),
            Constraint::Percentage(45),
            Constraint::Percentage(15),
            Constraint::Percentage(15),
        ])
        .header(header)
        .block(Block::default().borders(Borders::ALL).title(" Managed Feeds ").border_style(Style::default().fg(colors.primary)));
        f.render_widget(table, chunks[0]);
    }

    let footer = Paragraph::new(" [p] Pause  [e] Enable  [d] Disable  [x] Remove  [a] Add Feed  [y] Sync Feeds ")
        .style(Style::default().fg(colors.secondary))
        .alignment(Alignment::Center)
        .block(Block::default().borders(Borders::ALL));
    f.render_widget(footer, chunks[1]);
}

fn draw_settings(f: &mut Frame, app: &mut App, area: Rect, colors: &ThemeColors) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Length(3), Constraint::Min(5), Constraint::Length(5)])
        .split(area);

    let header = Paragraph::new(vec![
        Line::from(vec![
            Span::styled("  Current Theme: ", Style::default().fg(colors.secondary)),
            Span::styled(app.current_theme.name(), Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)),
            Span::styled("  |  Press ", Style::default().fg(Color::DarkGray)),
            Span::styled("Enter", Style::default().fg(colors.accent)),
            Span::styled(" to cycle themes", Style::default().fg(Color::DarkGray)),
        ]),
    ])
    .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.primary)).title(" Settings "));
    f.render_widget(header, chunks[0]);

    let themes = [Theme::Default, Theme::Cyberpunk, Theme::Monokai, Theme::Ocean, Theme::Dracula, Theme::Gruvbox];
    
    let items: Vec<ListItem> = themes.iter().map(|t| {
        let is_active = *t == app.current_theme;
        let prefix = if is_active { "  ● " } else { "  ○ " };
        let style = if is_active {
            Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.text)
        };

        let preview = ThemeColors::from(*t);
        ListItem::new(vec![
            Line::from(vec![
                Span::styled(prefix, style),
                Span::styled(t.name(), style),
                Span::raw("  "),
                Span::styled("███", Style::default().fg(preview.primary)),
                Span::styled("███", Style::default().fg(preview.accent)),
                Span::styled("███", Style::default().fg(preview.secondary)),
            ]),
        ])
    }).collect();

    let list = List::new(items)
        .block(Block::default().title(" Available Themes ").borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)));
    f.render_widget(list, chunks[1]);

    let info_lines = vec![
        Line::from(vec![
            Span::styled("  Keyboard Shortcuts:  ", Style::default().fg(colors.secondary)),
            Span::styled("[Tab] ", Style::default().fg(colors.accent)),
            Span::raw("Switch tabs  "),
            Span::styled("[q] ", Style::default().fg(colors.accent)),
            Span::raw("Quit  "),
            Span::styled("[1-4] ", Style::default().fg(colors.accent)),
            Span::raw("Jump to tab"),
        ]),
        Line::from(vec![
            Span::styled("  TUI Commands: ", Style::default().fg(colors.secondary)),
            Span::styled("[i] Install  ", Style::default().fg(colors.accent)),
            Span::styled("[f] Fetch  ", Style::default().fg(colors.accent)),
            Span::styled("[y] Feed Sync  ", Style::default().fg(colors.accent)),
            Span::styled("[a] Add Feed", Style::default().fg(colors.accent)),
        ]),
        Line::from(vec![
            Span::styled("  More: ", Style::default().fg(colors.secondary)),
            Span::styled("[t] Status  ", Style::default().fg(colors.accent)),
            Span::styled("[c] Config  ", Style::default().fg(colors.accent)),
            Span::styled("[l] Cleanup  ", Style::default().fg(colors.accent)),
            Span::styled("[r] Server  ", Style::default().fg(colors.accent)),
            Span::styled("[R] Run", Style::default().fg(colors.accent)),
        ]),
        Line::from(""),
        Line::from(Span::styled("  HaxNews v0.1.0 — Made with ❤ by team Haxnation", Style::default().fg(Color::DarkGray).add_modifier(Modifier::ITALIC))),
    ];
    let info = Paragraph::new(info_lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Rounded).border_style(Style::default().fg(colors.secondary)).title(" Info "));
    f.render_widget(info, chunks[2]);
}

fn draw_popup(f: &mut Frame, popup: &PopupState, colors: &ThemeColors) {
    let area = f.area();
    let popup_w = 60.min(area.width.saturating_sub(4));
    let popup_h = 9.min(area.height.saturating_sub(4));
    let x = (area.width.saturating_sub(popup_w)) / 2;
    let y = (area.height.saturating_sub(popup_h)) / 2;
    let popup_area = Rect::new(x, y, popup_w, popup_h);

    let clear = ratatui::widgets::Clear;
    f.render_widget(clear, popup_area);

    let mut popup_lines = vec![Line::from("")];
    let title = match popup {
        PopupState::PauseFeedInput { input } => {
            popup_lines.push(Line::from(Span::styled("  Pause feed for how many minutes?", Style::default().fg(colors.text))));
            popup_lines.push(Line::from(""));
            popup_lines.push(Line::from(vec![
                Span::styled("  ❯ ", Style::default().fg(colors.accent)),
                Span::styled(input, Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(colors.accent)),
            ]));
            popup_lines.push(Line::from(Span::styled("  [Enter] Confirm  [Esc] Cancel", Style::default().fg(Color::DarkGray))));
            " Pause Feed "
        }
        PopupState::AddFeedInput { stage, name, url, priority, category } => {
            let prompt = match stage {
                FeedAddStage::Name => "Enter feed name:",
                FeedAddStage::Url => "Enter feed URL:",
                FeedAddStage::Priority => "Enter priority (default 10):",
                FeedAddStage::Category => "Enter category (default News):",
            };
            let value = match stage {
                FeedAddStage::Name => name,
                FeedAddStage::Url => url,
                FeedAddStage::Priority => priority,
                FeedAddStage::Category => category,
            };
            popup_lines.push(Line::from(Span::styled(format!("  {}", prompt), Style::default().fg(colors.text))));
            popup_lines.push(Line::from(""));
            popup_lines.push(Line::from(vec![
                Span::styled("  ❯ ", Style::default().fg(colors.accent)),
                Span::styled(value, Style::default().fg(colors.primary).add_modifier(Modifier::BOLD)),
                Span::styled("█", Style::default().fg(colors.accent)),
            ]));
            popup_lines.push(Line::from(Span::styled("  [Enter] Next  [Esc] Cancel  [Backspace] Edit", Style::default().fg(Color::DarkGray))));
            " Add Feed "
        }
        PopupState::None => "",
    };

    let popup = Paragraph::new(popup_lines)
        .block(Block::default().borders(Borders::ALL).border_type(BorderType::Double).border_style(Style::default().fg(colors.accent)).title(title));
    f.render_widget(popup, popup_area);
}

fn draw_footer(f: &mut Frame, app: &App, area: Rect, colors: &ThemeColors) {
    let footer_text = if let Some(err) = &app.error_message {
        Span::styled(format!(" ERROR: {} ", err), Style::default().fg(Color::Red).add_modifier(Modifier::BOLD))
    } else if let Some(status) = &app.status_message {
        Span::styled(format!(" {} ", status), Style::default().fg(colors.accent))
    } else {
        Span::styled("[Tab] Switch  [1-4] Tabs  [q] Quit  [s] Search  [f] Fetch  [y] Sync Feeds", Style::default().fg(colors.secondary))
    };

    let footer = Paragraph::new(Line::from(footer_text))
        .block(Block::default().borders(Borders::TOP).border_style(Style::default().fg(colors.highlight)));
    f.render_widget(footer, area);
}
