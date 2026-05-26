use crawl_ipc::types::{FolderInfo, FolderKind};
use sqlx::{Row, SqlitePool};

pub struct FoldersStore<'a> {
    pool: &'a SqlitePool,
}

impl<'a> FoldersStore<'a> {
    pub fn new(pool: &'a SqlitePool) -> Self {
        Self { pool }
    }

    pub async fn list_folders(&self, account_id: &str) -> anyhow::Result<Vec<FolderInfo>> {
        let rows = sqlx::query(
            "SELECT name, display_name, unread_count, total_count, kind
             FROM folders WHERE account_id = ? ORDER BY name",
        )
        .bind(account_id)
        .fetch_all(self.pool)
        .await?;

        let folders = rows
            .into_iter()
            .map(|row| FolderInfo {
                name: row.get("name"),
                display_name: row.get("display_name"),
                unread: row.get::<i64, _>("unread_count") as u32,
                total: row.get::<i64, _>("total_count") as u32,
                kind: decode_folder_kind(row.get("kind")),
            })
            .collect();

        Ok(folders)
    }
}

fn decode_folder_kind(raw: String) -> FolderKind {
    match raw.as_str() {
        "inbox" => FolderKind::Inbox,
        "sent" => FolderKind::Sent,
        "drafts" => FolderKind::Drafts,
        "trash" => FolderKind::Trash,
        "spam" => FolderKind::Spam,
        "archive" => FolderKind::Archive,
        _ => FolderKind::Custom,
    }
}
