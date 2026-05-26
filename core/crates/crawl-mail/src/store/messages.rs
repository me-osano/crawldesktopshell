use crawl_ipc::types::{
    AttachmentInfo, GetMessage, ListMessages, MailFlag, MailSortOrder, MessageFull, MessageSummary,
    Search,
};
use sqlx::{Row, SqlitePool};
use uuid::Uuid;

pub struct MessagesStore<'a> {
    pool: &'a SqlitePool,
}

impl<'a> MessagesStore<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_messages(
        &self,
        params: &ListMessages,
    ) -> anyhow::Result<(Vec<MessageSummary>, u32)> {
        let total: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM messages WHERE account_id = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(&params.account_id)
        .bind(&params.account_id)
        .bind(&params.folder)
        .fetch_one(self.pool)
        .await
        .unwrap_or(0);

        let order_by = match params.sort {
            MailSortOrder::DateDesc => "date DESC",
            MailSortOrder::DateAsc => "date ASC",
            MailSortOrder::SenderAsc => "from_addr ASC",
            MailSortOrder::SubjectAsc => "subject ASC",
        };

        let query = format!(
            "SELECT uid, from_addr, subject, date, flags, has_attachments, snippet
             FROM messages WHERE account_id = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
             )
             ORDER BY {order_by} LIMIT ? OFFSET ?"
        );

        let rows = sqlx::query(&query)
            .bind(&params.account_id)
            .bind(&params.account_id)
            .bind(&params.folder)
            .bind(params.limit as i64)
            .bind(params.offset as i64)
            .fetch_all(self.pool)
            .await?;

        let messages = rows
            .into_iter()
            .map(|row| MessageSummary {
                uid: row.get::<i64, _>("uid") as u32,
                account_id: params.account_id.clone(),
                folder: params.folder.clone(),
                from: row.get("from_addr"),
                subject: row.get::<Option<String>, _>("subject").unwrap_or_default(),
                date: row.get("date"),
                flags: decode_flags(row.get("flags")),
                has_attachments: row.get::<i64, _>("has_attachments") != 0,
                snippet: row.get::<Option<String>, _>("snippet").unwrap_or_default(),
            })
            .collect();

        Ok((messages, total as u32))
    }

    pub async fn get_message(&self, params: &GetMessage) -> anyhow::Result<Option<MessageFull>> {
        let row = sqlx::query(
            "SELECT uid, message_id, from_addr, to_addrs, cc_addrs, subject, date, flags,
                    body_text, body_html, thread_id, folder_id
             FROM messages WHERE account_id = ? AND uid = ?",
        )
        .bind(&params.account_id)
        .bind(params.uid as i64)
        .fetch_optional(self.pool)
        .await?;

        let row = match row {
            Some(row) => row,
            None => return Ok(None),
        };

        let folder_name = sqlx::query_scalar("SELECT name FROM folders WHERE id = ?")
            .bind(row.get::<String, _>("folder_id"))
            .fetch_optional(self.pool)
            .await?
            .unwrap_or_default();

        Ok(Some(MessageFull {
            uid: row.get::<i64, _>("uid") as u32,
            account_id: params.account_id.clone(),
            folder: folder_name,
            message_id: row
                .get::<Option<String>, _>("message_id")
                .unwrap_or_default(),
            from: row.get("from_addr"),
            to: decode_json_list(row.get("to_addrs")),
            cc: decode_json_list(row.get::<Option<String>, _>("cc_addrs").unwrap_or_default()),
            subject: row.get::<Option<String>, _>("subject").unwrap_or_default(),
            date: row.get("date"),
            flags: decode_flags(row.get("flags")),
            body_text: row.get::<Option<String>, _>("body_text"),
            body_html: row.get::<Option<String>, _>("body_html"),
            attachments: Vec::<AttachmentInfo>::new(),
            thread_id: row.get::<Option<String>, _>("thread_id"),
        }))
    }

    pub async fn search_messages(&self, params: &Search) -> anyhow::Result<Vec<MessageSummary>> {
        let folder_filter = if params.folder.is_some() {
            "AND folder_id = (SELECT id FROM folders WHERE account_id = ? AND name = ?)"
        } else {
            ""
        };

        let sql = format!(
            "SELECT m.uid, m.from_addr, m.subject, m.date, m.flags, m.has_attachments, m.snippet
             FROM messages_fts f
             JOIN messages m ON m.rowid = f.rowid
             WHERE f MATCH ? AND m.account_id = ? {folder_filter}
             LIMIT ?"
        );

        let mut query = sqlx::query(&sql)
            .bind(&params.query)
            .bind(&params.account_id);

        if let Some(folder) = &params.folder {
            query = query.bind(&params.account_id).bind(folder);
        }

        let rows = query.bind(params.limit as i64).fetch_all(self.pool).await?;

        let messages = rows
            .into_iter()
            .map(|row| MessageSummary {
                uid: row.get::<i64, _>("uid") as u32,
                account_id: params.account_id.clone(),
                folder: params.folder.clone().unwrap_or_default(),
                from: row.get("from_addr"),
                subject: row.get::<Option<String>, _>("subject").unwrap_or_default(),
                date: row.get("date"),
                flags: decode_flags(row.get("flags")),
                has_attachments: row.get::<i64, _>("has_attachments") != 0,
                snippet: row.get::<Option<String>, _>("snippet").unwrap_or_default(),
            })
            .collect();

        Ok(messages)
    }

    pub async fn move_message(
        &self,
        account_id: &str,
        uid: u32,
        from_folder: &str,
        to_folder: &str,
    ) -> anyhow::Result<()> {
        let to_folder_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM folders WHERE account_id = ? AND name = ?")
                .bind(account_id)
                .bind(to_folder)
                .fetch_optional(self.pool)
                .await?;

        let to_folder_id = match to_folder_id {
            Some(id) => id,
            None => anyhow::bail!("Target folder '{}' not found", to_folder),
        };

        sqlx::query(
            "UPDATE messages SET folder_id = ? WHERE account_id = ? AND uid = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(&to_folder_id)
        .bind(account_id)
        .bind(uid as i64)
        .bind(account_id)
        .bind(from_folder)
        .execute(self.pool)
        .await?;

        Ok(())
    }

    pub async fn copy_message(
        &self,
        account_id: &str,
        uid: u32,
        to_folder: &str,
    ) -> anyhow::Result<()> {
        let to_folder_id: Option<String> =
            sqlx::query_scalar("SELECT id FROM folders WHERE account_id = ? AND name = ?")
                .bind(account_id)
                .bind(to_folder)
                .fetch_optional(self.pool)
                .await?;

        let target_folder_id = match to_folder_id {
            Some(id) => id,
            None => anyhow::bail!("Target folder '{}' not found", to_folder),
        };

        let source = sqlx::query(
            "SELECT uid, message_id, thread_id, from_addr, from_name, to_addrs, cc_addrs,
                    subject, date, flags, snippet, body_text, body_html, body_fetched,
                    has_attachments, raw_size
             FROM messages WHERE account_id = ? AND uid = ?",
        )
        .bind(account_id)
        .bind(uid as i64)
        .fetch_optional(self.pool)
        .await?;

        let row = match source {
            Some(r) => r,
            None => anyhow::bail!("Message uid={} not found", uid),
        };

        let new_id = Uuid::new_v4().to_string();
        let created_at = chrono::Utc::now().to_rfc3339();
        let flags: String = row.get("flags");

        let result = sqlx::query(
            "INSERT INTO messages (id, account_id, folder_id, uid, message_id, thread_id,
                    from_addr, from_name, to_addrs, cc_addrs, subject, date, flags,
                    snippet, body_text, body_html, body_fetched, has_attachments, raw_size, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&new_id)
        .bind(account_id)
        .bind(&target_folder_id)
        .bind(row.get::<i64, _>("uid"))
        .bind(row.get::<Option<String>, _>("message_id"))
        .bind(row.get::<Option<String>, _>("thread_id"))
        .bind(row.get::<String, _>("from_addr"))
        .bind(row.get::<Option<String>, _>("from_name"))
        .bind(row.get::<String, _>("to_addrs"))
        .bind(row.get::<Option<String>, _>("cc_addrs"))
        .bind(row.get::<Option<String>, _>("subject"))
        .bind(row.get::<String, _>("date"))
        .bind(&flags)
        .bind(row.get::<Option<String>, _>("snippet"))
        .bind(row.get::<Option<String>, _>("body_text"))
        .bind(row.get::<Option<String>, _>("body_html"))
        .bind(row.get::<i64, _>("body_fetched"))
        .bind(row.get::<i64, _>("has_attachments"))
        .bind(row.get::<Option<i64>, _>("raw_size"))
        .bind(&created_at)
        .execute(self.pool)
        .await?;

        let new_rowid = result.last_insert_rowid();
        let _ = sqlx::query(
            "INSERT INTO messages_fts (rowid, subject, body_text, from_addr) VALUES (?, ?, ?, ?)",
        )
        .bind(new_rowid)
        .bind(row.get::<Option<String>, _>("subject"))
        .bind(row.get::<Option<String>, _>("body_text"))
        .bind(row.get::<String, _>("from_addr"))
        .execute(self.pool)
        .await;

        Ok(())
    }

    pub async fn delete_message(
        &self,
        account_id: &str,
        folder: &str,
        uid: u32,
    ) -> anyhow::Result<()> {
        let result = sqlx::query(
            "DELETE FROM messages WHERE account_id = ? AND uid = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(account_id)
        .bind(uid as i64)
        .bind(account_id)
        .bind(folder)
        .execute(self.pool)
        .await?;

        if result.rows_affected() > 0 {
            let _ =
                sqlx::query("INSERT INTO messages_fts(messages_fts, rowid) VALUES('delete', ?)")
                    .bind(result.last_insert_rowid())
                    .execute(self.pool)
                    .await;
        }

        Ok(())
    }

    pub async fn set_flags(
        &self,
        account_id: &str,
        folder: &str,
        uid: u32,
        add: &[MailFlag],
        remove: &[MailFlag],
    ) -> anyhow::Result<Vec<MailFlag>> {
        let raw_flags: Option<String> = sqlx::query_scalar(
            "SELECT flags FROM messages WHERE account_id = ? AND uid = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(account_id)
        .bind(uid as i64)
        .bind(account_id)
        .bind(folder)
        .fetch_optional(self.pool)
        .await?;

        let mut flags: Vec<MailFlag> = match raw_flags {
            Some(f) => serde_json::from_str(&f).unwrap_or_default(),
            None => anyhow::bail!("Message uid={} not found in {}", uid, folder),
        };

        for flag in remove {
            flags.retain(|f| f != flag);
        }
        for flag in add {
            if !flags.contains(flag) {
                flags.push(flag.clone());
            }
        }

        let json = encode_flags(&flags);
        sqlx::query(
            "UPDATE messages SET flags = ? WHERE account_id = ? AND uid = ? AND folder_id = (
                SELECT id FROM folders WHERE account_id = ? AND name = ?
            )",
        )
        .bind(&json)
        .bind(account_id)
        .bind(uid as i64)
        .bind(account_id)
        .bind(folder)
        .execute(self.pool)
        .await?;

        Ok(flags)
    }

    pub async fn find_folder_for_message(
        &self,
        account_id: &str,
        uid: u32,
    ) -> anyhow::Result<Option<String>> {
        let folder_name: Option<String> = sqlx::query_scalar(
            "SELECT f.name FROM messages m
             JOIN folders f ON m.folder_id = f.id
             WHERE m.account_id = ? AND m.uid = ?",
        )
        .bind(account_id)
        .bind(uid as i64)
        .fetch_optional(self.pool)
        .await?;

        Ok(folder_name)
    }
}

fn decode_flags(raw: String) -> Vec<MailFlag> {
    serde_json::from_str(&raw).unwrap_or_default()
}

fn encode_flags(flags: &[MailFlag]) -> String {
    serde_json::to_string(flags).unwrap_or_else(|_| "[]".to_string())
}

fn decode_json_list(raw: String) -> Vec<String> {
    serde_json::from_str(&raw).unwrap_or_default()
}
