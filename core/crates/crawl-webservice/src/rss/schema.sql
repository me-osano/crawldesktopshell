PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS feeds (
    id          TEXT PRIMARY KEY,
    url         TEXT NOT NULL UNIQUE,
    title       TEXT DEFAULT '',
    description TEXT DEFAULT '',
    site_url    TEXT DEFAULT '',
    icon_url    TEXT DEFAULT '',
    category    TEXT DEFAULT '',
    error_count INTEGER DEFAULT 0,
    last_error  TEXT DEFAULT '',
    last_fetched TEXT DEFAULT '',
    fetch_interval INTEGER DEFAULT 1800,
    etag        TEXT DEFAULT '',
    last_modified TEXT DEFAULT '',
    created_at  TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE TABLE IF NOT EXISTS entries (
    id          TEXT PRIMARY KEY,
    feed_id     TEXT NOT NULL REFERENCES feeds(id) ON DELETE CASCADE,
    guid        TEXT NOT NULL,
    url         TEXT DEFAULT '',
    title       TEXT DEFAULT '',
    author      TEXT DEFAULT '',
    summary     TEXT DEFAULT '',
    content     TEXT DEFAULT '',
    content_type TEXT DEFAULT 'html',
    published   TEXT DEFAULT '',
    fetched_at  TEXT NOT NULL,
    is_read     INTEGER DEFAULT 0,
    is_starred  INTEGER DEFAULT 0,
    image_url   TEXT DEFAULT '',
    UNIQUE(feed_id, guid)
);

CREATE TABLE IF NOT EXISTS categories (
    id   TEXT PRIMARY KEY,
    name TEXT NOT NULL UNIQUE
);

CREATE TABLE IF NOT EXISTS opml_imports (
    id         TEXT PRIMARY KEY,
    source     TEXT,
    feed_count INTEGER DEFAULT 0,
    imported   INTEGER DEFAULT 0,
    failed     INTEGER DEFAULT 0,
    created_at TEXT NOT NULL DEFAULT (datetime('now'))
);

CREATE INDEX IF NOT EXISTS idx_entries_feed_id ON entries(feed_id);
CREATE INDEX IF NOT EXISTS idx_entries_published ON entries(published DESC);
CREATE INDEX IF NOT EXISTS idx_entries_is_read ON entries(is_read);
CREATE INDEX IF NOT EXISTS idx_entries_is_starred ON entries(is_starred);
CREATE INDEX IF NOT EXISTS idx_feeds_category ON feeds(category);
CREATE INDEX IF NOT EXISTS idx_feeds_last_fetched ON feeds(last_fetched);
