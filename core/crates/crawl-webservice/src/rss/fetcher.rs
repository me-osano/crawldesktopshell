use std::sync::Arc;

use anyhow::Result;
use feed_rs::parser;
use reqwest::header::{IF_MODIFIED_SINCE, IF_NONE_MATCH};
use tokio::sync::Semaphore;

use crate::rss::store::Store;

pub struct Fetcher {
    client: reqwest::Client,
    store: Arc<Store>,
    semaphore: Arc<Semaphore>,
}

impl Fetcher {
    pub fn new(store: Arc<Store>, max_concurrent: usize) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("CrawlDS/0.1 RSS Reader")
            .timeout(std::time::Duration::from_secs(30))
            .redirect(reqwest::redirect::Policy::limited(5))
            .gzip(true)
            .brotli(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            store,
            semaphore: Arc::new(Semaphore::new(max_concurrent)),
        }
    }

    pub async fn fetch_feed(&self, feed_id: &str, url: &str) -> Result<FetchResult> {
        let _permit = self.semaphore.acquire().await;

        // Conditional GET using cached ETag/Last-Modified
        let (etag, last_modified) = self.store.get_feed_http_cache(feed_id).await?;

        let mut request = self.client.get(url);
        if !etag.is_empty() {
            request = request.header(IF_NONE_MATCH, &etag);
        }
        if !last_modified.is_empty() {
            request = request.header(IF_MODIFIED_SINCE, &last_modified);
        }

        let response = request.send().await?;

        let status = response.status();

        // 304 Not Modified — feed unchanged since last fetch
        if status == reqwest::StatusCode::NOT_MODIFIED {
            self.store.mark_fetch_success(feed_id).await?;
            return Ok(FetchResult::NotModified);
        }

        if !status.is_success() {
            self.store
                .mark_fetch_error(feed_id, &format!("HTTP {status}"))
                .await?;
            return Ok(FetchResult::HttpError(status.as_u16()));
        }

        // Save new ETag/Last-Modified from response headers
        let new_etag = response
            .headers()
            .get(reqwest::header::ETAG)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        let new_lm = response
            .headers()
            .get(reqwest::header::LAST_MODIFIED)
            .and_then(|v| v.to_str().ok())
            .unwrap_or("")
            .to_string();
        if !new_etag.is_empty() || !new_lm.is_empty() {
            self.store
                .update_feed_http_cache(feed_id, &new_etag, &new_lm)
                .await?;
        }

        let bytes = response.bytes().await?;
        let feed = match parser::parse(&bytes[..]) {
            Ok(f) => f,
            Err(e) => {
                self.store
                    .mark_fetch_error(feed_id, &format!("Parse error: {e}"))
                    .await?;
                return Ok(FetchResult::ParseError(e.to_string()));
            }
        };

        // Update feed metadata
        let title = feed.title.map(|t| t.content).unwrap_or_default();
        let description = feed.description.map(|d| d.content).unwrap_or_default();
        let site_url = feed
            .links
            .iter()
            .find(|l| l.rel.as_deref() == Some("alternate") || l.rel.is_none())
            .and_then(|l| Some(l.href.clone()))
            .unwrap_or_default();
        let icon_url = feed
            .icon
            .clone()
            .map(|i| i.uri)
            .or_else(|| feed.logo.clone().map(|l| l.uri))
            .unwrap_or_default();

        self.store
            .update_feed_metadata(feed_id, &title, &description, &site_url, &icon_url)
            .await?;

        // Insert new entries
        let mut new_count = 0u32;
        for entry in &feed.entries {
            let guid = entry.id.clone();

            if self.store.entry_exists(feed_id, &guid).await? {
                continue;
            }

            let entry_url = entry
                .links
                .iter()
                .find(|l| l.rel.as_deref() != Some("enclosure"))
                .map(|l| l.href.clone());

            let author = entry.authors.first().map(|a| a.name.as_str()).unwrap_or("");

            let summary = entry
                .summary
                .as_ref()
                .map(|s| s.content.clone())
                .unwrap_or_default();

            let content = entry
                .content
                .as_ref()
                .and_then(|c| c.body.as_ref().cloned())
                .or_else(|| {
                    entry
                        .media
                        .first()
                        .and_then(|m| m.description.as_ref().map(|d| d.content.clone()))
                })
                .unwrap_or_default();

            let content_type = entry
                .content
                .as_ref()
                .map(|c| c.content_type.to_string())
                .unwrap_or_else(|| "html".to_string());

            let published = entry
                .published
                .or(entry.updated)
                .map(|dt| dt.to_rfc3339())
                .unwrap_or_default();

            let image_url = entry
                .media
                .iter()
                .find(|m| !m.thumbnails.is_empty())
                .and_then(|m| m.thumbnails.first())
                .map(|t| t.image.uri.clone())
                .or_else(|| {
                    entry
                        .media
                        .iter()
                        .find(|m| !m.content.is_empty())
                        .and_then(|m| m.content.first())
                        .map(|c| c.url.as_ref().map(|u| u.to_string()))
                        .flatten()
                })
                .unwrap_or_default();

            self.store
                .insert_entry(
                    feed_id,
                    &guid,
                    entry_url.as_deref(),
                    entry.title.as_ref().map(|t| t.content.as_str()),
                    Some(author),
                    if summary.is_empty() {
                        None
                    } else {
                        Some(&summary)
                    },
                    if content.is_empty() {
                        None
                    } else {
                        Some(&content)
                    },
                    &content_type,
                    if published.is_empty() {
                        None
                    } else {
                        Some(&published)
                    },
                    if image_url.is_empty() {
                        None
                    } else {
                        Some(&image_url)
                    },
                )
                .await?;

            new_count += 1;
        }

        self.store.mark_fetch_success(feed_id).await?;

        Ok(FetchResult::Success {
            title,
            new_entries: new_count,
        })
    }
}

#[derive(Debug)]
pub enum FetchResult {
    Success { title: String, new_entries: u32 },
    NotModified,
    HttpError(u16),
    ParseError(String),
}
