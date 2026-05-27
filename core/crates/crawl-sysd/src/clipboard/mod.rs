use std::collections::{HashSet, VecDeque};
use std::io::Read;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Arc, Mutex, OnceLock};
use std::time::{Duration, SystemTime, UNIX_EPOCH};

use anyhow::{Context, Result};
use crawl_ipc::events::{ClipboardEvent, CrawlEvent};
use crawl_ipc::types::{ClipContent, ClipEntry};
use sqlx::SqlitePool;
use tokio::runtime::Handle;
use tokio::sync::broadcast;
use tracing::{debug, info, warn};

use crate::config::ClipboardConfig;

// ── FNV-1a 64-bit hash ─────────────────────────────────────

fn fnv1a_hash(data: &[u8]) -> u64 {
    let mut hash: u64 = 0xcbf29ce484222325;
    for &byte in data {
        hash ^= byte as u64;
        hash = hash.wrapping_mul(0x100000001b3);
    }
    hash
}

fn now_ms() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_millis() as u64)
        .unwrap_or(0)
}

fn default_db_path() -> PathBuf {
    let base = std::env::var("XDG_DATA_HOME")
        .or_else(|_| std::env::var("HOME").map(|h| format!("{}/.local/share", h)))
        .unwrap_or_else(|_| "/tmp".to_string());
    PathBuf::from(base).join("crawlds").join("clipboard.db")
}

fn make_preview(content: &[u8], mime: &str) -> String {
    if mime.len() > 6 && mime.starts_with("image/") {
        let size = content.len();
        let units = ["B", "KiB", "MiB"];
        let mut fsize = size as f64;
        let mut i = 0;
        while fsize >= 1024.0 && i < units.len() - 1 {
            fsize /= 1024.0;
            i += 1;
        }
        return format!("[[ image {:.0} {} ]]", fsize, units[i]);
    }
    let text = String::from_utf8_lossy(content);
    let text = text.trim();
    let text: Vec<&str> = text.split_whitespace().collect();
    let text = text.join(" ");
    if text.len() > 120 {
        format!("{}…", &text[..120])
    } else {
        text.to_string()
    }
}

// ── Inner State (in-memory cache) ──────────────────────────

struct ClipboardInner {
    entries: VecDeque<ClipEntry>,
    hashes: HashSet<u64>,
    next_id: u64,
}

// ── Main Backend ───────────────────────────────────────────

pub struct ClipboardBackend {
    inner: Mutex<ClipboardInner>,
    event_tx: broadcast::Sender<CrawlEvent>,
    config: ClipboardConfig,
    pool: OnceLock<SqlitePool>,
    db_path: PathBuf,
    handle: OnceLock<Handle>,
    monitor_stop: Arc<AtomicBool>,
    pub(crate) last_copied_hash: AtomicU64,
}

impl ClipboardBackend {
    pub fn new(config: ClipboardConfig, event_tx: broadcast::Sender<CrawlEvent>) -> Self {
        let db_path = default_db_path();
        Self {
            inner: Mutex::new(ClipboardInner {
                entries: VecDeque::new(),
                hashes: HashSet::new(),
                next_id: 1,
            }),
            event_tx,
            config,
            pool: OnceLock::new(),
            db_path,
            handle: OnceLock::new(),
            monitor_stop: Arc::new(AtomicBool::new(false)),
            last_copied_hash: AtomicU64::new(0),
        }
    }

    pub async fn init(&self) -> Result<()> {
        if !self.config.enabled {
            info!("Clipboard service disabled by config");
            return Ok(());
        }
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            warn!("WAYLAND_DISPLAY not set — clipboard service will be inactive");
            return Ok(());
        }
        info!("Initializing clipboard backend");

        if self.config.persistent {
            if let Some(parent) = self.db_path.parent() {
                std::fs::create_dir_all(parent)
                    .context("Failed to create clipboard data directory")?;
            }

            let url = format!("sqlite://{}", self.db_path.display());
            let pool = SqlitePool::connect(&url)
                .await
                .context("Failed to connect to clipboard database")?;

            Self::migrate(&pool).await?;

            // Load entries into cache (newest first, pinned first)
            let (entries, hashes, max_id): (VecDeque<ClipEntry>, HashSet<u64>, u64) = {
                let rows = sqlx::query_as::<_, (i64, i64, Vec<u8>, String, String, i64, i32, i32, i64)>(
                    "SELECT id, hash, content, mime, preview, timestamp_ms, \
                     is_image, is_pinned, size FROM entries \
                     ORDER BY is_pinned DESC, timestamp_ms DESC",
                )
                .fetch_all(&pool)
                .await
                .context("Failed to load clipboard entries")?;

                let mut entries = VecDeque::new();
                let mut hashes = HashSet::new();
                let mut max_id: u64 = 0;

                for (id, hash, content, mime, preview, ts, is_img, is_pin, sz) in rows {
                    let id = id as u64;
                    let hash = hash as u64;
                    hashes.insert(hash);
                    if id > max_id {
                        max_id = id;
                    }
                    entries.push_back(ClipEntry {
                        id,
                        content: String::from_utf8_lossy(&content).to_string(),
                        mime,
                        preview,
                        timestamp_ms: ts as u64,
                        is_image: is_img != 0,
                        is_pinned: is_pin != 0,
                        size: sz as u64,
                    });
                }
                (entries, hashes, max_id)
            };

            {
                let mut inner = self.inner.lock().unwrap();
                inner.entries = entries;
                inner.hashes = hashes;
                inner.next_id = max_id + 1;
            }

            let _ = self.pool.set(pool);
            info!(
                "Clipboard: loaded {} entries from database",
                self.inner.lock().unwrap().entries.len()
            );
        }

        Ok(())
    }

    async fn migrate(pool: &SqlitePool) -> Result<()> {
        let schema = include_str!("schema.sql");
        for statement in schema.split(';') {
            let stmt = statement.trim();
            if stmt.is_empty() {
                continue;
            }
            sqlx::query(stmt)
                .execute(pool)
                .await
                .context("Clipboard: schema migration failed")?;
        }
        Ok(())
    }

    // ── Sync methods (no DB, or use block_on bridge) ────────

    pub fn list(&self) -> Vec<ClipEntry> {
        let inner = self.inner.lock().unwrap();
        inner.entries.iter().cloned().collect()
    }

    pub fn store(&self, content: &str, mime: &str) -> Result<Option<u64>> {
        let content_bytes = content.as_bytes();
        let size = content_bytes.len() as u64;
        if size == 0 || size > self.config.max_entry_size {
            return Ok(None);
        }

        let hash = fnv1a_hash(content_bytes);
        let is_image = mime.len() > 6 && mime.starts_with("image/");
        let preview = make_preview(content_bytes, mime);
        let now = now_ms();

        // ── Dedup check (in-memory) ──
        {
            let mut inner = self.inner.lock().unwrap();
            if inner.hashes.contains(&hash) {
                if let Some(entry) = inner.entries.iter_mut().find(|e| {
                    fnv1a_hash(e.content.as_bytes()) == hash
                }) {
                    entry.timestamp_ms = now;
                    let id = entry.id;
                    drop(inner);
                    if let (Some(pool), Some(handle)) = (self.pool.get(), self.handle.get()) {
                        let content_owned = content_bytes.to_vec();
                        let _ = handle.block_on(async {
                            sqlx::query(
                                "UPDATE entries SET timestamp_ms = ?1, content = ?2 WHERE id = ?3",
                            )
                            .bind(now as i64)
                            .bind(content_owned)
                            .bind(id as i64)
                            .execute(pool)
                            .await
                        });
                    }
                }
                return Ok(None);
            }
        }

        // ── New entry ──
        let (entry, removed_ids) = {
            let mut inner = self.inner.lock().unwrap();

            let id = inner.next_id;
            inner.next_id += 1;

            let entry = ClipEntry {
                id,
                content: content.to_string(),
                mime: mime.to_string(),
                preview,
                timestamp_ms: now,
                is_image,
                is_pinned: false,
                size,
            };

            inner.hashes.insert(hash);
            inner.entries.push_front(entry.clone());

            // Trim oldest unpinned entries from cache
            let mut removed_ids = Vec::new();
            while inner.entries.len() > self.config.history_size {
                let tail_pinned = inner.entries.back().map(|e| e.is_pinned).unwrap_or(true);
                if tail_pinned {
                    break;
                }
                let removed = inner.entries.pop_back().unwrap();
                inner.hashes.remove(&fnv1a_hash(removed.content.as_bytes()));
                removed_ids.push(removed.id);
            }

            (entry, removed_ids)
        };

        // ── Persist to SQLite (async via block_on) ──
        if let (Some(pool), Some(handle)) = (self.pool.get(), self.handle.get()) {
            let content_owned = content_bytes.to_vec();
            let mime_owned = entry.mime.clone();
            let preview_owned = entry.preview.clone();
            let _ = handle.block_on(async {
                let _ = sqlx::query(
                    "INSERT INTO entries (id, hash, content, mime, preview, \
                     timestamp_ms, is_image, is_pinned, size) \
                     VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9) \
                     ON CONFLICT(id) DO UPDATE SET \
                     hash=excluded.hash, content=excluded.content, \
                     mime=excluded.mime, preview=excluded.preview, \
                     timestamp_ms=excluded.timestamp_ms, \
                     is_image=excluded.is_image, is_pinned=excluded.is_pinned, \
                     size=excluded.size",
                )
                .bind(entry.id as i64)
                .bind(hash as i64)
                .bind(content_owned)
                .bind(&mime_owned)
                .bind(&preview_owned)
                .bind(now as i64)
                .bind(is_image as i32)
                .bind(false as i32)
                .bind(size as i64)
                .execute(pool)
                .await;

                for rid in &removed_ids {
                    let _ = sqlx::query("DELETE FROM entries WHERE id = ?")
                        .bind(*rid as i64)
                        .execute(pool)
                        .await;
                }
            });
        }

        // Broadcast event
        let _ = self.event_tx.send(CrawlEvent::Clipboard(
            ClipboardEvent::Changed {
                entry: entry.clone(),
            },
        ));

        debug!("Clipboard: stored id={} hash={} size={}", entry.id, hash, size);
        Ok(Some(entry.id))
    }

    pub fn paste_text(&self, text: &str) -> Result<()> {
        set_clipboard_inner(text, "text/plain")?;
        let hash = fnv1a_hash(text.as_bytes());
        self.last_copied_hash.store(hash, Ordering::Release);
        Ok(())
    }

    pub fn set_clipboard(&self, text: &str, mime: &str) -> Result<()> {
        set_clipboard_inner(text, mime)?;
        let hash = fnv1a_hash(text.as_bytes());
        self.last_copied_hash.store(hash, Ordering::Release);
        Ok(())
    }

    pub fn start_monitor(self: Arc<Self>) {
        if !self.config.enabled {
            return;
        }
        if std::env::var("WAYLAND_DISPLAY").is_err() {
            return;
        }

        // Capture the tokio runtime handle for sync→async bridging
        if let Ok(h) = Handle::try_current() {
            let _ = self.handle.set(h);
        }

        let stop = self.monitor_stop.clone();
        let backend = self.clone();
        let poll_ms = self.config.poll_interval_ms;

        info!(
            "Clipboard: starting monitor thread (poll={}ms, watch_primary={})",
            poll_ms, self.config.watch_primary
        );

        std::thread::spawn(move || {
            let mut last_hash: u64 = 0;
            let mut last_primary_hash: u64 = 0;

            while !stop.load(Ordering::Relaxed) {
                std::thread::sleep(Duration::from_millis(poll_ms));
                if stop.load(Ordering::Relaxed) {
                    break;
                }

                if let Some((content, mime)) = read_clipboard_inner() {
                    let hash = fnv1a_hash(content.as_bytes());
                    let last_copied = backend.last_copied_hash.load(Ordering::Acquire);
                    if hash != last_hash && hash != last_copied {
                        last_hash = hash;
                        if let Err(e) = backend.store(&content, &mime) {
                            warn!("Clipboard monitor: store error: {e}");
                        }
                    }
                }

                if backend.config.watch_primary {
                    if let Some((content, mime)) = read_primary_inner() {
                        let hash = fnv1a_hash(content.as_bytes());
                        if hash != last_primary_hash {
                            last_primary_hash = hash;
                            if let Err(e) = backend.store(&content, &mime) {
                                warn!("Clipboard primary monitor: store error: {e}");
                            }
                        }
                    }
                }
            }

            info!("Clipboard monitor thread stopped");
        });
    }

    pub fn stop(&self) {
        self.monitor_stop.store(true, Ordering::Release);
    }

    // ── Async methods (called from tokio service handlers) ──

    pub async fn get_content(&self, id: u64) -> Result<Option<ClipContent>> {
        let entry = {
            let inner = self.inner.lock().unwrap();
            inner.entries.iter().find(|e| e.id == id).cloned()
        };

        if let Some(entry) = entry {
            if entry.mime.len() > 6 && entry.mime.starts_with("image/") {
                let bytes = if let Some(pool) = self.pool.get() {
                    sqlx::query_scalar::<_, Vec<u8>>("SELECT content FROM entries WHERE id = ?")
                        .bind(id as i64)
                        .fetch_optional(pool)
                        .await?
                        .unwrap_or_default()
                } else {
                    Vec::new()
                };
                let b64 = if bytes.is_empty() {
                    None
                } else {
                    Some(base64_encode(&bytes))
                };
                Ok(Some(ClipContent {
                    content: String::new(),
                    mime: entry.mime,
                    data_base64: b64,
                    size: entry.size,
                }))
            } else {
                Ok(Some(ClipContent {
                    content: entry.content,
                    mime: entry.mime,
                    data_base64: None,
                    size: entry.size,
                }))
            }
        } else {
            Ok(None)
        }
    }

    pub async fn delete(&self, id: u64) -> Result<bool> {
        let found = {
            let mut inner = self.inner.lock().unwrap();
            if let Some(pos) = inner.entries.iter().position(|e| e.id == id) {
                let removed = inner.entries.remove(pos).unwrap();
                inner.hashes.remove(&fnv1a_hash(removed.content.as_bytes()));
                true
            } else {
                false
            }
        };

        if found {
            if let Some(pool) = self.pool.get() {
                sqlx::query("DELETE FROM entries WHERE id = ?")
                    .bind(id as i64)
                    .execute(pool)
                    .await?;
            }
            let _ = self
                .event_tx
                .send(CrawlEvent::Clipboard(ClipboardEvent::Deleted { id }));
            Ok(true)
        } else {
            Ok(false)
        }
    }

    pub async fn wipe(&self) -> Result<()> {
        {
            let mut inner = self.inner.lock().unwrap();
            inner.entries.clear();
            inner.hashes.clear();
        }

        if let Some(pool) = self.pool.get() {
            sqlx::query("DELETE FROM entries")
                .execute(pool)
                .await?;
        }

        let _ = self
            .event_tx
            .send(CrawlEvent::Clipboard(ClipboardEvent::Cleared));
        Ok(())
    }

    pub async fn toggle_pin(&self, id: u64) -> Result<bool> {
        let pinned = {
            let mut inner = self.inner.lock().unwrap();
            if let Some(entry) = inner.entries.iter_mut().find(|e| e.id == id) {
                entry.is_pinned = !entry.is_pinned;
                entry.is_pinned
            } else {
                return Ok(false);
            }
        };

        if let Some(pool) = self.pool.get() {
            sqlx::query("UPDATE entries SET is_pinned = ?1 WHERE id = ?2")
                .bind(pinned as i32)
                .bind(id as i64)
                .execute(pool)
                .await?;
        }

        let _ = self
            .event_tx
            .send(CrawlEvent::Clipboard(ClipboardEvent::Pinned { id, pinned }));
        Ok(true)
    }

    pub async fn copy_to_clipboard(&self, id: u64) -> Result<bool> {
        let entry = {
            let inner = self.inner.lock().unwrap();
            inner.entries.iter().find(|e| e.id == id).cloned()
        };

        if let Some(entry) = entry {
            set_clipboard_inner(&entry.content, &entry.mime)?;
            let hash = fnv1a_hash(entry.content.as_bytes());
            self.last_copied_hash.store(hash, Ordering::Release);

            let now = now_ms();
            {
                let mut inner = self.inner.lock().unwrap();
                if let Some(e) = inner.entries.iter_mut().find(|e| e.id == id) {
                    e.timestamp_ms = now;
                }
            }

            if let Some(pool) = self.pool.get() {
                sqlx::query("UPDATE entries SET timestamp_ms = ?1 WHERE id = ?2")
                    .bind(now as i64)
                    .bind(id as i64)
                    .execute(pool)
                    .await?;
            }

            Ok(true)
        } else {
            Ok(false)
        }
    }
}

// ── Wayland Clipboard I/O ──────────────────────────────────

fn read_clipboard_inner() -> Option<(String, String)> {
    use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
    match get_contents(ClipboardType::Regular, Seat::Unspecified, MimeType::Text) {
        Ok(result) => {
            let (mut reader, mime) = result;
            let mut content = String::new();
            if reader.read_to_string(&mut content).is_ok() && !content.is_empty() {
                Some((content, mime.to_string()))
            } else {
                None
            }
        }
        Err(e) => {
            debug!("Clipboard read error: {e}");
            None
        }
    }
}

fn read_primary_inner() -> Option<(String, String)> {
    use wl_clipboard_rs::paste::{get_contents, ClipboardType, MimeType, Seat};
    match get_contents(ClipboardType::Primary, Seat::Unspecified, MimeType::Text) {
        Ok(result) => {
            let (mut reader, mime) = result;
            let mut content = String::new();
            if reader.read_to_string(&mut content).is_ok() && !content.is_empty() {
                Some((content, mime.to_string()))
            } else {
                None
            }
        }
        Err(e) => {
            debug!("Clipboard primary read error: {e}");
            None
        }
    }
}

fn set_clipboard_inner(content: &str, mime: &str) -> Result<()> {
    use wl_clipboard_rs::copy::{copy, Options, Source, MimeType as CopyMime};
    let options = Options::new();
    let mime_type = if mime == "text/plain" || mime.is_empty() {
        CopyMime::Text
    } else {
        CopyMime::Specific(mime.to_string())
    };
    let bytes: Vec<u8> = content.as_bytes().to_vec();
    let source = Source::Bytes(bytes.into_boxed_slice());
    copy(options, source, mime_type).map_err(|e| anyhow::anyhow!("Failed to set clipboard: {e}"))
}

fn base64_encode(data: &[u8]) -> String {
    const CHARS: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZabcdefghijklmnopqrstuvwxyz0123456789+/";
    let mut result = String::with_capacity(data.len() * 4 / 3 + 4);
    for chunk in data.chunks(3) {
        let b0 = chunk[0] as u32;
        let b1 = chunk.get(1).copied().unwrap_or(0) as u32;
        let b2 = chunk.get(2).copied().unwrap_or(0) as u32;
        let triple = (b0 << 16) | (b1 << 8) | b2;
        result.push(CHARS[((triple >> 18) & 0x3F) as usize] as char);
        result.push(CHARS[((triple >> 12) & 0x3F) as usize] as char);
        if chunk.len() > 1 {
            result.push(CHARS[((triple >> 6) & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
        if chunk.len() > 2 {
            result.push(CHARS[(triple & 0x3F) as usize] as char);
        } else {
            result.push('=');
        }
    }
    result
}
