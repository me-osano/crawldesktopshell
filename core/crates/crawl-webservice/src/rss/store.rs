use anyhow::Result;
use sqlx::{Row, SqlitePool};

use crawl_ipc::types::{EntryFull, EntryInfo, FeedInfo, RssListEntriesParams, RssSortOrder};

const SCHEMA: &str = include_str!("schema.sql");

pub struct Store {
    pool: SqlitePool,
}

impl Store {
    pub fn new(pool: SqlitePool) -> Self {
        Self { pool }
    }

    pub fn pool(&self) -> &SqlitePool {
        &self.pool
    }

    pub async fn open(path: &std::path::Path) -> Result<SqlitePool> {
        if let Some(parent) = path.parent() {
            if !parent.exists() {
                tokio::fs::create_dir_all(parent).await?;
            }
        }
        let url = format!("sqlite://{}", path.display());
        Ok(SqlitePool::connect(&url).await?)
    }

    pub async fn migrate(pool: &SqlitePool) -> Result<()> {
        for statement in SCHEMA.split(';') {
            let stmt = statement.trim();
            if stmt.is_empty() {
                continue;
            }
            sqlx::query(stmt).execute(pool).await?;
        }
        Ok(())
    }

    // ── Feeds ──────────────────────────────────────────────────────────────

    pub async fn list_feeds(&self) -> Result<Vec<FeedInfo>> {
        let rows = sqlx::query(
            "SELECT f.id, f.url, f.title, f.description, f.site_url, f.icon_url,
                    f.category, f.error_count, f.last_error, f.last_fetched,
                    COALESCE((SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id AND e.is_read = 0), 0) as unread_count
             FROM feeds f
             ORDER BY f.title ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| FeedInfo {
                id: r.get("id"),
                url: r.get("url"),
                title: r.get::<Option<String>, _>("title").unwrap_or_default(),
                description: r
                    .get::<Option<String>, _>("description")
                    .unwrap_or_default(),
                site_url: r.get::<Option<String>, _>("site_url").unwrap_or_default(),
                icon_url: r.get::<Option<String>, _>("icon_url").unwrap_or_default(),
                category: r.get::<Option<String>, _>("category").unwrap_or_default(),
                unread_count: r.get::<Option<i64>, _>("unread_count").unwrap_or(0) as u32,
                error_count: r.get::<Option<i64>, _>("error_count").unwrap_or(0) as u32,
                last_error: r.get::<Option<String>, _>("last_error").unwrap_or_default(),
                last_fetched: r
                    .get::<Option<String>, _>("last_fetched")
                    .unwrap_or_default(),
            })
            .collect())
    }

    pub async fn get_feed(&self, feed_id: &str) -> Result<Option<FeedInfo>> {
        let r = sqlx::query(
            "SELECT f.id, f.url, f.title, f.description, f.site_url, f.icon_url,
                    f.category, f.error_count, f.last_error, f.last_fetched,
                    COALESCE((SELECT COUNT(*) FROM entries e WHERE e.feed_id = f.id AND e.is_read = 0), 0) as unread_count
             FROM feeds f WHERE f.id = ?",
        )
        .bind(feed_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| FeedInfo {
            id: r.get("id"),
            url: r.get("url"),
            title: r.get::<Option<String>, _>("title").unwrap_or_default(),
            description: r
                .get::<Option<String>, _>("description")
                .unwrap_or_default(),
            site_url: r.get::<Option<String>, _>("site_url").unwrap_or_default(),
            icon_url: r.get::<Option<String>, _>("icon_url").unwrap_or_default(),
            category: r.get::<Option<String>, _>("category").unwrap_or_default(),
            unread_count: r.get::<Option<i64>, _>("unread_count").unwrap_or(0) as u32,
            error_count: r.get::<Option<i64>, _>("error_count").unwrap_or(0) as u32,
            last_error: r.get::<Option<String>, _>("last_error").unwrap_or_default(),
            last_fetched: r
                .get::<Option<String>, _>("last_fetched")
                .unwrap_or_default(),
        }))
    }

    pub async fn add_feed(&self, url: &str, category: &str) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        sqlx::query("INSERT INTO feeds (id, url, category) VALUES (?, ?, ?)")
            .bind(&id)
            .bind(url)
            .bind(category)
            .execute(&self.pool)
            .await?;
        Ok(id)
    }

    pub async fn remove_feed(&self, feed_id: &str) -> Result<bool> {
        let result = sqlx::query("DELETE FROM feeds WHERE id = ?")
            .bind(feed_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn update_feed(&self, feed_id: &str, category: Option<&str>) -> Result<bool> {
        if let Some(cat) = category {
            let result = sqlx::query("UPDATE feeds SET category = ? WHERE id = ?")
                .bind(cat)
                .bind(feed_id)
                .execute(&self.pool)
                .await?;
            Ok(result.rows_affected() > 0)
        } else {
            Ok(true)
        }
    }

    pub async fn feed_exists(&self, url: &str) -> Result<bool> {
        let r: Option<i64> = sqlx::query_scalar("SELECT 1 FROM feeds WHERE url = ? LIMIT 1")
            .bind(url)
            .fetch_optional(&self.pool)
            .await?;
        Ok(r.is_some())
    }

    pub async fn get_feed_id_by_url(&self, url: &str) -> Result<Option<String>> {
        sqlx::query_scalar("SELECT id FROM feeds WHERE url = ?")
            .bind(url)
            .fetch_optional(&self.pool)
            .await
            .map_err(Into::into)
    }

    pub async fn get_all_feeds_for_fetch(&self) -> Result<Vec<(String, String)>> {
        let rows = sqlx::query("SELECT id, url FROM feeds ORDER BY last_fetched ASC")
            .fetch_all(&self.pool)
            .await?;

        Ok(rows
            .iter()
            .map(|r| (r.get::<String, _>("id"), r.get::<String, _>("url")))
            .collect())
    }

    pub async fn get_feed_http_cache(&self, feed_id: &str) -> Result<(String, String)> {
        let row = sqlx::query("SELECT etag, last_modified FROM feeds WHERE id = ?")
            .bind(feed_id)
            .fetch_optional(&self.pool)
            .await?;
        match row {
            Some(r) => Ok((
                r.get::<Option<String>, _>("etag").unwrap_or_default(),
                r.get::<Option<String>, _>("last_modified")
                    .unwrap_or_default(),
            )),
            None => Ok((String::new(), String::new())),
        }
    }

    pub async fn update_feed_http_cache(
        &self,
        feed_id: &str,
        etag: &str,
        last_modified: &str,
    ) -> Result<()> {
        sqlx::query("UPDATE feeds SET etag = ?, last_modified = ? WHERE id = ?")
            .bind(etag)
            .bind(last_modified)
            .bind(feed_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn update_feed_metadata(
        &self,
        feed_id: &str,
        title: &str,
        description: &str,
        site_url: &str,
        icon_url: &str,
    ) -> Result<()> {
        sqlx::query(
            "UPDATE feeds SET title = ?, description = ?, site_url = ?, icon_url = ? WHERE id = ?",
        )
        .bind(title)
        .bind(description)
        .bind(site_url)
        .bind(icon_url)
        .bind(feed_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_fetch_success(&self, feed_id: &str) -> Result<()> {
        sqlx::query(
            "UPDATE feeds SET error_count = 0, last_error = '', last_fetched = datetime('now') WHERE id = ?",
        )
        .bind(feed_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    pub async fn mark_fetch_error(&self, feed_id: &str, error: &str) -> Result<()> {
        sqlx::query(
            "UPDATE feeds SET error_count = error_count + 1, last_error = ?, last_fetched = datetime('now') WHERE id = ?",
        )
        .bind(error)
        .bind(feed_id)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    // ── Entries ────────────────────────────────────────────────────────────

    pub async fn entry_exists(&self, feed_id: &str, guid: &str) -> Result<bool> {
        let r: Option<i64> =
            sqlx::query_scalar("SELECT 1 FROM entries WHERE feed_id = ? AND guid = ? LIMIT 1")
                .bind(feed_id)
                .bind(guid)
                .fetch_optional(&self.pool)
                .await?;
        Ok(r.is_some())
    }

    pub async fn insert_entry(
        &self,
        feed_id: &str,
        guid: &str,
        url: Option<&str>,
        title: Option<&str>,
        author: Option<&str>,
        summary: Option<&str>,
        content: Option<&str>,
        content_type: &str,
        published: Option<&str>,
        image_url: Option<&str>,
    ) -> Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let now = chrono::Utc::now().to_rfc3339();
        sqlx::query(
            "INSERT OR IGNORE INTO entries (id, feed_id, guid, url, title, author, summary, content, content_type, published, fetched_at, image_url)
             VALUES (?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?, ?)",
        )
        .bind(&id)
        .bind(feed_id)
        .bind(guid)
        .bind(url)
        .bind(title)
        .bind(author)
        .bind(summary)
        .bind(content)
        .bind(content_type)
        .bind(published.unwrap_or(""))
        .bind(&now)
        .bind(image_url)
        .execute(&self.pool)
        .await?;
        Ok(id)
    }

    pub async fn list_entries(
        &self,
        params: &RssListEntriesParams,
    ) -> Result<(Vec<EntryInfo>, u32)> {
        let mut conditions = vec!["1=1".to_string()];
        let mut bind_idx = 0u32;

        struct QueryBuilder {
            sql: String,
            params: Vec<String>,
        }

        if let Some(ref feed_id) = params.feed_id {
            if !feed_id.is_empty() {
                conditions.push(format!("e.feed_id = ?{}", bind_idx));
                bind_idx += 1;
            }
        }

        if let Some(ref category) = params.category {
            if !category.is_empty() {
                conditions.push(format!("f.category = ?{}", bind_idx));
                bind_idx += 1;
            }
        }

        if params.only_unread {
            conditions.push("e.is_read = 0".to_string());
        }

        if params.only_starred {
            conditions.push("e.is_starred = 1".to_string());
        }

        let where_clause = conditions.join(" AND ");

        let order = match params.sort {
            RssSortOrder::NewestFirst => "e.published DESC, e.fetched_at DESC",
            RssSortOrder::OldestFirst => "e.published ASC, e.fetched_at ASC",
        };

        let count_sql = format!(
            "SELECT COUNT(*) FROM entries e JOIN feeds f ON e.feed_id = f.id WHERE {where_clause}"
        );

        let mut count_query = sqlx::query_scalar::<_, i64>(&count_sql);
        if let Some(ref feed_id) = params.feed_id {
            if !feed_id.is_empty() {
                count_query = count_query.bind(feed_id);
            }
        }
        if let Some(ref category) = params.category {
            if !category.is_empty() {
                count_query = count_query.bind(category);
            }
        }
        let total: i64 = count_query.fetch_one(&self.pool).await?;

        let query_sql = format!(
            "SELECT e.id, e.feed_id, COALESCE(f.title, '') as feed_title, e.title, e.author,
                    e.summary, e.published, e.is_read, e.is_starred, e.image_url
             FROM entries e JOIN feeds f ON e.feed_id = f.id
             WHERE {where_clause}
             ORDER BY {order}
             LIMIT ?{bind_idx} OFFSET ?{b}",
            b = bind_idx + 1
        );

        let mut query = sqlx::query(&query_sql);
        if let Some(ref feed_id) = params.feed_id {
            if !feed_id.is_empty() {
                query = query.bind(feed_id);
            }
        }
        if let Some(ref category) = params.category {
            if !category.is_empty() {
                query = query.bind(category);
            }
        }
        query = query.bind(params.limit as i64).bind(params.offset as i64);

        let rows = query.fetch_all(&self.pool).await?;

        let entries = rows
            .iter()
            .map(|r| EntryInfo {
                id: r.get("id"),
                feed_id: r.get("feed_id"),
                feed_title: r.get::<Option<String>, _>("feed_title").unwrap_or_default(),
                title: r.get::<Option<String>, _>("title").unwrap_or_default(),
                author: r.get::<Option<String>, _>("author").unwrap_or_default(),
                summary: r.get::<Option<String>, _>("summary").unwrap_or_default(),
                published: r.get::<Option<String>, _>("published").unwrap_or_default(),
                is_read: r.get::<Option<i64>, _>("is_read").unwrap_or(0) != 0,
                is_starred: r.get::<Option<i64>, _>("is_starred").unwrap_or(0) != 0,
                image_url: r.get::<Option<String>, _>("image_url").unwrap_or_default(),
            })
            .collect();

        Ok((entries, total as u32))
    }

    pub async fn get_entry(&self, entry_id: &str) -> Result<Option<EntryFull>> {
        let r = sqlx::query(
            "SELECT e.id, e.feed_id, COALESCE(f.title, '') as feed_title, e.url, e.title,
                    e.author, e.summary, e.content, e.content_type, e.published,
                    e.is_read, e.is_starred, e.image_url
             FROM entries e JOIN feeds f ON e.feed_id = f.id
             WHERE e.id = ?",
        )
        .bind(entry_id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(r.map(|r| EntryFull {
            id: r.get("id"),
            feed_id: r.get("feed_id"),
            feed_title: r.get::<Option<String>, _>("feed_title").unwrap_or_default(),
            url: r.get::<Option<String>, _>("url").unwrap_or_default(),
            title: r.get::<Option<String>, _>("title").unwrap_or_default(),
            author: r.get::<Option<String>, _>("author").unwrap_or_default(),
            summary: r.get::<Option<String>, _>("summary").unwrap_or_default(),
            content: r.get::<Option<String>, _>("content").unwrap_or_default(),
            content_type: r
                .get::<Option<String>, _>("content_type")
                .unwrap_or_default(),
            published: r.get::<Option<String>, _>("published").unwrap_or_default(),
            is_read: r.get::<Option<i64>, _>("is_read").unwrap_or(0) != 0,
            is_starred: r.get::<Option<i64>, _>("is_starred").unwrap_or(0) != 0,
            image_url: r.get::<Option<String>, _>("image_url").unwrap_or_default(),
        }))
    }

    pub async fn set_entry_read(&self, entry_id: &str, is_read: bool) -> Result<bool> {
        let result = sqlx::query("UPDATE entries SET is_read = ? WHERE id = ?")
            .bind(if is_read { 1 } else { 0 })
            .bind(entry_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn set_entry_starred(&self, entry_id: &str, is_starred: bool) -> Result<bool> {
        let result = sqlx::query("UPDATE entries SET is_starred = ? WHERE id = ?")
            .bind(if is_starred { 1 } else { 0 })
            .bind(entry_id)
            .execute(&self.pool)
            .await?;
        Ok(result.rows_affected() > 0)
    }

    pub async fn mark_all_read(&self, feed_id: &str) -> Result<()> {
        sqlx::query("UPDATE entries SET is_read = 1 WHERE feed_id = ? AND is_read = 0")
            .bind(feed_id)
            .execute(&self.pool)
            .await?;
        Ok(())
    }

    pub async fn count_new_entries(&self, feed_id: &str, since_threshold: &str) -> Result<u32> {
        let count: Option<i64> =
            sqlx::query_scalar("SELECT COUNT(*) FROM entries WHERE feed_id = ? AND fetched_at > ?")
                .bind(feed_id)
                .bind(since_threshold)
                .fetch_optional(&self.pool)
                .await?;
        Ok(count.unwrap_or(0) as u32)
    }

    // ── Categories ─────────────────────────────────────────────────────────

    pub async fn list_categories(&self) -> Result<Vec<String>> {
        let rows: Vec<String> = sqlx::query_scalar(
            "SELECT DISTINCT category FROM feeds WHERE category != '' ORDER BY category ASC",
        )
        .fetch_all(&self.pool)
        .await?;
        Ok(rows)
    }

    // ── OPML ───────────────────────────────────────────────────────────────

    pub async fn get_all_feeds_opml(&self) -> Result<Vec<FeedInfo>> {
        let rows = sqlx::query(
            "SELECT f.id, f.url, f.title, f.description, f.site_url, f.icon_url,
                    f.category, f.error_count, f.last_error, f.last_fetched, 0 as unread_count
             FROM feeds f ORDER BY f.category ASC, f.title ASC",
        )
        .fetch_all(&self.pool)
        .await?;

        Ok(rows
            .iter()
            .map(|r| FeedInfo {
                id: r.get("id"),
                url: r.get("url"),
                title: r.get::<Option<String>, _>("title").unwrap_or_default(),
                description: r
                    .get::<Option<String>, _>("description")
                    .unwrap_or_default(),
                site_url: r.get::<Option<String>, _>("site_url").unwrap_or_default(),
                icon_url: r.get::<Option<String>, _>("icon_url").unwrap_or_default(),
                category: r.get::<Option<String>, _>("category").unwrap_or_default(),
                unread_count: 0,
                error_count: r.get::<Option<i64>, _>("error_count").unwrap_or(0) as u32,
                last_error: r.get::<Option<String>, _>("last_error").unwrap_or_default(),
                last_fetched: r
                    .get::<Option<String>, _>("last_fetched")
                    .unwrap_or_default(),
            })
            .collect())
    }
}
