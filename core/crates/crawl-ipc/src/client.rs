//! IPC client runtime for crawl.
//! Provides CrawlClient for sending commands to the daemon.
//! For event subscriptions, use `crawl_ipc::subscription::EventSubscription`.

use anyhow::{Context, Result};
use serde_json::Value;
use std::path::PathBuf;
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::Duration;
use tokio::io::{AsyncBufReadExt, AsyncWriteExt, BufReader};
use tokio::net::UnixStream;
use tokio::time::timeout;

use crate::commands::CrawlCommand;
use crate::protocol::{CrawlRequest, CrawlResponse};

static REQUEST_ID: AtomicU64 = AtomicU64::new(1);

fn next_id() -> u64 {
    REQUEST_ID.fetch_add(1, Ordering::SeqCst)
}

pub struct CrawlClient {
    socket_path: PathBuf,
}

impl CrawlClient {
    pub fn new(socket_path: impl Into<PathBuf>) -> Self {
        Self {
            socket_path: socket_path.into(),
        }
    }

    #[allow(dead_code)]
    pub fn socket_path(&self) -> &PathBuf {
        &self.socket_path
    }

    pub async fn cmd(&self, method: &str, params: Value) -> Result<Value> {
        let id = next_id();
        let request = CrawlRequest::with_id(method, params, serde_json::json!(id));

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(&self.socket_path),
        )
        .await
        .with_context(|| {
            format!(
                "failed to connect to crawl daemon at {:?}\n\
             Is crawl-sysd running? Try: systemctl --user start crawl",
                self.socket_path
            )
        })??;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let req_str = serde_json::to_string(&request)?;
        writer.write_all(req_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut line))
            .await
            .with_context(|| "request timed out")??;

        let response: CrawlResponse =
            serde_json::from_str(&line).context("failed to parse JSON-RPC response")?;

        if let Some(error) = response.error {
            anyhow::bail!("daemon error: {}", error.message);
        }

        Ok(response.result.unwrap_or(serde_json::json!(null)))
    }

    pub async fn command(&self, command: CrawlCommand) -> Result<Value> {
        let id = next_id();
        let mut value = serde_json::to_value(command)?;
        if let serde_json::Value::Object(ref mut obj) = value {
            obj.insert(
                "jsonrpc".to_string(),
                serde_json::Value::String("2.0".to_string()),
            );
            obj.insert("id".to_string(), serde_json::json!(id));
        }

        let stream = timeout(
            Duration::from_secs(5),
            UnixStream::connect(&self.socket_path),
        )
        .await
        .with_context(|| {
            format!(
                "failed to connect to crawl daemon at {:?}\n\
             Is crawl-sysd running? Try: systemctl --user start crawl",
                self.socket_path
            )
        })??;

        let (reader, mut writer) = tokio::io::split(stream);
        let mut reader = BufReader::new(reader);

        let req_str = serde_json::to_string(&value)?;
        writer.write_all(req_str.as_bytes()).await?;
        writer.write_all(b"\n").await?;
        writer.flush().await?;

        let mut line = String::new();
        timeout(Duration::from_secs(5), reader.read_line(&mut line))
            .await
            .with_context(|| "request timed out")??;

        let response: CrawlResponse =
            serde_json::from_str(&line).context("failed to parse JSON-RPC response")?;

        if let Some(error) = response.error {
            anyhow::bail!("daemon error: {}", error.message);
        }

        Ok(response.result.unwrap_or(serde_json::json!(null)))
    }
}
