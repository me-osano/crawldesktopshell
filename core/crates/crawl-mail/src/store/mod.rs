use std::path::Path;

use sqlx::{Row, SqlitePool};

use crate::{imap::ImapFolder, parser::ParsedMessage};

pub mod accounts;
pub mod folders;
pub mod messages;
pub mod outbox;

#[derive(Clone)]
pub struct Store {
    pool: SqlitePool,
}

#[derive(Debug, Clone)]
pub struct FolderState {
    pub uidvalidity: u32,
    pub uid_next: u32,
}

pub async fn open(path: &Path) -> anyhow::Result<SqlitePool> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            std::fs::create_dir_all(parent)?;
        }
    }
    let url = format!("sqlite://{}", path.display());
    Ok(SqlitePool::connect(&url).await?)
}

pub async fn migrate(pool: &SqlitePool) -> anyhow::Result<()> {
    let schema = include_str!("schema.sql");
    for statement in schema.split(';') {
        let stmt = statement.trim();
        if stmt.is_empty() {
            continue;
        }
        sqlx::query(stmt).execute(pool).await?;
    }

    // Migrations for existing databases
    let _ = sqlx::query("ALTER TABLE accounts ADD COLUMN password TEXT NOT NULL DEFAULT ''")
        .execute(pool)
        .await;
    let _ = sqlx::query("CREATE INDEX IF NOT EXISTS idx_outbox_status ON outbox(status)")
        .execute(pool)
        .await;

    Ok(())
}

impl Store {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub fn accounts(&self) -> accounts::AccountsStore<'_> {
        accounts::AccountsStore::new(&self.pool)
    }

    pub fn folders(&self) -> folders::FoldersStore<'_> {
        folders::FoldersStore::new(&self.pool)
    }

    pub fn messages(&self) -> messages::MessagesStore<'_> {
        messages::MessagesStore::new(&self.pool)
    }

    pub fn outbox(&self) -> outbox::OutboxStore<'_> {
        outbox::OutboxStore::new(&self.pool)
    }

    pub async fn upsert_folder(&self, account_id: &str, folder: &ImapFolder) -> anyhow::Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query(
            "INSERT INTO folders (id, account_id, name, display_name, kind, uidvalidity, uid_next, unread_count, total_count)
             VALUES (?, ?, ?, ?, ?, 0, 0, 0, 0)
             ON CONFLICT(account_id, name) DO UPDATE SET
                 display_name = excluded.display_name,
                 kind = excluded.kind",
        )
        .bind(&id)
        .bind(account_id)
        .bind(&folder.name)
        .bind(&folder.display_name)
        .bind(&folder.kind)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn get_folder_state(
        &self,
        account_id: &str,
        name: &str,
    ) -> anyhow::Result<Option<FolderState>> {
        let row = sqlx::query(
            "SELECT uidvalidity, uid_next FROM folders WHERE account_id = ? AND name = ?",
        )
        .bind(account_id)
        .bind(name)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|r| {
            let uidv: Option<i64> = r.get("uidvalidity");
            let uidn: Option<i64> = r.get("uid_next");
            FolderState {
                uidvalidity: uidv.unwrap_or(0) as u32,
                uid_next: uidn.unwrap_or(0) as u32,
            }
        }))
    }

    pub async fn clear_folder_messages(&self, account_id: &str, name: &str) -> anyhow::Result<()> {
        sqlx::query(
            "DELETE FROM messages WHERE account_id = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(account_id)
        .bind(account_id)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn max_uid(&self, account_id: &str, name: &str) -> anyhow::Result<Option<u32>> {
        let uid: Option<i64> = sqlx::query_scalar(
            "SELECT MAX(uid) FROM messages WHERE account_id = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(account_id)
        .bind(account_id)
        .bind(name)
        .fetch_one(&self.pool)
        .await?;

        Ok(uid.map(|u| u as u32))
    }

    pub async fn update_folder_state(
        &self,
        account_id: &str,
        name: &str,
        uidvalidity: u32,
        uid_next: u32,
    ) -> anyhow::Result<()> {
        sqlx::query(
            "UPDATE folders SET uidvalidity = ?, uid_next = ? WHERE account_id = ? AND name = ?",
        )
        .bind(uidvalidity as i64)
        .bind(uid_next as i64)
        .bind(account_id)
        .bind(name)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn upsert_message(
        &self,
        account_id: &str,
        folder: &str,
        message: &ParsedMessage,
    ) -> anyhow::Result<()> {
        let id = uuid::Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let to_json = serde_json::to_string(&message.to_addrs)?;
        let cc_json = serde_json::to_string(&message.cc_addrs)?;
        let flags_json = serde_json::to_string(&message.flags)?;

        let folder_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM folders WHERE account_id = ? AND name = ?")
                .bind(account_id)
                .bind(folder)
                .fetch_optional(&self.pool)
                .await?;

        let folder_id = match folder_id {
            Some(fid) => fid,
            None => {
                let fid = uuid::Uuid::new_v4().to_string();
                sqlx::query(
                    "INSERT INTO folders (id, account_id, name, display_name, kind)
                     VALUES (?, ?, ?, ?, 'custom')",
                )
                .bind(&fid)
                .bind(account_id)
                .bind(folder)
                .bind(folder)
                .execute(&self.pool)
                .await?;
                fid
            }
        };

        let result = sqlx::query(
            "INSERT INTO messages (id, account_id, folder_id, uid, message_id, thread_id,
                    from_addr, from_name, to_addrs, cc_addrs, subject, date, flags,
                    snippet, body_text, body_html, body_fetched, has_attachments, raw_size, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)
             ON CONFLICT(account_id, folder_id, uid) DO UPDATE SET
                 flags = excluded.flags,
                 message_id = excluded.message_id,
                 from_addr = excluded.from_addr,
                 from_name = excluded.from_name,
                 to_addrs = excluded.to_addrs,
                 cc_addrs = excluded.cc_addrs,
                 subject = excluded.subject,
                 date = excluded.date,
                 snippet = excluded.snippet,
                 body_text = CASE WHEN messages.body_fetched = 0 THEN excluded.body_text ELSE messages.body_text END,
                 body_html = CASE WHEN messages.body_fetched = 0 THEN excluded.body_html ELSE messages.body_html END,
                 body_fetched = excluded.body_fetched,
                 has_attachments = excluded.has_attachments,
                 raw_size = excluded.raw_size",
        )
        .bind(&id)
        .bind(account_id)
        .bind(&folder_id)
        .bind(message.uid as i64)
        .bind(&message.message_id)
        .bind(&message.thread_id)
        .bind(&message.from_addr)
        .bind(&message.from_name)
        .bind(&to_json)
        .bind(&cc_json)
        .bind(&message.subject)
        .bind(&message.date)
        .bind(&flags_json)
        .bind(&message.snippet)
        .bind(&message.body_text)
        .bind(&message.body_html)
        .bind(message.body_fetched as i64)
        .bind(message.has_attachments as i64)
        .bind(message.raw_size)
        .bind(&created_at)
        .execute(&self.pool)
        .await?;

        let rowid = result.last_insert_rowid();
        sqlx::query(
            "INSERT INTO messages_fts (rowid, subject, body_text, from_addr)
             VALUES (?, ?, ?, ?)
             ON CONFLICT(rowid) DO UPDATE SET
                 subject = excluded.subject,
                 body_text = excluded.body_text,
                 from_addr = excluded.from_addr",
        )
        .bind(rowid)
        .bind(&message.subject)
        .bind(&message.body_text)
        .bind(&message.from_addr)
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    pub async fn update_flags(
        &self,
        account_id: &str,
        folder: &str,
        uid: u32,
        flags: Vec<String>,
    ) -> anyhow::Result<()> {
        let flags_json = serde_json::to_string(&flags)?;
        sqlx::query(
            "UPDATE messages SET flags = ? WHERE account_id = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            ) AND uid = ?",
        )
        .bind(&flags_json)
        .bind(account_id)
        .bind(account_id)
        .bind(folder)
        .bind(uid as i64)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn rebuild_fts(&self) -> anyhow::Result<()> {
        sqlx::query("INSERT INTO messages_fts(messages_fts) VALUES('rebuild')")
            .execute(&self.pool)
            .await?;
        Ok(())
    }
}
