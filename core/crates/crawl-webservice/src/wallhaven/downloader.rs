use std::path::PathBuf;
use std::sync::Arc;

use anyhow::Result;
use futures_util::StreamExt;
use tokio::sync::{Semaphore, broadcast};
use tracing::{error, info};

use crawl_ipc::CrawlEvent;
use crawl_ipc::events::WallhavenEvent;

pub struct DownloadManager {
    client: reqwest::Client,
    event_tx: broadcast::Sender<CrawlEvent>,
    semaphore: Arc<Semaphore>,
}

impl DownloadManager {
    pub fn new(event_tx: broadcast::Sender<CrawlEvent>, max_parallel: usize) -> Self {
        let client = reqwest::Client::builder()
            .user_agent("CrawlDS/0.1 Wallhaven")
            .timeout(std::time::Duration::from_secs(120))
            .gzip(true)
            .build()
            .expect("Failed to build HTTP client");

        Self {
            client,
            event_tx,
            semaphore: Arc::new(Semaphore::new(max_parallel)),
        }
    }

    pub async fn download(
        &self,
        wallpaper_id: &str,
        url: &str,
        dest_dir: &str,
        filename: Option<&str>,
    ) -> Result<()> {
        let _permit = self.semaphore.acquire().await?;

        let dest = PathBuf::from(dest_dir);
        if !dest.exists() {
            tokio::fs::create_dir_all(&dest).await?;
        }

        let file_name = match filename {
            Some(name) => name.to_string(),
            None => format!("wallhaven_{wallpaper_id}.jpg"),
        };

        let local_path = dest.join(&file_name);
        let local_path_str = local_path.to_string_lossy().to_string();

        // Notify start
        let _ = self
            .event_tx
            .send(CrawlEvent::Wallhaven(WallhavenEvent::DownloadStarted {
                wallpaper_id: wallpaper_id.to_string(),
                local_path: local_path_str.clone(),
            }));

        match self.client.get(url).send().await {
            Ok(response) => {
                let total = response.content_length().unwrap_or(0);
                let mut downloaded = 0u64;

                let mut file = tokio::fs::File::create(&local_path).await?;
                let mut stream = response.bytes_stream();

                while let Some(chunk) = stream.next().await {
                    match chunk {
                        Ok(bytes) => {
                            use tokio::io::AsyncWriteExt;
                            file.write_all(&bytes).await?;
                            downloaded += bytes.len() as u64;

                            let _ = self.event_tx.send(CrawlEvent::Wallhaven(
                                WallhavenEvent::DownloadProgress {
                                    wallpaper_id: wallpaper_id.to_string(),
                                    bytes_downloaded: downloaded,
                                    total_bytes: total,
                                },
                            ));
                        }
                        Err(e) => {
                            let _ = self.event_tx.send(CrawlEvent::Wallhaven(
                                WallhavenEvent::DownloadFailed {
                                    wallpaper_id: wallpaper_id.to_string(),
                                    error: format!("Download stream error: {e}"),
                                },
                            ));
                            return Err(e.into());
                        }
                    }
                }

                info!("Wallhaven download complete: {local_path_str}");
                let _ =
                    self.event_tx
                        .send(CrawlEvent::Wallhaven(WallhavenEvent::DownloadComplete {
                            wallpaper_id: wallpaper_id.to_string(),
                            local_path: local_path_str,
                        }));
                Ok(())
            }
            Err(e) => {
                error!("Wallhaven download failed for {wallpaper_id}: {e}");
                let _ = self
                    .event_tx
                    .send(CrawlEvent::Wallhaven(WallhavenEvent::DownloadFailed {
                        wallpaper_id: wallpaper_id.to_string(),
                        error: e.to_string(),
                    }));
                Err(e.into())
            }
        }
    }
}
