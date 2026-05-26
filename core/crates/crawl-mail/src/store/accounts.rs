use chrono::Utc;
use crawl_ipc::types::{AccountInfo, AddAccount};
use sqlx::{Row, SqlitePool};

pub struct AccountsStore<'a> {
    pool: &'a SqlitePool,
}

#[derive(Debug, Clone)]
pub struct AccountSmtp {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

#[derive(Debug, Clone)]
pub struct AccountImap {
    pub host: String,
    pub port: u16,
    pub username: String,
    pub password: String,
}

impl<'a> AccountsStore<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_accounts(&self) -> anyhow::Result<Vec<AccountInfo>> {
        let rows =
            sqlx::query("SELECT id, display_name, email FROM accounts ORDER BY created_at DESC")
                .fetch_all(self.pool)
                .await?;

        let accounts = rows
            .into_iter()
            .map(|row| AccountInfo {
                id: row.get("id"),
                display_name: row.get("display_name"),
                email: row.get("email"),
                unread_count: 0,
            })
            .collect();

        Ok(accounts)
    }

    pub async fn add_account(&self, payload: &AddAccount) -> anyhow::Result<String> {
        let account_id = uuid::Uuid::new_v4().to_string();
        let created_at = Utc::now().to_rfc3339();

        sqlx::query(
            "INSERT INTO accounts (id, display_name, email, imap_host, imap_port, smtp_host, smtp_port, username, password, created_at)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&account_id)
        .bind(&payload.display_name)
        .bind(&payload.email)
        .bind(&payload.imap_host)
        .bind(payload.imap_port as i64)
        .bind(&payload.smtp_host)
        .bind(payload.smtp_port as i64)
        .bind(&payload.username)
        .bind(&payload.password)
        .bind(created_at)
        .execute(self.pool)
        .await?;

        Ok(account_id)
    }

    pub async fn get_smtp(&self, account_id: &str) -> anyhow::Result<Option<AccountSmtp>> {
        let row = sqlx::query_as::<_, (String, i64, String, String)>(
            "SELECT smtp_host, smtp_port, username, password FROM accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|(host, port, username, password)| AccountSmtp {
            host,
            port: port as u16,
            username,
            password,
        }))
    }

    pub async fn delete_account(&self, account_id: &str) -> anyhow::Result<()> {
        let mut tx = self.pool.begin().await?;
        sqlx::query("DELETE FROM outbox WHERE account_id = ?")
            .bind(account_id)
            .execute(&mut *tx)
            .await?;
        sqlx::query("DELETE FROM accounts WHERE id = ?")
            .bind(account_id)
            .execute(&mut *tx)
            .await?;
        tx.commit().await?;
        Ok(())
    }

    pub async fn get_imap(&self, account_id: &str) -> anyhow::Result<Option<AccountImap>> {
        let row = sqlx::query_as::<_, (String, i64, String, String)>(
            "SELECT imap_host, imap_port, username, password FROM accounts WHERE id = ?",
        )
        .bind(account_id)
        .fetch_optional(self.pool)
        .await?;

        Ok(row.map(|(host, port, username, password)| AccountImap {
            host,
            port: port as u16,
            username,
            password,
        }))
    }
}
