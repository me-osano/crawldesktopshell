# Crawl Clipboard Daemon (`crawl-sysd` clipboard service)

A self-contained clipboard history backend with Wayland integration, SQLite persistence, and a Unix-socket IPC interface, designed for the Crawl desktop environment.

---

## Table of Contents

- [Overview](#overview)
- [Architecture](#architecture)
- [Startup Sequence](#startup-sequence)
- [IPC Protocol](#ipc-protocol)
  - [Commands](#commands)
  - [Responses](#responses)
  - [Push Events](#push-events)
- [Data Types](#data-types)
  - [ClipEntry](#clipentry)
  - [ClipContent](#clipcontent)
- [Storage Layer](#storage-layer)
  - [Schema](#schema)
  - [Cache Architecture](#cache-architecture)
- [Monitor Thread](#monitor-thread)
  - [Polling Loop](#polling-loop)
  - [Deduplication](#deduplication)
  - [Self-copy Suppression](#self-copy-suppression)
- [QML Integration](#qml-integration)
  - [ClipboardService.qml](#clipboardserviceqml)
  - [CrawlService Wrappers](#crawlservice-wrappers)
  - [ClipboardPanel.qml](#clipboardpanelqml)
- [Configuration](#configuration)
- [Development](#development)

---

## Overview

The clipboard backend in `crawl-sysd` provides:

- **Clipboard monitoring** — Polls the Wayland clipboard every 200ms via `wl-clipboard-rs`, detects content changes via FNV-1a hashing
- **History storage** — SQLite-backed persistent history with WAL journaling, auto-pruning to configurable size (default 200 entries)
- **Deduplication** — FNV-1a 64-bit hash in memory prevents duplicate entries; re-copies bump timestamp to top
- **Self-copy suppression** — Tracks hash of last programmatic copy to suppress re-recording when replying content from history
- **IPC surface** — 8 JSON-RPC commands for list, get, copy, delete, wipe, pin, paste-text, set
- **Event broadcast** — `Changed`, `Deleted`, `Cleared`, `Pinned` events pushed to connected clients
- **No shell dependencies** — Pure Rust, no `cliphist`, `wl-paste`, or `wl-copy` required

---

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      ClipboardBackend                            │
│                                                                   │
│  ┌──────────────────────────────────────────┐                    │
│  │              In-Memory Cache              │                    │
│  │  VecDeque<ClipEntry> + HashSet<u64> (FNV) │                    │
│  │  LRU eviction (oldest unpinned dropped)   │                    │
│  └──────────────────────────────────────────┘                    │
│                                                                   │
│  ┌──────────────────────────────────────────┐                    │
│  │            Monitor Thread (std::thread)   │                    │
│  │  loop { sleep(200ms)                      │                    │
│  │    read_clipboard_inner() → hash ≠ last?  │                    │
│  │      → store() → INSERT INTO entries      │                    │
│  │    [optional] read_primary_inner()        │                    │
│  │  }                                        │                    │
│  └──────────────────────────────────────────┘                    │
│                                                                   │
│  ┌──────────────────────────────────────────┐                    │
│  │         SQLite (via sqlx 0.7)             │                    │
│  │  $XDG_DATA_HOME/crawlds/clipboard.db      │                    │
│  │  entries(id, hash, content, mime, ...)    │                    │
│  └──────────────────────────────────────────┘                    │
│                                                                   │
│  ┌──────────────────────────────────────────┐                    │
│  │         broadcast::Sender<CrawlEvent>     │                    │
│  │  → CrawlEvent::Clipboard(Changed|Deleted  │                    │
│  │    |Cleared|Pinned)                       │                    │
│  └──────────────────────────────────────────┘                    │
└─────────────────────────────────────────────────────────────────┘
                                                                   │
                           │ IPC (Unix socket)
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                   ClipboardService (trait Service)               │
│  handle(ClipboardList|GetContent|Copy|Delete|Wipe|Pin|PasteText  │
│         |Set) → CrawlResponse                                    │
└─────────────────────────────────────────────────────────────────┘
```

### Data Flow

```
Wayland Clipboard ──[poll]──► read_clipboard_inner()
                                  │
                                  ▼ hash ≠ last?
                                  │
                  ┌───────────────┴───────────────┐
                  ▼                               ▼
             hash match                     new content
             bump timestamp                INSERT into cache
             UPDATE entries DB              INSERT into SQLite
                  │                               │
                  └───────────────┬───────────────┘
                                  ▼
                    broadcast ClipboardEvent::Changed
                                  │
                                  ▼
                    CrawlService._dispatchEvent("clipboard")
                                  │
                                  ▼
                    ClipboardService.qml → list()
```

---

## Startup Sequence

1. **`ClipboardService::start()`** called by daemon bootstrap
2. **`ClipboardBackend::init()`**:
   - Checks `config.clipboard.enabled` — exits early if disabled
   - Checks `WAYLAND_DISPLAY` env var — exits early if not set
   - If `persistent`: creates parent directory, opens SQLite pool, runs `schema.sql` migration, loads all entries into in-memory cache (pinned first, newest first)
3. **`ClipboardBackend::start_monitor()`**:
   - Captures tokio `Handle` for sync→async bridge
   - Spawns `std::thread` with polling loop
4. **Service registered** — ready to handle IPC commands

### Database Path Resolution

```
$CRAWL_CLIPBOARD_DB                (env var, optional)
  → $XDG_DATA_HOME/crawlds/clipboard.db
    → $HOME/.local/share/crawlds/clipboard.db
      → /tmp/crawlds/clipboard.db  (fallback)
```

---

## IPC Protocol

The clipboard service is registered under the `"clipboard"` domain in the daemon's IPC dispatcher. Commands follow JSON-RPC 2.0 over the daemon's Unix socket at `$XDG_RUNTIME_DIR/crawl.sock`.

### Commands

All 8 clipboard commands:

| Command | Direction | Params | Returns | Description |
|---------|-----------|--------|---------|-------------|
| `ClipboardList` | Request | `{}` | `[ClipEntry, ...]` | List all cached entries (newest first) |
| `ClipboardGetContent` | Request | `{ id: u64 }` | `ClipContent` | Get full content for an entry (base64 for images) |
| `ClipboardCopy` | Request | `{ id: u64 }` | `{ ok: true }` | Copy entry to system clipboard, bump timestamp |
| `ClipboardDelete` | Request | `{ id: u64 }` | `{ ok: true }` | Remove entry from cache and database |
| `ClipboardWipe` | Request | `{}` | `{ ok: true }` | Clear all entries from cache and database |
| `ClipboardPin` | Request | `{ id: u64, pinned: bool }` | `{ ok: true }` | Toggle pin state for an entry |
| `ClipboardPasteText` | Request | `{ text: string }` | `{ ok: true }` | Set clipboard text + simulate Ctrl+Shift+V |
| `ClipboardSet` | Request | `{ text: string, mime: string }` | `{ ok: true }` | Set clipboard text with explicit MIME type |

**Request example:**
```json
{"jsonrpc": "2.0", "method": "ClipboardList", "params": {}, "id": 5}
```

**Response example:**
```json
{"jsonrpc": "2.0", "result": [{"id": 1, "preview": "Hello world", "mime": "text/plain", ...}], "id": 5}
```

### Responses

Every response is a JSON-RPC `CrawlResponse`:

| Outcome | Shape |
|---------|-------|
| Success | `{ "jsonrpc": "2.0", "result": <data>, "id": <id> }` |
| Error | `{ "jsonrpc": "2.0", "error": { "code": -1, "message": "..." }, "id": <id> }` |

Common error cases:
- `"entry not found"` — `ClipboardGetContent`/`Copy`/`Delete`/`Pin` with non-existent id
- `"not connected"` — request sent before daemon socket connects (QML frontend)

### Push Events

Events are dispatched as `CrawlEvent::Clipboard(ClipboardEvent)` on the subscribe socket:

```json
{"domain": "clipboard", "data": {"event": "changed", "entry": {...}}}
```

| Event | Payload | Trigger |
|-------|---------|---------|
| `"changed"` | `{ "entry": ClipEntry }` | New clipboard content detected by monitor |
| `"deleted"` | `{ "id": u64 }` | Entry removed via `ClipboardDelete` |
| `"cleared"` | `{}` | All entries wiped via `ClipboardWipe` |
| `"pinned"` | `{ "id": u64, "pinned": bool }` | Pin state toggled via `ClipboardPin` |

---

## Data Types

Defined in `crawl-ipc/src/types.rs`:

### ClipEntry

```rust
pub struct ClipEntry {
    pub id: u64,           // Auto-incrementing unique ID
    pub content: String,   // Full content (text) or empty for images
    pub mime: String,      // MIME type ("text/plain", "image/png", etc.)
    pub preview: String,   // Truncated preview (120 chars text, "[[ image N MiB ]]" for images)
    pub timestamp_ms: u64, // Unix timestamp in milliseconds
    pub is_image: bool,    // Whether MIME starts with "image/"
    pub is_pinned: bool,   // Whether entry is pinned (survives LRU eviction)
    pub size: u64,         // Content size in bytes
}
```

### ClipContent

```rust
pub struct ClipContent {
    pub content: String,          // Full text content (empty for images)
    pub mime: String,             // MIME type
    pub data_base64: Option<String>,  // Base64-encoded binary data (only for images)
    pub size: u64,                // Content size in bytes
}
```

The `content` field is populated inline for text entries. For image entries, `content` is empty and `data_base64` carries the base64 blob so the QML frontend can construct a `data:` URL for display.

---

## Storage Layer

### Schema

File: `core/crates/crawl-sysd/src/clipboard/schema.sql`

```sql
PRAGMA journal_mode = WAL;

CREATE TABLE IF NOT EXISTS entries (
    id           INTEGER PRIMARY KEY,
    hash         INTEGER NOT NULL,
    content      BLOB NOT NULL,
    mime         TEXT NOT NULL,
    preview      TEXT NOT NULL,
    timestamp_ms INTEGER NOT NULL,
    is_image     INTEGER NOT NULL DEFAULT 0,
    is_pinned    INTEGER NOT NULL DEFAULT 0,
    size         INTEGER NOT NULL
);

CREATE INDEX IF NOT EXISTS idx_entries_timestamp ON entries(timestamp_ms DESC);
CREATE INDEX IF NOT EXISTS idx_entries_hash ON entries(hash);
```

Key design decisions:

- **WAL mode** — Enables concurrent reads from the in-memory cache while the monitor thread writes.
- **`hash` column** — Stores FNV-1a 64-bit hash for offline dedup and index lookups (not used at runtime, which uses in-memory `HashSet`).
- **`content` as BLOB** — Stores arbitrary binary data (text encoded as UTF-8, image bytes as-is).
- **`is_pinned` flag** — Pinned entries survive LRU eviction when the history size limit is reached.
- **No foreign keys** — Clipboard entries are independent records with no relational constraints.

### Cache Architecture

The backend maintains a dual in-memory + on-disk cache:

**In-memory (`ClipboardInner`):**
- `VecDeque<ClipEntry>` — Ordered list (newest + pinned first), bounded by `config.history_size` (default 200)
- `HashSet<u64>` — FNV-1a hashes for O(1) dedup lookups
- `next_id: u64` — Auto-incrementing ID counter (persisted between restarts as `MAX(id)` on load)

**On-disk (SQLite):**
- Writes happen asynchronously via `Handle::block_on` bridge from the monitor thread
- All entries are loaded into memory on startup from `SELECT ... ORDER BY is_pinned DESC, timestamp_ms DESC`
- Deleted/evicted entries are cleaned up via `DELETE FROM entries WHERE id = ?`
- The `ON CONFLICT(id) DO UPDATE` upsert pattern handles the dedup-bump case

**LRU eviction:**
When the in-memory cache exceeds `config.history_size`, the oldest unpinned entry is evicted from both cache and database. Pinned entries are preserved until explicitly unpinned.

---

## Monitor Thread

### Polling Loop

```
std::thread::spawn(|| loop {
    sleep(config.poll_interval_ms);  // default 200ms

    // Read Wayland clipboard
    let (content, mime) = wl_clipboard_rs::paste::get_contents(
        ClipboardType::Regular, Seat::Unspecified, MimeType::Text
    );

    // Compute FNV-1a hash
    let hash = fnv1a_hash(content);

    // Skip if:
    //   - hash == last_hash (unchanged clipboard)
    //   - hash == last_copied_hash (programmatic copy, self-suppression)

    // If new content:
    backend.store(&content, &mime);  // INSERT + broadcast event
    last_hash = hash;

    // Optional: primary selection monitoring
    if config.watch_primary {
        let (primary, mime) = read_primary_inner();
        if hash != last_primary_hash { backend.store(...); }
    }
});
```

### Deduplication

Each clipboard entry is fingerprinted with FNV-1a 64-bit hash:

```rust
fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}
```

When a hash already exists in the in-memory `HashSet`:
1. The existing entry's `timestamp_ms` is updated to `now()` (bumps to top of list)
2. The database row is updated via `UPDATE entries SET timestamp_ms = ?1, content = ?2 WHERE id = ?3`
3. No new entry is created — duplicates are silently absorbed

### Self-copy Suppression

When the frontend calls `ClipboardCopy` or `ClipboardSet`/`ClipboardPasteText`, the backend records the content's hash in `last_copied_hash` (an `AtomicU64`). The monitor thread skips any clipboard change whose hash matches `last_copied_hash`, preventing the system clipboard from being re-recorded when the user simply activates an entry from history.

The suppression is one-shot: after the monitor observes a non-matching change, normal recording resumes.

---

## QML Integration

### ClipboardService.qml

`quickshell/Services/ClipboardService.qml` is the frontend singleton providing the QML API:

| Property/Method | Description |
|-----------------|-------------|
| `active: true` | Always active (backed by Rust daemon, no external deps) |
| `loading: bool` | True during IPC list fetch |
| `items: [ClipEntry]` | Current clipboard history list |
| `imageDataById: {}` | Cache of base64 data URLs by entry id |
| `contentCache: {}` | Cache of full text content by entry id |
| `list()` | Fetch clipboard list from backend |
| `decode(id, cb)` | Get text content for an entry (async) |
| `decodeToDataUrl(id, mime, cb)` | Get image content as data URL (async) |
| `getContent(id)` | Sync text content cache lookup |
| `getImageData(id)` | Sync image data URL cache lookup |
| `copyToClipboard(id)` | Copy entry to system clipboard |
| `pasteFromClipboard(id, mime)` | Copy + simulate paste keystroke |
| `pasteText(text)` | Set clipboard + simulate Ctrl+Shift+V paste |
| `deleteById(id)` | Delete entry from history |
| `wipeAll()` | Clear all clipboard history |
| `parseImageMeta(preview)` | Parse "[[ image ... ]]" preview string |

The service listens for `CrawlService.clipboardChanged` events and auto-refreshes on any clipboard backend event.

### CrawlService Wrappers

`quickshell/Services/CrawlService.qml` provides 8 convenience methods that delegate to `sendRequest`:

| Wrapper | Calls method | Notes |
|---------|-------------|-------|
| `clipboardList(cb)` | `ClipboardList` | Returns `[ClipEntry]` |
| `clipboardGetContent(id, cb)` | `ClipboardGetContent` | Returns `ClipContent` |
| `clipboardCopy(id, cb)` | `ClipboardCopy` | Copies entry to system clipboard |
| `clipboardDelete(id, cb)` | `ClipboardDelete` | Removes entry |
| `clipboardWipe(cb)` | `ClipboardWipe` | Clears all |
| `clipboardPin(id, cb)` | `ClipboardPin` | Toggles pin |
| `clipboardPasteText(text, cb)` | `ClipboardPasteText` | Set text + paste |
| `clipboardSet(text, mime, cb)` | `ClipboardSet` | Set text with MIME |

### ClipboardPanel.qml

`quickshell/Modules/Panels/Clipboard/ClipboardPanel.qml` is the standalone clipboard popup panel (opened from the bar's clipboard icon). It provides:

- **Header** with clipboard icon, clear-all button, settings, close
- **Search bar** to filter text entries
- **Item list** with copy/paste/delete context menus
- **Preview panel** (`ClipboardPreviewPanel.qml`) for selected items — text content or image rendering
- **Loading state** during IPC fetch
- **Empty state** when no history exists
- **Disabled state** when the launcher's clipboard setting is off

The panel uses `ClipboardService` exclusively for data — no shell commands.

---

## Configuration

Defined in `core/crates/crawl-sysd/src/config.rs`:

```rust
pub struct ClipboardConfig {
    pub enabled: bool,             // Enable clipboard monitoring (default: true)
    pub history_size: usize,       // Max in-memory entries (default: 200)
    pub poll_interval_ms: u64,     // Monitor poll interval (default: 200ms)
    pub max_entry_size: u64,       // Max content size in bytes (default: 5 MiB)
    pub persistent: bool,          // Enable SQLite persistence (default: true)
    pub watch_primary: bool,       // Monitor primary selection (default: false)
}
```

Config file: `/etc/crawl/config.toml`

```toml
[clipboard]
enabled = true
history_size = 200
poll_interval_ms = 200
max_entry_size = 5242880
persistent = true
watch_primary = false
```

Environment variable override: `CRAWL_CLIPBOARD__HISTORY_SIZE=500` (uses `__` as nested separator).

---

## Development

### Module Map

| File | Lines | Responsibility |
|------|-------|----------------|
| `clipboard/mod.rs` | 633 | `ClipboardBackend`: init, store, list, paste_text, set, start/stop monitor, get_content, delete, wipe, toggle_pin, copy_to_clipboard, FNV-1a hash, preview generation, base64, clipboard I/O via `wl-clipboard-rs` |
| `clipboard/schema.sql` | 17 | SQLite DDL: `entries` table with indexes |
| `services/clipboard.rs` | 133 | `ClipboardService`: Service trait impl, IPC handler dispatching 8 commands |
| `config.rs` | (22) | `ClipboardConfig` struct + defaults |

### Key Dependencies

| Crate | Version | Purpose |
|-------|---------|---------|
| `wl-clipboard-rs` | 0.9 | Wayland clipboard read/write (paste + copy) |
| `sqlx` | 0.7 | SQLite async driver with `runtime-tokio` + `sqlite` features |
| `tokio` | 1 | Async runtime (for `Handle::block_on` bridge) |
| `anyhow` | 1 | Error handling |
| `tracing` | 0.1 | Structured logging |
| `serde` / `serde_json` | 1 | JSON serialisation for IPC |

### Threading Model

| Thread | Purpose |
|--------|---------|
| **Main tokio runtime** | IPC handler (async service methods: `get_content`, `delete`, `wipe`, `toggle_pin`, `copy_to_clipboard`) |
| **Monitor thread** | `std::thread` polling Wayland clipboard every 200ms, calls `store()` (which does `Handle::block_on` for DB writes) |

The monitor thread uses `Handle::block_on` to bridge sync→async for SQLite writes, since `sqlx` requires an async context.

### Testing

The clipboard backend has no dedicated test suite yet. Recommended approach:

- **Unit tests**: Test `fnv1a_hash`, `make_preview`, `base64_encode` with known inputs/outputs
- **Integration tests**: Test `ClipboardBackend::store` + `list` with in-memory SQLite
- **Wayland tests**: Require a compositor; test `read_clipboard_inner`/`set_clipboard_inner` with `wl-clipboard-rs`

### IPC Debugging

```bash
# Watch clipboard events from the daemon
crawl-cli --watch | grep clipboard

# List clipboard history
echo '{"method":"ClipboardList","id":1}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/crawl.sock

# Get content for an entry
echo '{"method":"ClipboardGetContent","params":{"id":1},"id":2}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/crawl.sock
```

### Future Work

- [ ] **Primary selection monitoring** — Complete `watch_primary` option for Wayland primary selection
- [ ] **Image paste support** — Full binary content handling for `wl-clipboard-rs` paste (currently reads text only)
- [ ] **Clipboard sync** — Sync clipboard state across machines (optional)
- [ ] **Entry editing** — Allow editing stored clipboard entries before pasting
- [ ] **Categories/tags** — Organise clipboard entries by category for better search
- [ ] **History search** — Full-text search over entry previews via SQLite FTS
