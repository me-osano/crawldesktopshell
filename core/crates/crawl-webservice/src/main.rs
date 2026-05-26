use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Duration;

use tokio::sync::{Mutex, broadcast};
use tracing::{error, info};
use tracing_subscriber::EnvFilter;

use crawl_ipc::CrawlEvent;
use crawl_ipc::events::{RssEvent, WallhavenEvent};

mod config;
mod ipc;
mod rss;
mod wallhaven;

use rss::fetcher::Fetcher;
use rss::store::Store;
use wallhaven::client::WallhavenClient;
use wallhaven::downloader::DownloadManager;

struct SyncEngine {
    store: Arc<Store>,
    fetcher: Arc<Fetcher>,
    event_tx: broadcast::Sender<CrawlEvent>,
    fetch_tx: tokio::sync::mpsc::UnboundedSender<(String, String)>,
    rss_enabled: Arc<AtomicBool>,
}

impl SyncEngine {
    fn new(
        store: Arc<Store>,
        fetcher: Arc<Fetcher>,
        event_tx: broadcast::Sender<CrawlEvent>,
        fetch_tx: tokio::sync::mpsc::UnboundedSender<(String, String)>,
        rss_enabled: Arc<AtomicBool>,
    ) -> Self {
        Self {
            store,
            fetcher,
            event_tx,
            fetch_tx,
            rss_enabled,
        }
    }

    fn set_enabled(&self, enabled: bool) {
        self.rss_enabled.store(enabled, Ordering::Relaxed);
    }

    fn is_enabled(&self) -> bool {
        self.rss_enabled.load(Ordering::Relaxed)
    }

    async fn enqueue(&self, feed_id: &str, url: &str) {
        if !self.rss_enabled.load(Ordering::Relaxed) {
            return;
        }
        let _ = self.fetch_tx.send((feed_id.to_string(), url.to_string()));
    }

    async fn refresh_all(&self) {
        if !self.rss_enabled.load(Ordering::Relaxed) {
            return;
        }
        match self.store.get_all_feeds_for_fetch().await {
            Ok(feeds) => {
                for (feed_id, url) in feeds {
                    let _ = self.fetch_tx.send((feed_id, url));
                }
            }
            Err(e) => {
                error!("Failed to get feeds for refresh: {e}");
            }
        }
    }

    async fn run_polling_loop(
        store: Arc<Store>,
        fetcher: Arc<Fetcher>,
        event_tx: broadcast::Sender<CrawlEvent>,
        fetch_rx: tokio::sync::mpsc::UnboundedReceiver<(String, String)>,
        rss_enabled: Arc<AtomicBool>,
    ) {
        let polling_interval = Duration::from_secs(60);
        let mut timer = tokio::time::interval(polling_interval);
        tokio::pin!(fetch_rx);

        loop {
            tokio::select! {
                Some((feed_id, url)) = fetch_rx.recv() => {
                    if rss_enabled.load(Ordering::Relaxed) {
                        Self::fetch_one(&fetcher, &event_tx, &feed_id, &url).await;
                    }
                }
                _ = timer.tick() => {
                    if !rss_enabled.load(Ordering::Relaxed) {
                        continue;
                    }
                    if let Ok(feeds) = store.get_all_feeds_for_fetch().await {
                        for (feed_id, url) in feeds {
                            Self::fetch_one(&fetcher, &event_tx, &feed_id, &url).await;
                        }
                    }
                }
            }
        }
    }

    async fn fetch_one(
        fetcher: &Fetcher,
        event_tx: &broadcast::Sender<CrawlEvent>,
        feed_id: &str,
        url: &str,
    ) {
        let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncStarted {
            feed_id: Some(feed_id.to_string()),
        }));

        match fetcher.fetch_feed(feed_id, url).await {
            Ok(rss::fetcher::FetchResult::Success { new_entries, .. }) => {
                if new_entries > 0 {
                    let _ = event_tx.send(CrawlEvent::Rss(RssEvent::NewEntries {
                        feed_id: feed_id.to_string(),
                        count: new_entries,
                    }));
                }
                let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncComplete {
                    feed_id: Some(feed_id.to_string()),
                }));
            }
            Ok(rss::fetcher::FetchResult::NotModified) => {
                let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncComplete {
                    feed_id: Some(feed_id.to_string()),
                }));
            }
            Ok(rss::fetcher::FetchResult::HttpError(code)) => {
                let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncError {
                    feed_id: feed_id.to_string(),
                    error: format!("HTTP {code}"),
                }));
            }
            Ok(rss::fetcher::FetchResult::ParseError(e)) => {
                let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncError {
                    feed_id: feed_id.to_string(),
                    error: format!("Parse error: {e}"),
                }));
            }
            Err(e) => {
                tracing::warn!("Feed fetch failed: {e}");
                let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncError {
                    feed_id: feed_id.to_string(),
                    error: e.to_string(),
                }));
            }
        }
    }
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    info!("Starting crawl-webservice...");

    let cfg = config::load()?;

    // ── Runtime enabled flags ─────────────────────────────────────────────
    let rss_enabled = Arc::new(AtomicBool::new(cfg.rss.enabled));
    let wh_enabled = Arc::new(AtomicBool::new(cfg.wallhaven.enabled));

    // ── RSS Store ──────────────────────────────────────────────────────────
    let db = Store::open(&cfg.db_path).await?;
    Store::migrate(&db).await?;
    let store = Arc::new(Store::new(db));

    // ── Event bus ──────────────────────────────────────────────────────────
    let (push_tx, _) = broadcast::channel::<CrawlEvent>(256);

    // ── RSS Fetcher ────────────────────────────────────────────────────────
    let fetcher = Arc::new(Fetcher::new(
        Arc::clone(&store),
        cfg.rss.max_concurrent_fetches,
    ));

    // ── Sync engine ────────────────────────────────────────────────────────
    let (fetch_tx, fetch_rx) = tokio::sync::mpsc::unbounded_channel::<(String, String)>();
    let sync_engine = Arc::new(SyncEngine::new(
        Arc::clone(&store),
        Arc::clone(&fetcher),
        push_tx.clone(),
        fetch_tx,
        Arc::clone(&rss_enabled),
    ));

    // Spawn polling loop
    {
        let store = Arc::clone(&store);
        let fetcher = Arc::clone(&fetcher);
        let event_tx = push_tx.clone();
        tokio::spawn(async move {
            SyncEngine::run_polling_loop(store, fetcher, event_tx, fetch_rx, rss_enabled).await;
        });
    }

    // ── Wallhaven client ───────────────────────────────────────────────────
    let wh_api_key = std::env::var("CRAWLDS_WALLHAVEN_API_KEY")
        .unwrap_or_else(|_| cfg.wallhaven.api_key.clone());

    let wh_client = Arc::new(Mutex::new(WallhavenClient::new(
        wh_api_key.clone(),
        cfg.wallhaven.rate_per_min,
    )));

    // ── Wallhaven download manager ─────────────────────────────────────────
    let downloader = Arc::new(DownloadManager::new(
        push_tx.clone(),
        cfg.max_parallel_downloads,
    ));

    // ── IPC server ─────────────────────────────────────────────────────────
    let socket_path = cfg.socket_path.clone();
    info!("Starting IPC server on {}", socket_path.display());

    ipc::serve(
        socket_path,
        store,
        wh_client,
        downloader,
        sync_engine,
        push_tx,
        wh_enabled,
    )
    .await?;

    Ok(())
}
