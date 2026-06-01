# HaxNews

HaxNews is a production-ready RSS feed aggregator and parser written in Rust. This README is written for users and product teams who want to run, integrate, or ship HaxNews as a service or CLI tool. It explains install steps, the available APIs, configuration, screenshots, and deployment notes.

Key outputs:
- `haxnews-core` — core library (parsing, dedup, db, API)
- `haxnews-cli` — command-line interface and TUI

--

## Quick Start

Install Rust toolchain (if not already installed):

```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
rustup update stable
```

Build and run the API server (development):

```bash
cargo run -p haxnews-cli --bin haxnews-cli
```

Run the CLI help:

```bash
cd haxnews-cli
cargo run -- --help
```

If you want the example parser test:

```bash
cargo run --example test-parser
```

--

## Configuration

- Global config lives in `config/feeds.toml` — add your feed sources there. Example entry:

```toml
[[feeds]]
name = "Tech News"
url = "https://example.com/rss"
category = "technology"
```

- Environment variables:
  - `HAXNEWS_DB` — path to DB file (default: `news.db`)
  - `HAXNEWS_PORT` — HTTP port for API (default: `8080`)

--

## Command-Line Usage (`haxnews-cli`)

Common commands:

- `haxnews-cli run` — start the HTTP API and fetch loop
- `haxnews-cli fetch` — fetch configured feeds once (useful for CI)
- `haxnews-cli parse <file>` — parse an RSS XML file locally
- `haxnews-cli tui` — start the local text UI (ncurses-style) for browsing items

Example: run one-time fetch and print items:

```bash
cd haxnews-cli
cargo run -- fetch --verbose
```

--

## HTTP API (stable endpoints)

Base URL: `http://<host>:<port>` (default `http://localhost:8080`)

- `GET /health`
  - Returns: 200 OK if service is healthy

- `GET /feeds`
  - Returns: JSON array of configured feed sources

- `POST /feeds`
  - Body: `{ "name": "Feed name", "url": "https://..." }`
  - Adds a new feed to the repository and returns the created feed object

- `GET /items`
  - Query params: `?limit=50&offset=0`
  - Returns: paginated list of news items

- `GET /items/:id`
  - Returns: single item by id

- `GET /search?q=<term>`
  - Performs full-text search across items (uses normalized `search_text`)
  - Optional: `?limit=20&sort=published_at`

Response format (example `GET /items`):

```json
[
  {
    "id":"...",
    "title":"...",
    "summary":"...",
    "link":"https://...",
    "image_url":"https://...",
    "published_at":"2026-05-30T06:41:26Z"
  }
]
```

--

## TUI / Local Browser

The `haxnews-cli tui` command provides a terminal UI for quick browsing, marking read, and opening links. It uses local configuration and the same DB as the API.

--

## Screenshots

Add your product screenshots to `docs/images/` and reference them in the README. Below are placeholders used for product dispatch. Replace the files with your actual images (PNG preferred):

- `docs/images/Screenshot (57).png` — CLI output and example fetch
- `docs/images/Screenshot (58).png` — API dashboard / sample curl responses
- `docs/images/Screenshot (60).png` — TUI browsing view

Example screenshots included below:

CLI example:

![CLI screenshot](docs/images/Screenshot%20(57).png)

API example:

![API screenshot](docs/images/Screenshot%20(58).png)

TUI example:

![TUI screenshot](docs/images/Screenshot%20(60).png)

If you want, upload the screenshots and I will add them inline and update captions.

--

## Deploying

Minimal notes to deploy as a service:

- Use a systemd unit or Windows service to run `haxnews-cli run`.
- Persist `news.db` on disk or mount an external storage volume.
- Expose the API behind a reverse proxy (NGINX) and enable TLS.
- Add healthchecks pointing to `/health` for autoscaling.

--

## Integrations

- Webhook support: You can subscribe external services by implementing a small forwarder that polls `GET /items`.
- Embed in other Rust apps: add `haxnews-core` as a dependency and call parsing/aggregator functions directly.

--

## Contributing

Please open issues or PRs on the GitHub repo. For code style follow existing Rust patterns in the workspace and run `cargo fmt` before opening a PR.

--
