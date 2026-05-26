use chrono::Utc;
use crawl_ipc::types::SendMessage;
use sqlx::{Row, SqlitePool};

use super::accounts::AccountSmtp;

pub struct OutboxStore<'a> {
    pool: &'a SqlitePool,
}

#[derive(Debug, Clone)]
pub struct QueuedMessage {
    pub id: String,
    pub payload: SendMessage,
    pub smtp: AccountSmtp,
}

impl<'a> OutboxStore<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn enqueue(&self, payload: &SendMessage) -> anyhow::Result<String> {
        let outbox_id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();
        let serialized = serde_json::to_string(payload)?;

        sqlx::query(
            "INSERT INTO outbox (id, account_id, payload, status, attempts, created_at)
             VALUES (?, ?, ?, 'queued', 0, ?)",
        )
        .bind(&outbox_id)
        .bind(&payload.account_id)
        .bind(serialized)
        .bind(created_at)
        .execute(self.pool)
        .await?;

        Ok(outbox_id)
    }

    pub async fn list_queued(&self, limit: u32) -> anyhow::Result<Vec<QueuedMessage>> {
        let rows = sqlx::query(
            "SELECT o.id, o.payload, a.smtp_host, a.smtp_port, a.username, a.password
             FROM outbox o
             JOIN accounts a ON a.id = o.account_id
             WHERE o.status = 'queued'
             ORDER BY o.created_at ASC
             LIMIT ?",
        )
        .bind(limit as i64)
        .fetch_all(self.pool)
        .await?;

        let messages = rows
            .into_iter()
            .filter_map(|row| {
                let payload_str: String = row.get("payload");
                let payload: SendMessage = serde_json::from_str(&payload_str).ok()?;
                Some(QueuedMessage {
                    id: row.get("id"),
                    payload,
                    smtp: AccountSmtp {
                        host: row.get("smtp_host"),
                        port: row.get::<i64, _>("smtp_port") as u16,
                        username: row.get("username"),
                        password: row.get("password"),
                    },
                })
            })
            .collect();

        Ok(messages)
    }

    pub async fn mark_status(
        &self,
        id: &str,
        status: &str,
        error: Option<&str>,
    ) -> anyhow::Result<()> {
        let now = Utc::now().to_rfc3339();
        match status {
            "sent" => {
                sqlx::query(
                    "UPDATE outbox SET status = ?, sent_at = ?, attempts = attempts + 1 WHERE id = ?",
                )
                .bind(status)
                .bind(&now)
                .bind(id)
                .execute(self.pool)
                .await?;
            }
            _ => {
                sqlx::query(
                    "UPDATE outbox SET status = ?, last_error = ?, attempts = attempts + 1 WHERE id = ?",
                )
                .bind(status)
                .bind(error)
                .bind(id)
                .execute(self.pool)
                .await?;
            }
        }
        Ok(())
    }
}
