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
