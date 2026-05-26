use std::path::Path;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};

use tokio::sync::{Mutex, broadcast};
use tracing::warn;

use crawl_ipc::commands::CrawlCommand;
use crawl_ipc::events::{RssEvent, WallhavenEvent};
use crawl_ipc::protocol::error_code;
use crawl_ipc::types::{RssErrorCode, RssResponse, WallhavenResponse};
use crawl_ipc::{CrawlEvent, CrawlResponse, IpcServer, RequestDispatcher};

use crate::SyncEngine;
use crate::rss::opml::{OpmlImporter, generate_opml};
use crate::rss::store::Store;
use crate::wallhaven::client::{WallhavenClient, WhApiResponse};
use crate::wallhaven::downloader::DownloadManager;

pub async fn serve(
    path: impl AsRef<Path>,
    store: Arc<Store>,
    wh_client: Arc<Mutex<WallhavenClient>>,
    downloader: Arc<DownloadManager>,
    sync_engine: Arc<SyncEngine>,
    push_tx: broadcast::Sender<CrawlEvent>,
    wh_enabled: Arc<AtomicBool>,
) -> anyhow::Result<()> {
    let socket_path = path.as_ref().to_path_buf();
    let mut server = IpcServer::new(socket_path, push_tx.clone());

    let dispatcher: RequestDispatcher = Arc::new(move |method, params, id| {
        let store = Arc::clone(&store);
        let wh_client = Arc::clone(&wh_client);
        let downloader = Arc::clone(&downloader);
        let sync_engine = Arc::clone(&sync_engine);
        let event_tx = push_tx.clone();
        let wh_enabled = Arc::clone(&wh_enabled);

        Box::pin(async move {
            let mut command_value = serde_json::json!({"method": method.clone()});
            let include_params = !params.is_null()
                && !params
                    .as_object()
                    .map(|obj| obj.is_empty())
                    .unwrap_or(false);
            if include_params {
                if let serde_json::Value::Object(ref mut obj) = command_value {
                    obj.insert("params".to_string(), params.clone());
                }
            }

            let command: CrawlCommand = match serde_json::from_value(command_value) {
                Ok(cmd) => cmd,
                Err(_) => {
                    return CrawlResponse::error(
                        id,
                        error_code::INVALID_PARAMS,
                        "Invalid webservice command",
                    );
                }
            };

            match command {
                // ── RSS Commands ──────────────────────────────────────────────
                CrawlCommand::RssListFeeds => match store.list_feeds().await {
                    Ok(feeds) => ok(
                        id,
                        serde_json::to_value(RssResponse::FeedList { feeds }).unwrap_or_default(),
                    ),
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                CrawlCommand::RssAddFeed { url, category } => {
                    let cat = category.unwrap_or_default();
                    if url.trim().is_empty() {
                        return err_response(id, RssErrorCode::InvalidUrl, "URL is empty");
                    }
                    match store.feed_exists(&url).await {
                        Ok(true) => {
                            err_response(id, RssErrorCode::Duplicate, "Feed already exists")
                        }
                        Ok(false) => {
                            match store.add_feed(&url, &cat).await {
                                Ok(feed_id) => {
                                    let _ = event_tx.send(CrawlEvent::Rss(RssEvent::FeedAdded {
                                        feed_id: feed_id.clone(),
                                        title: String::new(),
                                        category: cat.clone(),
                                    }));
                                    // Trigger immediate fetch
                                    sync_engine.enqueue(&feed_id, &url).await;
                                    ok(
                                        id,
                                        serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                                    )
                                }
                                Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                            }
                        }
                        Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                    }
                }

                CrawlCommand::RssRemoveFeed { feed_id } => {
                    match store.remove_feed(&feed_id).await {
                        Ok(true) => {
                            let _ = event_tx.send(CrawlEvent::Rss(RssEvent::FeedRemoved {
                                feed_id: feed_id.clone(),
                            }));
                            ok(
                                id,
                                serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                            )
                        }
                        Ok(false) => err_response(id, RssErrorCode::NotFound, "Feed not found"),
                        Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                    }
                }

                CrawlCommand::RssUpdateFeed { feed_id, category } => {
                    match store.update_feed(&feed_id, category.as_deref()).await {
                        Ok(true) => ok(
                            id,
                            serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                        ),
                        Ok(false) => err_response(id, RssErrorCode::NotFound, "Feed not found"),
                        Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                    }
                }

                CrawlCommand::RssListEntries(params) => match store.list_entries(&params).await {
                    Ok((entries, total)) => ok(
                        id,
                        serde_json::to_value(RssResponse::EntryList { entries, total })
                            .unwrap_or_default(),
                    ),
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                CrawlCommand::RssGetEntry { entry_id } => match store.get_entry(&entry_id).await {
                    Ok(Some(entry)) => ok(
                        id,
                        serde_json::to_value(RssResponse::Entry { entry }).unwrap_or_default(),
                    ),
                    Ok(None) => err_response(id, RssErrorCode::NotFound, "Entry not found"),
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                CrawlCommand::RssSetEntryRead { entry_id, is_read } => {
                    match store.set_entry_read(&entry_id, is_read).await {
                        Ok(true) => ok(
                            id,
                            serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                        ),
                        Ok(false) => err_response(id, RssErrorCode::NotFound, "Entry not found"),
                        Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                    }
                }

                CrawlCommand::RssSetEntryStarred {
                    entry_id,
                    is_starred,
                } => match store.set_entry_starred(&entry_id, is_starred).await {
                    Ok(true) => ok(
                        id,
                        serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                    ),
                    Ok(false) => err_response(id, RssErrorCode::NotFound, "Entry not found"),
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                CrawlCommand::RssMarkAllRead { feed_id } => {
                    match store.mark_all_read(&feed_id).await {
                        Ok(()) => ok(
                            id,
                            serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                        ),
                        Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                    }
                }

                CrawlCommand::RssRefreshFeed { feed_id } => {
                    let url = match store.get_feed(&feed_id).await {
                        Ok(Some(feed)) => feed.url,
                        Ok(None) => {
                            return err_response(id, RssErrorCode::NotFound, "Feed not found");
                        }
                        Err(e) => return err_response(id, RssErrorCode::DbError, &e.to_string()),
                    };

                    let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncStarted {
                        feed_id: Some(feed_id.clone()),
                    }));

                    sync_engine.enqueue(&feed_id, &url).await;
                    ok(
                        id,
                        serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                    )
                }

                CrawlCommand::RssRefreshAll => {
                    let _ = event_tx.send(CrawlEvent::Rss(RssEvent::SyncStarted { feed_id: None }));
                    sync_engine.refresh_all().await;
                    ok(
                        id,
                        serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                    )
                }

                CrawlCommand::RssListCategories => match store.list_categories().await {
                    Ok(categories) => ok(
                        id,
                        serde_json::to_value(RssResponse::Categories { categories })
                            .unwrap_or_default(),
                    ),
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                CrawlCommand::RssImportOpml { path } => {
                    match tokio::fs::read_to_string(&path).await {
                        Ok(xml) => {
                            let importer = OpmlImporter::new(&store);
                            match importer.import(&xml).await {
                                Ok((total, imported, failed)) => ok(
                                    id,
                                    serde_json::to_value(RssResponse::ImportResult {
                                        total,
                                        imported,
                                        failed,
                                    })
                                    .unwrap_or_default(),
                                ),
                                Err(e) => {
                                    err_response(id, RssErrorCode::ParseFailed, &e.to_string())
                                }
                            }
                        }
                        Err(e) => err_response(
                            id,
                            RssErrorCode::InvalidUrl,
                            &format!("Cannot read file: {e}"),
                        ),
                    }
                }

                CrawlCommand::RssSetEnabled { enabled } => {
                    sync_engine.set_enabled(enabled);
                    let _ = event_tx.send(CrawlEvent::Rss(RssEvent::StateChanged { enabled }));
                    ok(
                        id,
                        serde_json::to_value(RssResponse::Ok).unwrap_or_default(),
                    )
                }

                CrawlCommand::RssExportOpml => match store.get_all_feeds_opml().await {
                    Ok(feeds) => {
                        let opml = generate_opml(&feeds);
                        ok(
                            id,
                            serde_json::to_value(RssResponse::ExportData { opml })
                                .unwrap_or_default(),
                        )
                    }
                    Err(e) => err_response(id, RssErrorCode::DbError, &e.to_string()),
                },

                // ── Wallhaven Commands ─────────────────────────────────────────
                CrawlCommand::WallhavenSetEnabled { enabled } => {
                    wh_enabled.store(enabled, Ordering::Relaxed);
                    let _ = event_tx.send(CrawlEvent::Wallhaven(WallhavenEvent::StateChanged {
                        enabled,
                    }));
                    ok(
                        id,
                        serde_json::to_value(WallhavenResponse::Ok).unwrap_or_default(),
                    )
                }

                CrawlCommand::WallhavenSearch(params) => {
                    if !wh_enabled.load(Ordering::Relaxed) {
                        return err_response_str(id, "disabled", "Wallhaven is disabled");
                    }
                    let mut client = wh_client.lock().await;
                    match client.search(&params).await {
                        Ok(WhApiResponse::Success { results, meta }) => ok(
                            id,
                            serde_json::to_value(WallhavenResponse::SearchResults {
                                results,
                                meta,
                            })
                            .unwrap_or_default(),
                        ),
                        Ok(WhApiResponse::AuthError(msg)) => {
                            err_response_str(id, "auth_error", &msg)
                        }
                        Ok(WhApiResponse::HttpError(code, msg)) => {
                            err_response_str(id, "http_error", &format!("HTTP {code}: {msg}"))
                        }
                        Err(e) => err_response_str(id, "network_error", &e.to_string()),
                    }
                }

                CrawlCommand::WallhavenDownload {
                    wallpaper_id,
                    url,
                    dest_dir,
                    filename,
                } => {
                    if !wh_enabled.load(Ordering::Relaxed) {
                        return err_response_str(id, "disabled", "Wallhaven is disabled");
                    }
                    let dl = Arc::clone(&downloader);
                    let wid = wallpaper_id.clone();
                    // Spawn download in background
                    tokio::spawn(async move {
                        if let Err(e) = dl
                            .download(&wid, &url, &dest_dir, filename.as_deref())
                            .await
                        {
                            warn!("Wallhaven download failed for {wid}: {e}");
                        }
                    });
                    ok(
                        id,
                        serde_json::to_value(WallhavenResponse::Ok).unwrap_or_default(),
                    )
                }

                _ => CrawlResponse::error(
                    id,
                    error_code::METHOD_NOT_FOUND,
                    "Unsupported command for crawl-webservice",
                ),
            }
        })
    });

    server.set_dispatcher(dispatcher);
    server.run().await?;
    Ok(())
}

fn ok(id: Option<serde_json::Value>, value: serde_json::Value) -> CrawlResponse {
    CrawlResponse::success(id, value)
}

fn err_response(id: Option<serde_json::Value>, code: RssErrorCode, message: &str) -> CrawlResponse {
    CrawlResponse::success(
        id,
        serde_json::to_value(RssResponse::Error {
            code,
            message: message.to_string(),
        })
        .unwrap_or_default(),
    )
}

fn err_response_str(id: Option<serde_json::Value>, code: &str, message: &str) -> CrawlResponse {
    CrawlResponse::success(
        id,
        serde_json::to_value(WallhavenResponse::Error {
            code: code.to_string(),
            message: message.to_string(),
        })
        .unwrap_or_default(),
    )
}
