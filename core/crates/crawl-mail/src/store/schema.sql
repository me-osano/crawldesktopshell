-- Enable WAL for concurrent reads during sync
PRAGMA journal_mode = WAL;
PRAGMA foreign_keys = ON;

CREATE TABLE IF NOT EXISTS accounts (
    id           TEXT PRIMARY KEY,
    display_name TEXT NOT NULL,
    email        TEXT NOT NULL UNIQUE,
    imap_host    TEXT NOT NULL,
    imap_port    INTEGER NOT NULL,
    smtp_host    TEXT NOT NULL,
    smtp_port    INTEGER NOT NULL,
    username     TEXT NOT NULL,
    password     TEXT NOT NULL DEFAULT '',
    created_at   TEXT NOT NULL
);

CREATE TABLE IF NOT EXISTS folders (
    id           TEXT PRIMARY KEY,
    account_id   TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    name         TEXT NOT NULL,   -- IMAP path e.g. "INBOX" or "INBOX.Sent"
    display_name TEXT NOT NULL,
    kind         TEXT NOT NULL DEFAULT 'custom',
    uidvalidity  INTEGER,
    uid_next     INTEGER,
    unread_count INTEGER NOT NULL DEFAULT 0,
    total_count  INTEGER NOT NULL DEFAULT 0,
    last_synced  TEXT,
    UNIQUE(account_id, name)
);

CREATE TABLE IF NOT EXISTS messages (
    id              TEXT PRIMARY KEY,  -- internal UUID
    account_id      TEXT NOT NULL REFERENCES accounts(id) ON DELETE CASCADE,
    folder_id       TEXT NOT NULL REFERENCES folders(id) ON DELETE CASCADE,
    uid             INTEGER NOT NULL,
    message_id      TEXT,              -- RFC 5322 Message-ID header
    thread_id       TEXT,              -- computed via References/In-Reply-To
    from_addr       TEXT NOT NULL,
    from_name       TEXT,
    to_addrs        TEXT NOT NULL,     -- JSON array
    cc_addrs        TEXT,              -- JSON array
    subject         TEXT,
    date            TEXT NOT NULL,     -- ISO 8601
    flags           TEXT NOT NULL DEFAULT '[]',  -- JSON array
    snippet         TEXT,
    body_text       TEXT,              -- NULL if not yet fetched
    body_html       TEXT,              -- NULL if not yet fetched
    body_fetched    INTEGER NOT NULL DEFAULT 0,
    has_attachments INTEGER NOT NULL DEFAULT 0,
    raw_size        INTEGER,
    created_at      TEXT NOT NULL,
    UNIQUE(account_id, folder_id, uid)
);

-- Full-text search on subject + body_text + from
CREATE VIRTUAL TABLE IF NOT EXISTS messages_fts USING fts5(
    subject, body_text, from_addr,
    content='messages', content_rowid='rowid'
);

CREATE TABLE IF NOT EXISTS attachments (
    id          TEXT PRIMARY KEY,
    message_id  TEXT NOT NULL REFERENCES messages(id) ON DELETE CASCADE,
    filename    TEXT NOT NULL,
    mime_type   TEXT NOT NULL,
    size        INTEGER NOT NULL,
    content_id  TEXT,           -- for inline images
    cached_path TEXT,           -- local path if downloaded
    part_index  INTEGER NOT NULL
);

CREATE TABLE IF NOT EXISTS outbox (
    id           TEXT PRIMARY KEY,
    account_id   TEXT NOT NULL REFERENCES accounts(id),
    payload      TEXT NOT NULL,  -- JSON of SendMessageParams
    status       TEXT NOT NULL DEFAULT 'queued',  -- queued|sending|sent|failed
    attempts     INTEGER NOT NULL DEFAULT 0,
    last_error   TEXT,
    created_at   TEXT NOT NULL,
    sent_at      TEXT
);

-- Indexes
CREATE INDEX IF NOT EXISTS idx_messages_folder    ON messages(folder_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_messages_thread    ON messages(thread_id);
CREATE INDEX IF NOT EXISTS idx_messages_date      ON messages(account_id, date DESC);
CREATE INDEX IF NOT EXISTS idx_attachments_msg    ON attachments(message_id);
CREATE INDEX IF NOT EXISTS idx_outbox_status      ON outbox(status);