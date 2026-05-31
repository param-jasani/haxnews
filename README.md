# HaxNews - RSS Feed Aggregator

A high-performance RSS feed parser and aggregator built in Rust with deduplication, full-text search, and REST API support.

## Features

✅ **RSS Feed Parser** - Parses RSS 2.0 feeds with support for:
  - Item extraction (title, description, link, author, publication date)
  - Image URLs from enclosures
  - SHA256-based deduplication hashing

✅ **Feed Fetcher** - Asynchronous HTTP client for fetching feeds
  - Configurable timeouts (15 seconds default)
  - User-agent support
  - Error handling and retries

✅ **Deduplication Engine** - Intelligent duplicate detection:
  - Jaro-Winkler similarity scoring
  - Configurable similarity threshold
  - Batch duplicate detection and filtering

✅ **Database Layer** - Persistent storage using ReDB:
  - JSON serialization for complex types
  - Transaction support
  - Automatic cleanup of old items

✅ **REST API** - Axum-based web server:
  - Health check endpoint
  - Items retrieval
  - Full-text search

✅ **Text Processing** - Utilities for:
  - Text normalization
  - HTML tag stripping
  - Search text generation

## Project Structure

```
haxnews-core/
├── src/
│   ├── api/               # REST API handlers, routes, responses
│   ├── config/            # Configuration loading
│   ├── db/                # Database abstraction and repository
│   ├── dedup/             # Deduplication engine and similarity scoring
│   ├── feed/              # RSS parser and feed fetcher
│   ├── models/            # Data models (FeedSource, NewsItem)
│   ├── service/           # Business logic (AggregatorService)
│   ├── utils/             # Utilities (hashing, normalization)
│   └── lib.rs
└── Cargo.toml
```

## Data Models

### FeedSource
Represents an RSS feed source with metadata:
- `id`: Unique identifier (UUID)
- `name`: Feed name
- `url`: Feed URL
- `priority`: Processing priority
- `category`: Optional category
- `status`: Active, Error, Disabled, or Paused
- `etag` / `last_modified`: HTTP caching headers

### NewsItem
Represents a parsed news item from an RSS feed:
- `id`: Unique identifier
- `feed_id`: Reference to source feed
- `title`: Article title
- `summary`: Article summary/description
- `image_url`: Associated image
- `author`: Article author
- `link`: Article URL
- `published_at`: Publication timestamp
- `dedup_hash`: SHA256 hash for deduplication
- `search_text`: Normalized text for search

## Usage

### Parsing an RSS Feed

```rust
use haxnews_core::feed::parser::FeedParser;
use uuid::Uuid;

let feed_id = Uuid::new_v4();
let feed_content = "<?xml version=\"1.0\"?>...</rss>";

let items = FeedParser::parse(feed_id, feed_content, "Feed Name")?;
for item in items {
    println!("{}: {}", item.title, item.link);
}
```

### Fetching and Parsing a Feed

```rust
use haxnews_core::feed::fetcher::FeedFetcher;
use haxnews_core::feed::parser::FeedParser;

let fetcher = FeedFetcher::new();
let content = fetcher.fetch("https://example.com/feed.xml").await?;
let items = FeedParser::parse(feed_id, &content, "Example")?;
```

### Using the Aggregator Service

```rust
use haxnews_core::service::AggregatorService;

let aggregator = AggregatorService::new(0.85); // 85% similarity threshold

let feeds = vec![
    (uuid1, "https://feed1.com/rss".to_string(), "Feed 1".to_string()),
    (uuid2, "https://feed2.com/rss".to_string(), "Feed 2".to_string()),
];

let items = aggregator.fetch_feeds(feeds).await?;
```

### Deduplication

```rust
use haxnews_core::dedup::DedupEngine;

let engine = DedupEngine::new(0.85); // 85% threshold
let duplicates = engine.find_duplicates(&items);
let unique_items = engine.deduplicate(items);
```

### Database Operations

```rust
use haxnews_core::db::Repository;

let repo = Repository::new("./news.db")?;

// Save a feed
repo.save_feed(&feed_source)?;

// Get all feeds
let feeds = repo.get_all_feeds()?;

// Save an item
repo.save_item(&news_item)?;

// Delete old items (older than N days)
repo.delete_old_items(30)?;
```

## Running the Example

```bash
cargo run --example test-parser
```

Output:
```
🔍 Testing RSS Feed Parser
========================

✅ Successfully parsed 3 items

Item 1:
  Title: PAN-OS GlobalProtect Authentication Bypass (CVE-2026-0257)...
  Link: https://thehackernews.com/2026/05/pan-os-globalprotect-authentication.html
  Author: author
  Published: Sat, 30 May 2026 06:41:26 +0000
  Image URL: https://blogger.googleusercontent.com/img/b/R29vZ2xl/AVv...
  Dedup Hash: 06730d5729720c59
```

## Dependencies

Key dependencies in Cargo.toml:
- `tokio` - Async runtime
- `axum` - Web framework
- `feed-rs` - RSS/Atom feed parsing
- `redb` - Embedded database
- `serde_json` - JSON serialization
- `chrono` - Date/time handling
- `uuid` - UUID generation
- `sha2` - SHA256 hashing
- `hex` - Hex encoding
- `strsim` - String similarity

## API Endpoints (Planned)

- `GET /health` - Health check
- `GET /items` - Get all news items
- `GET /search?q=query` - Search news items
- `GET /feeds` - List configured feeds
- `POST /feeds` - Add a new feed

## Future Enhancements

- [ ] Feed caching with ETags
- [ ] Rate limiting
- [ ] User authentication
- [ ] Saved articles/bookmarks
- [ ] Category filtering
- [ ] Advanced search syntax
- [ ] Feed validation
- [ ] Atom feed support
- [ ] JSON Feed support
- [ ] WebSocket updates

## Testing

Run tests:
```bash
cargo test
```

Run tests with output:
```bash
cargo test -- --nocapture
```

## Build

Development build:
```bash
cargo build
```

Release build:
```bash
cargo build --release
```

## License

MIT
