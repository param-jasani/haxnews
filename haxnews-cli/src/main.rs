mod commands;
mod tui;

use clap::{Parser, Subcommand, Args};
use anyhow::Result;
use commands::*;
use tracing::Level;
use tracing_subscriber::FmtSubscriber;
use std::path::PathBuf;

#[derive(Parser)]
#[command(
    name = "haxnews",
    version = "0.1.0",
    author = "Param Jasani / Haxnation",
    about = "HaxNews - RSS Feed Aggregator CLI",
    long_about = "A high-performance RSS feed parser and aggregator with TUI support"
)]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Run the interactive TUI (default if no command given)
    #[arg(short, long)]
    interactive: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// First time setup (create config dir, default feeds.toml)
    Install,

    /// Run in foreground (API + fetcher)
    Run,

    /// Run the server API mode
    Server,

    /// Manually trigger fetch
    Fetch {
        /// Fetch only specific feed
        #[arg(long)]
        feed: Option<String>,
    },

    /// Manage feeds
    Feeds(FeedsArgs),

    /// Show latest news items
    Items {
        /// Search items
        #[arg(long)]
        search: Option<String>,
        
        /// Limit output
        #[arg(long, default_value = "50")]
        limit: usize,
    },

    /// Overall system status
    Status,

    /// Delete old items (older than 60 days)
    Cleanup,

    /// Show config paths
    Config,
    
    /// Run the interactive TUI
    Tui,
}

#[derive(Args)]
struct FeedsArgs {
    #[command(subcommand)]
    action: FeedsAction,
}

#[derive(Subcommand)]
enum FeedsAction {
    List,
    Add,
    Sync,
}

pub fn get_data_dir() -> PathBuf {
    let mut path = dirs::config_dir().unwrap_or_else(|| PathBuf::from("."));
    path.push("haxnews");
    path
}

pub fn get_db_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("haxnews.db");
    path
}

pub fn get_config_path() -> PathBuf {
    let mut path = get_data_dir();
    path.push("feeds.toml");
    path
}

#[tokio::main]
async fn main() -> Result<()> {
    // Setup tracing
    let subscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    if let Err(err) = tracing::subscriber::set_global_default(subscriber) {
        eprintln!("Unable to set tracing subscriber: {}", err);
    }

    let cli = Cli::parse();

    match cli.command {
        Some(Commands::Install) => install_command().await?,
        Some(Commands::Run) => run_command_fg().await?,
        Some(Commands::Server) => server_start().await?,
        Some(Commands::Fetch { feed }) => fetch_command(feed).await?,
        Some(Commands::Feeds(args)) => match args.action {
            FeedsAction::List => feeds_list().await?,
            FeedsAction::Add => feeds_add().await?,
            FeedsAction::Sync => feeds_sync().await?,
        },
        Some(Commands::Items { search, limit }) => items_command(search, limit).await?,
        Some(Commands::Status) => status_command().await?,
        Some(Commands::Cleanup) => cleanup_command().await?,
        Some(Commands::Config) => config_command().await?,
        Some(Commands::Tui) => tui::run_tui().await?,
        None => {
            // Default to TUI
            tui::run_tui().await?;
        }
    }

    Ok(())
}
