# Crawl Mail Daemon (`crawl-mail`)

A self-contained IMAP/SMTP mail daemon with a Unix-socket IPC interface, designed for the Crawl desktop environment.

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
- [Storage Layer](#storage-layer)
  - [Schema](#schema)
  - [Store Accessors](#store-accessors)
- [IMAP Integration](#imap-integration)
  - [Session](#session)
  - [Sync Engine](#sync-engine)
  - [IDLE Loop](#idle-loop)
- [SMTP Outbox](#smtp-outbox)
- [Email Parsing](#email-parsing)
- [Account Lifecycle](#account-lifecycle)
- [Desktop Notifications](#desktop-notifications)
- [Configuration](#configuration)
- [Development](#development)

---

## Overview

`crawl-mail` is a background daemon that:

- Connects to arbitrary IMAP servers (over TLS)
- Synchronises mailbox state to a local SQLite database
- Provides a JSON-RPC-over-Unix-socket API for clients (CLI, QML)
- Sends email via SMTP through a persistent outbox queue
- Emits push events on new mail, flag changes, sync completion
- Sends D-Bus desktop notifications for new mail (via `org.freedesktop.Notifications`)

It is designed as a single-binary daemon with no HTTP surface — all IPC is over a Unix socket at a well-known path.

---

## Architecture

```
┌──────────────────────────────────────────────────────┐
│                    main.rs                            │
│                                                        │
│  config::load() ──► Config { db_path, socket_path }    │
│                                                        │
│  store::open() + migrate() ──► SqlitePool              │
│       │                                                │
│       ▼                                                │
│    ┌──────────────────────────────┐                    │
│    │           Store              │                    │
│    │  ┌──────────┐ ┌───────────┐  │                    │
│    │  │ Accounts │ │ Messages  │  │                    │
│    │  ├──────────┤ ├───────────┤  │                    │
│    │  │ Folders  │ │ Outbox    │  │                    │
│    │  └──────────┘ └───────────┘  │                    │
│    └──────────────────────────────┘                    │
│                                                        │
│  AccountManager ──► run_sync_loop() per account        │
│       │              └── SyncEngine::full_sync()       │
│       │                  ├── list IMAP folders          │
│       │                  ├── sync_folder() each         │
│       │                  └── fetch_uid_range()          │
│                                                        │
│  SmtpQueue (background tokio task)                     │
│       └── process_queue() every 10s                    │
│            └── send_single() via lettre                 │
│                                                        │
│  ipc::serve() (Unix socket listener)                   │
│       └── dispatches CrawlCommand → Store methods      │
│            └── emits CrawlEvent via broadcast channel   │
└──────────────────────────────────────────────────────┘
```

The daemon has three background loops running concurrently:

| Loop | Interval | Purpose |
|------|----------|---------|
| **Sync** | 5 min per account | Fetch new mail, sync flags |
| **Outbox** | 10 s | Process queued outbound messages |
| **IDLE** | persistent | IMAP IDLE for real-time new-mail notification |
| **IPC** | event-driven | Listen on Unix socket for client commands |

---

## Startup Sequence

1. **Initialise** tracing/logging
2. **Load config** from `$CRAWL_MAIL_CONFIG` (or `$XDG_CONFIG_HOME/crawl/config.toml`)
3. **Open SQLite DB** at `$CRAWL_MAIL_DB` (or `$XDG_DATA_HOME/crawl/mail.db`)
4. **Run migrations**: execute `schema.sql` + any ALTER TABLE patches
5. **Create AccountManager**, call `start_all()` to spawn per-account sync loops
6. **Spawn SmtpQueue** as a background tokio task
7. **Start IPC server** on `$CRAWL_MAIL_SOCKET` (or `$XDG_RUNTIME_DIR/crawl-mail.sock`)

---

## IPC Protocol

The daemon speaks a JSON-RPC-like protocol over a Unix stream socket. Each message is a newline-delimited JSON object.

### Commands

Every command is serialised as a `CrawlCommand` enum (tagged by the `"method"` field):

```json
{"method": "ListAccounts"}
{"method": "AddAccount", "params": { ... }}
{"method": "SendMessage", "params": { ... }}
```

Full command list (18 mail-related variants):

| Command | Direction | Description |
|---------|-----------|-------------|
| `ListAccounts` | Request | List all configured accounts |
| `AddAccount` | Request | Add a new IMAP/SMTP account |
| `RemoveAccount` | Request | Delete an account and all its data |
| `ListFolders` | Request | List IMAP folders for an account |
| `SelectFolder` | Request | Signal folder selection to daemon (no-op) |
| `ListMessages` | Request | Paginated message listing with sort |
| `GetMessage` | Request | Get single message full body |
| `SearchMessages` | Request | Full-text search via FTS5 |
| `SendMessage` | Request | Enqueue an outbound message |
| `MoveMessage` | Request | Move message between folders |
| `CopyMessage` | Request | Copy message to another folder |
| `DeleteMessage` | Request | Remove message from folder |
| `SetFlags` | Request | Add/remove flags on a message |
| `SyncNow` | Request | Trigger immediate IMAP sync for account |
| `FetchBody` | Request | Fetch full message body from IMAP |
| `SaveAttachment` | Request | Save an attachment to disk |

### Responses

Every response is a `CrawlResponse` JSON object. Success:

```json
{"jsonrpc": "2.0", "result": {"type": "account_list", "data": {"accounts": [...]}}, "id": 1}
```

Error:

```json
{"jsonrpc": "2.0", "result": {"type": "error", "data": {"code": "not_found", "message": "..."}}, "id": 1}
```

The `result` payload is a `MailResponse` enum tagged by `"type"`:

| Response Type | When | Data |
|---------------|------|------|
| `"ok"` | Operation succeeded | `null` |
| `"error"` | Operation failed | `{ code, message }` |
| `"account_list"` | ListAccounts | `{ accounts: [...] }` |
| `"folder_list"` | ListFolders | `{ folders: [...] }` |
| `"message_list"` | ListMessages | `{ messages: [...], total }` |
| `"message"` | GetMessage / FetchBody | `{ message: {...} }` |
| `"search_results"` | SearchMessages | `{ messages: [...] }` |
| `"sync_status"` | SyncNow (deprecated) | `{ account_id, status }` |
| `"sync_complete"` | SyncNow success | `{ account_id }` |
| `"send_queued"` | SendMessage | `{ queue_id }` |

### Push Events

The IPC server maintains a persistent event stream for subscribed clients. Events are `CrawlEvent::Mail(MailEvent)`:

```json
{"domain": "mail", "data": {"event": "new_messages", "account_id": "...", "folder": "INBOX", "count": 3}}
```

| Event | Triggered by |
|-------|-------------|
| `"account_added"` | AddAccount command |
| `"new_messages"` | IDLE detects new mail, CopyMessage |
| `"flags_updated"` | SetFlags, MoveMessage, DeleteMessage commands |
| `"sync_complete"` | Periodic sync or SyncNow completes |
| `"sync_status"` | Sync state transitions (Running, Idle, Error) |
| `"attachment_saved"` | SaveAttachment command |

---

## Data Types

All types defined in `crawl-ipc/src/types.rs`:

### AccountInfo

```rust
pub struct AccountInfo {
    pub id: String,
    pub display_name: String,
    pub email: String,
    pub unread_count: u32,
}
```

### AddAccount

```rust
pub struct AddAccount {
    pub display_name: String,
    pub email: String,
    pub imap_host: String,
    pub imap_port: u16,
    pub smtp_host: String,
    pub smtp_port: u16,
    pub username: String,
    pub password: String,
}
```

### ListMessages / GetMessage / Search

```rust
pub struct ListMessages {
    pub account_id: String,
    pub folder: String,
    pub offset: u32,
    pub limit: u32,
    pub sort: MailSortOrder,
}

pub struct GetMessage {
    pub account_id: String,
    pub uid: u32,
    pub fetch_remote: bool,
}

pub struct Search {
    pub account_id: String,
    pub query: String,
    pub folder: Option<String>,
    pub limit: u32,
}
```

### SendMessage / MoveMessage / CopyMessage / SetFlags

```rust
pub struct SendMessage {
    pub account_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub bcc: Vec<String>,
    pub subject: String,
    pub body_text: String,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentRef>,
    pub in_reply_to: Option<String>,
}

pub struct MoveMessage {
    pub account_id: String,
    pub uid: u32,
    pub from_folder: String,
    pub to_folder: String,
}

pub struct CopyMessage {
    pub account_id: String,
    pub uid: u32,
    pub to_folder: String,
}

pub struct SetFlags {
    pub account_id: String,
    pub folder: String,
    pub uid: u32,
    pub add: Vec<MailFlag>,
    pub remove: Vec<MailFlag>,
}
```

### MessageFull / MessageSummary

```rust
pub struct MessageSummary {
    pub uid: u32,
    pub account_id: String,
    pub folder: String,
    pub from: String,
    pub subject: String,
    pub date: String,
    pub flags: Vec<MailFlag>,
    pub has_attachments: bool,
    pub snippet: String,
}

pub struct MessageFull {
    pub uid: u32,
    pub account_id: String,
    pub folder: String,
    pub message_id: String,
    pub from: String,
    pub to: Vec<String>,
    pub cc: Vec<String>,
    pub subject: String,
    pub date: String,
    pub flags: Vec<MailFlag>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub attachments: Vec<AttachmentInfo>,
    pub thread_id: Option<String>,
}
```

### Enums

```rust
pub enum MailSortOrder { DateDesc, DateAsc, SenderAsc, SubjectAsc }
pub enum MailFlag { Seen, Answered, Flagged, Deleted, Draft }
pub enum FolderKind { Inbox, Sent, Drafts, Trash, Spam, Archive, Custom }
pub enum SyncStatusKind { Running, Idle, Error }
pub enum MailErrorCode {
    AuthFailed, NetworkError, NotFound, InvalidParams,
    ImapError, SmtpError, DbError, Unknown,
}
```

---

## Storage Layer

### Schema

6 tables in SQLite:

| Table | Purpose | Key Constraints |
|-------|---------|----------------|
| `accounts` | Mail account credentials + IMAP/SMTP config | PK `id`, UNIQUE `email` |
| `folders` | IMAP folders with sync state | PK `id`, UNIQUE `(account_id, name)`, FK → `accounts` CASCADE |
| `messages` | Email headers, body, flags | PK `id`, UNIQUE `(account_id, folder_id, uid)`, FK → `folders` CASCADE |
| `messages_fts` | FTS5 full-text search on subject + body + from | External content table linked to `messages.rowid` |
| `attachments` | Attachment metadata | FK → `messages` CASCADE |
| `outbox` | Outbound message queue | FK → `accounts` (no cascade) |

Key design decisions:

- **Foreign keys with ON DELETE CASCADE**: Deleting an account or folder cleans up all related records automatically. The outbox table is exempt (must explicitly delete outbox entries before account deletion).
- **WAL mode**: Enables concurrent reads during sync operations.
- **FTS5 external content**: The FTS index references `messages` by rowid. Inserts manually sync via `INSERT INTO messages_fts(rowid, ...)`. Deletes use `INSERT INTO messages_fts(messages_fts, rowid) VALUES('delete', ?)`.
- **body_fetched flag**: The full body text/html is only fetched on demand (via `FetchBody` command). The initial sync only fetches headers to minimise bandwidth.
- **Flags stored as JSON array**: The `flags` column stores `["\\Seen", "\\Flagged"]` serialised as JSON. Deserialised to `Vec<MailFlag>` for IPC responses.

### Store Accessors

The `Store` struct wraps `SqlitePool` and exposes typed sub-stores:

```rust
impl Store {
    pub fn pool(&self) -> &SqlitePool;
    pub fn accounts(&self) -> AccountsStore;
    pub fn folders(&self) -> FoldersStore;
    pub fn messages(&self) -> MessagesStore;
    pub fn outbox(&self) -> OutboxStore;

    // Direct SQL methods on Store:
    pub async fn upsert_folder(...);
    pub async fn get_folder_state(...);
    pub async fn clear_folder_messages(...);
    pub async fn max_uid(...);
    pub async fn update_folder_state(...);
    pub async fn upsert_message(...);   // preserves existing fetched body
    pub async fn update_flags(...);
    pub async fn rebuild_fts(...);
}
```

#### AccountsStore

| Method | SQL | Description |
|--------|-----|-------------|
| `list_accounts()` | `SELECT id, display_name, email` | List all accounts |
| `add_account(payload)` | `INSERT INTO accounts` | Create account with IMAP/SMTP config |
| `delete_account(account_id)` | `DELETE FROM outbox` + `DELETE FROM accounts` (transaction) | Remove account and cascade |
| `get_smtp(account_id)` | `SELECT smtp_host, smtp_port, username, password` | Get SMTP credentials |
| `get_imap(account_id)` | `SELECT imap_host, imap_port, username, password` | Get IMAP credentials |

#### MessagesStore

| Method | SQL | Description |
|--------|-----|-------------|
| `list_messages(params)` | `SELECT ... ORDER BY {sort} LIMIT ? OFFSET ?` | Paginated + sorted message list |
| `get_message(params)` | `SELECT ... FROM messages JOIN folders` | Single message full body |
| `search_messages(params)` | `messages_fts MATCH ?` JOIN messages | Full-text search via FTS5 |
| `move_message(...)` | `UPDATE messages SET folder_id = ?` | Move between folders |
| `copy_message(...)` | `INSERT INTO messages ... SELECT` | Duplicate message to folder |
| `delete_message(...)` | `DELETE FROM messages` + FTS delete | Remove message |
| `set_flags(...)` | Read-modify-write on flags JSON | Add/remove flags atomically |
| `find_folder_for_message(...)` | `SELECT f.name FROM messages m JOIN folders f` | Lookup folder by account+uid |

#### FoldersStore

| Method | SQL | Description |
|--------|-----|-------------|
| `list_folders(account_id)` | `SELECT ... FROM folders WHERE account_id = ?` | List folders with counts |

#### OutboxStore

| Method | SQL | Description |
|--------|-----|-------------|
| `enqueue(payload)` | `INSERT INTO outbox` | Queue outbound message |
| `list_queued(limit)` | `SELECT ... JOIN accounts WHERE status = 'queued'` | Get pending messages + SMTP creds |
| `mark_status(id, status, error)` | `UPDATE outbox SET status = ?, attempts++` | Atomic state transition |

---

## IMAP Integration

### Session (`imap/session.rs`)

`ImapSession` wraps `async_imap::Session` over TLS:

```rust
pub struct ImapSession {
    session: Option<async_imap::Session<TlsTcpStream>>,
    selected: Option<String>,
    uidvalidity: u32,
    uidnext: u32,
}
```

**Methods:**

| Method | Description |
|--------|-------------|
| `connect(host, port, username, password)` | TCP + TLS + read greeting + IMAP LOGIN |
| `list(reference, pattern)` | List mailboxes, classify by `\Inbox`, `\Sent`, etc. |
| `select(folder)` | SELECT folder, cache uidvalidity/uidnext |
| `uid_validity_and_next()` | Return cached (uidvalidity, uidnext) |
| `uid_fetch(range, query)` | Fetch messages with arbitrary IMAP query |
| `fetch_raw_body(uid)` | Fetch BODY[] for a single UID |
| `idle_once()` | Execute IMAP IDLE, return `(bool, Option<Self>)` — `None` means session lost |

**Folder classification** maps IMAP attributes to kind strings:

| IMAP Attribute | Kind |
|----------------|------|
| `name == "INBOX"` (case-insensitive) | `"inbox"` |
| `\Sent` | `"sent"` |
| `\Drafts` | `"drafts"` |
| `\Trash` | `"trash"` |
| `\Junk` | `"spam"` |
| `\Archive` | `"archive"` |
| (none of the above) | `"custom"` |

**Flag mapping** converts `async_imap::Flag` to `String`:

| IMAP Flag | String |
|-----------|--------|
| `\Seen` | `"\\Seen"` |
| `\Answered` | `"\\Answered"` |
| `\Flagged` | `"\\Flagged"` |
| `\Deleted` | `"\\Deleted"` |
| `\Draft` | `"\\Draft"` |
| `\*` (MayCreate) | `"\\*"` |
| `Custom(s)` | `s` as-is |

### Sync Engine (`imap/sync.rs`)

`SyncEngine` orchestrates one full sync for an account:

```
full_sync()
  ├── list("*") all folders from IMAP
  ├── upsert each folder in DB
  └── for each folder:
       └── sync_folder()
            ├── SELECT folder on IMAP
            ├── compare uidvalidity with stored state
            ├── if uidvalidity changed:
            │     clear folder messages
            │     fetch_uid_range(1, uid_next)
            ├── else:
            │     fetch_uid_range(last_uid + 1, uid_next)
            ├── sync_flags() for 1:*
            └── update_folder_state(uidvalidity, uid_next)
```

**Incremental sync strategy:**

- The folder's `uidvalidity` and `uid_next` are stored in the `folders` table after each successful sync.
- On next sync, `uidvalidity` is compared first. If the server-side folder has been recreated (uidvalidity changed), all existing messages for that folder are deleted and a full refetch is done.
- If uidvalidity matches, only UIDs from `max(uid) + 1` to `uid_next` are fetched.
- After header fetch, `FLAGS` for `1:*` are fetched to sync read/unread/flagged state.
- There is **no flag push** to the IMAP server yet — changes made locally (via `SetFlags`) only update the local database.

### IDLE Loop (`imap/idle.rs`)

An infinite background task per account that:

1. Calls `session.idle_once()` which sends IMAP IDLE and waits
2. On `(true, Some(session))`:
   - Session is restored and ready for reuse
   - Sends a D-Bus notification
   - Emits `MailEvent::NewMessages { count: 0 }` (exact count unknown from IDLE)
3. On `(false, Some(session))`: sleeps 15s and retries
4. On `(false, None)`: session lost (IDLE DONE failed), exits loop

**Note:** The IDLE loop operates independently of the periodic sync loop. IDLE provides near-real-time notification; the sync loop performs the actual data fetch.

---

## SMTP Outbox

The `SmtpQueue` runs as a background tokio task that polls the `outbox` table every 10 seconds:

```
run()
  └── loop:
       └── process_queue()
            ├── list_queued(10) → Vec<QueuedMessage>
            └── for each message:
                 ├── mark_status("sending")
                 ├── send_single(message)
                 │    ├── build lettre::Message (multipart/alternative)
                 │    ├── connect via STARTTLS relay
                 │    ├── authenticate with SMTP credentials
                 │    └── send via tokio async transport
                 ├── mark_status("sent") on success
                 └── mark_status("failed", error) on failure
```

**Send flow:**

1. Build a `lettre::Message::builder()` with `From`, `To`, `Cc`, `Bcc`, `Subject`
2. Create multi-part body: `MultiPart::alternative(plain, html)` if HTML is present
3. Connect via `AsyncSmtpTransport::starttls_relay()` using the account's SMTP host
4. Authenticate with username/password
5. Send the message asynchronously
6. Update outbox status (`"queued" → "sending" → "sent"` or `"failed"`)

**Limitations:**

- Attachments are not yet implemented (logs a warning and skips them)
- Max 10 messages processed per poll cycle
- Messages with empty passwords are immediately marked as failed

---

## Email Parsing

`parser.rs` handles RFC 5322 parsing via `mail-parser 0.9` (Stalwart Labs):

```rust
pub struct ParsedMessage {
    pub uid: u32,
    pub message_id: Option<String>,
    pub thread_id: Option<String>,
    pub from_addr: String,
    pub from_name: Option<String>,
    pub to_addrs: Vec<String>,
    pub cc_addrs: Vec<String>,
    pub subject: Option<String>,
    pub date: String,           // RFC 3339
    pub flags: Vec<String>,     // IMAP flag strings
    pub snippet: Option<String>,
    pub body_text: Option<String>,
    pub body_html: Option<String>,
    pub body_fetched: bool,
    pub has_attachments: bool,
    pub raw_size: Option<i64>,
}
```

`parse_message(&ImapMessage)` extracts from raw IMAP headers:

| Field | Source |
|-------|--------|
| uid | `ImapMessage.uid` |
| message_id | `msg.message_id()` |
| from_addr / from_name | `msg.from()` → `Address::List(Vec<Addr>)`, uses `Addr::first()` |
| to_addrs | `msg.to()` → `Address::List(Vec<Addr>)` / `Address::Group(Vec<Group>)`, iterates `.address` field |
| cc_addrs | `msg.cc()` |
| subject | `msg.subject()` |
| date | `msg.date().to_rfc3339()` fallback to `Utc::now()` |
| snippet | `msg.body_preview(150)` |
| body_text | `msg.body_text(0)` |
| body_html | `msg.body_html(0)` |
| has_attachments | `msg.attachment_count() > 0` |
| flags | `ImapMessage.flags()` |

---

## Account Lifecycle

`accounts.rs` manages per-account sync loops and one-shot syncs.

### Startup

`AccountManager::start_all()`:

1. `list_accounts()` from DB
2. For each account:
   - Get IMAP credentials via `get_imap(id)`
   - Skip if IMAP config missing or password empty
   - Spawn a `tokio::spawn(run_sync_loop(...))` task

### Periodic Sync Loop

```rust
loop {
    emit SyncStatus::Running
    match try_sync(account_id, store, imap).await {
        Ok(())  => emit SyncComplete
        Err(e)  => emit SyncStatus::Error, sleep 60s, continue
    }
    emit SyncStatus::Idle
    sleep 300s (5 minutes)
}
```

### One-Shot Sync

`AccountManager::sync_now(account_id)` performs the same `try_sync` inline with event emissions, used by the `SyncNow` IPC command.

---

## Desktop Notifications

`notify.rs` sends D-Bus notifications via `org.freedesktop.Notifications`:

```rust
pub async fn notify_new_mail(summary: &str) -> anyhow::Result<()>
```

Uses `zbus 4` to:
1. Connect to the session D-Bus bus
2. Call `Notify` method with app name "crawl-mail"
3. Set the summary text (e.g., "New mail in INBOX")
4. No actions, hints, or timeout specified

Called by the IDLE loop when new mail is detected.

---

## Configuration

### Config File

Path resolution (first match wins):

1. `$CRAWL_MAIL_CONFIG` environment variable
2. `$XDG_CONFIG_HOME/crawl/config.toml`
3. `$HOME/.config/crawl/config.toml`
4. `./config.toml` (CWD fallback)

```toml
# $XDG_CONFIG_HOME/crawl/config.toml
db_path = "/path/to/mail.db"
socket_path = "/path/to/crawl-mail.sock"
```

If the config file does not exist, both fields use environment-derived defaults.

### Environment Variables

| Variable | Overrides | Default |
|----------|-----------|---------|
| `CRAWL_MAIL_CONFIG` | Config file path | `$XDG_CONFIG_HOME/crawl/config.toml` |
| `CRAWL_MAIL_DB` | Database file path | `$XDG_DATA_HOME/crawl/mail.db` |
| `CRAWL_MAIL_SOCKET` | Unix socket path | `$XDG_RUNTIME_DIR/crawl-mail.sock` |
| `XDG_CONFIG_HOME` | Config directory | `$HOME/.config` |
| `XDG_DATA_HOME` | Data directory | `$HOME/.local/share` |
| `XDG_RUNTIME_DIR` | Runtime directory | `/tmp/crawl-mail.sock` fallback |

### Default Paths

```
Config:   $HOME/.config/crawl/config.toml
Database: $HOME/.local/share/crawl/mail.db
Socket:   $XDG_RUNTIME_DIR/crawl-mail.sock  (or /tmp/crawl-mail.sock)
```

---

## Development

### Dependencies

Key crates:

| Crate | Version | Purpose |
|-------|---------|---------|
| `async-imap` | 0.9 | IMAP client |
| `async-native-tls` | 0.5 | TLS for IMAP connections |
| `lettre` | 0.11 | SMTP client |
| `mail-parser` | 0.9 | RFC 5322 / MIME parsing |
| `sqlx` | 0.7 | SQLite async driver |
| `zbus` | 4 | D-Bus client for notifications |
| `tokio` | 1 | Async runtime |
| `serde` / `serde_json` | 1 | JSON serialisation |

### Module Map

| File | Lines | Responsibility |
|------|-------|----------------|
| `main.rs` | 49 | Entrypoint, wiring |
| `config.rs` | 67 | Config file loading |
| `error.rs` | 18 | Error types |
| `ipc.rs` | ~445 | IPC dispatcher + body/attachment fetch handlers |
| `accounts.rs` | 148 | Account lifecycle + sync orchestration |
| `notify.rs` | 32 | D-Bus desktop notifications |
| `parser.rs` | 93 | RFC 5322 email parsing |
| `smtp.rs` | 134 | SMTP outbox queue |
| `imap/mod.rs` | 7 | Module re-exports |
| `imap/session.rs` | 173 | IMAP session abstraction |
| `imap/idle.rs` | 36 | IMAP IDLE loop |
| `imap/sync.rs` | 95 | IMAP sync engine |
| `store/mod.rs` | 299 | SQLite pool, core queries |
| `store/accounts.rs` | 119 | Accounts CRUD |
| `store/messages.rs` | 380 | Messages CRUD, search, flags |
| `store/folders.rs` | 48 | Folders listing |
| `store/outbox.rs` | 108 | Outbound queue |
| `store/schema.sql` | 89 | DDL |

### Testing

The daemon has no test suite yet. Recommended testing approach:

- **Unit tests**: Test `ParsedMessage` parsing with synthetic IMAP fetch data
- **Integration tests**: Use a local IMAP test server (e.g., `greenmail`) to verify sync logic
- **SQL tests**: Verify SQL queries against an in-memory SQLite database

### IPC Debugging

Use the `crawl-cli` tool or `socat`:

```bash
# Listen to events
crawl-cli mail --watch

# Send raw commands
echo '{"method":"ListAccounts","id":1}' | socat - UNIX-CONNECT:$XDG_RUNTIME_DIR/crawl-mail.sock
```

### Future Work

- [ ] **Secret Service integration**: Migrate password storage from `accounts.password` column to `org.freedesktop.SecretService`
- [ ] **Flag push to IMAP**: Sync local flag changes back to IMAP server
- [ ] **SMTP attachments**: Encode and send MIME attachments
- [x] **Attachment caching**: Download, cache, and serve attachments on demand
- [ ] **Threading**: Populate `thread_id` from `References` / `In-Reply-To` headers
- [ ] **Push IMAP flag sync**: Sync flag changes bidirectionally
- [ ] **Multiple IDLE connections**: Maintain parallel IDLE for all folders (currently INBOX only)
