# crawl-webservice — RSS & Wallhaven Backend Daemon

## Architecture

```
┌──────────────────────────────────────────────────────────────────┐
│                     crawl-webservice                              │
│                                                                  │
│  ┌──────────┐   ┌────────────┐   ┌────────────────────────┐     │
│  │IPC Server │   │Sync Engine │   │  HTTP Client Pool      │     │
│  │(JSON-RPC) │──▶│ (polling)  │──▶│  reqwest (rustls-tls)  │     │
│  │crawl-ws.sock│  │⚠ enabled? │   │  with ETag cache       │     │
│  └─────┬─────┘   └─────┬──────┘   └────┬───────────┬───────┘     │
│        │               │               │           │             │
│        ▼               ▼               ▼           ▼             │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │                    SQLite Store                          │     │
│  │      (feeds, entries, categories, opml_imports)         │     │
│  │      (etag, last_modified per feed)                     │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │  Wallhaven Client (rate-limited API proxy + downloader) │     │
│  │  ┌────────────┐  ┌────────────────┐  ┌──────────────┐  │     │
│  │  │ Search API │  │ Download Mgr   │  │⚠ enabled?    │  │     │
│  │  │ (45 req/m) │  │ (async stream) │  │ (AtomicBool) │  │     │
│  │  └────────────┘  └────────────────┘  └──────────────┘  │     │
│  └─────────────────────────────────────────────────────────┘     │
│                                                                  │
│  ┌─────────────────────────────────────────────────────────┐     │
│  │     Push Events (broadcast channel → IPC subscribers)   │     │
│  │  Rss: FeedAdded, FeedRemoved, NewEntries, SyncComplete │     │
│  │  Rss: StateChanged                                      │     │
│  │  WH:  DownloadStarted, DownloadProgress, DownloadDone   │     │
│  │  WH:  StateChanged                                      │     │
│  └─────────────────────────────────────────────────────────┘     │
└──────────────────────────────────────────────────────────────────┘
```

### Socket Design

**Separate Unix socket** (`$XDG_RUNTIME_DIR/crawl-webservice.sock`).

Rationale for a separate socket (not sharing `crawl.sock` with `crawl-sysd`):

| Factor | Shared Socket | Separate Socket |
|--------|--------------|-----------------|
| Lifecycle | Must start/stop with sysd | Independent |
| Complexity | Need IPC proxy layer | Standalone server |
| Failure isolation | One crashing takes both down | Independent |
| Socket conflict | Only one process can bind | No conflict |
| Frontend routing | Single connection | Multiple connections |

The QML frontend (`RssService.qml`, `WallhavenService.qml`) connects directly to
`crawl-webservice.sock` via `WebServiceConnection` (a dedicated
`CrawlSocket`-wrapper), keeping `CrawlService.qml` dedicated to the main
`crawl-sysd` daemon.

---

## Build & Run

```bash
# Build
cargo build -p crawl-webservice

# Run (with defaults)
cargo run -p crawl-webservice

# With env overrides
CRAWL_WEBSERVICE_SOCKET=/tmp/crawl-ws.sock \
CRAWL_WEBSERVICE_DB=~/.local/share/crawl/ws.db \
CRAWLDS_WALLHAVEN_API_KEY=your_key \
  cargo run -p crawl-webservice
```

### Configuration File (`~/.config/crawl/webservice.toml`)

```toml
db_path = "/home/user/.local/share/crawl/webservice.db"
socket_path = "/run/user/1000/crawl-webservice.sock"
max_parallel_downloads = 3

[rss]
enabled = true
max_concurrent_fetches = 8
default_fetch_interval_secs = 1800
timeout_secs = 30
user_agent = "CrawlDS/0.1 RSS Reader"

[wallhaven]
enabled = true
api_key = "your-wallhaven-api-key"
rate_per_min = 45
```

Environment variables override file config:
- `CRAWL_WEBSERVICE_CONFIG` — config file path
- `CRAWL_WEBSERVICE_DB` — database path
- `CRAWL_WEBSERVICE_SOCKET` — socket path
- `CRAWLDS_WALLHAVEN_API_KEY` — Wallhaven API key (takes priority over file)

---

## JSON-RPC IPC Protocol

### Naming Convention
All commands use `Rss` prefix for RSS domain and `Wallhaven` prefix for Wallhaven
domain. Serialized with `rename_all = "PascalCase"` matching the existing pattern.

### RSS Commands

| Command | Params | Returns |
|---------|--------|---------|
| `RssListFeeds` | — | `FeedList { feeds: Vec<FeedInfo> }` |
| `RssAddFeed` | `{ url, category? }` | `Ok` |
| `RssRemoveFeed` | `{ feed_id }` | `Ok` |
| `RssUpdateFeed` | `{ feed_id, category? }` | `Ok` |
| `RssListEntries` | `{ feed_id?, category?, offset, limit, only_unread, only_starred, sort }` | `EntryList { entries, total }` |
| `RssGetEntry` | `{ entry_id }` | `Entry { entry: EntryFull }` |
| `RssSetEntryRead` | `{ entry_id, is_read }` | `Ok` |
| `RssSetEntryStarred` | `{ entry_id, is_starred }` | `Ok` |
| `RssMarkAllRead` | `{ feed_id }` | `Ok` |
| `RssRefreshFeed` | `{ feed_id }` | `Ok` (triggers async fetch) |
| `RssRefreshAll` | — | `Ok` (triggers all) |
| `RssListCategories` | — | `Categories { categories: Vec<String> }` |
| `RssImportOpml` | `{ path }` | `ImportResult { total, imported, failed }` |
| `RssExportOpml` | — | `ExportData { opml: String }` |
| `RssSetEnabled` | `{ enabled: bool }` | `Ok` |

### Wallhaven Commands

| Command | Params | Returns |
|---------|--------|---------|
| `WallhavenSearch` | `{ query, categories, purity, sorting, order, page, seed?, top_range?, atleast?, resolutions?, ratios?, colors? }` | `SearchResults { results, meta }` |
| `WallhavenDownload` | `{ wallpaper_id, url, dest_dir, filename? }` | `Ok` (downloads in background) |
| `WallhavenSetEnabled` | `{ enabled: bool }` | `Ok` |

### Response Format

All responses follow JSON-RPC 2.0 envelope with domain-typed bodies:

```json
{
  "jsonrpc": "2.0",
  "result": {
    "type": "feed_list",
    "data": {
      "feeds": [{ "id": "...", "url": "...", "title": "...", ... }]
    }
  },
  "id": 1
}

{
  "jsonrpc": "2.0",
  "result": {
    "type": "entry_list",
    "data": {
      "entries": [{ "id": "...", "title": "...", ... }],
      "total": 42
    }
  },
  "id": 2
}

{
  "jsonrpc": "2.0",
  "result": {
    "type": "search_results",
    "data": {
      "results": [{ "id": "...", "path": "...", "thumbs": {...}, ... }],
      "meta": { "current_page": 1, "last_page": 10, "total": 240 }
    }
  },
  "id": 3
}

{
  "jsonrpc": "2.0",
  "result": {
    "type": "error",
    "data": {
      "code": "fetch_failed",
      "message": "HTTP 404"
    }
  },
  "id": 4
}
```

### Push Events

Clients subscribe via the standard `Subscribe` command. Events are pushed
as NDJSON `event` messages with domain-tagged bodies:

```json
{"jsonrpc":"2.0","method":"event","params":{"domain":"rss","data":{"event":"new_entries","data":{"feed_id":"abc","count":5}}}}
{"jsonrpc":"2.0","method":"event","params":{"domain":"rss","data":{"event":"sync_complete","data":{"feed_id":"abc"}}}}
{"jsonrpc":"2.0","method":"event","params":{"domain":"wallhaven","data":{"event":"download_complete","data":{"wallpaper_id":"xyz","local_path":"/path/to/file.jpg"}}}}
{"jsonrpc":"2.0","method":"event","params":{"domain":"wallhaven","data":{"event":"download_progress","data":{"wallpaper_id":"xyz","bytes_downloaded":50000,"total_bytes":200000}}}}
```

#### RSS Events

| Event | Data |
|-------|------|
| `feed_added` | `{ feed_id, title, category }` |
| `feed_removed` | `{ feed_id }` |
| `entry_updated` | `{ feed_id, entry_id }` |
| `new_entries` | `{ feed_id, count }` |
| `sync_started` | `{ feed_id: Option<String> }` |
| `sync_complete` | `{ feed_id: Option<String> }` |
| `sync_error` | `{ feed_id, error }` |
| `state_changed` | `{ enabled: bool }` |

#### Wallhaven Events

| Event | Data |
|-------|------|
| `download_started` | `{ wallpaper_id, local_path }` |
| `download_progress` | `{ wallpaper_id, bytes_downloaded, total_bytes }` |
| `download_complete` | `{ wallpaper_id, local_path }` |
| `download_failed` | `{ wallpaper_id, error }` |
| `state_changed` | `{ enabled: bool }` |

---

## RSS Data Model (SQLite)

```sql
-- Pragmas
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE feeds (
    id              TEXT PRIMARY KEY,           -- UUID v4
    url             TEXT NOT NULL UNIQUE,       -- RSS/Atom feed URL
    title           TEXT DEFAULT '',            -- fetched from feed
    description     TEXT DEFAULT '',            -- feed description
    site_url        TEXT DEFAULT '',            -- link to website
    icon_url        TEXT DEFAULT '',            -- favicon / feed icon
    category        TEXT DEFAULT '',            -- user-assigned category
    error_count     INTEGER DEFAULT 0,         -- consecutive fetch failures
    last_error      TEXT DEFAULT '',            -- last error message
    last_fetched    TEXT DEFAULT '',            -- ISO 8601 timestamp
    fetch_interval  INTEGER DEFAULT 1800,      -- seconds
    etag            TEXT DEFAULT '',            -- HTTP ETag for conditional GET
    last_modified   TEXT DEFAULT '',            -- HTTP Last-Modified
    created_at      TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE entries (
    id              TEXT PRIMARY KEY,           -- UUID v4
    feed_id         TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid            TEXT NOT NULL,              -- RSS <guid> / Atom <id>
    url             TEXT DEFAULT '',
    title           TEXT DEFAULT '',
    author          TEXT DEFAULT '',
    summary         TEXT DEFAULT '',
    content         TEXT DEFAULT '',            -- full HTML content
    content_type    TEXT DEFAULT 'html',        -- 'html' | 'text'
    published       TEXT DEFAULT '',            -- ISO 8601
    fetched_at      TEXT NOT NULL,              -- when we fetched it
    is_read         INTEGER DEFAULT 0,
    is_starred      INTEGER DEFAULT 0,
    image_url       TEXT DEFAULT '',            -- lead image / thumbnail
    UNIQUE(feed_id, guid)
);

-- Indexes
CREATE INDEX idx_entries_feed_id ON entries(feed_id);
CREATE INDEX idx_entries_published ON entries(published DESC);
CREATE INDEX idx_entries_is_read ON entries(is_read);
CREATE INDEX idx_entries_is_starred ON entries(is_starred);
CREATE INDEX idx_feeds_category ON feeds(category);
CREATE INDEX idx_feeds_last_fetched ON feeds(last_fetched);
```

### Sync Engine

- Background task runs every **60 seconds**, checks all feeds
- Feeds fetched when `last_fetched + fetch_interval < now()`
- On startup / `RssRefreshAll`: all feeds queued immediately
- **Max 8 parallel fetches** (configurable), semaphore-gated
- **Exponential backoff**: after consecutive failures, `fetch_interval` doubles
  (capped at 24h). Error count resets on success.

### Fetch Pipeline

1. **Conditional GET** — sends `If-None-Match` / `If-Modified-Since` when available
2. **Timeout** — 30s per request
3. **Redirects** — follows up to 5
4. **Parse** — `feed-rs` handles RSS 2.0, RSS 1.0, Atom, JSON Feed
5. **Dedup** — matches by `(feed_id, guid)` UNIQUE constraint
6. **Batch insert** — `INSERT OR IGNORE` for new entries
7. **Notify** — emits `RssEvent::NewEntries` with count

### OPML

- **Import** (`RssImportOpml`): Recursive `<outline>` parsing with category
  nesting. Reports total/imported/failed counts.
- **Export** (`RssExportOpml`): Groups feeds by category into nested
  `<outline>` elements, valid OPML 1.0.

---

## Wallhaven Backend

### Why a Backend Proxy?

Previously, the Wallhaven API was called directly from QML via `XMLHttpRequest`.
Moving it to the Rust daemon provides:

| Feature | Before (QML) | After (Rust daemon) |
|---------|-------------|---------------------|
| API key security | Exposed in QML env/settings | Hidden in daemon process |
| Rate limiting | None (hitting 429 frequently) | Enforced 45 req/min with backoff |
| Download mechanism | Shelling to `curl`/`wget` | Async reqwest with progress events |
| Error recovery | Basic | Automatic retry on 429 |
| Caching | None | Future: result caching |

### Rate Limiting

The Wallhaven API allows **45 requests per minute**. The client tracks request
timestamps and sleeps when approaching the limit. On HTTP 429, it waits 60s
and retries automatically.

### Download Manager

Downloads are fully async using `reqwest` streaming:

```
WallhavenDownload command
  └─▶ spawn background task
       ├─▶ DownloadStarted event
       ├─▶ DownloadProgress event (streaming chunks)
       ├─▶ DownloadComplete event (on success)
       └─▶ DownloadFailed event (on error)
```

- Max **3 parallel downloads** (configurable)
- Semaphore-gated to prevent overwhelming I/O
- Progress events include `bytes_downloaded` / `total_bytes`

---

## Frontend Integration (QML)

### RssService.qml
Connects to `crawl-webservice.sock` via `WebSocketConnection` (a dedicated
`CrawlSocket` wrapper). All `_send()` calls go to this socket instead of
`CrawlService`.

The RSS state management and method signatures remain unchanged — only the
transport layer changes.

Runtime toggle:
- **`rssEnabled`** (bool property) — reflects current state from daemon
- **`setRssEnabled(bool)`** — sends `RssSetEnabled` IPC command

### WallhavenService.qml
Replaces `XMLHttpRequest` with IPC calls to `crawl-webservice.sock`.
- `search()` → `WallhavenSearch` command
- `downloadWallpaper()` → `WallhavenDownload` command + event listener

Runtime toggle:
- **`whEnabled`** (bool property) — reflects current state from daemon
- **`setWhEnabled(bool)`** — sends `WallhavenSetEnabled` IPC command

### Events
Both services subscribe to the event stream to receive push notifications:
- RSS: sync status, new entries, errors, state_changed
- Wallhaven: download progress/completion, state_changed

---

## Crate Structure

```
crates/crawl-webservice/
├── Cargo.toml
└── src/
    ├── main.rs              # Entry point, wiring
    ├── config.rs            # TOML config + env overrides
    ├── ipc.rs               # JSON-RPC dispatcher (RSS + Wallhaven)
    ├── rss/
    │   ├── mod.rs
    │   ├── store.rs         # SQLite CRUD for feeds/entries/categories
    │   ├── schema.sql       # DDL with indexes
    │   ├── fetcher.rs       # HTTP fetch + feed-rs parsing
    │   └── opml.rs          # OPML import/export
    └── wallhaven/
        ├── mod.rs
        ├── client.rs        # API proxy with rate limiting
        └── downloader.rs    # Async streaming downloader
```

---
## Resource Optimization

### Runtime Feature Toggles

Both RSS and Wallhaven domains can be enabled/disabled at runtime via IPC
commands, with state persisted in config (`webservice.toml`):

| Domain | Config field | IPC command | Event |
|--------|-------------|-------------|-------|
| RSS | `[rss].enabled` (default: true) | `RssSetEnabled` | `RssEvent::StateChanged` |
| Wallhaven | `[wallhaven].enabled` (default: true) | `WallhavenSetEnabled` | `WallhavenEvent::StateChanged` |

When disabled:
- **RSS**: polling loop skips all feeds; `RssRefreshFeed` / `RssRefreshAll` / `RssAddFeed` return immediately without fetching
- **Wallhaven**: `WallhavenSearch` / `WallhavenDownload` return a `"disabled"` error

### ETag / Last-Modified HTTP Caching

The fetcher uses **conditional GETs** to avoid re-downloading unchanged feeds:

1. Before each HTTP request, the fetcher looks up cached `ETag` and `Last-Modified`
   headers from the `feeds` table
2. Sends `If-None-Match` / `If-Modified-Since` headers
3. If the server responds `304 Not Modified`:
   - Feed metadata is NOT updated (nothing changed)
   - `FetchResult::NotModified` is returned
   - `SyncComplete` event is emitted (no new entries)
4. On a successful `200` response:
   - New `ETag` / `Last-Modified` headers from the response are saved
   - Feed is parsed and entries inserted as normal

This eliminates bandwidth and parsing overhead for feeds that haven't changed
since the last poll — most RSS feeds return 304 the majority of the time.

### Concurrency Controls

| Resource | Default | Config key | Mechanism |
|----------|---------|-----------|-----------|
| RSS parallel fetches | 8 | `[rss].max_concurrent_fetches` | `tokio::sync::Semaphore` |
| Wallhaven parallel downloads | 3 | `max_parallel_downloads` | `tokio::sync::Semaphore` |
| Wallhaven API rate | 45/min | `[wallhaven].rate_per_min` | Sliding window + 429 backoff |

### Dependencies

| Crate | Purpose |
|-------|---------|
| `crawl-ipc` | Shared types, IPC server, JSON-RPC protocol |
| `tokio` | Async runtime |
| `reqwest` (rustls-tls) | HTTP client for RSS fetches + Wallhaven API |
| `feed-rs` | RSS/Atom/JSON Feed parsing |
| `sqlx` (sqlite) | Persistence layer |
| `quick-xml` | OPML XML parsing |
| `uuid` | ID generation |
| `chrono` | Timestamps |
| `futures-util` | Async streaming |
